#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_span;

mod processor;
pub use processor::process_findings;

use clippy_wrappers::span_lint_and_help;
use if_chain::if_chain;
use rustc_ast::{
    token::{Delimiter, Token, TokenKind},
    tokenstream::{TokenStream, TokenTree},
    AttrArgs, AttrKind, Attribute, Item, ItemKind,
};
use rustc_lint::{EarlyContext, EarlyLintPass};
use rustc_span::{Span, Symbol};
use std::collections::VecDeque;

const LINT_MESSAGE: &str = "This `#[scout_allow]` attribute may be unnecessary. Consider removing it if the lint is no longer triggered.";

dylint_linting::declare_pre_expansion_lint! {
    pub UNNECESSARY_LINT_ALLOW,
    Warn,
    LINT_MESSAGE,
    {
        name: "Unnecessary Lint Allow",
        long_message: "The `#[scout_allow]` attribute may be unnecessary. Consider removing it if the lint is no longer triggered.",
        severity: "Enhancement",
        help: "https://coinfabrik.github.io/scout-soroban/docs/detectors/unnecessary-lint-allow",
        vulnerability_class: "Code Quality",
    }
}

impl UnnecessaryLintAllow {
    fn extract_lint_names(&self, tokens: &TokenStream) -> Vec<String> {
        let mut lint_names = Vec::new();
        let mut stack = VecDeque::from([tokens]);

        while let Some(current_stream) = stack.pop_front() {
            for tree in current_stream.trees() {
                match tree {
                    TokenTree::Token(
                        Token {
                            kind: TokenKind::Ident(ident, _),
                            ..
                        },
                        _,
                    ) => {
                        lint_names.push(ident.to_string());
                    }
                    TokenTree::Delimited(_, _, Delimiter::Parenthesis, inner_stream) => {
                        stack.push_back(inner_stream);
                    }
                    _ => {}
                }
            }
        }

        lint_names
    }

    fn check_scout_allow_attrs(&self, cx: &EarlyContext<'_>, attrs: &[Attribute], span: Span) {
        for attr in attrs {
            if_chain! {
                if !attr.span.from_expansion();
                if attr.has_name(Symbol::intern("scout_allow"));
                if let AttrKind::Normal(item) = &attr.kind;
                if let AttrArgs::Delimited(delimited_args) = &item.item.args;
                then {
                    let lint_names = self.extract_lint_names(&delimited_args.tokens);
                    for lint_name in lint_names {
                        span_lint_and_help(
                            cx,
                            UNNECESSARY_LINT_ALLOW,
                            span,
                            LINT_MESSAGE,
                            None,
                            format!("The detector `{}` is no longer triggered. Consider removing the `#[scout_allow({})]` attribute if the lint is no longer triggered.", lint_name, lint_name)
                        );
                    }
                }
            }
        }
    }
}

impl EarlyLintPass for UnnecessaryLintAllow {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        self.check_scout_allow_attrs(cx, &item.attrs, item.span);

        if let ItemKind::Impl(impl_) = &item.kind {
            for impl_item in &impl_.items {
                self.check_scout_allow_attrs(cx, &impl_item.attrs, impl_item.span);
            }
        }
    }
}
