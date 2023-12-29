#![feature(rustc_private)]
#![recursion_limit = "256"]
extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_span;

use if_chain::if_chain;
use rustc_hir::def::DefKind;
use rustc_hir::StmtKind;
use rustc_hir::{
    def::Res,
    intravisit::{walk_body, walk_expr, FnKind, Visitor},
    Body, Expr, ExprKind, FnDecl, HirId, LangItem, MatchSource, QPath,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::{def_id::LocalDefId, Span};
use scout_audit_internal::Detector;

dylint_linting::declare_late_lint!(
    pub DOS_UNBOUNDED_OPERATION,
    Warn,
    "This loop seems to do not have a fixed number of iterations"
);

struct ForLoopVisitor {
    constants: Vec<HirId>,
    span_constant: Vec<Span>,
}

impl ForLoopVisitor {
    fn is_qpath_constant(&self, path: &QPath) -> bool {
        if let QPath::Resolved(_, path) = path {
            // We search the path, if it has been previously defined or is a constant then we are good
            match path.res {
                Res::Def(def_kind, _) => matches!(
                    def_kind,
                    DefKind::AnonConst
                        | DefKind::AssocConst
                        | DefKind::Const
                        | DefKind::InlineConst
                ),
                Res::Local(hir_id) => self.constants.contains(&hir_id),
                _ => false,
            }
        } else {
            false
        }
    }

    fn is_expr_constant(&self, current_expr: &Expr) -> bool {
        match current_expr.kind {
            ExprKind::Array(expr_array) => expr_array
                .iter()
                .all(|expr_in_array| self.is_expr_constant(expr_in_array)),
            ExprKind::Binary(_, left_expr, right_expr) => {
                self.is_expr_constant(left_expr) && self.is_expr_constant(right_expr)
            }
            ExprKind::Cast(cast_expr, _) => self.is_expr_constant(cast_expr),
            ExprKind::Field(field_expr, _) => self.is_expr_constant(field_expr),
            ExprKind::Index(array_expr, index_expr, _) => {
                self.is_expr_constant(array_expr) && self.is_expr_constant(index_expr)
            }
            ExprKind::Lit(_) => true,
            ExprKind::MethodCall(_, call_expr, _, _) => self.is_expr_constant(call_expr),
            ExprKind::Path(qpath_expr) => self.is_qpath_constant(&qpath_expr),
            ExprKind::Repeat(repeat_expr, _) => self.is_expr_constant(repeat_expr),
            ExprKind::Struct(_, expr_fields, _) => expr_fields
                .iter()
                .all(|field_expr| self.is_expr_constant(field_expr.expr)),
            _ => false,
        }
    }
}

impl<'tcx> Visitor<'tcx> for ForLoopVisitor {
    fn visit_expr(&mut self, expr: &'tcx Expr<'tcx>) {
        // Constant detection
        if let ExprKind::Block(a, _) = expr.kind {
            a.stmts.iter().for_each(|func| {
                if let StmtKind::Local(sd) = func.kind {
                    if sd.init.is_some() && self.is_expr_constant(sd.init.as_ref().unwrap()) {
                        self.constants.push(sd.pat.hir_id);
                    }
                }
            })
        }

        // For loop detection
        if_chain! {
            // Check if the expression is a for loop
            if let ExprKind::Match(match_expr, _, MatchSource::ForLoopDesugar) = expr.kind;
            if let ExprKind::Call(call_func, call_args) = match_expr.kind;
            // Check the function call
            if let ExprKind::Path(qpath) = &call_func.kind;
            if let QPath::LangItem(LangItem::IntoIterIntoIter, _, _) = qpath;
            // Check if a Range is used
            if let ExprKind::Struct(struct_lang_item, struct_expr, _) = call_args.first().unwrap().kind;
            if let QPath::LangItem(
                LangItem::Range | LangItem::RangeInclusiveStruct | LangItem::RangeInclusiveNew,
                _,
                _,
            ) = struct_lang_item;
            // Get the start and end of the range
            if let Some(start_expr) = struct_expr.first();
            if let Some(end_expr) = struct_expr.last();
            then {
                if !self.is_expr_constant(start_expr.expr) || !self.is_expr_constant(end_expr.expr) {
                    self.span_constant.push(expr.span);
                }
            }
        }
        walk_expr(self, expr);
    }
}

impl<'tcx> LateLintPass<'tcx> for DosUnboundedOperation {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: FnKind<'tcx>,
        _: &'tcx FnDecl<'tcx>,
        body: &'tcx Body<'tcx>,
        _: Span,
        _: LocalDefId,
    ) {
        let mut visitor = ForLoopVisitor {
            span_constant: Vec::new(),
            constants: Vec::new(),
        };

        walk_body(&mut visitor, body);

        for span in visitor.span_constant {
            Detector::DosUnboundedOperation.span_lint_and_help(
                cx,
                DOS_UNBOUNDED_OPERATION,
                span,
                "This loop seems to do not have a fixed number of iterations",
            );
        }
    }
}
