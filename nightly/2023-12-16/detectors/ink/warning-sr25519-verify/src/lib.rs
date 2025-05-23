#![feature(rustc_private)]
#![feature(let_chains)]

extern crate rustc_hir;
extern crate rustc_span;

use clippy_utils::diagnostics::span_lint_and_help;
use common::{
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use rustc_hir::{
    intravisit::{walk_body, walk_expr, Visitor},
    Expr, ExprKind, QPath,
};
use rustc_lint::LateLintPass;
use rustc_span::Span;

const LINT_MESSAGE: &str = "This function is from the unstable interface, which is unsafe and normally is not available on production chains.";

#[expose_lint_info]
pub static WARNING_SR25519_VERIFY_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: LINT_MESSAGE,
    severity: Severity::Medium,
    help: "https://coinfabrik.github.io/scout-audit/docs/detectors/ink/warning-sr25519-verify",
    vulnerability_class: VulnerabilityClass::KnownBugs,
};

dylint_linting::declare_late_lint! {
    pub WARNING_SR25519_VERIFY,
    Warn,
    LINT_MESSAGE
}

impl<'tcx> LateLintPass<'tcx> for WarningSr25519Verify {
    fn check_fn(
        &mut self,
        cx: &rustc_lint::LateContext<'tcx>,
        _: rustc_hir::intravisit::FnKind<'tcx>,
        _: &'tcx rustc_hir::FnDecl<'tcx>,
        body: &'tcx rustc_hir::Body<'tcx>,
        _: rustc_span::Span,
        _: rustc_hir::def_id::LocalDefId,
    ) {
        struct WarningSr25519VerifyVisitor {
            has_sr25519_verify: bool,
            has_sr25519_verify_span: Vec<Span>,
        }

        impl<'tcx> Visitor<'tcx> for WarningSr25519VerifyVisitor {
            fn visit_expr(&mut self, expr: &'tcx Expr<'_>) {
                if let ExprKind::Path(QPath::Resolved(_, path)) = &expr.kind
                    && path
                        .segments
                        .iter()
                        .any(|x| x.ident.name.to_string() == "sr25519_verify")
                {
                    self.has_sr25519_verify = true;
                    self.has_sr25519_verify_span.push(expr.span);
                }

                walk_expr(self, expr);
            }
        }

        let mut visitor = WarningSr25519VerifyVisitor {
            has_sr25519_verify: false,
            has_sr25519_verify_span: Vec::new(),
        };

        walk_body(&mut visitor, body);

        for span in visitor.has_sr25519_verify_span {
            span_lint_and_help(
                cx,
                WARNING_SR25519_VERIFY,
                span,
                LINT_MESSAGE,
                None,
                "Do not use it",
            );
        }
    }
}
