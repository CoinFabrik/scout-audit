#![feature(rustc_private)]
#![warn(unused_extern_crates)]

extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;

use clippy_utils::diagnostics::span_lint_and_help;
use common::{
    analysis::{hir_utils::get_expr_hir_id_opt, is_soroban_storage, SorobanStorageType},
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use rustc_hir::{
    intravisit::{walk_expr, walk_local, FnKind, Visitor},
    Body, BorrowKind, Expr, ExprKind, FnDecl, HirId, LetStmt, Mutability, Pat, PatKind,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::ty::Ref;
use rustc_span::{def_id::LocalDefId, Span};
use std::collections::HashMap;

const LINT_MESSAGE: &str =
    "detects re-reads of modified storage variables without intervening writes";

#[expose_lint_info]
pub static UNCACHED_STORAGE_MODIFICATION_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "Checks for storage variables that are read, modified, and then re-read without being written back to storage.",
    severity: Severity::Medium,
    help: "Write the modified value back to storage before re-reading it.",
    vulnerability_class: VulnerabilityClass::BestPractices,
};

dylint_linting::impl_late_lint! {
    pub UNCACHED_STORAGE_MODIFICATION,
    Warn,
    LINT_MESSAGE,
    UncachedStorageModification
}

#[derive(Default)]
pub struct UncachedStorageModification;

impl<'tcx> LateLintPass<'tcx> for UncachedStorageModification {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: FnKind<'tcx>,
        _: &'tcx FnDecl<'tcx>,
        body: &'tcx Body<'tcx>,
        _: Span,
        _: LocalDefId,
    ) {
        let mut visitor = StateVisitor {
            cx,
            dirty_state: HashMap::new(),
            key_map: HashMap::new(),
        };
        visitor.visit_expr(body.value);
    }
}

pub struct StateVisitor<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    dirty_state: HashMap<HirId, CopyState>,
    key_map: HashMap<HirId, HirId>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum CopyState {
    Clean,
    Modified(Span),
}

impl<'a, 'tcx> Visitor<'tcx> for StateVisitor<'a, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx Expr<'tcx>) {
        // Control Flow Handling
        if let ExprKind::If(cond, then, else_opt) = expr.kind {
            self.visit_expr(cond);

            let pre_state = self.dirty_state.clone();

            self.visit_expr(then);
            let state_then = self.dirty_state.clone();

            self.dirty_state = pre_state.clone(); // Restore for else

            let state_else = if let Some(else_expr) = else_opt {
                self.visit_expr(else_expr);
                self.dirty_state.clone()
            } else {
                pre_state // Else branch does nothing (keeps pre-state)
            };

            // Merge logic: Union of modifications
            self.dirty_state = self.merge_states(state_then, state_else);
            return;
        }

        match expr.kind {
            ExprKind::MethodCall(path, receiver, args, span) => {
                let method_name = path.ident.as_str();

                // Heuristic type check: ensure receiver is related to Soroban storage
                let is_storage_call = self.is_soroban_storage_type(receiver);

                if is_storage_call && method_name == "get" {
                    if let Some(key_expr) = args.first() {
                        if let Some(key_id) = self.get_local_var_id(key_expr) {
                            // Check for stale read
                            for (var_id, state) in &self.dirty_state {
                                if let Some(mapped_key_id) = self.key_map.get(var_id) {
                                    if *mapped_key_id == key_id {
                                        if let CopyState::Modified(mod_span) = state {
                                            span_lint_and_help(
                                                self.cx,
                                                UNCACHED_STORAGE_MODIFICATION,
                                                span,
                                                "uncached storage modification detected",
                                                Some(*mod_span),
                                                "variable modified here but not written back before re-read",
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                } else if is_storage_call && matches!(method_name, "set") {
                    // Write barrier
                    if let Some(key_expr) = args.first() {
                        if let Some(key_id) = self.get_local_var_id(key_expr) {
                            self.clean_vars_for_key(key_id);
                        }
                    }
                }

                self.check_method_receiver_mutation(expr, receiver, expr.span);

                for arg in args {
                    self.check_arg_is_mut_borrow(arg, expr.span);
                }
            }
            ExprKind::Assign(lhs, _, _) | ExprKind::AssignOp(_, lhs, _) => {
                // Check if we are modifying a tracked variable
                if let Some(hir_id) = get_expr_hir_id_opt(lhs) {
                    if self.dirty_state.contains_key(&hir_id) {
                        self.dirty_state
                            .insert(hir_id, CopyState::Modified(expr.span));
                    }
                }
            }
            ExprKind::Call(_, args) => {
                for arg in args {
                    self.check_arg_is_mut_borrow(arg, expr.span);
                }
            }
            _ => {}
        }

        walk_expr(self, expr);
    }

    fn visit_local(&mut self, local: &'tcx LetStmt<'tcx>) {
        if let Some(init) = local.init {
            if let Some(args) = Self::find_storage_get_args(init) {
                if let Some(key_expr) = args.first() {
                    if let Some(key_id) = self.get_local_var_id(key_expr) {
                        self.process_pat(local.pat, key_id);
                    }
                }
            }
        }

        walk_local(self, local);
    }
}

impl<'a, 'tcx> StateVisitor<'a, 'tcx> {
    fn is_soroban_storage_type(&self, expr: &'tcx Expr<'tcx>) -> bool {
        let ty = self.cx.typeck_results().expr_ty(expr);
        let ty = ty.peel_refs();
        is_soroban_storage(self.cx, ty, SorobanStorageType::Any)
    }

    fn check_arg_is_mut_borrow(&mut self, arg: &'tcx Expr<'tcx>, span: Span) {
        if let ExprKind::AddrOf(BorrowKind::Ref, Mutability::Mut, sub_expr) = arg.kind {
            if let Some(hir_id) = get_expr_hir_id_opt(sub_expr) {
                if self.dirty_state.contains_key(&hir_id) {
                    self.dirty_state.insert(hir_id, CopyState::Modified(span));
                }
            }
        }
    }

    fn check_method_receiver_mutation(
        &mut self,
        expr: &'tcx Expr<'tcx>,
        receiver: &'tcx Expr<'tcx>,
        span: Span,
    ) {
        if let Some(def_id) = self.cx.typeck_results().type_dependent_def_id(expr.hir_id) {
            let sig = self.cx.tcx.fn_sig(def_id).skip_binder();
            if let Some(self_ty) = sig.inputs().skip_binder().first() {
                if let Ref(_, _, Mutability::Mut) = self_ty.kind() {
                    if let Some(hir_id) = get_expr_hir_id_opt(receiver) {
                        if self.dirty_state.contains_key(&hir_id) {
                            self.dirty_state.insert(hir_id, CopyState::Modified(span));
                        }
                    }
                }
            }
        }
    }

    fn find_storage_get_args(expr: &'tcx Expr<'tcx>) -> Option<&'tcx [Expr<'tcx>]> {
        match expr.kind {
            ExprKind::MethodCall(path, receiver, args, _) => {
                let name = path.ident.as_str();
                if name == "get" {
                    return Some(args);
                }
                if name == "unwrap" || name == "unwrap_or" || name == "expect" {
                    return Self::find_storage_get_args(receiver);
                }
                None
            }
            _ => None,
        }
    }

    fn merge_states(
        &self,
        s1: HashMap<HirId, CopyState>,
        s2: HashMap<HirId, CopyState>,
    ) -> HashMap<HirId, CopyState> {
        let mut result = s1;
        for (k, v2) in s2 {
            match (result.get(&k), v2) {
                // If it was already modified in s1, keep it.
                // If s2 has it modified, mark it modified.
                (Some(CopyState::Modified(_)), _) => {}
                (_, CopyState::Modified(span)) => {
                    result.insert(k, CopyState::Modified(span));
                }
                // If both are clean or unset, it stays as is.
                _ => {}
            }
        }
        result
    }

    fn process_pat(&mut self, pat: &'tcx Pat<'tcx>, key_id: HirId) {
        if let PatKind::Binding(_, hir_id, _, _) = pat.kind {
            self.dirty_state.insert(hir_id, CopyState::Clean);
            self.key_map.insert(hir_id, key_id);
        }
    }

    fn get_local_var_id(&self, expr: &'tcx Expr<'tcx>) -> Option<HirId> {
        let mut curr = expr;
        while let ExprKind::AddrOf(_, _, sub) = curr.kind {
            curr = sub;
        }
        get_expr_hir_id_opt(curr)
    }

    fn clean_vars_for_key(&mut self, key_id: HirId) {
        for (var_id, mapped_key_id) in &self.key_map {
            if *mapped_key_id == key_id {
                self.dirty_state.insert(*var_id, CopyState::Clean);
            }
        }
    }
}
