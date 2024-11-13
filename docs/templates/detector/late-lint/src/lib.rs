#![feature(rustc_private)]

extern crate rustc_hir;
extern crate rustc_span;

use common::{
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use rustc_hir::{
    intravisit::{walk_expr, FnKind, Visitor},
    Body, Expr, FnDecl,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::{def_id::LocalDefId, Span};

const SHORT_MESSAGE: &str = "This is a short message";
const LONG_MESSAGE: &str = "This is a long message";

#[expose_lint_info]
pub static YOUR_LINT_NAME_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: SHORT_MESSAGE,
    long_message: LONG_MESSAGE,
    severity: Severity::Medium,
    help: "https://github.com/CoinFabrik/{project}/tree/main/detectors/{lint}",
    vulnerability_class: VulnerabilityClass::KnownBugs,
};

dylint_linting::declare_late_lint! {
    pub YOUR_LINT_NAME,
    Warn,
    "Short description of the lint"
}

struct YourVisitor {
    // Visitor fields
}

impl<'tcx> Visitor<'tcx> for YourVisitor {
    fn visit_expr(&mut self, expr: &'tcx Expr<'tcx>) {
        // Implement the logic of your lint here

        // Call `walk_expr` to visit the descendants of `expr`
        walk_expr(self, expr);
    }
}

impl<'tcx> LateLintPass<'tcx> for YourLintName {
    fn check_fn(
        &mut self,
        _: &LateContext<'tcx>,
        _: FnKind<'tcx>,
        _: &'tcx FnDecl<'tcx>,
        body: &'tcx Body<'tcx>,
        _: Span,
        _: LocalDefId,
    ) {
        let mut visitor = YourVisitor {
            // Initialize visitor fields
        };
        visitor.visit_expr(body.value);
    }
}
