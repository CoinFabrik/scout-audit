#![feature(rustc_private)]

extern crate rustc_hir;
extern crate rustc_span;

use common::{
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use if_chain::if_chain;
use rustc_hir::{
    def::Res,
    def_id::LocalDefId,
    intravisit::{walk_expr, FnKind, Visitor},
    Body, Expr, ExprKind, FnDecl, PatKind, QPath,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::Span;

const LINT_MESSAGE: &str = "Passing arguments to the target of a delegate call is not safe, as it allows the caller to set a malicious hash as the target.";

#[expose_lint_info]
pub static DELEGATE_CALL_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "It is important to validate and restrict delegate calls to trusted contracts, implement proper access control mechanisms, and carefully review external contracts to prevent unauthorized modifications, unexpected behavior, and potential exploits.",
    severity: Severity::Critical,
    help: "https://coinfabrik.github.io/scout-audit/docs/detectors/ink/delegate-call",
    vulnerability_class: VulnerabilityClass::Authorization,
};

dylint_linting::declare_late_lint! {
    pub DELEGATE_CALL,
    Warn,
    LINT_MESSAGE
}

impl<'tcx> LateLintPass<'tcx> for DelegateCall {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: FnKind<'tcx>,
        _: &'tcx FnDecl<'_>,
        body: &'tcx Body<'_>,
        _: Span,
        _: LocalDefId,
    ) {
        struct DelegateCallStorage<'tcx> {
            span: Option<Span>,
            has_vulnerable_delegate: bool,
            the_body: &'tcx Body<'tcx>,
        }

        fn check_delegate_call(expr: &Expr, body: &Body<'_>) -> Option<Span> {
            if_chain! {
                if let ExprKind::MethodCall(func, _, arguments, _) = &expr.kind;
                if let function_name = func.ident.name.to_string();
                if function_name == "delegate";
                then {
                    let mut param_hir_ids = Vec::new();
                    let mut arg_hir_ids = Vec::new();

                    for i in 0..body.params.len() {
                        if let PatKind::Binding(_, hir_id, _, _) = body.params[i].pat.kind {
                            param_hir_ids.push(hir_id);
                        }
                    }

                    for i in 0..arguments.len() {
                        arg_hir_ids.push(arguments[i].hir_id);

                        if let ExprKind::Path(QPath::Resolved(_, path)) = &arguments[i].kind {
                            if let Res::Local(hir_id) = path.res {
                                arg_hir_ids.push(hir_id);
                            }
                            for j in 0..path.segments.len() {
                                arg_hir_ids.push(path.segments[j].hir_id);
                            }
                        }

                    }

                    for param_id in param_hir_ids {
                        if arg_hir_ids.contains(&param_id) {
                            return Some(expr.span);
                        }
                    }

                    return None;
                }
            }
            None
        }

        impl<'tcx> Visitor<'tcx> for DelegateCallStorage<'_> {
            fn visit_expr(&mut self, expr: &'tcx Expr<'_>) {
                if let Some(delegate_call_span) = check_delegate_call(expr, self.the_body) {
                    self.has_vulnerable_delegate = true;
                    self.span = Some(delegate_call_span);
                };

                walk_expr(self, expr);
            }
        }

        let mut delegate_storage = DelegateCallStorage {
            span: None,
            has_vulnerable_delegate: false,
            the_body: body,
        };

        walk_expr(&mut delegate_storage, body.value);

        if delegate_storage.has_vulnerable_delegate {
            clippy_utils::diagnostics::span_lint_and_help(
                cx,
                DELEGATE_CALL,
                delegate_storage.span.unwrap(),
                LINT_MESSAGE,
                None,
                "Consider using a memory value (self.target) as the target of the delegate call.",
            );
        }
    }
}
