#![feature(rustc_private)]

extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_span;

use clippy_wrappers::span_lint;
use common::expose_lint_info;
use edit_distance::edit_distance;
use if_chain::if_chain;
use rustc_errors::MultiSpan;
use rustc_hir::{intravisit::FnKind, Body, FnDecl, Item, ItemKind, Node};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::{def_id::DefId, def_id::LocalDefId, Span};
use std::{
    collections::HashSet,
    ops::{Div, Mul},
    vec,
};

const LINT_MESSAGE: &str =
    "This contract seems like a Token, consider implementing the Token Interface trait";
const CANONICAL_FUNCTIONS_AMOUNT: u16 = 10;
const INCLUDED_FUNCTIONS_THRESHOLD: u16 = 60;
const TOKEN_INTERFACE_PATH: &str = "soroban_sdk::token::TokenInterface";

#[expose_lint_info]
pub static TOKEN_INTERFACE_INFERENCE_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "Implementing the Token Interface trait helps to ensure proper compliance of the SEP-41 standard.",
    severity: "Enhancement",
    help: "https://coinfabrik.github.io/scout-soroban/docs/detectors/token-interface-inference",
    vulnerability_class: "Best Practices",
};

dylint_linting::impl_late_lint! {
    pub TOKEN_INTERFACE_INFERENCE,
    Warn,
    LINT_MESSAGE,
    TokenInterfaceInference::default()
}

#[derive(Default)]
struct TokenInterfaceInference {
    canonical_funcs_def_id: HashSet<DefId>,
    impl_token_interface_trait: bool,
    detected_canonical_functions_count: u16,
    funcs_spans: Vec<Span>,
}

impl<'tcx> LateLintPass<'tcx> for TokenInterfaceInference {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx Item<'tcx>) {
        if_chain! {
            if let ItemKind::Impl(impl_block) = item.kind;
            if let Some(trait_ref) = impl_block.of_trait;
            if let Some(trait_def_id) = trait_ref.path.res.opt_def_id();
            if cx.tcx.def_path_str(trait_def_id) == TOKEN_INTERFACE_PATH;
            then {
                self.impl_token_interface_trait = true;
            }
        }
    }

    fn check_crate_post(&mut self, cx: &LateContext<'tcx>) {
        // Verify if the contract implements the token interface.
        if self.impl_token_interface_trait {
            return;
        }

        if self
            .detected_canonical_functions_count
            .mul(100)
            .div(CANONICAL_FUNCTIONS_AMOUNT)
            >= INCLUDED_FUNCTIONS_THRESHOLD
        {
            span_lint(
                cx,
                TOKEN_INTERFACE_INFERENCE,
                MultiSpan::from_spans(self.funcs_spans.clone()),
                LINT_MESSAGE,
            );
        }
    }

    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: FnKind<'tcx>,
        _: &'tcx FnDecl<'tcx>,
        _: &'tcx Body<'tcx>,
        span: Span,
        local_def_id: LocalDefId,
    ) {
        if span.from_expansion() {
            return;
        }

        let def_id = local_def_id.to_def_id();
        let fn_name = cx.tcx.def_path_str(def_id);
        let fn_name_span = if let Some(node) = cx.tcx.hir().get_if_local(def_id) {
            match node {
                Node::Item(item) => Some(item.ident.span),
                Node::ImplItem(impl_item) => Some(impl_item.ident.span),
                _ => None,
            }
        } else {
            None
        };

        // If the function is part of the token interface, I store its defid.
        if verify_token_interface_function_similarity(fn_name.clone()) {
            self.detected_canonical_functions_count += 1;
            self.canonical_funcs_def_id.insert(def_id);
            if let Some(span) = fn_name_span {
                self.funcs_spans.push(span);
            }
        }
    }
}

fn verify_token_interface_function_similarity(fn_name: String) -> bool {
    let canonical_functions_formatted = [
        "allowance",
        "approve",
        "balance",
        "transfer",
        "transferfrom",
        "burn",
        "burnfrom",
        "decimals",
        "name",
        "symbol",
        "mint",
    ];
    let function_name = String::from(fn_name.split("::").last().unwrap());
    let formatted_name: String = function_name
        .to_lowercase()
        .replace("_", "")
        .split_whitespace()
        .collect();

    canonical_functions_formatted
        .iter()
        .any(|cf| edit_distance(formatted_name.as_str(), cf) <= 1)
}
