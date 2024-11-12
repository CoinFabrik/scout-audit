#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_span;

use clippy_utils::sym;
use clippy_wrappers::span_lint_and_help;
use common::{declarations::{Severity, VulnerabilityClass}, macros::expose_lint_info};
use if_chain::if_chain;
use rustc_ast::{
    ptr::P,
    tokenstream::{TokenStream, TokenTree},
    AttrArgs, AttrKind, Expr, ExprKind, Item, MacCall, Stmt, StmtKind,
};
use rustc_lint::{EarlyContext, EarlyLintPass};
use rustc_span::{sym, Span};

const LINT_MESSAGE: &str = "Assert causes panic. Instead, return a proper error.";

#[expose_lint_info]
pub static ASSERT_VIOLATION_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "Assert causes panic. Instead, return a proper error.",
    severity: Severity::Medium,
    help: "https://coinfabrik.github.io/scout-soroban/docs/detectors/assert-violation",
    vulnerability_class: VulnerabilityClass::Panic,
};

dylint_linting::impl_pre_expansion_lint! {
    pub ASSERT_VIOLATION,
    Warn,
    LINT_MESSAGE,
    AssertViolation::default()
}

#[derive(Default)]
struct AssertViolation {
    in_test_span: Option<Span>,
}

impl AssertViolation {
    fn in_test_item(&self) -> bool {
        self.in_test_span.is_some()
    }
}

impl EarlyLintPass for AssertViolation {
    fn check_item(&mut self, _cx: &EarlyContext, item: &Item) {
        match (is_test_item(item), self.in_test_span) {
            (true, None) => self.in_test_span = Some(item.span),
            (true, Some(test_span)) => {
                if !test_span.contains(item.span) {
                    self.in_test_span = Some(item.span);
                }
            }
            (false, None) => {}
            (false, Some(test_span)) => {
                if !test_span.contains(item.span) {
                    self.in_test_span = None;
                }
            }
        };
    }

    fn check_stmt(&mut self, cx: &EarlyContext, stmt: &Stmt) {
        if self.in_test_item() {
            return;
        }

        if let StmtKind::MacCall(mac) = &stmt.kind {
            check_macro_call(cx, stmt.span, &mac.mac)
        }
    }
    fn check_expr(&mut self, cx: &EarlyContext, expr: &Expr) {
        if self.in_test_item() {
            return;
        }

        if let ExprKind::MacCall(mac) = &expr.kind {
            check_macro_call(cx, expr.span, mac)
        }
    }
}

fn check_macro_call(cx: &EarlyContext, span: Span, mac: &P<MacCall>) {
    if [
        sym!(assert),
        sym!(assert_eq),
        sym!(assert_ne),
        sym!(debug_assert),
        sym!(debug_assert_eq),
        sym!(debug_assert_ne),
    ]
    .iter()
    .any(|sym| &mac.path == sym)
    {
        span_lint_and_help(
            cx,
            ASSERT_VIOLATION,
            span,
            LINT_MESSAGE,
            None,
            "You could use instead an Error enum.",
        );
    }
}

fn is_test_item(item: &Item) -> bool {
    item.attrs.iter().any(|attr| {
        // Find #[cfg(all(test, feature = "e2e-tests"))]
        if_chain!(
            if let AttrKind::Normal(normal) = &attr.kind;
            if let AttrArgs::Delimited(delim_args) = &normal.item.args;
            if is_test_token_present(&delim_args.tokens);
            then {
                return true;
            }
        );

        // Find unit or integration tests
        if attr.has_name(sym::test) {
            return true;
        }

        if_chain! {
            if attr.has_name(sym::cfg);
            if let Some(items) = attr.meta_item_list();
            if let [item] = items.as_slice();
            if let Some(feature_item) = item.meta_item();
            if feature_item.has_name(sym::test);
            then {
                return true;
            }
        }

        false
    })
}

fn is_test_token_present(token_stream: &TokenStream) -> bool {
    token_stream.trees().any(|tree| match tree {
        TokenTree::Token(token, _) => token.is_ident_named(sym::test),
        TokenTree::Delimited(_, _, _, token_stream) => is_test_token_present(token_stream),
    })
}
