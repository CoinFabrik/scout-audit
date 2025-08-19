#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_span;

use clippy_utils::diagnostics::span_lint_and_help;
use common::{
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use rustc_ast::{tokenstream::TokenTree, AttrArgs, AttrKind, Item, MacCall};
use rustc_lint::{EarlyContext, EarlyLintPass};
use rustc_span::{sym, Span};

const LINT_MESSAGE: &str = "The panic! macro is used in a function that returns Result. \
    Consider using the ? operator or return Err() instead.";
const HELP_MESSAGE: &str =
    "Consider using '?' to propagate errors or 'return Err()' to return early with an error";

#[expose_lint_info]
pub static AVOID_PANIC_ERROR_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message:
        "Using panic! in functions that return Result defeats the purpose of error handling. \
        Consider propagating the error using ? or return Err() instead.",
    severity: Severity::Enhancement,
    help: "https://coinfabrik.github.io/scout-audit/docs/detectors/substrate/avoid-panic-error",
    vulnerability_class: VulnerabilityClass::ErrorHandling,
};

dylint_linting::impl_pre_expansion_lint! {
    pub AVOID_PANIC_ERROR,
    Warn,
    LINT_MESSAGE,
    AvoidPanicError::default()
}

#[derive(Default)]
pub struct AvoidPanicError {
    test_spans: Vec<Span>,
}

impl AvoidPanicError {
    fn is_within_test(&self, span: Span) -> bool {
        self.test_spans
            .iter()
            .any(|test_span| test_span.contains(span))
    }

    fn is_test_token_present(args: &AttrArgs) -> bool {
        matches!(args, AttrArgs::Delimited(delim_args) if delim_args
            .tokens
            .iter()
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
}
impl EarlyLintPass for AvoidPanicError {
    fn check_item(&mut self, _: &EarlyContext<'_>, item: &rustc_ast::Item) {
        if Self::is_test_item(item) {
            self.test_spans.push(item.span);
        }
    }

    fn check_mac(&mut self, cx: &EarlyContext<'_>, mac: &MacCall) {
        if mac.path != sym::panic {
            return;
        }

        // Early return if within a test function
        if self.is_within_test(mac.span()) {
            return;
        }

        span_lint_and_help(
            cx,
            AVOID_PANIC_ERROR,
            mac.span(),
            LINT_MESSAGE,
            None,
            HELP_MESSAGE,
        );
    }
}
