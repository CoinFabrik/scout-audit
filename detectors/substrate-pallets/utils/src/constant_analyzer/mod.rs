extern crate rustc_hir;
extern crate rustc_lint;

use clippy_utils::consts::{constant, Constant};
use if_chain::if_chain;
use rustc_hir::{
    def::{DefKind, Res},
    intravisit::{walk_local, Visitor},
    Expr, ExprKind, HirId, Node, QPath,
};
use rustc_lint::LateContext;
use std::collections::HashSet;

/// Analyzes expressions to determine if they are constants or known at compile-time.
pub struct ConstantAnalyzer<'a, 'tcx> {
    pub cx: &'a LateContext<'tcx>,
    pub constants: HashSet<HirId>,
}

impl<'a, 'tcx> ConstantAnalyzer<'a, 'tcx> {
    /// Checks if a QPath refers to a constant.
    fn is_qpath_constant(&self, path: &QPath) -> bool {
        if let QPath::Resolved(_, path) = path {
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

    /// Determines if an expression is constant or known at compile-time.
    fn is_expr_constant(&self, expr: &Expr<'tcx>) -> bool {
        if constant(self.cx, self.cx.typeck_results(), expr).is_some() {
            return true;
        }

        match expr.kind {
            ExprKind::Array(expr_array) => expr_array
                .iter()
                .all(|expr_in_array| self.is_expr_constant(expr_in_array)),
            ExprKind::Binary(_, left_expr, right_expr) => {
                self.is_expr_constant(left_expr) && self.is_expr_constant(right_expr)
            }
            ExprKind::Cast(cast_expr, _) => self.is_expr_constant(cast_expr),
            ExprKind::Field(field_expr, _) => self.is_expr_constant(field_expr),
            ExprKind::Index(array_expr, index_expr, _) => {
                self.is_array_index_constant(array_expr, index_expr)
            }
            ExprKind::Lit(_) => true,
            ExprKind::Path(qpath_expr) => self.is_qpath_constant(&qpath_expr),
            ExprKind::Repeat(repeat_expr, _) => self.is_expr_constant(repeat_expr),
            ExprKind::Struct(_, expr_fields, _) => expr_fields
                .iter()
                .all(|field_expr| self.is_expr_constant(field_expr.expr)),
            _ => false,
        }
    }

    /// Checks if an array index operation results in a constant value.
    fn is_array_index_constant(&self, array_expr: &Expr<'tcx>, index_expr: &Expr<'tcx>) -> bool {
        match (
            &array_expr.kind,
            constant(self.cx, self.cx.typeck_results(), index_expr),
        ) {
            (ExprKind::Array(array_elements), Some(Constant::Int(index))) => {
                self.is_array_element_constant(array_elements, index)
            }
            (ExprKind::Path(QPath::Resolved(_, path)), Some(Constant::Int(index))) => {
                if_chain! {
                    if let Res::Local(hir_id) = path.res;
                    if let Some(Node::Local(let_stmt)) = self.cx.tcx.hir().find_parent(hir_id);
                    if let Some(ExprKind::Array(array_elements)) = let_stmt.init.map(|init| &init.kind);
                    then {
                        self.is_array_element_constant(array_elements, index)
                    } else {
                        false
                    }
                }
            }
            _ => false,
        }
    }

    /// Checks if a specific array element is constant.
    fn is_array_element_constant(&self, elements: &[Expr<'tcx>], index: u128) -> bool {
        elements
            .get(index as usize)
            .map_or(false, |element| self.is_expr_constant(element))
    }

    /// Public method to check if an expression is constant.
    pub fn is_constant(&self, expr: &Expr<'tcx>) -> bool {
        self.is_expr_constant(expr)
    }
}

impl<'a, 'tcx> Visitor<'tcx> for ConstantAnalyzer<'a, 'tcx> {
    fn visit_local(&mut self, l: &'tcx rustc_hir::Local<'tcx>) {
        if let Some(init) = l.init {
            if self.is_expr_constant(init) {
                self.constants.insert(l.pat.hir_id);
            }
        }
        walk_local(self, l);
    }
}
