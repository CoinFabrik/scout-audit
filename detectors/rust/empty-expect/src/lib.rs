#![feature(rustc_private)]
#![allow(clippy::enum_variant_names)]

extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_span;

use clippy_utils::{diagnostics::span_lint, is_from_proc_macro};
use common::{
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use if_chain::if_chain;
use rustc_ast::LitKind;
use rustc_hir::{
    def_id::LocalDefId,
    intravisit::{walk_expr, FnKind, Visitor},
    Body, Expr, ExprKind, FnDecl,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::{sym, Span};

const LINT_MESSAGE: &str = "`.expect()` called with an empty string message";

#[expose_lint_info]
pub static EMPTY_EXPECT_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "Using `expect` with an empty string makes error diagnosis \
        harder as it provides no context about what went wrong",
    severity: Severity::Medium,
    help: "https://coinfabrik.github.io/scout-rust/docs/detectors/empty-expect",
    vulnerability_class: VulnerabilityClass::BestPractices,
};

dylint_linting::declare_late_lint! {
    pub EMPTY_EXPECT,
    Warn,
    LINT_MESSAGE
}

struct EmptyExpectVisitor<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
}

impl<'a, 'tcx> Visitor<'tcx> for EmptyExpectVisitor<'a, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx Expr<'_>) {
        if_chain! {
            // Ignore expressions from proc macros
            if !is_from_proc_macro(self.cx, expr);
            // Check if the expression is `expect`
            if let ExprKind::MethodCall(path_segment, _, args, _) = &expr.kind;
            if path_segment.ident.name == sym::expect;
            // Check if the argument is a string literal and empty
            if let Some(arg) = args.first();
            if let ExprKind::Lit(lit) = arg.kind;
            if let LitKind::Str(s, _) = lit.node;
            if s.is_empty();
            then {
                span_lint(self.cx, EMPTY_EXPECT, expr.span, LINT_MESSAGE);
            }
        }

        walk_expr(self, expr);
    }
}

impl<'tcx> LateLintPass<'tcx> for EmptyExpect {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: FnKind<'tcx>,
        _: &'tcx FnDecl<'tcx>,
        body: &'tcx Body<'tcx>,
        _: Span,
        _: LocalDefId,
    ) {
        let mut visitor = EmptyExpectVisitor { cx };
        walk_expr(&mut visitor, body.value);
    }
}
