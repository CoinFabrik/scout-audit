#![feature(rustc_private)]

extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;

use clippy_wrappers::span_lint;
use if_chain::if_chain;
use rustc_hir::{
    intravisit::{walk_expr, FnKind, Visitor},
    Body, Expr, ExprKind, FnDecl,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::ty::{Ty, TyKind};
use rustc_span::{def_id::LocalDefId, Span, Symbol};
use utils::{get_node_type_opt, is_soroban_storage, SorobanStorageType};

const LINT_MESSAGE: &str = "Using dynamic types in instance or persistent storage can lead to unnecessary growth or storage-related vulnerabilities.";

dylint_linting::impl_late_lint! {
    pub DYNAMIC_STORAGE,
    Warn,
    LINT_MESSAGE,
    DynamicStorage,
    {
        name: "Dynamic Storage Analyzer",
        long_message: "Using dynamic types in instance or persistent storage can lead to unnecessary growth or storage-related vulnerabilities.",
        severity: "Warning",
        help: "https://coinfabrik.github.io/scout-soroban/docs/detectors/dynamic-storage",
        vulnerability_class: "Resource Management",
    }
}

#[derive(Default)]
struct DynamicStorage;

impl<'tcx> LateLintPass<'tcx> for DynamicStorage {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: FnKind<'tcx>,
        _: &'tcx FnDecl<'tcx>,
        body: &'tcx Body<'tcx>,
        span: Span,
        _: LocalDefId,
    ) {
        if span.from_expansion() {
            return;
        }

        let mut storage_warn_visitor = DynamicStorageVisitor { cx };
        storage_warn_visitor.visit_body(body);
    }
}

struct DynamicStorageVisitor<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
}

impl<'a, 'tcx> Visitor<'tcx> for DynamicStorageVisitor<'a, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx Expr<'tcx>) {
        if_chain! {
            // Detect calls to `set` method
            if let ExprKind::MethodCall(path, receiver, args, _) = &expr.kind;
            if path.ident.name == Symbol::intern("set");
            // Get the type of the receiver and check if it is an instance or persistent storage
            if let Some(receiver_ty) = get_node_type_opt(self.cx, &receiver.hir_id);
            if is_soroban_storage(self.cx, receiver_ty, SorobanStorageType::Instance)
                || is_soroban_storage(self.cx, receiver_ty, SorobanStorageType::Persistent);
            // Check if the value being set is a dynamic type
            if args.len() >= 2;
            if let Some(value_type) = get_node_type_opt(self.cx, &args[1].hir_id);
            if is_dynamic_type(self.cx, &value_type);
            then {
                span_lint(self.cx, DYNAMIC_STORAGE, expr.span, LINT_MESSAGE)
            }
        }

        walk_expr(self, expr)
    }
}

fn is_dynamic_type(cx: &LateContext, ty: &Ty) -> bool {
    match ty.kind() {
        TyKind::Str => true,
        TyKind::Slice(_) => true,
        TyKind::Dynamic(..) => true,
        TyKind::Array(element_ty, _) => is_dynamic_type(cx, element_ty),
        TyKind::Adt(adt_def, _) => {
            let type_name = cx.tcx.item_name(adt_def.did());
            matches!(type_name.as_str(), "Vec" | "String" | "Map" | "LinkedList")
        }
        TyKind::RawPtr(ty, _) => is_dynamic_type(cx, ty),
        TyKind::Ref(_, ty, _) => is_dynamic_type(cx, ty),
        TyKind::Tuple(substs) => substs.iter().any(|ty| is_dynamic_type(cx, &ty)),
        _ => false,
    }
}
