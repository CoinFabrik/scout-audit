#![feature(rustc_private)]
#![feature(let_chains)]

extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;

use std::collections::{HashMap, HashSet};

use clippy_wrappers::span_lint_and_help;
use if_chain::if_chain;
use rustc_hir::{
    intravisit::{walk_expr, FnKind, Visitor},
    Body, Expr, ExprKind, FnDecl, HirId,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::ty::Ty;
use rustc_span::{
    def_id::{DefId, LocalDefId},
    Span, Symbol,
};
use utils::{is_soroban_address, is_soroban_env, is_soroban_function, FunctionCallVisitor};

const LINT_MESSAGE: &str = "This update_current_contract_wasm is called without access control";

dylint_linting::impl_late_lint! {
    pub UNPROTECTED_UPDATE_CURRENT_CONTRACT_WASM,
    Warn,
    LINT_MESSAGE,
    UnprotectedUpdateCurrentContractWasm::default(),
    {
        name: "Unprotected Update Current Contract Wasm",
        long_message: "If users are allowed to call update_current_contract_wasm, they can intentionally modify the contract behaviour, leading to the loss of all associated data/tokens and functionalities given by this contract or by others that depend on it. To prevent this, the function should be restricted to administrators or authorized users only.    ",
        severity: "Critical",
        help: "https://coinfabrik.github.io/scout-soroban/docs/detectors/unprotected-update-current-contract-wasm",
        vulnerability_class: "Authorization",
    }
}

#[derive(Default)]
struct UnprotectedUpdateCurrentContractWasm {
    function_call_graph: HashMap<DefId, HashSet<DefId>>,
    authorized_functions: HashSet<DefId>,
    checked_functions: HashSet<String>,
    unauthorized_update_wasm_calls: HashMap<DefId, Vec<Span>>,
}

impl<'tcx> LateLintPass<'tcx> for UnprotectedUpdateCurrentContractWasm {
    fn check_crate_post(&mut self, cx: &LateContext<'tcx>) {
        for (callee_def_id, storage_spans) in &self.unauthorized_update_wasm_calls {
            let is_callee_soroban = is_soroban_function(cx, &self.checked_functions, callee_def_id);
            let (is_called_by_soroban, is_soroban_caller_authed) = self
                .function_call_graph
                .iter()
                .fold((false, true), |acc, (caller, callees)| {
                    if callees.contains(callee_def_id) {
                        let is_caller_soroban =
                            is_soroban_function(cx, &self.checked_functions, caller);
                        // Update if the caller is Soroban and check if it's authorized only if it's a Soroban caller
                        (
                            acc.0 || is_caller_soroban,
                            acc.1
                                && (!is_caller_soroban
                                    || self.authorized_functions.contains(caller)),
                        )
                    } else {
                        acc
                    }
                });

            // Determine if a warning should be emitted
            if is_callee_soroban || (is_called_by_soroban && !is_soroban_caller_authed) {
                for span in storage_spans {
                    span_lint_and_help(
                        cx,
                        UNPROTECTED_UPDATE_CURRENT_CONTRACT_WASM,
                        *span,
                        LINT_MESSAGE,
                        None,
                        "",
                    );
                }
            }
        }
    }
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: FnKind<'tcx>,
        _: &'tcx FnDecl<'tcx>,
        body: &'tcx Body<'tcx>,
        span: Span,
        local_def_id: LocalDefId,
    ) {
        // Fetch the DefId of the current function for future reference on public functions implemented inside the soroban contract
        let def_id = local_def_id.to_def_id();
        self.checked_functions.insert(cx.tcx.def_path_str(def_id));

        // If the function comes from a macro, we don't analyze it
        if span.from_expansion() {
            return;
        }

        // First visitor: build the function call graph
        let mut function_call_visitor =
            FunctionCallVisitor::new(cx, def_id, &mut self.function_call_graph);
        function_call_visitor.visit_body(body);

        // Second visitor: check for authed functions and calls to update wasm
        let mut unprotected_wasm_visitor = UnprotectedUpdateVisitor {
            cx,
            auth_found: false,
            update_contract_wasm_spans: Vec::new(),
        };
        unprotected_wasm_visitor.visit_expr(body.value);

        // If the function calls to update the wasm without authorization, we store the spans.
        // If the function is authorized, we store it.
        if !unprotected_wasm_visitor
            .update_contract_wasm_spans
            .is_empty()
            && !unprotected_wasm_visitor.auth_found
        {
            self.unauthorized_update_wasm_calls
                .insert(def_id, unprotected_wasm_visitor.update_contract_wasm_spans);
        } else if unprotected_wasm_visitor.auth_found {
            self.authorized_functions.insert(def_id);
        }
    }
}

struct UnprotectedUpdateVisitor<'tcx, 'a> {
    cx: &'a LateContext<'tcx>,
    auth_found: bool,
    update_contract_wasm_spans: Vec<Span>,
}

impl<'tcx> UnprotectedUpdateVisitor<'tcx, '_> {
    fn get_node_type(&self, hir_id: HirId) -> Ty<'tcx> {
        self.cx.typeck_results().node_type(hir_id)
    }
}

impl<'tcx> Visitor<'tcx> for UnprotectedUpdateVisitor<'tcx, '_> {
    fn visit_expr(&mut self, expr: &'tcx Expr<'_>) {
        if self.auth_found {
            return;
        }

        if let ExprKind::MethodCall(path_segment, receiver, ..) = expr.kind {
            let receiver_type = self.get_node_type(receiver.hir_id);

            // Check if the method call is require_auth() on an address
            if is_soroban_address(self.cx, receiver_type)
                && path_segment.ident.name == Symbol::intern("require_auth")
            {
                self.auth_found = true;
            }

            if_chain! {
                if path_segment.ident.name == Symbol::intern("update_current_contract_wasm");
                if let ExprKind::MethodCall(deployer_path_segment, receiver, ..) = receiver.kind;
                if deployer_path_segment.ident.name == Symbol::intern("deployer");
                let receiver_type = self.get_node_type(receiver.hir_id);
                if is_soroban_env(self.cx, receiver_type);
                then {
                    // Found `env.deployer().update_current_contract_wasm()`
                    self.update_contract_wasm_spans.push(expr.span);
                }
            }
        }

        walk_expr(self, expr);
    }
}
