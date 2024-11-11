#![feature(rustc_private)]

extern crate rustc_hir;
extern crate rustc_span;

use clippy_wrappers::span_lint_and_help;
use rustc_hir::{
    intravisit::{walk_expr, Visitor},
    Expr, ExprKind,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::{def_id::DefId, Span};
use std::{
    collections::{HashMap, HashSet},
    vec,
};
use utils::{verify_token_interface_function, FunctionCallVisitor};

const LINT_MESSAGE: &str = "This function belongs to the Token Interface and should emit an event";

dylint_linting::impl_late_lint! {
    pub TOKEN_INTERFACE_EVENTS,
    Warn,
    "",
    TokenInterfaceEvents::default(),
    {
        name: "Token Interface events checker",
        long_message: "Not emiting the established events breaks compatibility with the token standard and can lead to interoperability problems between the contract and its observers",
        severity: "Medium",
        help: "https://coinfabrik.github.io/scout-soroban/docs/detectors/token-interface-events",
        vulnerability_class: "Standard Compliance",
    }
}

#[derive(Default)]
struct TokenInterfaceEvents {
    function_call_graph: HashMap<DefId, HashSet<DefId>>,
    checked_functions: HashSet<String>,
    //eventless_storage_changers: HashSet<DefId>,
    defids_with_events: HashSet<DefId>,
    canonical_funcs_def_id: HashSet<DefId>,
    impl_token_interface_trait: bool,
}

/// Used to verify if, starting from a specific parent in the call graph, an event is emitted at any point of the flow.
/// # Params:
///     - fcg: function call graph
///     - parent: the item from which the analysis starts.
///     - check_against: a HashSet that is used to compare the defids. This HashSet is supposed to contain all the defids of the functions that emit events (collected by the `visit_expr` and `check_func` functions).
fn check_events_children(
    fcg: &HashMap<DefId, HashSet<DefId>>,
    parent: &DefId,
    check_against: &HashSet<DefId>,
) -> bool {
    if check_against.contains(parent) {
        return true;
    }
    let children = fcg.get(parent);
    if children.is_some() {
        for c in children.unwrap() {
            if check_against.contains(c) || check_events_children(fcg, c, check_against) {
                return true;
            }
        }
    }
    false
}

impl<'tcx> LateLintPass<'tcx> for TokenInterfaceEvents {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx rustc_hir::Item<'tcx>) {
        if let rustc_hir::ItemKind::Impl(impl_block) = item.kind {
            if let Some(trait_ref) = impl_block.of_trait {
                let trait_def_id = trait_ref.path.res.def_id();
                let trait_name = cx.tcx.def_path_str(trait_def_id);

                if trait_name == "soroban_sdk::token::TokenInterface" {
                    self.impl_token_interface_trait = true;
                }
            }
        }
    }

    fn check_crate_post(&mut self, cx: &LateContext<'tcx>) {
        let functions_that_emit_events: [String; 6] = [
            "burn".to_string(),
            "approve".to_string(),
            "transfer_from".to_string(),
            "burn_from".to_string(),
            "transfer".to_string(),
            "mint".to_string(),
        ];
        // Verify if the contract implements the token interface trait.
        if !self.impl_token_interface_trait {
            return;
        }

        // Emit the alerts for the considered "unsafe" functions.
        for func in self.function_call_graph.keys() {
            // Only take into account those functions that are public and exposed in a soroban contract (entrypoints that can be called externally). We do not advise on functions that are used auxiliarily.
            if self.canonical_funcs_def_id.contains(func)
                && functions_that_emit_events.contains(
                    &cx.tcx
                        .def_path_str(func)
                        .split("::")
                        .last()
                        .unwrap()
                        .to_string(),
                )
            {
                // Verify if the function itself or the ones it calls (directly or indirectly) emit an event at any point of the flow.
                let emits_event_in_flow = check_events_children(
                    &self.function_call_graph,
                    func,
                    &self.defids_with_events,
                );

                // If both conditions are met, emit an warning.
                if !emits_event_in_flow {
                    span_lint_and_help(
                        cx,
                        TOKEN_INTERFACE_EVENTS,
                        cx.tcx.hir().span_if_local(*func).unwrap(),
                        LINT_MESSAGE,
                        /* cx.tcx.hir().span_if_local(r) */ None,
                        "",
                    );
                }
            }
        }
    }

    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: rustc_hir::intravisit::FnKind<'tcx>,
        fn_decl: &'tcx rustc_hir::FnDecl<'tcx>,
        body: &'tcx rustc_hir::Body<'tcx>,
        span: Span,
        local_def_id: rustc_span::def_id::LocalDefId,
    ) {
        let def_id = local_def_id.to_def_id();
        self.checked_functions.insert(cx.tcx.def_path_str(def_id));

        if span.from_expansion() {
            return;
        }

        let fn_name = cx.tcx.def_path_str(def_id);

        let mut function_call_visitor =
            FunctionCallVisitor::new(cx, def_id, &mut self.function_call_graph);
        function_call_visitor.visit_body(body);

        // If the function is part of the token interface, I store its defid.
        if verify_token_interface_function(fn_name.clone(), fn_decl.inputs, fn_decl.output) {
            self.canonical_funcs_def_id.insert(def_id);
        }
        let mut token_interface_events_visitor = TokenInterfaceEventsVisitor {
            _cx: cx,
            emits_event: false,
        };

        token_interface_events_visitor.visit_body(body);

        // If the function emits an event, we storage its defid.
        if token_interface_events_visitor.emits_event {
            self.defids_with_events.insert(def_id);
        }
    }
}

struct TokenInterfaceEventsVisitor<'a, 'tcx> {
    _cx: &'a LateContext<'tcx>,
    emits_event: bool,
}

impl<'a, 'tcx> Visitor<'tcx> for TokenInterfaceEventsVisitor<'a, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx Expr<'_>) {
        if let ExprKind::MethodCall(path, _receiver, _, _) = expr.kind {
            let name = path.ident.name.as_str();

            // verify if it is an event emission
            if name == "events" {
                self.emits_event = true;
            }
        }
        walk_expr(self, expr);
    }
}
