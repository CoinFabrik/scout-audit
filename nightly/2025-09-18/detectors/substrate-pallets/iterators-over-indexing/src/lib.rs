#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;
extern crate rustc_type_ir;

use std::collections::HashSet;

use clippy_utils::diagnostics::span_lint_and_help;
use common::{
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use rustc_hir::Expr;
use rustc_lint::{LateContext, LateLintPass};

const LINT_MESSAGE: &str =
    "Hardcoding an index could lead to panic if the top bound is out of bounds.";

#[expose_lint_info]
pub static ITERATORS_OVER_INDEXING_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "Instead, use an iterator or index to `.len()`.",
    severity: Severity::Medium,
    help:
        "https://coinfabrik.github.io/scout-audit/docs/detectors/substrate/iterators-over-indexing",
    vulnerability_class: VulnerabilityClass::Arithmetic,
};

dylint_linting::declare_late_lint! {
    pub ITERATORS_OVER_INDEXING,
    Warn,
    LINT_MESSAGE
}

fn make_config() -> common_detectors::iterators_over_indexing::IteratorsOverIndexingConfig {
    let mut set = HashSet::<String>::new();
    set.insert("bounded_collections::bounded_vec::BoundedVec".to_string());
    common_detectors::iterators_over_indexing::IteratorsOverIndexingConfig {
        check_get: false,
        check_index: true,
        relevant_object_types: set,
    }
}

impl<'tcx> LateLintPass<'tcx> for IteratorsOverIndexing {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'_>) {
        let config = make_config();
        let span_constant =
            common_detectors::iterators_over_indexing::check_expr(cx, expr, &config);
        for span in span_constant {
            span_lint_and_help(
                cx,
                ITERATORS_OVER_INDEXING,
                span,
                LINT_MESSAGE,
                None,
                "Instead, use an iterator or index to `.len()`.",
            );
        }
    }
}
