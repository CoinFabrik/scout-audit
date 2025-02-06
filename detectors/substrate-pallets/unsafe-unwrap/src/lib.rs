#![feature(rustc_private)]

extern crate rustc_hir;
extern crate rustc_span;

use common::{
    analysis::ConstantAnalyzer,
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use common_detectors::unsafe_checks::UnsafeChecks;
use rustc_hir::{
    def_id::LocalDefId,
    intravisit::{walk_expr, FnKind, Visitor},
    Body, FnDecl,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::{sym, Span};
use std::collections::HashMap;

const LINT_MESSAGE: &str = "Unsafe usage of `unwrap`";

#[expose_lint_info]
pub static UNSAFE_UNWRAP_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "This vulnerability class pertains to the inappropriate usage of the unwrap method in Rust, which is commonly employed for error handling. The unwrap method retrieves the inner value of an Option or Result, but if an error or None occurs, it triggers a panic and crashes the program.    ",
    severity: Severity::Medium,
    help: "https://coinfabrik.github.io/scout-audit/docs/detectors/substrate/unsafe-unwrap",
    vulnerability_class: VulnerabilityClass::ErrorHandling,
};

dylint_linting::declare_late_lint! {
    pub UNSAFE_UNWRAP,
    Warn,
    LINT_MESSAGE
}

impl<'tcx> LateLintPass<'tcx> for UnsafeUnwrap {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: FnKind<'tcx>,
        _: &'tcx FnDecl<'tcx>,
        body: &'tcx Body<'tcx>,
        _: Span,
        _: LocalDefId,
    ) {
        let mut constant_analyzer = ConstantAnalyzer::new(cx);
        constant_analyzer.visit_body(body);

        let mut visitor = UnsafeChecks::new(cx, UNSAFE_UNWRAP, constant_analyzer, sym::unwrap);

        walk_expr(&mut visitor, body.value);
    }
}
