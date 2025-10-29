extern crate rustc_data_structures;
extern crate rustc_hir;

use rustc_data_structures::fx::FxHashSet;
use rustc_hir::{Expr, ExprKind, HirId, QPath};
use std::collections::HashMap;

/// Analyzes expressions for linear forms and structural equivalence
pub struct ExprAnalyzer<'a, 'tcx> {
    locals: &'a HashMap<HirId, &'tcx Expr<'tcx>>,
}

impl<'a, 'tcx> ExprAnalyzer<'a, 'tcx> {
    pub fn new(locals: &'a HashMap<HirId, &'tcx Expr<'tcx>>) -> Self {
        Self { locals }
    }

    /// Check if two expressions are structurally equivalent
    pub fn are_equivalent(&self, expr1: &Expr<'tcx>, expr2: &Expr<'tcx>) -> bool {
        self.are_equivalent_impl(expr1, expr2, &mut FxHashSet::default())
    }

    fn are_equivalent_impl(
        &self,
        expr1: &Expr<'tcx>,
        expr2: &Expr<'tcx>,
        visited: &mut FxHashSet<(HirId, HirId)>,
    ) -> bool {
        if expr1.hir_id == expr2.hir_id {
            return true;
        }

        match (&expr1.kind, &expr2.kind) {
            (ExprKind::Lit(l1), ExprKind::Lit(l2)) => l1.node == l2.node,
            (ExprKind::Path(p1), ExprKind::Path(p2)) => self.are_paths_equiv(p1, p2, visited),
            (ExprKind::Binary(op1, l1, r1), ExprKind::Binary(op2, l2, r2)) => {
                op1.node == op2.node
                    && self.are_equivalent_impl(l1, l2, visited)
                    && self.are_equivalent_impl(r1, r2, visited)
            }
            (ExprKind::Unary(op1, i1), ExprKind::Unary(op2, i2)) => {
                op1 == op2 && self.are_equivalent_impl(i1, i2, visited)
            }
            (ExprKind::Cast(i1, t1), ExprKind::Cast(i2, t2)) => {
                t1.hir_id == t2.hir_id && self.are_equivalent_impl(i1, i2, visited)
            }
            (ExprKind::DropTemps(i1), ExprKind::DropTemps(i2)) => {
                self.are_equivalent_impl(i1, i2, visited)
            }
            (ExprKind::Block(b1, _), ExprKind::Block(b2, _)) => match (b1.expr, b2.expr) {
                (Some(e1), Some(e2)) => self.are_equivalent_impl(e1, e2, visited),
                (None, None) => true,
                _ => false,
            },
            (ExprKind::DropTemps(inner), _) => self.are_equivalent_impl(inner, expr2, visited),
            (_, ExprKind::DropTemps(inner)) => self.are_equivalent_impl(expr1, inner, visited),
            (ExprKind::Block(b, _), _) if b.expr.is_some() => {
                self.are_equivalent_impl(b.expr.unwrap(), expr2, visited)
            }
            (_, ExprKind::Block(b, _)) if b.expr.is_some() => {
                self.are_equivalent_impl(expr1, b.expr.unwrap(), visited)
            }
            _ => false,
        }
    }

    fn are_paths_equiv(
        &self,
        p1: &QPath<'tcx>,
        p2: &QPath<'tcx>,
        visited: &mut FxHashSet<(HirId, HirId)>,
    ) -> bool {
        match (p1, p2) {
            (QPath::Resolved(_, path1), QPath::Resolved(_, path2)) => {
                match (path1.res, path2.res) {
                    (rustc_hir::def::Res::Local(id1), rustc_hir::def::Res::Local(id2)) => {
                        id1 == id2
                            || {
                                let key = (id1, id2);
                                visited.insert(key)
                                    && {
                                        let result = matches!(
                                            (self.locals.get(&id1), self.locals.get(&id2)),
                                            (Some(i1), Some(i2)) if self.are_equivalent_impl(i1, i2, visited)
                                        );
                                        visited.remove(&key);
                                        result
                                    }
                            }
                    }
                    _ => path1.res == path2.res,
                }
            }
            _ => false,
        }
    }
}

/// Collect local variable bindings from a function body
pub fn collect_locals<'tcx>(
    body: &'tcx rustc_hir::Body<'tcx>,
    locals: &mut HashMap<HirId, &'tcx Expr<'tcx>>,
) {
    use rustc_hir::{
        intravisit::{walk_body, walk_local, Visitor},
        LetStmt, PatKind,
    };

    struct Collector<'a, 'tcx> {
        locals: &'a mut HashMap<HirId, &'tcx Expr<'tcx>>,
    }

    impl<'tcx> Visitor<'tcx> for Collector<'_, 'tcx> {
        fn visit_local(&mut self, local: &'tcx LetStmt<'tcx>) {
            if let (Some(init), PatKind::Binding(..)) = (local.init, &local.pat.kind) {
                self.locals.insert(local.pat.hir_id, init);
            }
            walk_local(self, local);
        }
    }

    walk_body(&mut Collector { locals }, body);
}
