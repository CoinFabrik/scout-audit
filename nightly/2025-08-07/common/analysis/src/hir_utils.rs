extern crate rustc_ast;
extern crate rustc_hir;

use rustc_ast::LitKind;
use rustc_hir::{def::Res, Expr, ExprKind, HirId, QPath, TyKind};

pub fn get_expr_hir_id_opt(expr: &Expr<'_>) -> Option<HirId> {
    match &expr.kind {
        ExprKind::Path(qpath) => match qpath {
            QPath::Resolved(_, path) => match path.res {
                Res::Local(id) => Some(id),
                _ => None,
            },
            QPath::TypeRelative(ty, _) => match &ty.kind {
                TyKind::Path(_) => Some(ty.hir_id),
                _ => None,
            },
            QPath::LangItem(_, _) => Some(expr.hir_id),
        },
        ExprKind::Lit(lit) => match lit.node {
            LitKind::Int(_, _) => Some(expr.hir_id),
            _ => None,
        },
        _ => None,
    }
}
