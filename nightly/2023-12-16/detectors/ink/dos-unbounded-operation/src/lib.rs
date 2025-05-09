#![feature(rustc_private)]
#![feature(let_chains)]

extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_span;

use common::{
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use rustc_ast::LitKind;
use rustc_hir::{
    intravisit::{walk_expr, FnKind, Visitor},
    ExprKind, LangItem, MatchSource, QPath,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::{def_id::LocalDefId, Span};

const LINT_MESSAGE: &str =
    "In order to prevent a single transaction from consuming all the gas in a block, unbounded operations must be avoided";

#[expose_lint_info]
pub static DOS_UNBOUNDED_OPERATION_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "In order to prevent a single transaction from consuming all the gas in a block, unbounded operations must be avoided. This includes loops that do not have a bounded number of iterations, and recursive calls.    ",
    severity: Severity::Medium,
    help: "https://coinfabrik.github.io/scout-audit/docs/detectors/ink/dos-unbounded-operation",
    vulnerability_class: VulnerabilityClass::DoS,
};

dylint_linting::declare_late_lint! {
    pub DOS_UNBOUNDED_OPERATION,
    Warn,
    LINT_MESSAGE
}

struct ForLoopVisitor {
    span_constant: Vec<Span>,
}
impl<'tcx> Visitor<'tcx> for ForLoopVisitor {
    fn visit_expr(&mut self, expr: &'tcx rustc_hir::Expr<'tcx>) {
        if let ExprKind::Match(match_expr, _arms, source) = expr.kind
            && source == MatchSource::ForLoopDesugar
            && let ExprKind::Call(func, args) = match_expr.kind
            && let ExprKind::Path(qpath) = &func.kind
            && let QPath::LangItem(item, _span) = qpath
            && item == &LangItem::IntoIterIntoIter
        {
            if args.first().is_some()
                && let ExprKind::Struct(qpath, fields, _) = args.first().unwrap().kind
                && let QPath::LangItem(langitem, _span) = qpath
                && (LangItem::Range == *langitem
                    || LangItem::RangeInclusiveStruct == *langitem
                    || LangItem::RangeInclusiveNew == *langitem)
                && fields.last().is_some()
                && let ExprKind::Lit(lit) = &fields.last().unwrap().expr.kind
                && let LitKind::Int(_v, _typ) = lit.node
            {
                walk_expr(self, expr);
            } else {
                self.span_constant.push(expr.span);
            }
        }
        walk_expr(self, expr);
    }
}
impl<'tcx> LateLintPass<'tcx> for DosUnboundedOperation {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        kind: rustc_hir::intravisit::FnKind<'tcx>,
        _: &'tcx rustc_hir::FnDecl<'tcx>,
        body: &'tcx rustc_hir::Body<'tcx>,
        _: Span,
        _: LocalDefId,
    ) {
        if let FnKind::Method(_ident, _sig) = kind {
            let mut visitor = ForLoopVisitor {
                span_constant: vec![],
            };
            walk_expr(&mut visitor, body.value);

            for span in visitor.span_constant {
                clippy_utils::diagnostics::span_lint_and_help(
                    cx,
                    DOS_UNBOUNDED_OPERATION,
                    span,
                    LINT_MESSAGE,
                    None,
                    "This loop seems to do not have a fixed number of iterations",
                );
            }
        }
    }
}
