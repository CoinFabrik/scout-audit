#![feature(rustc_private)]

extern crate rustc_ast;

use clippy_utils::diagnostics::span_lint_and_help;
use common::{
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use rustc_ast::{
    token::TokenKind, tokenstream::TokenTree, AssocItemKind, HasTokens, Item, ItemKind,
};
use rustc_lint::{EarlyContext, EarlyLintPass};

const LINT_MESSAGE: &str =
    "Invalid weight attribute: Each extrinsic must use its own unique weight calculation function.";

#[expose_lint_info]
pub static INVALID_EXTRINSIC_WEIGHT_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "The weight attribute is using a weight calculation function that doesn't match the extrinsic name. \
                    Each extrinsic must have its own dedicated weight calculation to accurately reflect its resource consumption. \
                    Reusing weight calculations from other functions can lead to incorrect resource estimation and potential issues in production.",
    severity: Severity::Enhancement,
    help: "https://coinfabrik.github.io/scout-audit/docs/detectors/substrate/invalid-weight-info",
    vulnerability_class: VulnerabilityClass::KnownBugs,
};

dylint_linting::declare_pre_expansion_lint! {
    pub INVALID_EXTRINSIC_WEIGHT,
    Warn,
    LINT_MESSAGE
}

struct WeightInfoValidator {
    function_name: String,
    collected_text: Vec<String>,
}

impl WeightInfoValidator {
    fn new(function_name: String) -> Self {
        Self {
            function_name,
            collected_text: Vec::new(),
        }
    }

    fn process_token_trees(&mut self, trees: &[TokenTree]) {
        for tree in trees {
            match tree {
                TokenTree::Delimited(.., token_stream) => {
                    self.process_token_trees(&token_stream.trees().cloned().collect::<Vec<_>>());
                }
                TokenTree::Token(token, _) => {
                    if let TokenKind::Ident(symbol, _) = token.kind {
                        self.collected_text.push(symbol.to_string());
                    }
                }
            }
        }
    }

    fn is_valid_weight_info(&self) -> bool {
        // Less than 4 elements means we are not in the weight attribute
        if self.collected_text.len() < 4 {
            return true;
        }

        // We are looking for the following pattern: #[pallet::weight(...::WeightInfo::FUNCTION_NAME)]
        // First two items should be "pallet" and "weight"
        let is_pallet_attr = self.collected_text[0] == "pallet";
        let is_weight_attr = self.collected_text[1] == "weight";

        // Last item should be the function name
        let function_name = &self.collected_text[self.collected_text.len() - 1];
        let is_function_attr = function_name == &self.function_name;

        // Second to last should be "WeightInfo"
        let second_last = &self.collected_text[self.collected_text.len() - 2];
        let is_weight_info = second_last == "WeightInfo";

        is_pallet_attr && is_weight_attr && is_function_attr && is_weight_info
    }
}

impl EarlyLintPass for InvalidExtrinsicWeight {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        if let ItemKind::Impl(impl_) = &item.kind {
            for impl_item in &impl_.items {
                if let AssocItemKind::Fn(..) = &impl_item.kind {
                    let fn_name = impl_item.ident.name.to_string();

                    for attr in &impl_item.attrs {
                        if attr.is_doc_comment() {
                            continue;
                        }

                        if let Some(token_stream) = attr.tokens() {
                            let mut validator = WeightInfoValidator::new(fn_name.clone());

                            let attr_token_stream = token_stream.to_attr_token_stream();
                            let attr_token_trees = attr_token_stream.to_token_trees();

                            validator.process_token_trees(&attr_token_trees);

                            if !validator.is_valid_weight_info() {
                                span_lint_and_help(
                                    cx,
                                    INVALID_EXTRINSIC_WEIGHT,
                                    impl_item.span,
                                    LINT_MESSAGE,
                                    Some(attr.span),
                                    "used here",
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
