#![feature(rustc_private)]
#![feature(let_chains)]

extern crate rustc_ast;
extern crate rustc_span;

use clippy_wrappers::span_lint;
use common::expose_lint_info;
use rustc_ast::{BlockCheckMode, Expr, ExprKind, UnsafeSource};
use rustc_lint::{EarlyContext, EarlyLintPass};

const LINT_MESSAGE: &str = "Avoid using unsafe blocks as it may lead to undefined behavior.";

#[expose_lint_info]
pub static AVOID_UNSAFE_BLOCK_INFO: LintInfo = LintInfo {
    name: "Avoid unsafe block",
    short_message: LINT_MESSAGE,
    long_message: "The unsafe block is used to bypass Rust's safety checks. It is recommended to avoid using unsafe blocks as much as possible, and to use them only when necessary.",
    severity: "Enhancement",
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/avoid-unsafe-block",
    vulnerability_class: "Best practices",
};

dylint_linting::declare_pre_expansion_lint! {
    pub AVOID_UNSAFE_BLOCK,
    Warn,
    LINT_MESSAGE
}

impl EarlyLintPass for AvoidUnsafeBlock {
    fn check_expr(&mut self, cx: &EarlyContext, expr: &Expr) {
        if let ExprKind::Block(block, ..) = &expr.kind
            && block.rules == BlockCheckMode::Unsafe(UnsafeSource::UserProvided)
        {
            span_lint(cx, AVOID_UNSAFE_BLOCK, expr.span, LINT_MESSAGE)
        }
    }
}
