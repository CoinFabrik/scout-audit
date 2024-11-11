#![feature(rustc_private)]

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
use utils::FunctionCallVisitor;

const LINT_MESSAGE: &str = "Abitrary users should not have control over keys because it implies writing any value of left mapping, lazy variable, or the main struct of the contract located in position 0 of the storage";

dylint_linting::impl_late_lint! {
    /// ### What it does
    /// Checks for calls to env.storage() without a prior call to env.require_auth()
    ///
    /// ### Why is this bad?
    /// Functions using keys as variables without proper access control or input sanitation can allow users to perform changes in arbitrary memory locations.
    ///
    /// ### Known problems
    /// Only check the function call, so false positives could result.
    ///
    /// ### Example
    /// ```rust
    /// fn set_contract_storage(env: Env) {
    ///   let _storage = env.storage().instance();
    /// }
    /// ```
    /// Use instead:
    /// ```rust
    /// fn set_contract_storage(env: Env, user: Address) {
    ///   user.require_auth();
    ///   let _storage = env.storage().instance();
    /// }
    /// ```
    pub SET_CONTRACT_STORAGE,
    Warn,
    LINT_MESSAGE,
    SetContractStorage::default(),
    {
        name: "Set Contract Storage",
        long_message: "Functions using keys as variables without proper access control or input sanitation can allow users to perform changes in arbitrary memory locations.",
        severity: "Critical",
        help: "https://coinfabrik.github.io/scout-soroban/docs/detectors/set-contract-storage",
        vulnerability_class: "Authorization",
    }
}

const SOROBAN_INSTANCE_STORAGE: &str = "soroban_sdk::storage::Instance";
const SOROBAN_TEMPORARY_STORAGE: &str = "soroban_sdk::storage::Temporary";
const SOROBAN_PERSISTENT_STORAGE: &str = "soroban_sdk::storage::Persistent";
const SOROBAN_ADDRESS: &str = "soroban_sdk::Address";

#[derive(Default)]
struct SetContractStorage {
    function_call_graph: HashMap<DefId, HashSet<DefId>>,
    authorized_functions: HashSet<DefId>,
    checked_functions: HashSet<String>,
    unauthorized_storage_calls: HashMap<DefId, Vec<Span>>,
}

impl<'tcx> SetContractStorage {
    fn is_soroban_function(&self, cx: &LateContext<'tcx>, def_id: &DefId) -> bool {
        let def_path_str = cx.tcx.def_path_str(*def_id);
        let mut parts = def_path_str.rsplitn(2, "::");

        let function_name = parts.next().unwrap();
        let contract_path = parts.next().unwrap_or("");

        if contract_path.is_empty() {
            return false;
        }

        // Define the patterns to check against
        let patterns = [
            format!("{}Client::<'a>::try_{}", contract_path, function_name),
            format!("{}::{}", contract_path, function_name),
            format!("{}::spec_xdr_{}", contract_path, function_name),
            format!("{}Client::<'a>::{}", contract_path, function_name),
        ];

        patterns
            .iter()
            .all(|pattern| self.checked_functions.contains(pattern.as_str()))
    }
}

impl<'tcx> LateLintPass<'tcx> for SetContractStorage {
    fn check_crate_post(&mut self, cx: &LateContext<'tcx>) {
        for (callee_def_id, storage_spans) in &self.unauthorized_storage_calls {
            let is_callee_soroban = self.is_soroban_function(cx, callee_def_id);
            let (is_called_by_soroban, is_soroban_caller_authed) = self
                .function_call_graph
                .iter()
                .fold((false, true), |acc, (caller, callees)| {
                    if callees.contains(callee_def_id) {
                        let is_caller_soroban = self.is_soroban_function(cx, caller);
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

impl<'tcx> SetStorageWarnVisitor<'_, 'tcx> {
    fn get_node_type(&self, hir_id: HirId) -> Ty<'tcx> {
        self.cx.typeck_results().node_type(hir_id)
    }

    fn is_soroban_address(&self, type_: Ty<'tcx>) -> bool {
        type_.to_string().contains(SOROBAN_ADDRESS)
    }

    fn is_soroban_storage(&self, type_: Ty<'tcx>) -> bool {
        let type_string = type_.to_string();
        type_string.contains(SOROBAN_INSTANCE_STORAGE)
            || type_string.contains(SOROBAN_TEMPORARY_STORAGE)
            || type_string.contains(SOROBAN_PERSISTENT_STORAGE)
    }
}

impl<'a, 'tcx> Visitor<'tcx> for SetStorageWarnVisitor<'a, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx Expr<'tcx>) {
        if self.auth_found {
            return;
        }

        if let ExprKind::MethodCall(path, object, args, _) = &expr.kind {
            let object_type = self.get_node_type(object.hir_id);

            // Check if the method call is require_auth() on an address
            if self.is_soroban_address(object_type)
                && path.ident.name == Symbol::intern("require_auth")
            {
                self.auth_found = true;
            }

            if_chain! {
                // Look for calls to set() on storage
                if self.is_soroban_storage(object_type);
                if path.ident.name == Symbol::intern("set");
                if let Some(first_arg) = args.first();
                // TODO: add support for various structures
                // Check if the first argument is an address
                if self.is_soroban_address(self.get_node_type(first_arg.hir_id));
                then {
                    self.storage_spans.push(expr.span);
                }
            }
        }

        walk_expr(self, expr)
    }
}
