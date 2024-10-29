#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_span;

use clippy_wrappers::span_lint_and_help;
use rustc_hir::{
    def::{DefKind, Res},
    Expr, ExprKind, Path, QPath, Ty,
};
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_span::{def_id::DefId, Span};

pub const LINT_MESSAGE: &str = "Avoid using DispatchError::Other for error codes.";

scout_audit_dylint_linting::impl_late_lint! {
    pub AVOID_DISPATCH_ERROR_OTHER,
    Warn,
    LINT_MESSAGE,
    AvoidDispatchErrorOther::default(),
    {
        name: "Avoid DispatchError::Other",
        long_message: "Avoid using DispatchError::Other for error codes, as it makes writing smart contracts more difficult.",
        severity: "Enhancement",
        help: "TODO",
        vulnerability_class: "TODO",
    }
}

#[derive(Default)]
pub struct AvoidDispatchErrorOther {}
impl AvoidDispatchErrorOther {
    pub fn new() -> Self {
        Self {}
    }
}

fn expr_to_call<'hir>(
    kind: &'hir ExprKind<'hir>,
) -> Result<(&'hir Expr<'hir>, &'hir [Expr<'hir>]), ()> {
    if let ExprKind::Call(a, b) = kind {
        Ok((a, b))
    } else {
        Err(())
    }
}

fn expr_to_path<'hir>(kind: &'hir ExprKind<'hir>) -> Result<QPath<'hir>, ()> {
    if let ExprKind::Path(a) = kind {
        Ok(*a)
    } else {
        Err(())
    }
}

fn path_to_resolved<'hir>(
    path: &'hir QPath<'hir>,
) -> Result<(&'hir Option<&'hir Ty<'hir>>, &'hir Path<'hir>), ()> {
    if let QPath::Resolved(a, b) = path {
        Ok((a, b))
    } else {
        Err(())
    }
}

fn resolution_to_def(resolution: &Res) -> Result<(&DefKind, &DefId), ()> {
    if let Res::Def(a, b) = resolution {
        Ok((a, b))
    } else {
        Err(())
    }
}

pub fn definition_to_string(
    cx: &rustc_lint::LateContext<'_>,
    did: rustc_hir::def_id::DefId,
) -> String {
    cx.get_def_path(did)
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

fn check_expr<'a>(cx: &rustc_lint::LateContext<'a>, expr: &'a ExprKind<'a>) -> Result<Span, ()> {
    let (function, _arguments) = expr_to_call(expr)?;
    let path = expr_to_path(&function.kind)?;
    let (_, path) = path_to_resolved(&path)?;
    let (_, def_id) = resolution_to_def(&path.res)?;
    let function_name = definition_to_string(cx, *def_id);

    if function_name == "sp_runtime::DispatchError::Other" {
        Ok(path.span)
    } else {
        Err(())
    }
}

impl<'tcx> LateLintPass<'tcx> for AvoidDispatchErrorOther {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
        if let Ok(span) = check_expr(cx, &expr.kind) {
            span_lint_and_help(
                cx,
                AVOID_DISPATCH_ERROR_OTHER,
                span,
                LINT_MESSAGE,
                None,
                "Instead, define an error enum with #[pallet::error] and return that.",
            );
        }
    }
}
