#![feature(rustc_private)]
#![allow(clippy::enum_variant_names)]

extern crate rustc_hir;
extern crate rustc_span;

use common::{
    analysis::{fn_returns, ConstantAnalyzer},
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
use std::collections::HashSet;

const LINT_MESSAGE: &str = "Unsafe usage of `expect`";

#[expose_lint_info]
pub static UNSAFE_EXPECT_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "In Rust, the expect method is commonly used for error handling. It retrieves the value from a Result or Option and panics with a specified error message if an error occurs. However, using expect can lead to unexpected program crashes.    ",
    severity: Severity::Medium,
    help: "https://coinfabrik.github.io/scout-rust/docs/detectors/unsafe-expect",
    vulnerability_class: VulnerabilityClass::ErrorHandling,
};

dylint_linting::declare_late_lint! {
    pub UNSAFE_EXPECT,
    Warn,
    LINT_MESSAGE
}

impl<'tcx> LateLintPass<'tcx> for UnsafeExpect {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: FnKind<'tcx>,
        fn_decl: &'tcx FnDecl<'tcx>,
        body: &'tcx Body<'tcx>,
        span: Span,
        _: LocalDefId,
    ) {
        // If the function comes from a macro expansion or does not return a Result<(), ()> or Option<()>, we don't want to analyze it.
        if span.from_expansion()
            || !fn_returns(fn_decl, sym::Result) && !fn_returns(fn_decl, sym::Option)
        {
            return;
        }

        let mut constant_analyzer = ConstantAnalyzer {
            cx,
            constants: HashSet::new(),
        };
        constant_analyzer.visit_body(body);

        let mut visitor = UnsafeChecks::new(cx, UNSAFE_EXPECT, constant_analyzer, sym::expect);

        walk_expr(&mut visitor, body.value);
    }
}
