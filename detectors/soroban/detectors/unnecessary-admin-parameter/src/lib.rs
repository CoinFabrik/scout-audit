#![feature(rustc_private)]

extern crate rustc_hir;
extern crate rustc_span;

use clippy_wrappers::span_lint_and_help;
use edit_distance::edit_distance;
use if_chain::if_chain;
use rustc_hir::{
    def::Res,
    intravisit::{walk_expr, FnKind, Visitor},
    Body, Expr, ExprKind, FnDecl, HirId, Param, PatKind, QPath,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::{
    def_id::{DefId, LocalDefId},
    Span, Symbol,
};
use std::collections::{HashMap, HashSet};
use utils::{get_node_type_opt, is_soroban_address, is_soroban_function};

const LINT_MESSAGE: &str = "Usage of admin parameter might be unnecessary";

dylint_linting::impl_late_lint! {
    pub UNNECESSARY_ADMIN_PARAMETER,
    Warn,
    LINT_MESSAGE,
    UnnecessaryAdminParameter::default(),
    {
        name: "Unnecessary Admin Parameter",
        long_message: "This function has an admin parameter that might be unnecessary. Consider retrieving the admin from storage instead.",
        severity: "Medium",
        help: "https://coinfabrik.github.io/scout-soroban/docs/detectors/unnecessary-admin-parameter",
        vulnerability_class: "Access Control",
    }
}

struct AdminInfo {
    param_span: Span,
    usage_span: Option<Span>,
}

#[derive(Default)]
struct UnnecessaryAdminParameter {
    checked_functions: HashSet<String>,
    admin_params: HashMap<DefId, AdminInfo>,
}

impl<'tcx> LateLintPass<'tcx> for UnnecessaryAdminParameter {
    fn check_crate_post(&mut self, cx: &LateContext<'tcx>) {
        for (function_def_id, admin_info) in &self.admin_params {
            if is_soroban_function(cx, &self.checked_functions, function_def_id) {
                let help_message = if admin_info.usage_span.is_some() {
                    "Consider retrieving the admin from storage instead of passing it as a parameter"
                } else {
                    "This admin parameter is not used for access control. Consider removing it or implementing proper access control"
                };

                span_lint_and_help(
                    cx,
                    UNNECESSARY_ADMIN_PARAMETER,
                    admin_info.param_span,
                    LINT_MESSAGE,
                    admin_info.usage_span,
                    help_message,
                )
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
        let def_id = local_def_id.to_def_id();
        self.checked_functions.insert(cx.tcx.def_path_str(def_id));

        if span.from_expansion() {
            return;
        }

        // Skip analysis for functions named "initialize"
        let fn_name = cx.tcx.item_name(def_id);
        if is_similar_to(&fn_name.as_str().to_lowercase(), "initialize") {
            return;
        }

        // Step 1: Check for admin parameter
        if let Some(admin_param) = find_admin_param(cx, body.params) {
            // Step 2: Check function body for proper use of admin parameter
            let mut visitor = UnnecessaryAdminParameterVisitor {
                admin_param_id: admin_param.pat.hir_id,
                access_control_span: None,
            };
            visitor.visit_body(body);

            // Step 3: Store the information for later analysis
            self.admin_params.insert(
                def_id,
                AdminInfo {
                    param_span: admin_param.span,
                    usage_span: visitor.access_control_span,
                },
            );
        }
    }
}

fn find_admin_param<'tcx>(
    cx: &LateContext<'tcx>,
    params: &'tcx [Param<'tcx>],
) -> Option<&'tcx Param<'tcx>> {
    params.iter().find(|param| {
        matches!(param.pat.kind, PatKind::Binding(_, _, ident, _)
            if is_similar_to(&ident.name.as_str().to_lowercase(), "admin"))
            && get_node_type_opt(cx, &param.hir_id)
                .map_or(false, |type_| is_soroban_address(cx, type_))
    })
}

fn is_similar_to(s1: &str, s2: &str) -> bool {
    edit_distance(s1, s2) <= 1
}

struct UnnecessaryAdminParameterVisitor {
    admin_param_id: HirId,
    access_control_span: Option<Span>,
}

impl<'tcx> Visitor<'tcx> for UnnecessaryAdminParameterVisitor {
    fn visit_expr(&mut self, expr: &'tcx Expr<'tcx>) {
        if_chain! {
            if let ExprKind::MethodCall(path_segment, receiver, ..) = expr.kind;
            if path_segment.ident.name == Symbol::intern("require_auth");
            if let ExprKind::Path(QPath::Resolved(_, path)) = receiver.kind;
            if let Res::Local(id) = path.res;
            if id == self.admin_param_id;
            then {
                self.access_control_span = Some(expr.span);
            }
        }

        walk_expr(self, expr);
    }
}
