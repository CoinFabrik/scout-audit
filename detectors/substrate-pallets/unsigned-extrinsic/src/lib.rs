#![feature(rustc_private)]

extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_span;

use clippy_utils::{diagnostics::span_lint_and_sugg, is_from_proc_macro, source::snippet};
use common::{
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use if_chain::if_chain;
use rustc_errors::Applicability;
use rustc_hir::{
    def_id::LocalDefId,
    intravisit::{walk_expr, FnKind, Visitor},
    Body, Expr, ExprKind, FnDecl, QPath,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::{Span, Symbol};

const LINT_MESSAGE: &str = "Using unsigned extrinsics without fees exposes the chain to potential DoS attacks";

#[expose_lint_info]
pub static UNSIGNED_EXTRINSIC_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "Unsigned extrinsics allow transactions to be submitted without any associated fees \
                   or signatures. This can be exploited by malicious actors to flood the network with \
                   transactions at no cost, potentially causing denial of service. Consider using signed \
                   extrinsics with appropriate fee mechanisms unless there's a specific security reason \
                   for allowing unsigned transactions.",
    severity: Severity::Critical,
    help: "https://coinfabrik.github.io/scout-substrate/docs/detectors/unsigned-extrinsic",
    vulnerability_class: VulnerabilityClass::DoS,
};

dylint_linting::declare_late_lint! {
    pub UNSIGNED_EXTRINSIC,
    Warn,
    LINT_MESSAGE
}

struct UnsignedExtrinsicVisitor<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
}

impl<'a, 'tcx> Visitor<'tcx> for UnsignedExtrinsicVisitor<'a, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx Expr<'tcx>) {
        if_chain! {
            // Ignore expressions from proc macros
            if !is_from_proc_macro(self.cx, expr);
            // Check if the expression is `ensure_none`
            if let ExprKind::Call(callee, args) = &expr.kind;
            if let ExprKind::Path(QPath::Resolved(None, path)) = &callee.kind;
            if path.segments.len() == 1;
            if path.segments[0].ident.name == Symbol::intern("ensure_none");
            then {
                let first_arg_str = snippet(self.cx, args[0].span, "..");
                span_lint_and_sugg(
                    self.cx,
                    UNSIGNED_EXTRINSIC,
                    expr.span,
                    LINT_MESSAGE,
                    "consider signing this extrinsic to prevent DoS attacks",
                    format!("ensure_signed({})", first_arg_str),
                    Applicability::MaybeIncorrect,
                );
            }
        }

        walk_expr(self, expr);
    }
}

impl<'tcx> LateLintPass<'tcx> for UnsignedExtrinsic {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: FnKind<'tcx>,
        _: &'tcx FnDecl<'tcx>,
        body: &'tcx Body<'tcx>,
        _: Span,
        _: LocalDefId,
    ) {
        let mut visitor = UnsignedExtrinsicVisitor { cx };
        walk_expr(&mut visitor, body.value);
    }
}
