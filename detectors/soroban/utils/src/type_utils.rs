extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_middle;
extern crate rustc_span;

use rustc_hir::{FnDecl, FnRetTy, HirId, QPath};
use rustc_lint::LateContext;
use rustc_middle::ty::{Ty, TyKind};
use rustc_span::Symbol;

/// Get the type of a node, if it exists.
pub fn get_node_type_opt<'tcx>(cx: &LateContext<'tcx>, hir_id: &HirId) -> Option<Ty<'tcx>> {
    cx.typeck_results().node_type_opt(*hir_id)
}

/// Match the type of an expression to a string.
pub fn match_type_to_str(cx: &LateContext<'_>, expr_type: Ty<'_>, type_str: &str) -> bool {
    match expr_type.kind() {
        TyKind::Adt(adt_def, _) => cx.tcx.def_path_str(adt_def.did()).contains(type_str),
        TyKind::Ref(_, ty, _) => match_type_to_str(cx, *ty, type_str),
        _ => false,
    }
}

/// Check the return type of a function.
pub fn fn_returns(decl: &FnDecl<'_>, type_symbol: Symbol) -> bool {
    if let FnRetTy::Return(ty) = decl.output {
        matches!(ty.kind, rustc_hir::TyKind::Path(QPath::Resolved(_, path)) if path
            .segments
            .last()
            .map_or(false, |seg| seg.ident.name == type_symbol))
    } else {
        false
    }
}
