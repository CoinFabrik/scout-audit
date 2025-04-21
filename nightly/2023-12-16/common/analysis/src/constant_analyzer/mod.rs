extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_lint;

use clippy_utils::consts::{constant, Constant};
use if_chain::if_chain;
use rustc_ast::LitKind;
use rustc_hir::{
    def::{DefKind, Res},
    intravisit::{walk_local, Visitor},
    Expr, ExprKind, HirId, Node, QPath,
};
use rustc_lint::LateContext;
use std::collections::HashMap;

use crate::get_expr_hir_id_opt;

/// Analyzes expressions to determine if they are constants or known at compile-time.
pub struct ConstantAnalyzer<'a, 'tcx> {
    pub cx: &'a LateContext<'tcx>,
    pub current_constant: Option<Constant<'tcx>>,
    pub constants: HashMap<HirId, Option<Constant<'tcx>>>,
}

impl<'a, 'tcx> ConstantAnalyzer<'a, 'tcx> {
    pub fn new(cx: &'a LateContext<'tcx>) -> Self {
        Self {
            cx,
            current_constant: None,
            constants: HashMap::new(),
        }
    }

    /// Checks if a QPath refers to a constant.
    fn is_qpath_constant(&self, path: &QPath) -> bool {
        if let QPath::Resolved(_, path) = path {
            match path.res {
                Res::Def(def_kind, def_id) => {
                    matches!(
                        def_kind,
                        DefKind::AnonConst
                            | DefKind::AssocConst
                            | DefKind::Const
                            | DefKind::InlineConst
                    ) || {
                        // Allow both Some and Ok variant constructors
                        if let DefKind::Ctor(..) = def_kind {
                            let def_path = self.cx.tcx.def_path_str(def_id);
                            def_path.ends_with("::Some") || def_path.ends_with("::Ok")
                        } else {
                            false
                        }
                    }
                }
                Res::Local(hir_id) => self.constants.contains_key(&hir_id),
                _ => false,
            }
        } else {
            false
        }
    }

    /// Determines if an expression is constant or known at compile-time.
    fn is_expr_constant(&mut self, expr: &Expr<'tcx>) -> bool {
        if let Some(const_val) = constant(self.cx, self.cx.typeck_results(), expr) {
            self.current_constant = Some(const_val);
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
            ExprKind::Lit(lit) => {
                match lit.node {
                    LitKind::Int(val, _) => {
                        self.current_constant = Some(Constant::Int(val));
                    }
                    _ => {}
                };
                true
            }
            ExprKind::Path(qpath_expr) => self.is_qpath_constant(&qpath_expr),
            ExprKind::Repeat(repeat_expr, _) => self.is_expr_constant(repeat_expr),
            ExprKind::Struct(_, expr_fields, _) => expr_fields
                .iter()
                .all(|field_expr| self.is_expr_constant(field_expr.expr)),
            ExprKind::Call(func, args) => {
                self.is_expr_constant(func) && args.iter().all(|arg| self.is_expr_constant(arg))
            }
            _ => false,
        }
    }

    /// Checks if an array index operation results in a constant value.
    fn is_array_index_constant(
        &mut self,
        array_expr: &Expr<'tcx>,
        index_expr: &Expr<'tcx>,
    ) -> bool {
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
                    if let Node::Local(let_stmt) = self.cx.tcx.hir_node(hir_id);
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
    fn is_array_element_constant(&mut self, elements: &[Expr<'tcx>], index: u128) -> bool {
        elements
            .get(index as usize)
            .map_or(false, |element| self.is_expr_constant(element))
    }

    /// Public method to check if an expression is constant.
    pub fn is_constant(&mut self, expr: &Expr<'tcx>) -> bool {
        self.is_expr_constant(expr)
    }

    pub fn get_constant(&self, expr: &Expr<'tcx>) -> Option<Constant<'tcx>> {
        if let Some(constant) = constant(self.cx, self.cx.typeck_results(), expr) {
            return Some(constant);
        }

        let hir_id = get_expr_hir_id_opt(expr)?;
        if let Some(constant) = self.constants.get(&hir_id) {
            return constant.clone();
        }
        None
    }
}

impl<'a, 'tcx> Visitor<'tcx> for ConstantAnalyzer<'a, 'tcx> {
    fn visit_local(&mut self, l: &'tcx rustc_hir::Local<'tcx>) {
        if let Some(init) = l.init {
            if self.is_expr_constant(init) {
                self.constants
                    .insert(l.pat.hir_id, self.current_constant.take());
            }
        }
        walk_local(self, l);
    }
}
