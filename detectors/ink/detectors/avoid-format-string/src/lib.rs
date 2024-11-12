#![feature(rustc_private)]


extern crate rustc_ast;
extern crate rustc_span;

use clippy_utils::sym;
use common::expose_lint_info;
use if_chain::if_chain;
use rustc_ast::{
    tokenstream::{TokenStream, TokenTree},
    AttrArgs, AttrKind, Expr, ExprKind, Item,
};
use rustc_lint::{EarlyContext, EarlyLintPass};
use rustc_span::{sym, Span};

const LINT_MESSAGE: &str = "The format! macro should not be used.";

#[expose_lint_info]
pub static AVOID_FORMAT_STRING_INFO: LintInfo = LintInfo {
    name: "Avoid format! macro",
    short_message: LINT_MESSAGE,
    long_message: "The format! macro is used to create a String from a given set of arguments. This macro is not recommended, it is better to use a custom error type enum.    ",
    severity: "Enhancement",
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/avoid-format-string",
    vulnerability_class: "Validations and error handling",
};

dylint_linting::impl_pre_expansion_lint! {
    pub AVOID_FORMAT_STRING,
    Warn,
    LINT_MESSAGE,
    AvoidFormatString::default()
}

#[derive(Default)]
pub struct AvoidFormatString {
    in_test_span: Option<Span>,
}

impl AvoidFormatString {
    fn in_test_item(&self) -> bool {
        self.in_test_span.is_some()
    }
}

impl EarlyLintPass for AvoidFormatString {
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

    fn check_expr(&mut self, cx: &EarlyContext, expr: &Expr) {
        if_chain! {
            if !self.in_test_item();
            if let ExprKind::MacCall(mac) = &expr.kind;
            if mac.path == sym!(format);

            then {
                clippy_wrappers::span_lint_and_help(
                    cx,
                    AVOID_FORMAT_STRING,
                    expr.span,
                    LINT_MESSAGE,
                    None,
                    "Instead, if this is returning an error, define a new error type",
                );
            }
        }
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
