use clippy_utils::{peel_blocks, sym};
use edit_distance::edit_distance;
use rustc_hir::{def_id::DefId, Expr, ExprKind, UnOp};
use rustc_lint::LateContext;
use rustc_span::Symbol;
use std::{collections::HashMap, hash::Hash};

const STORAGE_GET_METHODS: [Symbol; 2] = [sym::get, sym::get_unchecked];
const UNWRAP_METHODS: [Symbol; 5] = [
    sym::unwrap,
    sym::expect,
    sym::unwrap_or,
    sym::unwrap_or_else,
    sym::unwrap_or_default,
];

pub fn is_initialize_fn(cx: &LateContext<'_>, def_id: DefId) -> bool {
    cx.tcx
        .opt_item_name(def_id)
        .is_some_and(|name| name.as_str() == "initialize")
}

pub fn is_unwrap_method(name: Symbol) -> bool {
    UNWRAP_METHODS.contains(&name)
}

pub fn is_get_method(name: Symbol) -> bool {
    STORAGE_GET_METHODS.contains(&name)
}

pub fn strip_identity<'tcx>(expr: &'tcx Expr<'tcx>) -> &'tcx Expr<'tcx> {
    let expr = peel_blocks(expr);

    match expr.kind {
        ExprKind::AddrOf(_, _, inner) => strip_identity(inner),
        ExprKind::Unary(UnOp::Deref, inner) => strip_identity(inner),
        ExprKind::MethodCall(seg, receiver, _args, _)
            if matches!(seg.ident.name, sym::clone | sym::to_owned | sym::into) =>
        {
            strip_identity(receiver)
        }
        _ => expr,
    }
}

/// Check if a name matches privileged role patterns (admin, owner, etc.)
pub fn is_privileged_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    const TARGETS: [&str; 14] = [
        "admin",
        "owner",
        "new_admin",
        "new_owner",
        "newadmin",
        "newowner",
        "next_admin",
        "next_owner",
        "nextadmin",
        "nextowner",
        "pending_admin",
        "pending_owner",
        "pendingadmin",
        "pendingowner",
    ];
    TARGETS
        .iter()
        .any(|target| edit_distance(&lower, target) <= 1)
}

/// Helper to get a slice from a HashMap<K, Vec<V>>, returning empty slice if key not found
pub fn get_vec_slice<'a, K, V>(map: &'a HashMap<K, Vec<V>>, key: &K) -> &'a [V]
where
    K: Eq + Hash,
{
    map.get(key).map(Vec::as_slice).unwrap_or(&[])
}
