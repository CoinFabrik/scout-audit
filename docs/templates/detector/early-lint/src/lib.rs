#![feature(rustc_private)]

extern crate rustc_ast;

use common::{
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use rustc_ast::{
    visit::{walk_expr, Visitor},
    Expr,
};
use rustc_lint::{EarlyContext, EarlyLintPass};

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

dylint_linting::declare_early_lint! {
    pub YOUR_LINT_NAME,
    Warn,
    "Short description of the lint"
}

struct YourVisitor {
    // Visitor fields
}

impl<'ast> Visitor<'ast> for YourVisitor {
    fn visit_expr(&mut self, expr: &'ast Expr) {
        // Implement the logic of your lint here

        // Call `walk_expr` to visit the descendants of `ex`
        walk_expr(self, expr);
    }
}

impl EarlyLintPass for YourLintName {
    fn check_expr(&mut self, _: &EarlyContext<'_>, expr: &Expr) {
        let mut visitor = YourVisitor {
            // Initialize visitor fields
        };
        visitor.visit_expr(expr);
    }
}
