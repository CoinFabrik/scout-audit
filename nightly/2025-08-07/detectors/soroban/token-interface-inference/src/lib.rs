#![feature(rustc_private)]

extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_span;

use clippy_utils::diagnostics::span_lint;
use common::{
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use edit_distance::edit_distance;
use if_chain::if_chain;
use rustc_errors::MultiSpan;
use rustc_hir::{intravisit::FnKind, Body, FnDecl, Item, ItemKind, Node};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::{def_id::LocalDefId, Span};
use std::collections::HashSet;

const LINT_MESSAGE: &str =
    "This contract seems like a Token, consider implementing the Token Interface trait";
const INCLUDED_FUNCTIONS_THRESHOLD: usize = 60;
const TOKEN_INTERFACE_PATH: &str = "soroban_sdk::token::TokenInterface";
const TOKEN_INTERFACE_FUNCTIONS: [&str; 10] = [
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
];

#[expose_lint_info]
pub static TOKEN_INTERFACE_INFERENCE_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "Implementing the Token Interface trait helps to ensure proper compliance of the SEP-41 standard.",
    severity: Severity::Enhancement,
    help: "https://coinfabrik.github.io/scout-audit/docs/detectors/soroban/token-interface-inference",
    vulnerability_class: VulnerabilityClass::BestPractices,
};

dylint_linting::impl_late_lint! {
    pub TOKEN_INTERFACE_INFERENCE,
    Warn,
    LINT_MESSAGE,
    TokenInterfaceInference::default()
}

#[derive(Default)]
struct TokenInterfaceInference {
    impl_token_interface_trait: bool,
    detected_canonical_functions: HashSet<&'static str>,
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

        if self.detected_canonical_functions.len() * 100
            >= TOKEN_INTERFACE_FUNCTIONS.len() * INCLUDED_FUNCTIONS_THRESHOLD
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
        let fn_name_span = if let Some(node) = cx.tcx.hir_get_if_local(def_id) {
            match node {
                Node::Item(item) => match item.kind {
                    ItemKind::ExternCrate(_, ident)
                    | ItemKind::Static(_, ident, _, _)
                    | ItemKind::Const(ident, _, _, _)
                    | ItemKind::Fn { ident, .. }
                    | ItemKind::Macro(ident, _, _)
                    | ItemKind::Mod(ident, _)
                    | ItemKind::TyAlias(ident, _, _)
                    | ItemKind::Enum(ident, _, _)
                    | ItemKind::Struct(ident, _, _)
                    | ItemKind::Union(ident, _, _)
                    | ItemKind::Trait(_, _, _, ident, _, _, _)
                    | ItemKind::TraitAlias(ident, _, _) => Some(ident.span),
                    _ => None,
                },
                Node::ImplItem(impl_item) => Some(impl_item.ident.span),
                _ => None,
            }
        } else {
            None
        };

        if let Some(canonical_function) = verify_token_interface_function_similarity(&fn_name) {
            if self.detected_canonical_functions.insert(canonical_function) {
                if let Some(span) = fn_name_span {
                    self.funcs_spans.push(span);
                }
            }
        }
    }
}

fn verify_token_interface_function_similarity(fn_name: &str) -> Option<&'static str> {
    let function_name = fn_name.split("::").last().unwrap_or(fn_name);
    let formatted_name: String = function_name
        .to_lowercase()
        .replace('_', "")
        .split_whitespace()
        .collect();

    TOKEN_INTERFACE_FUNCTIONS
        .iter()
        .copied()
        .find(|cf| edit_distance(formatted_name.as_str(), cf) <= 1)
}
