#![feature(rustc_private)]
#![warn(unused_extern_crates)]
#![feature(let_chains)]

extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;

use rustc_hir::intravisit::walk_expr;
use rustc_hir::intravisit::Visitor;
use rustc_hir::{Expr, ExprKind};
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::ty::Ty;
use rustc_span::def_id::DefId;
use rustc_span::Span;
use scout_audit_clippy_utils::diagnostics::span_lint;

const LINT_MESSAGE: &str = "This vector operation is called without access control";

dylint_linting::impl_late_lint! {
    pub UNEXPECTED_REVERT_WARN,
    Warn,
    "",
    UnexpectedRevertWarn::default(),
    {
        name: "Unexpected Revert Inserting to Storage",
        long_message: " It occurs by preventing transactions by other users from being successfully executed forcing the blockchain state to revert to its original state.",
        severity: "Medium",
        help: "https://coinfabrik.github.io/scout-soroban/docs/detectors/dos-unexpected-revert-with-vector",
        vulnerability_class: "Denial of Service",
    }
}

#[derive(Default)]
pub struct UnexpectedRevertWarn {}
impl UnexpectedRevertWarn {
    pub fn new() -> Self {
        Self {}
    }
}

impl<'tcx> LateLintPass<'tcx> for UnexpectedRevertWarn {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: rustc_hir::intravisit::FnKind<'tcx>,
        _: &'tcx rustc_hir::FnDecl<'tcx>,
        body: &'tcx rustc_hir::Body<'tcx>,
        _: Span,
        _localdef: rustc_span::def_id::LocalDefId,
    ) {
        struct UnprotectedVectorFinder<'tcx, 'tcx_ref> {
            cx: &'tcx_ref LateContext<'tcx>,
            push_def_id: Option<DefId>,
            push_span: Option<Span>,
            require_auth: bool,
        }
        impl<'tcx> Visitor<'tcx> for UnprotectedVectorFinder<'tcx, '_> {
            fn visit_expr(&mut self, expr: &'tcx Expr<'_>) {
                if let ExprKind::MethodCall(path, _receiver, ..) = expr.kind {
                    let defid = self.cx.typeck_results().type_dependent_def_id(expr.hir_id);
                    let ty = Ty::new_foreign(self.cx.tcx, defid.unwrap());
                    if path.ident.name.to_string() == "require_auth" {
                        self.require_auth = true;
                    }
                    if ty.to_string().contains("soroban_sdk::Vec")
                        && (path.ident.name.to_string() == "push_back"
                            || path.ident.name.to_string() == "push_front")
                    {
                        self.push_def_id = defid;
                        self.push_span = Some(path.ident.span);
                    }
                }
                walk_expr(self, expr);
            }
        }

        let mut uvf_storage = UnprotectedVectorFinder {
            cx,
            push_def_id: None,
            push_span: None,
            require_auth: false,
        };

        walk_expr(&mut uvf_storage, body.value);

        if uvf_storage.push_def_id.is_some() && !uvf_storage.require_auth {
            span_lint(
                uvf_storage.cx,
                UNEXPECTED_REVERT_WARN,
                uvf_storage.push_span.unwrap(),
                LINT_MESSAGE,
            );
        }
    }
}
