#![feature(rustc_private)]

extern crate rustc_hir;
extern crate rustc_span;

use clippy_wrappers::span_lint_and_help;
use if_chain::if_chain;
use rustc_hir::{BinOpKind, Expr, ExprKind};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::Symbol;

const LINT_MESSAGE: &str = "Use env.prng() to generate random numbers, and remember that all random numbers are under the control of validators";

dylint_linting::declare_late_lint! {
    /// ### What it does
    /// This detector prevents the usage of timestamp/sequence number and modulo operator as a random number source.
    ///
    /// ### Why is this bad?
    /// The value of the block timestamp and block sequence can be manipulated by validators, which means they're not a secure source of randomness. Therefore, they shouldn't be used for generating random numbers, especially in the context of a betting contract where the outcomes of bets could be manipulated.
    ///
    /// ### Example
    /// ```rust
    ///  let pseudo_random = env.ledger().timestamp() % max_val;
    /// ```
    ///
    pub INSUFFICIENTLY_RANDOM_VALUES,
    Warn,
    LINT_MESSAGE,
    {
        name: "Insufficiently Random Values",
        long_message: "Use env.prng() to generate random numbers, and remember that all random numbers are under the control of validators.",
        severity: "Critical",
        help: "https://coinfabrik.github.io/scout-soroban/docs/detectors/insufficiently-random-values",
        vulnerability_class: "Block attributes",
    }
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
