#![feature(rustc_private)]

extern crate rustc_hir;
extern crate rustc_span;

use clippy_utils::diagnostics::span_lint_and_help;
use common::{
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use rustc_hir::{
    def_id::LocalDefId,
    intravisit::{walk_expr, FnKind, Visitor},
    Body, Expr, ExprKind, FnDecl,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::Span;

const LINT_MESSAGE: &str = "'^' It is not an exponential operator. It is a bitwise XOR.";
const LINT_HELP: &str = "If you want to use XOR, use bitxor(). If you want to raise a number use .checked_pow() or .pow() ";

#[expose_lint_info]
pub static INCORRECT_EXPONENTIATION_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: LINT_MESSAGE,
    severity: Severity::Critical,
    help: "https://coinfabrik.github.io/scout-audit/docs/detectors/rust/incorrect-exponentiation",
    vulnerability_class: VulnerabilityClass::Arithmetic,
};

dylint_linting::declare_late_lint! {
    pub INCORRECT_EXPONENTIATION,
    Warn,
    LINT_MESSAGE
}

impl<'tcx> LateLintPass<'tcx> for IncorrectExponentiation {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: FnKind<'tcx>,
        _: &'tcx FnDecl<'_>,
        body: &'tcx Body<'_>,
        _: Span,
        _: LocalDefId,
    ) {
        struct IncorrectExponentiationVisitor {
            span: Vec<Span>,
        }

        impl<'tcx> Visitor<'tcx> for IncorrectExponentiationVisitor {
            fn visit_expr(&mut self, expr: &'tcx Expr<'_>) {
                if let ExprKind::AssignOp(binop, _, _) = &expr.kind {
                    if binop.node == rustc_hir::BinOpKind::BitXor {
                        self.span.push(expr.span);
                    }
                }

                if let ExprKind::Binary(op, _, _) = &expr.kind {
                    if op.node == rustc_hir::BinOpKind::BitXor {
                        self.span.push(expr.span);
                    }
                }

                walk_expr(self, expr);
            }
        }

        let mut exponentiation_visitor = IncorrectExponentiationVisitor { span: Vec::new() };

        walk_expr(&mut exponentiation_visitor, body.value);

        for span in exponentiation_visitor.span.iter() {
            span_lint_and_help(
                cx,
                INCORRECT_EXPONENTIATION,
                *span,
                LINT_MESSAGE,
                None,
                LINT_HELP,
            );
        }
    }
}
