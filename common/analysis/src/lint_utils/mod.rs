extern crate rustc_driver;
extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;
extern crate rustc_type_ir;
extern crate rustc_lint;

use rustc_hir::{Expr, ExprKind, LetStmt, QPath};
use rustc_span::Symbol;
use rustc_ast::{Label, LitIntType, LitKind};
use rustc_hir::{
    def::Res,
    BindingMode, Block, ExprField, HirId, LangItem, LoopSource, MatchSource,
    Pat, PatField, PatKind, Path, PathSegment, StmtKind, Ty, BorrowKind, Mutability,
};
use rustc_middle::ty::{Interner, TyCtxt, TyKind};
use rustc_span::{symbol::Ident, Span};

pub fn get_receiver_ident_name(receiver: &Expr) -> Symbol {
    if let ExprKind::Path(QPath::Resolved(_, path)) = &receiver.kind {
        let name = path.segments.first().map(|segment| segment.ident.name);
        return name.unwrap_or(Symbol::intern(""));
    }
    Symbol::intern("")
}

//---------------------------------------------------------------------

pub fn type_to_adt<'hir>(
    kind: &'hir rustc_type_ir::TyKind<TyCtxt<'hir>>,
) -> Result<
    (
        &'hir <TyCtxt<'hir> as Interner>::AdtDef,
        &'hir <TyCtxt<'hir> as Interner>::GenericArgs,
    ),
    (),
> {
    if let TyKind::Adt(a, b) = kind {
        Ok((&a, &b))
    } else {
        Err(())
    }
}

//---------------------------------------------------------------------

pub fn stmt_to_expr<'hir>(kind: &'hir StmtKind<'hir>) -> Result<&'hir Expr<'hir>, ()> {
    if let StmtKind::Expr(a) = kind {
        Ok(a)
    } else {
        Err(())
    }
}

//---------------------------------------------------------------------

pub fn expr_to_drop_temps<'hir>(kind: &'hir ExprKind<'hir>) -> Result<&'hir Expr<'hir>, ()> {
    if let ExprKind::DropTemps(a) = kind {
        Ok(a)
    } else {
        Err(())
    }
}

pub fn expr_to_match<'hir>(
    kind: &'hir ExprKind<'hir>,
) -> Result<(&'hir Expr<'hir>, &'hir [rustc_hir::Arm<'hir>], MatchSource), ()> {
    if let ExprKind::Match(a, b, c) = kind {
        Ok((a, b, *c))
    } else {
        Err(())
    }
}

pub fn expr_to_call<'hir>(
    kind: &'hir ExprKind<'hir>,
) -> Result<(&'hir Expr<'hir>, &'hir [Expr<'hir>]), ()> {
    if let ExprKind::Call(a, b) = kind {
        Ok((a, b))
    } else {
        Err(())
    }
}

pub fn expr_to_path<'hir>(kind: &'hir ExprKind<'hir>) -> Result<QPath<'hir>, ()> {
    if let ExprKind::Path(a) = kind {
        Ok(*a)
    } else {
        Err(())
    }
}

pub fn expr_to_struct<'hir>(
    kind: &'hir ExprKind<'hir>,
) -> Result<
    (
        &'hir QPath<'hir>,
        &'hir [ExprField<'hir>],
        Option<&'hir Expr<'hir>>,
    ),
    (),
> {
    if let ExprKind::Struct(a, b, c) = kind {
        Ok((a, b, *c))
    } else {
        Err(())
    }
}

pub fn expr_to_lit<'hir>(kind: &'hir ExprKind<'hir>) -> Result<&'hir rustc_hir::Lit, ()> {
    if let ExprKind::Lit(a) = kind {
        Ok(a)
    } else {
        Err(())
    }
}

pub fn expr_to_loop<'hir>(
    kind: &'hir ExprKind<'hir>,
) -> Result<(&'hir Block<'hir>, &Option<Label>, LoopSource, &Span), ()> {
    if let ExprKind::Loop(a, b, c, d) = kind {
        Ok((a, b, *c, d))
    } else {
        Err(())
    }
}

pub fn expr_to_method_call<'hir>(
    kind: &'hir ExprKind<'hir>,
) -> Result<
    (
        &'hir PathSegment<'hir>,
        &'hir Expr<'hir>,
        &'hir [Expr<'hir>],
        Span,
    ),
    (),
> {
    if let ExprKind::MethodCall(a, b, c, d) = kind {
        Ok((a, b, c, *d))
    } else {
        Err(())
    }
}

//---------------------------------------------------------------------

pub fn path_to_lang_item(path: &QPath) -> Result<(LangItem, Span), ()> {
    if let QPath::LangItem(a, b) = path {
        Ok((*a, *b))
    } else {
        Err(())
    }
}

pub fn path_to_resolved<'hir>(
    path: &'hir QPath<'hir>,
) -> Result<(&'hir Option<&'hir Ty<'hir>>, &'hir Path<'hir>), ()> {
    if let QPath::Resolved(a, b) = path {
        Ok((a, b))
    } else {
        Err(())
    }
}

//---------------------------------------------------------------------

pub fn resolution_to_local(resolution: &Res) -> Result<&HirId, ()> {
    if let Res::Local(a) = resolution {
        Ok(a)
    } else {
        Err(())
    }
}

//---------------------------------------------------------------------

pub fn lit_to_int(kind: &LitKind) -> Result<(u128, LitIntType), ()> {
    if let LitKind::Int(a, b) = kind {
        Ok((a.get(), *b))
    } else {
        Err(())
    }
}

//---------------------------------------------------------------------

pub fn pattern_to_struct<'hir>(
    pat: &'hir PatKind<'hir>,
) -> Result<(&QPath<'hir>, &'hir [PatField<'hir>], bool), ()> {
    if let PatKind::Struct(a, b, c) = pat {
        Ok((a, b, *c))
    } else {
        Err(())
    }
}

pub fn pattern_to_binding<'hir>(
    pat: &'hir PatKind<'hir>,
) -> Result<(&BindingMode, &HirId, &Ident, &Option<&'hir Pat<'hir>>), ()> {
    if let PatKind::Binding(a, b, c, d) = pat {
        Ok((a, b, c, d))
    } else {
        Err(())
    }
}

//---------------------------------------------------------------------

pub fn is_range(item: LangItem) -> bool {
    matches!(
        item,
        LangItem::Range | LangItem::RangeInclusiveStruct | LangItem::RangeInclusiveNew
    )
}

//---------------------------------------------------------------------

pub fn type_to_path<'hir>(kind: &'hir rustc_hir::TyKind<'hir>) -> Result<&'hir QPath<'hir>, ()> {
    if let rustc_hir::TyKind::Path(a) = kind {
        Ok(a)
    } else {
        Err(())
    }
}

//---------------------------------------------------------------------

pub fn expr_to_address_of<'hir>(
    kind: &'hir ExprKind<'hir>,
) -> Result<(&BorrowKind, &Mutability, &'hir Expr<'hir>), ()> {
    if let ExprKind::AddrOf(a, b, c) = kind {
        Ok((a, b, c))
    } else {
        Err(())
    }
}

//---------------------------------------------------------------------

pub fn path_to_type_relative<'hir>(
    path: &'hir QPath<'hir>,
) -> Result<(&'hir Ty<'hir>, &'hir PathSegment<'hir>), ()> {
    if let QPath::TypeRelative(a, b) = path {
        Ok((a, b))
    } else {
        Err(())
    }
}

//---------------------------------------------------------------------

pub fn get_node_type<'a>(cx: &rustc_lint::LateContext<'a>, hir_id: &HirId) -> rustc_middle::ty::Ty<'a> {
    cx.typeck_results().node_type(*hir_id)
}

pub fn definition_to_string<'a>(cx: &rustc_lint::LateContext<'a>, did: rustc_hir::def_id::DefId) -> String{
    cx
        .get_def_path(did)
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

pub fn get_type_string<'a, 'hir>(cx: &rustc_lint::LateContext<'a>, hir_id: &HirId) -> Result<String, ()>{
    let (def, _generic_args) = type_to_adt(get_node_type(cx, hir_id).kind())?;
    Ok(definition_to_string(cx, def.did()))
}

pub fn stmt_to_local<'hir>(
    kind: &'hir StmtKind<'hir>,
) -> Result<
    &'hir LetStmt<'hir>,
    (),
> {
    if let StmtKind::Let(a) = kind {
        Ok(a)
    } else {
        Err(())
    }
}

/*
fn type_to_adt<'hir>(
    kind: &'hir rustc_type_ir::TyKind<TyCtxt<'hir>>,
) -> Result<
    (
        &'hir <TyCtxt<'hir> as Interner>::AdtDef,
        &'hir <TyCtxt<'hir> as Interner>::GenericArgs,
    ),
    (),
> {
    if let rustc_type_ir::TyKind::Adt(a, b) = kind {
        Ok((&a, &b))
    } else {
        Err(())
    }
}
*/
