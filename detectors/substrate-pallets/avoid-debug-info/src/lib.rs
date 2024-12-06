#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_span;

use clippy_utils::{diagnostics::span_lint_and_help, sym};
use common::{
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use rustc_ast::{tokenstream::TokenTree, AttrArgs, AttrKind, Item, MacCall};
use rustc_lint::{EarlyContext, EarlyLintPass};
use rustc_span::{sym, Span};

const LINT_MESSAGE: &str = "The debug! or info! macros are used in the code. \
    Consider emitting events instead, or removing them.";
const HELP_MESSAGE: &str =
    "Consider using emit! to emit events, or removing the debug! and info! macros.";

#[expose_lint_info]
pub static AVOID_DEBUG_INFO_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message:
    "The use of debugging macros, such as debug!() and info!(), has been detected in the contract code. \
    While useful during development and testing, these macros are not recommended for production. \
    Instead, consider emitting structured events with emit! to log relevant data more efficiently \
    and reduce unnecessary gas costs.",
    severity: Severity::Minor,
    help: "https://coinfabrik.github.io/scout/docs/detectors/avoid-debug-info",
    vulnerability_class: VulnerabilityClass::BestPractices,
};

dylint_linting::impl_pre_expansion_lint! {
    pub AVOID_DEBUG_INFO,
    Warn,
    LINT_MESSAGE,
    AvoidDebugInfo::default()
}

#[derive(Default)]
pub struct AvoidDebugInfo {
    test_spans: Vec<Span>,
}

impl AvoidDebugInfo {
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
}

impl EarlyLintPass for AvoidDebugInfo {
    fn check_item(&mut self, _: &EarlyContext<'_>, item: &rustc_ast::Item) {
        if Self::is_test_item(item) {
            self.test_spans.push(item.span);
        }
    }
    fn check_mac(&mut self, cx: &EarlyContext<'_>, mac: &MacCall) {
        if (mac.path != sym::debug) && (mac.path != sym!(info)) {
            return;
        }

        // Early return if within a test function
        if self.is_within_test(mac.span()) {
            return;
        }

        span_lint_and_help(
            cx,
            AVOID_DEBUG_INFO,
            mac.span(),
            LINT_MESSAGE,
            None,
            HELP_MESSAGE,
        );
    }
}
