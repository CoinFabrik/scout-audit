extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;
extern crate rustc_type_ir;
extern crate rustc_lint;

use rustc_ast::{BindingMode, Label, LitIntType, LitKind};
use rustc_hir::{
    def::Res,
    Block, Expr, ExprField, ExprKind, HirId, LangItem, LoopSource, MatchSource, Pat, PatField,
    PatKind, Path, QPath, StmtKind, Ty, PathSegment,
};
use rustc_middle::ty::{TyCtxt, TyKind};
use rustc_span::{symbol::Ident, Span};
use rustc_type_ir::Interner;

pub fn type_to_adt<'hir>(
    kind: &'hir rustc_type_ir::TyKind<TyCtxt<'hir>>,
) -> Option<
    (
        &'hir <TyCtxt<'hir> as Interner>::AdtDef,
        &'hir <TyCtxt<'hir> as Interner>::GenericArgs,
    )
> {
    if let TyKind::Adt(a, b) = kind {
        Some((a, b))
    } else {
        None
    }
}

//---------------------------------------------------------------------

pub fn stmt_to_expr<'hir>(kind: &'hir StmtKind<'hir>) -> Option<&'hir Expr<'hir>> {
    if let StmtKind::Expr(a) = kind {
        Some(a)
    } else {
        None
    }
}

//---------------------------------------------------------------------

pub fn expr_to_drop_temps<'hir>(kind: &'hir ExprKind<'hir>) -> Option<&'hir Expr<'hir>> {
    if let ExprKind::DropTemps(a) = kind {
        Some(a)
    } else {
        None
    }
}

pub fn expr_to_match<'hir>(
    kind: &'hir ExprKind<'hir>,
) -> Option<(&'hir Expr<'hir>, &'hir [rustc_hir::Arm<'hir>], MatchSource)> {
    if let ExprKind::Match(a, b, c) = kind {
        Some((a, b, *c))
    } else {
        None
    }
}

pub fn expr_to_call<'hir>(
    kind: &'hir ExprKind<'hir>,
) -> Option<(&'hir Expr<'hir>, &'hir [Expr<'hir>])> {
    if let ExprKind::Call(a, b) = kind {
        Some((a, b))
    } else {
        None
    }
}

pub fn expr_to_method_call<'hir>(
    kind: &'hir ExprKind<'hir>,
) -> Option<(&'hir PathSegment<'hir>, &'hir Expr<'hir>, &'hir [Expr<'hir>], Span)> {
    if let ExprKind::MethodCall(a, b, c, d) = kind {
        Some((a, b, c, *d))
    } else {
        None
    }
}

pub fn expr_to_path<'hir>(kind: &'hir ExprKind<'hir>) -> Option<QPath<'hir>> {
    if let ExprKind::Path(a) = kind {
        Some(*a)
    } else {
        None
    }
}

pub fn expr_to_struct<'hir>(
    kind: &'hir ExprKind<'hir>,
) -> Option<
    (
        &'hir QPath<'hir>,
        &'hir [ExprField<'hir>],
        Option<&'hir Expr<'hir>>,
    ),
> {
    if let ExprKind::Struct(a, b, c) = kind {
        Some((a, b, *c))
    } else {
        None
    }
}

pub fn expr_to_lit<'hir>(kind: &'hir ExprKind<'hir>) -> Option<&'hir rustc_hir::Lit> {
    if let ExprKind::Lit(a) = kind {
        Some(a)
    } else {
        None
    }
}

pub fn expr_to_loop<'hir>(
    kind: &'hir ExprKind<'hir>,
) -> Option<(&'hir Block<'hir>, &Option<Label>, LoopSource, &Span)> {
    if let ExprKind::Loop(a, b, c, d) = kind {
        Some((a, b, *c, d))
    } else {
        None
    }
}

//---------------------------------------------------------------------

pub fn path_to_lang_item(path: &QPath) -> Option<(LangItem, Span)> {
    if let QPath::LangItem(a, b) = path {
        Some((*a, *b))
    } else {
        None
    }
}

pub fn path_to_resolved<'hir>(
    path: &'hir QPath<'hir>,
) -> Option<(&'hir Option<&'hir Ty<'hir>>, &'hir Path<'hir>)> {
    if let QPath::Resolved(a, b) = path {
        Some((a, b))
    } else {
        None
    }
}

//---------------------------------------------------------------------

pub fn resolution_to_local(resolution: &Res) -> Option<&HirId> {
    if let Res::Local(a) = resolution {
        Some(a)
    } else {
        None
    }
}

//---------------------------------------------------------------------

pub fn lit_to_int(kind: &LitKind) -> Option<(u128, LitIntType)> {
    if let LitKind::Int(a, b) = kind {
        Some((a.get(), *b))
    } else {
        None
    }
}

//---------------------------------------------------------------------

pub fn pattern_to_struct<'hir>(
    pat: &'hir PatKind<'hir>,
) -> Option<(&QPath<'hir>, &'hir [PatField<'hir>], bool)> {
    if let PatKind::Struct(a, b, c) = pat {
        Some((a, b, *c))
    } else {
        None
    }
}

pub fn pattern_to_binding<'hir>(
    pat: &'hir PatKind<'hir>,
) -> Option<(&BindingMode, &HirId, &Ident, &Option<&'hir Pat<'hir>>)> {
    if let PatKind::Binding(a, b, c, d) = pat {
        Some((a, b, c, d))
    } else {
        None
    }
}