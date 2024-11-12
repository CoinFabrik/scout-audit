#![feature(rustc_private)]

extern crate rustc_hir;
extern crate rustc_span;

use clippy_wrappers::span_lint_and_help;
use common::macros::expose_lint_info;
use if_chain::if_chain;
use rustc_hir::{BinOpKind, Expr, ExprKind};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::Symbol;

const LINT_MESSAGE: &str = "Use env.prng() to generate random numbers, and remember that all random numbers are under the control of validators";

#[expose_lint_info]
pub static INSUFFICIENTLY_RANDOM_VALUES_INFO: LintInfo = LintInfo {
    name: "Insufficiently Random Values",
    short_message: LINT_MESSAGE,
    long_message: LINT_MESSAGE,
    severity: "Critical",
    help: "https://coinfabrik.github.io/scout-soroban/docs/detectors/insufficiently-random-values",
    vulnerability_class: "Block attributes",
};

dylint_linting::declare_late_lint! {
    pub INSUFFICIENTLY_RANDOM_VALUES,
    Warn,
    LINT_MESSAGE
}

impl<'tcx> LateLintPass<'tcx> for InsufficientlyRandomValues {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'_>) {
        if_chain! {
            if let ExprKind::Binary(op, lexp, _rexp) = expr.kind;
            if op.node == BinOpKind::Rem;
            if let ExprKind::MethodCall(path, _, _, _) = lexp.kind;
            if path.ident.name == Symbol::intern("timestamp") ||
                path.ident.name == Symbol::intern("sequence");
            then {
                span_lint_and_help(
                    cx,
                    INSUFFICIENTLY_RANDOM_VALUES,
                    expr.span,
                    LINT_MESSAGE,
                    None,
                    format!("This expression seems to use ledger().{}() as a pseudo random number",path.ident.as_str()),
                );
            }
        }
    }
}
