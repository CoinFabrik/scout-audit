#![feature(rustc_private)]

extern crate rustc_hir;

use common::expose_lint_info;
use if_chain::if_chain;
use rustc_hir::{BinOpKind, Expr, ExprKind};
use rustc_lint::{LateContext, LateLintPass};

const LINT_MESSAGE: &str = "In order to prevent randomness manipulations by validators block_timestamp should not be used as random number source";

#[expose_lint_info]
pub static INSUFFICIENTLY_RANDOM_VALUES_INFO: LintInfo = LintInfo {
    name: "Insufficiently Random Values",
    short_message: LINT_MESSAGE,
    long_message: "Using block attributes like block_timestamp or block_number for random number generation in ink! Substrate smart contracts is not recommended due to the predictability of these values. Block attributes are publicly visible and deterministic, making it easy for malicious actors to anticipate their values and manipulate outcomes to their advantage.",
    severity: "Critical",
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/insufficiently-random-values",
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
            if path.ident.as_str() == "block_timestamp" ||
            path.ident.as_str() == "block_number";
            then {
                clippy_wrappers::span_lint_and_help(
                    cx,
                    INSUFFICIENTLY_RANDOM_VALUES,
                    expr.span,
                    LINT_MESSAGE,
                    None,
                    "This expression seems to use block_timestamp as a pseudo random number",
                );
            }
        }
    }
}
