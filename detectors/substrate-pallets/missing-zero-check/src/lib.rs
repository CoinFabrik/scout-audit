#![feature(rustc_private)]
#![feature(let_chains)]

extern crate rustc_error_messages;
extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;

use common::{
    analysis::{
        decomposers::{ expr_to_call, expr_to_path, path_to_resolved, resolved_to_def },
        get_receiver_ident_name,
    },
    declarations::{ Severity, VulnerabilityClass },
    macros::expose_lint_info,
};
use rustc_hir::{ intravisit::{ walk_expr, Visitor }, BinOpKind, Expr, ExprKind, PatKind };
use rustc_lint::{ LateContext, LateLintPass };
use rustc_middle::ty::TyKind;
use rustc_span::{ def_id::DefId, Span };

const LINT_MESSAGE: &str =
    "Not checking for a zero value in the parameters may result in redundant operations";

#[expose_lint_info]
pub static MISSING_ZERO_CHECK_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "Failing to check for a zero value in the parameters may lead to unnecessary operations, potentially increasing resource usage and reducing the efficiency of the function. Consider checking the parameter to not be zero",
    severity: Severity::Minor,
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/missing-zero-check",
    vulnerability_class: VulnerabilityClass::BestPractices,
};

dylint_linting::impl_late_lint! {
    pub MISSING_ZERO_CHECK,
    Warn,
    LINT_MESSAGE,
    MissingZeroCheck::default()
}

#[derive(Default)]
pub struct MissingZeroCheck {
    pub param_infos: Vec<ParamInfo>,
}

impl MissingZeroCheck {
    pub fn new() -> Self {
        Self {
            param_infos: Vec::new(),
        }
    }
    pub fn add_param_info(
        &mut self,
        param_name: &str,
        def_path: &str,
        span: Span,
        is_checked: bool
    ) {
        self.param_infos.push(ParamInfo {
            param_name: param_name.to_string(),
            def_path: def_path.to_string(),
            span,
            is_checked,
        });
    }
}
#[derive(Debug, Clone)]
pub struct ParamInfo {
    pub param_name: String,
    pub def_path: String,
    pub span: Span,
    pub is_checked: bool,
}
struct MissingZeroCheckFinder<'tcx, 'tcx_ref> {
    cx: &'tcx_ref LateContext<'tcx>,
    pub param_infos: Vec<ParamInfo>,
}

fn expr_to_def_id<'hir>(kind: &'hir ExprKind<'hir>) -> Option<DefId> {
    let (call, _) = expr_to_call(kind)?;
    let qpath = expr_to_path(&call.kind)?;
    let (_, path) = path_to_resolved(&qpath)?;
    let (_, def_id) = resolved_to_def(&path.res)?;
    Some(*def_id)
}

impl<'tcx> Visitor<'tcx> for MissingZeroCheckFinder<'tcx, '_> {
    fn visit_expr(&mut self, expr: &'tcx Expr<'_>) {
        if let ExprKind::Binary(op, rvalue, lvalue) = expr.kind {
            if BinOpKind::Ne == op.node || BinOpKind::Eq == op.node {
                let rvalue_name = get_receiver_ident_name(rvalue);
                let lvalue_name = get_receiver_ident_name(lvalue);
                // If one of the values in the binary operation is a function parameter, verify if the other value is zero.
                self.param_infos.iter_mut().for_each(|param| {
                    let actual_fn = self.cx.tcx.def_path_str(expr.hir_id.owner);
                    if param.def_path == actual_fn {
                        if
                            param.param_name == lvalue_name.to_string() ||
                            param.param_name == rvalue_name.to_string()
                        {
                            if let Some(def_id) = expr_to_def_id(&lvalue.kind) {
                                let name = self.cx.tcx.def_path_str(def_id);
                                if name.to_string().contains("zero") {
                                    param.is_checked = true;
                                }
                            }
                        }
                    }
                });
            }
        }

        walk_expr(self, expr);
    }
}

impl<'tcx> LateLintPass<'tcx> for MissingZeroCheck {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: rustc_hir::intravisit::FnKind<'tcx>,
        _: &'tcx rustc_hir::FnDecl<'tcx>,
        body: &'tcx rustc_hir::Body<'tcx>,
        _: Span,
        localdef: rustc_span::def_id::LocalDefId
    ) {
        let mut utf_storage = MissingZeroCheckFinder {
            cx,
            param_infos: Vec::default(),
        };

        // Look for function params with AccountId type
        let mir_body = cx.tcx.optimized_mir(localdef);
        for (arg, hir_param) in mir_body.args_iter().zip(body.params.iter()) {
            let fn_name = &cx.tcx.def_path_str(localdef);
            if let TyKind::Alias(_alias_kind, alias_ty) = mir_body.local_decls[arg].ty.kind() {
                let def_id = &cx.tcx.def_path_str(alias_ty.def_id);
                if
                    def_id.contains("traits::Currency::Balance") ||
                    def_id.contains("Config::Balance")
                {
                    let mut param_name = "";
                    if let PatKind::Binding(_, _, ident, _) = &hir_param.pat.kind {
                        param_name = ident.name.as_str();
                    }
                    if !fn_name.contains("new_call_variant") {
                        self.add_param_info(
                            param_name,
                            fn_name,
                            mir_body.local_decls[arg].source_info.span,
                            false
                        );
                    }
                }
            }
        }

        utf_storage.param_infos = self.param_infos.clone();

        walk_expr(&mut utf_storage, body.value);

        utf_storage.param_infos
            .clone()
            .iter()
            .for_each(|p| {
                if p.is_checked {
                    self.param_infos.retain(|param| {
                        param.param_name != p.param_name || param.def_path != p.def_path
                    });
                }
            });
    }
    fn check_crate_post(&mut self, cx: &LateContext<'tcx>) {
        self.param_infos
            .iter()
            .for_each(|p| {
                clippy_utils::diagnostics::span_lint(cx, MISSING_ZERO_CHECK, p.span, LINT_MESSAGE)
            });
    }
}
