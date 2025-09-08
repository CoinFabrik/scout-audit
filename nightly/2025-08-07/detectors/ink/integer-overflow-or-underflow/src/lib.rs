#![feature(rustc_private)]

extern crate rustc_hir;
extern crate rustc_span;

use clippy_utils::{
    consts::{ConstEvalCtxt, Constant},
    is_integer_literal,
};
use common::{
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use rustc_hir::{self as hir, Body, Expr, ExprKind, UnOp};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::Span;

pub const LINT_MESSAGE: &str = "Potential for integer arithmetic overflow/underflow. Consider checked, wrapping or saturating arithmetic.";

#[expose_lint_info]
pub static INTEGER_OVERFLOW_OR_UNDERFLOW_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "An overflow/underflow is typically caught and generates an error. When it is not caught, the operation will result in an inexact result which could lead to serious problems.\n In Ink! 5.0.0, using raw math operations will result in `cargo contract build` failing with an error message.",
    severity: Severity::Critical,
    help: "https://coinfabrik.github.io/scout-audit/docs/detectors/ink/integer-overflow-or-underflow",
    vulnerability_class: VulnerabilityClass::Arithmetic,
};

dylint_linting::impl_late_lint! {
    pub INTEGER_OVERFLOW_OR_UNDERFLOW,
    Warn,
    LINT_MESSAGE,
    IntegerOverflowOrUnderflow::default()
}

#[derive(Default)]
pub struct IntegerOverflowOrUnderflow {
    arithmetic_context: ArithmeticContext,
}
impl IntegerOverflowOrUnderflow {
    pub fn new() -> Self {
        Self {
            arithmetic_context: ArithmeticContext::default(),
        }
    }
}

impl<'tcx> LateLintPass<'tcx> for IntegerOverflowOrUnderflow {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, e: &'tcx Expr<'_>) {
        match e.kind {
            ExprKind::Binary(op, lhs, rhs) => {
                self.arithmetic_context
                    .check_binary(cx, e, op.node, lhs, rhs);
            }
            ExprKind::AssignOp(op, lhs, rhs) => {
                self.arithmetic_context
                    .check_assignment(cx, e, op.node, lhs, rhs);
            }
            ExprKind::Unary(op, arg) => {
                if op == UnOp::Neg {
                    self.arithmetic_context.check_negate(cx, e, arg);
                }
            }
            _ => (),
        }
    }

    fn check_expr_post(&mut self, _: &LateContext<'_>, e: &Expr<'_>) {
        self.arithmetic_context.expr_post(e.hir_id);
    }

    fn check_body(&mut self, cx: &LateContext<'tcx>, b: &Body<'tcx>) {
        self.arithmetic_context.enter_body(cx, b);
    }

    fn check_body_post(&mut self, cx: &LateContext<'tcx>, b: &rustc_hir::Body<'_>) {
        self.arithmetic_context.body_post(cx, b);
    }
}

/// Attempts to evaluate an expression only if its value is not dependent on other items.
pub fn constant_simple<'tcx>(lcx: &LateContext<'tcx>, e: &Expr<'_>) -> Option<Constant<'tcx>> {
    ConstEvalCtxt::new(lcx).eval_simple(e)
}

#[derive(Default)]
pub struct ArithmeticContext {
    expr_id: Option<hir::HirId>,
    /// This field is used to check whether expressions are constants, such as in enum discriminants
    /// and consts
    const_span: Option<Span>,
}
impl ArithmeticContext {
    fn skip_expr(&mut self, e: &hir::Expr<'_>) -> bool {
        self.expr_id.is_some() || self.const_span.is_some_and(|span| span.contains(e.span))
    }

    pub fn check_binary<'tcx>(
        &mut self,
        cx: &LateContext<'tcx>,
        expr: &'tcx hir::Expr<'_>,
        op: hir::BinOpKind,
        l: &'tcx hir::Expr<'_>,
        r: &'tcx hir::Expr<'_>,
    ) {
        if self.skip_expr(expr) {
            return;
        }
        match op {
            hir::BinOpKind::And
            | hir::BinOpKind::Or
            | hir::BinOpKind::BitAnd
            | hir::BinOpKind::BitOr
            | hir::BinOpKind::BitXor
            | hir::BinOpKind::Eq
            | hir::BinOpKind::Lt
            | hir::BinOpKind::Le
            | hir::BinOpKind::Ne
            | hir::BinOpKind::Ge
            | hir::BinOpKind::Gt => return,
            _ => (),
        }

        let (l_ty, r_ty) = (
            cx.typeck_results().expr_ty(l),
            cx.typeck_results().expr_ty(r),
        );
        if l_ty.peel_refs().is_integral() && r_ty.peel_refs().is_integral() {
            match op {
                hir::BinOpKind::Div | hir::BinOpKind::Rem => match &r.kind {
                    hir::ExprKind::Lit(_lit) => (),
                    hir::ExprKind::Unary(hir::UnOp::Neg, expr) => {
                        if is_integer_literal(expr, 1) {
                            clippy_utils::diagnostics::span_lint_and_help(
                                cx,
                                INTEGER_OVERFLOW_OR_UNDERFLOW,
                                expr.span,
                                LINT_MESSAGE,
                                None,
                                "Potential for integer arithmetic overflow/underflow in unary operation with negative expression. Consider checked, wrapping or saturating arithmetic."
                            );
                            self.expr_id = Some(expr.hir_id);
                        }
                    }
                    _ => {
                        clippy_utils::diagnostics::span_lint_and_help(
                            cx,
                            INTEGER_OVERFLOW_OR_UNDERFLOW,
                            expr.span,
                            LINT_MESSAGE,
                            None,
                            format!("Potential for integer arithmetic overflow/underflow in operation '{}'. Consider checked, wrapping or saturating arithmetic.", op.as_str()),
                        );
                        self.expr_id = Some(expr.hir_id);
                    }
                },
                _ => {
                    clippy_utils::diagnostics::span_lint_and_help(
                        cx,
                        INTEGER_OVERFLOW_OR_UNDERFLOW,
                        expr.span,
                        LINT_MESSAGE,
                        None,
                        format!("Potential for integer arithmetic overflow/underflow in operation '{}'. Consider checked, wrapping or saturating arithmetic.", op.as_str()),
                    );
                    self.expr_id = Some(expr.hir_id);
                }
            }
        }
    }

    pub fn check_assignment<'tcx>(
        &mut self,
        cx: &LateContext<'tcx>,
        expr: &'tcx hir::Expr<'_>,
        op: hir::AssignOpKind,
        l: &'tcx hir::Expr<'_>,
        r: &'tcx hir::Expr<'_>,
    ) {
        if self.skip_expr(expr) {
            return;
        }

        let (l_ty, r_ty) = (
            cx.typeck_results().expr_ty(l),
            cx.typeck_results().expr_ty(r),
        );
        if l_ty.peel_refs().is_integral() && r_ty.peel_refs().is_integral() {
            match op {
                hir::AssignOpKind::DivAssign | hir::AssignOpKind::RemAssign => match &r.kind {
                    hir::ExprKind::Lit(_lit) => (),
                    hir::ExprKind::Unary(hir::UnOp::Neg, expr) => {
                        if is_integer_literal(expr, 1) {
                            clippy_utils::diagnostics::span_lint_and_help(
                                cx,
                                INTEGER_OVERFLOW_OR_UNDERFLOW,
                                expr.span,
                                LINT_MESSAGE,
                                None,
                                "Potential for integer arithmetic overflow/underflow in unary operation with negative expression. Consider checked, wrapping or saturating arithmetic."
                            );
                            self.expr_id = Some(expr.hir_id);
                        }
                    }
                    _ => {
                        clippy_utils::diagnostics::span_lint_and_help(
                            cx,
                            INTEGER_OVERFLOW_OR_UNDERFLOW,
                            expr.span,
                            LINT_MESSAGE,
                            None,
                            format!("Potential for integer arithmetic overflow/underflow in operation '{}'. Consider checked, wrapping or saturating arithmetic.", op.as_str()),
                        );
                        self.expr_id = Some(expr.hir_id);
                    }
                },
                _ => {
                    clippy_utils::diagnostics::span_lint_and_help(
                        cx,
                        INTEGER_OVERFLOW_OR_UNDERFLOW,
                        expr.span,
                        LINT_MESSAGE,
                        None,
                        format!("Potential for integer arithmetic overflow/underflow in operation '{}'. Consider checked, wrapping or saturating arithmetic.", op.as_str()),
                    );
                    self.expr_id = Some(expr.hir_id);
                }
            }
        }
    }

    pub fn check_negate<'tcx>(
        &mut self,
        cx: &LateContext<'tcx>,
        expr: &'tcx hir::Expr<'_>,
        arg: &'tcx hir::Expr<'_>,
    ) {
        if self.skip_expr(expr) {
            return;
        }
        let ty = cx.typeck_results().expr_ty(arg);
        if constant_simple(cx, expr).is_none() && ty.is_integral() {
            clippy_utils::diagnostics::span_lint_and_help(
                cx,
                INTEGER_OVERFLOW_OR_UNDERFLOW,
                expr.span,
                LINT_MESSAGE,
                None,
                "Potential for integer arithmetic overflow/underflow. Consider checked, wrapping or saturating arithmetic.",
            );
            self.expr_id = Some(expr.hir_id);
        }
    }

    pub fn expr_post(&mut self, id: hir::HirId) {
        if Some(id) == self.expr_id {
            self.expr_id = None;
        }
    }

    pub fn enter_body(&mut self, cx: &LateContext<'_>, body: &hir::Body<'_>) {
        let body_owner = cx.tcx.hir_body_owner(body.id());
        let body_owner_def_id = cx.tcx.hir_body_owner_def_id(body.id());

        match cx.tcx.hir_body_owner_kind(body_owner_def_id) {
            hir::BodyOwnerKind::Static(_) | hir::BodyOwnerKind::Const { .. } => {
                let body_span = cx.tcx.hir_span_with_body(body_owner);

                if let Some(span) = self.const_span {
                    if span.contains(body_span) {
                        return;
                    }
                }
                self.const_span = Some(body_span);
            }
            _ => (),
        }
    }

    pub fn body_post(&mut self, cx: &LateContext<'_>, body: &hir::Body<'_>) {
        let body_owner = cx.tcx.hir_body_owner(body.id());
        let body_span = cx.tcx.hir_span_with_body(body_owner);

        if let Some(span) = self.const_span {
            if span.contains(body_span) {
                return;
            }
        }
        self.const_span = None;
    }
}
