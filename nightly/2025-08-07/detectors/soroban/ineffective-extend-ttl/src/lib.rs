#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_span;

use clippy_utils::diagnostics::span_lint_and_help;
use common::{
    analysis::{get_expr_hir_id_opt, is_soroban_storage, SorobanStorageType},
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use if_chain::if_chain;
use rustc_ast::LitKind;
use rustc_hir::{
    intravisit::{walk_expr, FnKind, Visitor},
    Body, Expr, ExprKind, FnDecl,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::{def_id::LocalDefId, Span};

const LINT_MESSAGE: &str =
    "extend_ttl called with identical or smaller TTL arguments keeps refreshing the entry without enforcing expiration";

#[expose_lint_info]
pub static INEFFECTIVE_EXTEND_TTL_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "Soroban's extend_ttl can only increase an entry's lifetime. When both TTL parameters refer to the same binding, or the new TTL is smaller than the threshold, the call will run on every access making it ineffective",
    severity: Severity::Medium,
    help: "https://coinfabrik.github.io/scout-audit/docs/detectors/soroban/ineffective-extend-ttl",
    vulnerability_class: VulnerabilityClass::BestPractices,
};

dylint_linting::declare_late_lint! {
    pub INEFFECTIVE_EXTEND_TTL,
    Warn,
    LINT_MESSAGE
}

impl<'tcx> LateLintPass<'tcx> for IneffectiveExtendTtl {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: FnKind<'tcx>,
        _: &'tcx FnDecl<'tcx>,
        body: &'tcx Body<'tcx>,
        span: Span,
        _: LocalDefId,
    ) {
        if span.from_expansion() {
            return;
        }

        let mut visitor = IneffectiveExtendTtlVisitor { cx };
        walk_expr(&mut visitor, body.value);
    }
}

struct IneffectiveExtendTtlVisitor<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
}

impl<'tcx> Visitor<'tcx> for IneffectiveExtendTtlVisitor<'_, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx Expr<'_>) {
        if_chain! {
            if let ExprKind::MethodCall(path_segment, receiver, args, call_span) = &expr.kind;
            if path_segment.ident.name.as_str() == "extend_ttl";
            // Instance storage has 2 args (threshold, target), Persistent/Temporary have 3 (key, threshold, target)
            if args.len() == 2 || args.len() == 3;
            if is_soroban_storage(self.cx, self.cx.typeck_results().expr_ty(receiver), SorobanStorageType::Any);
            then {
                // Determine argument positions based on storage type
                let (threshold_idx, extend_to_idx) = if args.len() == 2 {
                    // Instance storage: extend_ttl(threshold, target)
                    (0, 1)
                } else {
                    // Persistent/Temporary storage: extend_ttl(key, threshold, target)
                    (1, 2)
                };

                if let Some(threshold) = get_expr_hir_id_opt(&args[threshold_idx]) {
                    if let Some(extend_to) = get_expr_hir_id_opt(&args[extend_to_idx]) {
                        let same_binding = threshold == extend_to;

                        if same_binding || is_extend_to_smaller_than_threshold(&args[threshold_idx], &args[extend_to_idx]) {
                            span_lint_and_help(
                                self.cx,
                                INEFFECTIVE_EXTEND_TTL,
                                *call_span,
                                LINT_MESSAGE,
                                None,
                                "ensure `extend_to` is strictly higher than `threshold`, or enforce the expiration through contract logic instead of extend_ttl",
                            );
                        }
                    }
                }
            }
        }
        walk_expr(self, expr);
    }
}

fn is_extend_to_smaller_than_threshold(
    threshold_expr: &Expr<'_>,
    extend_to_expr: &Expr<'_>,
) -> bool {
    let threshold_val = extract_literal_int(threshold_expr);
    let extend_to_val = extract_literal_int(extend_to_expr);

    if let (Some(threshold), Some(extend_to)) = (threshold_val, extend_to_val) {
        return extend_to <= threshold;
    }

    false
}

fn extract_literal_int(expr: &Expr<'_>) -> Option<u128> {
    if let ExprKind::Lit(lit) = &expr.kind {
        if let LitKind::Int(val, _) = lit.node {
            return Some(val.get());
        }
    }
    None
}
