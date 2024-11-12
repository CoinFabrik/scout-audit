#![feature(rustc_private)]
#![feature(let_chains)]
extern crate rustc_ast;
extern crate rustc_span;

use common::expose_lint_info;
use if_chain::if_chain;
use rustc_ast::ast::GenericArgs;
use rustc_ast::{
    tokenstream::{TokenStream, TokenTree},
    AngleBracketedArgs, AttrArgs, AttrKind, Item, ItemKind, TyKind,
};
use rustc_lint::{EarlyContext, EarlyLintPass};
use rustc_span::Span;

const LINT_MESSAGE: &str = "Delegate call with non-lazy, non-mapping storage";

#[expose_lint_info]
pub static LAZY_DELEGATE_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "A bug in ink! causes delegated calls to not modify the caller's storage unless Lazy with ManualKey or Mapping is used.",
    severity: "Critical",
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/lazy-delegate",
    vulnerability_class: "Known Bugs",
};

dylint_linting::impl_pre_expansion_lint! {
    pub LAZY_DELEGATE,
    Warn,
    LINT_MESSAGE,
    LazyDelegate::default()
}

#[derive(Default)]
pub struct LazyDelegate {
    non_lazy_manual_storage_spans: Vec<Span>,
    delegate_uses: Vec<Span>,
}

impl EarlyLintPass for LazyDelegate {
    fn check_item(&mut self, _: &EarlyContext<'_>, item: &Item) {
        if is_storage_item(item)
            && let ItemKind::Struct(strt, _) = &item.kind
        {
            for field in strt.fields() {
                if let Some(_) = field.ident
                    && let TyKind::Path(_, path) = &field.ty.kind
                    && path.segments.len() == 1
                    && (path.segments[0].ident.name.to_string() == *"Lazy"
                        || path.segments[0].ident.name.to_string() == "Mapping")
                    && let Some(arg) = &path.segments[0].args
                    && let GenericArgs::AngleBracketed(AngleBracketedArgs { args, .. }) =
                        arg.clone().into_inner()
                    && args.len() > 1
                {
                } else if !self.non_lazy_manual_storage_spans.contains(&item.span) {
                    self.non_lazy_manual_storage_spans.push(item.span);
                }
            }
        }
    }

    fn check_ident(&mut self, cx: &EarlyContext<'_>, id: rustc_span::symbol::Ident) {
        if id.name.to_string() == "delegate" {
            self.delegate_uses.push(id.span);
        }

        if !self.delegate_uses.is_empty() && !self.non_lazy_manual_storage_spans.is_empty() {
            clippy_wrappers::span_lint_and_help(
                cx,
                LAZY_DELEGATE,
                id.span,
                LINT_MESSAGE,
                None,
                "Use lazy storage with manual keys",
            );

            for span in &self.non_lazy_manual_storage_spans {
                clippy_wrappers::span_lint_and_help(
                    cx,
                    LAZY_DELEGATE,
                    *span,
                    LINT_MESSAGE,
                    None,
                    "Use lazy storage with manual keys. \nMore info in https://github.com/paritytech/ink/issues/1826 and https://github.com/paritytech/ink/issues/1825",
                );
            }

            self.delegate_uses.clear();
        }
    }
}

fn is_storage_item(item: &Item) -> bool {
    item.attrs.iter().any(|attr| {
        if_chain!(
            if let AttrKind::Normal(normal) = &attr.kind;
            if let AttrArgs::Delimited(delim_args) = &normal.item.args;
            if is_storage_present(&delim_args.tokens);
            then {
                return true
            }
        );
        false
    })
}

fn is_storage_present(token_stream: &TokenStream) -> bool {
    token_stream.trees().any(|tree| match tree {
        TokenTree::Token(token, _) => {
            if let Some(ident) = token.ident() {
                ident.0.name.to_ident_string().contains("storage")
            } else {
                false
            }
        }
        TokenTree::Delimited(_, _, _, token_stream) => is_storage_present(token_stream),
    })
}
