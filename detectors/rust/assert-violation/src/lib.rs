#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_span;

use clippy_utils::{diagnostics::span_lint, sym};
use common::{
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use rustc_ast::{tokenstream::TokenTree, AttrArgs, AttrKind, Item, MacCall};
use rustc_lint::{EarlyContext, EarlyLintPass};
use rustc_span::{sym, Span};

const LINT_MESSAGE: &str = "Assert causes panic. Instead, return a proper error.";

#[expose_lint_info]
pub static ASSERT_VIOLATION_ERROR_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "Using assert! macro in production code can cause unexpected panics. \
                    This violates best practices for smart contract error handling.",
    severity: Severity::Enhancement,
    help: "https://coinfabrik.github.io/scout/docs/detectors/assert-violation",
    vulnerability_class: VulnerabilityClass::ErrorHandling,
};

dylint_linting::impl_pre_expansion_lint! {
    pub ASSERT_VIOLATION,
    Warn,
    LINT_MESSAGE,
    AssertViolation::default()
}

#[derive(Default)]
pub struct AssertViolation {
    test_spans: Vec<Span>,
}

impl AssertViolation {
    fn is_within_test(&self, span: Span) -> bool {
        self.test_spans
            .iter()
            .any(|test_span| test_span.contains(span))
    }

    fn is_test_token_present(args: &AttrArgs) -> bool {
        matches!(args, AttrArgs::Delimited(delim_args) if delim_args
            .tokens
            .trees()
            .any(|tree| matches!(tree, TokenTree::Token(token, _) if token.is_ident_named(sym::test))))
    }

    fn is_test_item(item: &Item) -> bool {
        item.attrs.iter().any(|attr| {
            attr.has_name(sym::test)
                || (attr.has_name(sym::cfg)
                    && attr.meta_item_list().map_or(false, |list| {
                        list.iter().any(|item| item.has_name(sym::test))
                    }))
                || matches!(
                    &attr.kind,
                    AttrKind::Normal(normal) if Self::is_test_token_present(&normal.item.args)
                )
        })
    }

    fn is_assert_macro(mac: &MacCall) -> bool {
        mac.path == sym!(assert)
            || mac.path == sym!(assert_eq)
            || mac.path == sym!(assert_ne)
            || mac.path == sym!(debug_assert)
            || mac.path == sym!(debug_assert_eq)
            || mac.path == sym!(debug_assert_ne)
    }
}

impl EarlyLintPass for AssertViolation {
    fn check_item(&mut self, _: &EarlyContext<'_>, item: &rustc_ast::Item) {
        if Self::is_test_item(item) {
            self.test_spans.push(item.span);
        }
    }

    fn check_mac(&mut self, cx: &EarlyContext<'_>, mac: &MacCall) {
        if !Self::is_assert_macro(mac) {
            return;
        }

        // Early return if within a test function
        if self.is_within_test(mac.span()) {
            return;
        }

        span_lint(cx, ASSERT_VIOLATION, mac.span(), LINT_MESSAGE);
    }
}
