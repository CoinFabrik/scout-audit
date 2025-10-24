#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_span;

use clippy_utils::{consts::Constant, diagnostics::span_lint_and_help};
use common::{
    analysis::{get_expr_hir_id_opt, is_soroban_storage, ConstantAnalyzer, SorobanStorageType},
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use if_chain::if_chain;
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

        let mut constant_analyzer = ConstantAnalyzer::new(cx);
        constant_analyzer.visit_body(body);

        let mut visitor = IneffectiveExtendTtlVisitor {
            cx,
            constant_analyzer,
        };
        walk_expr(&mut visitor, body.value);
    }
}

struct IneffectiveExtendTtlVisitor<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    constant_analyzer: ConstantAnalyzer<'a, 'tcx>,
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

                let threshold_expr = &args[threshold_idx];
                let extend_to_expr = &args[extend_to_idx];

                let same_binding = match (
                    get_expr_hir_id_opt(threshold_expr),
                    get_expr_hir_id_opt(extend_to_expr),
                ) {
                    (Some(threshold), Some(extend_to)) => threshold == extend_to,
                    _ => false,
                };

                let shrink_extend = self.is_extend_to_smaller_than_threshold(
                    threshold_expr,
                    extend_to_expr,
                );

                if same_binding || shrink_extend {
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
        walk_expr(self, expr);
    }
}

impl<'a, 'tcx> IneffectiveExtendTtlVisitor<'a, 'tcx> {
    fn is_extend_to_smaller_than_threshold(
        &self,
        threshold_expr: &Expr<'tcx>,
        extend_to_expr: &Expr<'tcx>,
    ) -> bool {
        let threshold_val = self.resolve_constant_u128(threshold_expr);
        let extend_to_val = self.resolve_constant_u128(extend_to_expr);

        match (threshold_val, extend_to_val) {
            (Some(threshold), Some(extend_to)) => extend_to <= threshold,
            _ => false,
        }
    }

    fn resolve_constant_u128(&self, expr: &Expr<'tcx>) -> Option<u128> {
        if let Some(constant) = self.constant_analyzer.get_constant(expr) {
            if let Constant::Int(value) = constant {
                return Some(value);
            }
        }

        None
    }
}
