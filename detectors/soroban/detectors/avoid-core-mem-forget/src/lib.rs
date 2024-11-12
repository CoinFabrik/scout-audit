#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_span;

use clippy_wrappers::span_lint_and_help;
use common::macros::expose_lint_info;
use if_chain::if_chain;
use rustc_ast::{Expr, ExprKind, Item, NodeId};
use rustc_lint::{EarlyContext, EarlyLintPass};
use rustc_span::sym;

const LINT_MESSAGE: &str = "Use the `let _ = ...` pattern or `.drop()` method to forget the value";

#[expose_lint_info]
pub static AVOID_CORE_MEM_FORGET_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "The core::mem::forget function is used to forget about a value without running its destructor. This could lead to memory leaks and logic errors.",
    severity: "Enhancement",
    help: "https://coinfabrik.github.io/scout-soroban/docs/detectors/avoid-core-mem-forget",
    vulnerability_class: "Best practices",
};

dylint_linting::impl_pre_expansion_lint! {
    pub AVOID_CORE_MEM_FORGET,
    Warn,
    LINT_MESSAGE,
    AvoidCoreMemForget::default()
}

#[derive(Default)]
pub struct AvoidCoreMemForget {
    stack: Vec<NodeId>,
}

impl EarlyLintPass for AvoidCoreMemForget {
    fn check_item(&mut self, _cx: &EarlyContext, item: &Item) {
        if self.in_test_item() || is_test_item(item) {
            self.stack.push(item.id);
        }
    }

    fn check_expr(&mut self, cx: &EarlyContext, expr: &Expr) {
        if_chain! {
            if !self.in_test_item();
            if let ExprKind::Call(a, _) = &expr.kind;
            if let ExprKind::Path(_, path) = &a.kind;
            if path.segments.len() == 3;
            if path.segments[0].ident.name.to_string() == "core";
            if path.segments[1].ident.name.to_string() == "mem";
            if path.segments[2].ident.name.to_string() == "forget";
            then {
                span_lint_and_help(
                    cx,
                    AVOID_CORE_MEM_FORGET,
                    expr.span,
                    LINT_MESSAGE,
                    None,
                    "Instead, use the `let _ = ...` pattern or `.drop` method to forget the value."
                );
            }
        }
    }
}

fn is_test_item(item: &Item) -> bool {
    item.attrs.iter().any(|attr| {
        if attr.has_name(sym::test) {
            true
        } else {
            if_chain! {
                if attr.has_name(sym::cfg);
                if let Some(items) = attr.meta_item_list();
                if let [item] = items.as_slice();
                if let Some(feature_item) = item.meta_item();
                if feature_item.has_name(sym::test);
                then {
                    true
                } else {
                    false
                }
            }
        }
    })
}

impl AvoidCoreMemForget {
    fn in_test_item(&self) -> bool {
        !self.stack.is_empty()
    }
}
