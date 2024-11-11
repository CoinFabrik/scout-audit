#![feature(rustc_private)]

extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;

use clippy_wrappers::span_lint_and_help;
use if_chain::if_chain;
use rustc_hir::{
    intravisit::{walk_expr, FnKind, Visitor},
    Body, Expr, ExprKind, FnDecl,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::ty::{GenericArgKind, Ty, TyKind};
use rustc_span::{
    def_id::{DefId, LocalDefId},
    Span, Symbol,
};
use std::collections::{HashMap, HashSet};
use utils::{
    get_node_type_opt, is_soroban_address, is_soroban_function, is_soroban_map, FunctionCallVisitor,
};

const LINT_MESSAGE: &str = "This mapping operation is called without access control on a different key than the caller's address";

dylint_linting::impl_late_lint! {
    pub UNPROTECTED_MAPPING_OPERATION,
    Warn,
    LINT_MESSAGE,
    UnprotectedMappingOperation::default(),
    {
        name: "Unprotected Mapping Operation",
        long_message: "This mapping operation is called without access control on a different key than the caller's address",
        severity: "Critical",
        help: "https://coinfabrik.github.io/scout-soroban/docs/detectors/unprotected-mapping-operation",
        vulnerability_class: "Access Control",
    }
}

#[derive(Default)]
struct UnprotectedMappingOperation {
    function_call_graph: HashMap<DefId, HashSet<DefId>>,
    authorized_functions: HashSet<DefId>,
    checked_functions: HashSet<String>,
    unauthorized_mapping_calls: HashMap<DefId, Vec<Span>>,
}

impl<'tcx> LateLintPass<'tcx> for UnprotectedMappingOperation {
    fn check_crate_post(&mut self, cx: &LateContext<'tcx>) {
        for (callee_def_id, mapping_spans) in &self.unauthorized_mapping_calls {
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
                for span in mapping_spans {
                    span_lint_and_help(
                        cx,
                        UNPROTECTED_MAPPING_OPERATION,
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

        // If this function comes from a macro, don't analyze it
        if span.from_expansion() {
            return;
        }

        // First visitor: build the function call graph
        let mut function_call_visitor =
            FunctionCallVisitor::new(cx, def_id, &mut self.function_call_graph);
        function_call_visitor.visit_body(body);

        // Second visitor: check for authed functions and storage calls
        let mut unprotected_mapping_visitor = UnprotectedMappingOperationVisitor {
            cx,
            auth_found: false,
            mapping_spans: Vec::new(),
        };
        unprotected_mapping_visitor.visit_body(body);

        // If the function calls storage without auth, we store the spans
        if !unprotected_mapping_visitor.mapping_spans.is_empty()
            && !unprotected_mapping_visitor.auth_found
        {
            self.unauthorized_mapping_calls
                .insert(def_id, unprotected_mapping_visitor.mapping_spans);
        } else if unprotected_mapping_visitor.auth_found {
            self.authorized_functions.insert(def_id);
        }
    }
}

struct UnprotectedMappingOperationVisitor<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    auth_found: bool,
    mapping_spans: Vec<Span>,
}

impl<'tcx> UnprotectedMappingOperationVisitor<'_, 'tcx> {
    fn is_soroban_map_with_address(&self, receiver: &Expr, receiver_type: Ty<'_>) -> bool {
        if_chain! {
            // Check that the receiver expression is a field (e.g., accessing a struct's field).
            if let ExprKind::Field(..) = &receiver.kind;

            // Verify that the type of the receiver is a 'soroban_sdk::Map'.
            if is_soroban_map(self.cx, receiver_type);

            // Retrieve the first generic argument, ensure it exists and is of type Ty.
            if let TyKind::Adt(_, args) = receiver_type.kind();
            if let Some(first_arg) = args.first();
            if let GenericArgKind::Type(first_type) = first_arg.unpack();

            // Verify that the type of the first argument is 'soroban_sdk::Address'.
            if is_soroban_address(self.cx, first_type);
            then {
                return true;
            }
        }
        false
    }
}

impl<'a, 'tcx> Visitor<'tcx> for UnprotectedMappingOperationVisitor<'a, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx Expr<'tcx>) {
        if self.auth_found {
            return;
        }

        if let ExprKind::MethodCall(path_segment, receiver, _args, _) = &expr.kind {
            // Get the method expression type and check if it's a map with address
            let receiver_type = get_node_type_opt(self.cx, &receiver.hir_id);
            if let Some(type_) = receiver_type {
                // Check if the method call is require_auth() on an address
                if is_soroban_address(self.cx, type_)
                    && path_segment.ident.name == Symbol::intern("require_auth")
                {
                    self.auth_found = true;
                }

                // Look for usage of soroban map with address
                // Anything that looks like `soroban_sdk::Map::<soroban_sdk::Address, _>` is in our interest
                if self.is_soroban_map_with_address(receiver, type_)
                    && path_segment.ident.name == Symbol::intern("set")
                {
                    self.mapping_spans.push(expr.span);
                }
            }
        }

        walk_expr(self, expr);
    }
}
