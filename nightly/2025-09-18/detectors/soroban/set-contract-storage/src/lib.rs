#![feature(rustc_private)]

extern crate rustc_hir;
extern crate rustc_span;

use std::collections::{HashMap, HashSet};

use clippy_utils::diagnostics::span_lint_and_help;
use common::{
    analysis::{self, FunctionCallVisitor, SorobanStorageType},
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use if_chain::if_chain;
use rustc_hir::{
    intravisit::{walk_expr, FnKind, Visitor},
    Body, Expr, ExprKind, FnDecl,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::{
    def_id::{DefId, LocalDefId},
    Span, Symbol,
};

const LINT_MESSAGE: &str = "Abitrary users should not have control over keys because it implies writing any value of left mapping, lazy variable, or the main struct of the contract located in position 0 of the storage";

#[expose_lint_info]
pub static SET_CONTRACT_STORAGE_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "Functions using keys as variables without proper access control or input sanitation can allow users to perform changes in arbitrary memory locations.",
    severity: Severity::Critical,
    help: "https://coinfabrik.github.io/scout-audit/docs/detectors/soroban/set-contract-storage",
    vulnerability_class: VulnerabilityClass::Authorization,
};

dylint_linting::impl_late_lint! {
    pub SET_CONTRACT_STORAGE,
    Warn,
    LINT_MESSAGE,
    SetContractStorage::default()
}

#[derive(Default)]
struct SetContractStorage {
    function_call_graph: HashMap<DefId, HashSet<DefId>>,
    authorized_functions: HashSet<DefId>,
    checked_functions: HashSet<String>,
    unauthorized_storage_calls: HashMap<DefId, Vec<Span>>,
}

impl<'tcx> LateLintPass<'tcx> for SetContractStorage {
    fn check_crate_post(&mut self, cx: &LateContext<'tcx>) {
        for (callee_def_id, storage_spans) in &self.unauthorized_storage_calls {
            let is_callee_soroban =
                analysis::is_soroban_function(cx, &self.checked_functions, callee_def_id);
            let (is_called_by_soroban, is_soroban_caller_authed) = self
                .function_call_graph
                .iter()
                .fold((false, true), |acc, (caller, callees)| {
                    if callees.contains(callee_def_id) {
                        let is_caller_soroban =
                            analysis::is_soroban_function(cx, &self.checked_functions, caller);
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
                    span_lint_and_help(cx, SET_CONTRACT_STORAGE, *span, LINT_MESSAGE, None, "");
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

        // If this function comes from a macro, don't analyze it
        if span.from_expansion() {
            return;
        }

        // First visitor: build the function call graph
        let mut function_call_visitor =
            FunctionCallVisitor::new(cx, def_id, &mut self.function_call_graph);
        function_call_visitor.visit_body(body);

        // Second visitor: check for authed functions and storage calls
        let mut storage_warn_visitor = SetStorageWarnVisitor {
            cx,
            auth_found: false,
            storage_spans: Vec::new(),
        };
        storage_warn_visitor.visit_body(body);

        // If the function calls storage without auth, we store the spans
        if !storage_warn_visitor.storage_spans.is_empty() && !storage_warn_visitor.auth_found {
            self.unauthorized_storage_calls
                .insert(def_id, storage_warn_visitor.storage_spans);
        } else if storage_warn_visitor.auth_found {
            self.authorized_functions.insert(def_id);
        }
    }
}

struct SetStorageWarnVisitor<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    auth_found: bool,
    storage_spans: Vec<Span>,
}

impl<'a, 'tcx> Visitor<'tcx> for SetStorageWarnVisitor<'a, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx Expr<'tcx>) {
        if self.auth_found {
            return;
        }

        if let ExprKind::MethodCall(path, object, args, _) = &expr.kind {
            let object_type = analysis::get_node_type_opt(self.cx, &object.hir_id);
            if let Some(object_type) = object_type {
                // Check if the method call is require_auth() on an address
                if analysis::is_soroban_address(self.cx, object_type)
                    && path.ident.name == Symbol::intern("require_auth")
                {
                    self.auth_found = true;
                }

                if_chain! {
                    // Look for calls to set() on storage
                    if analysis::is_soroban_storage(self.cx, object_type, SorobanStorageType::Any);
                    if path.ident.name == Symbol::intern("set");
                    if let Some(first_arg) = args.first();
                    // Check if the first argument is an address
                    if let Some(first_arg_type) = analysis::get_node_type_opt(self.cx, &first_arg.hir_id);
                    if analysis::is_soroban_address(self.cx, first_arg_type);
                    then {
                        self.storage_spans.push(expr.span);
                    }
                }
            }
        }

        walk_expr(self, expr)
    }
}
