#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_hir;
extern crate rustc_span;

use clippy_utils::diagnostics::span_lint_and_help;
use common::{
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use rustc_hir::def_id::LocalDefId;
use rustc_hir::intravisit::Visitor;
use rustc_hir::intravisit::{walk_expr, FnKind};
use rustc_hir::{Body, FnDecl};
use rustc_hir::{Expr, ExprKind};
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
    help: LINT_HELP,
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
        struct IncorrectExponentiationStorage {
            span: Vec<Span>,
            incorrect_exponentiation: bool,
        }

        impl<'tcx> Visitor<'tcx> for IncorrectExponentiationStorage {
            fn visit_expr(&mut self, expr: &'tcx Expr<'_>) {
                if let ExprKind::AssignOp(binop, _, _) = &expr.kind {
                    if binop.node == rustc_hir::BinOpKind::BitXor {
                        self.incorrect_exponentiation = true;
                        self.span.push(expr.span);
                    }
                }

                if let ExprKind::Binary(op, _, _) = &expr.kind {
                    if op.node == rustc_hir::BinOpKind::BitXor {
                        self.incorrect_exponentiation = true;
                        self.span.push(expr.span);
                    }
                }

                walk_expr(self, expr);
            }
        }

        let mut expon_storage = IncorrectExponentiationStorage {
            span: Vec::new(),
            incorrect_exponentiation: false,
        };

        walk_expr(&mut expon_storage, body.value);

        if expon_storage.incorrect_exponentiation {
            for span in expon_storage.span.iter() {
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
}
