#![feature(rustc_private)]

extern crate rustc_hir;
extern crate rustc_span;

use common::{
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use if_chain::if_chain;
use rustc_hir::{
    def_id::LocalDefId,
    intravisit::{walk_expr, FnKind, Visitor},
    Body, Expr, ExprKind, FnDecl, QPath,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::Span;

const LINT_MESSAGE:&str = "Abitrary users should not have control over keys because it implies writing any value of left mapping, lazy variable, or the main struct of the contract located in position 0 of the storage";

#[expose_lint_info]
pub static SET_CONTRACT_STORAGE_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "In ink! the function set_contract_storage(key: &K, value: &V) can be used to modify the contract storage under a given key. When a smart contract uses this function, the contract needs to check if the caller should be able to alter this storage. If this does not happen, an arbitary caller may modify balances and other relevant contract storage.    ",
    severity: Severity::Critical,
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/set-contract-storage",
    vulnerability_class: VulnerabilityClass::Authorization,
};

dylint_linting::declare_late_lint! {
    pub SET_CONTRACT_STORAGE,
    Warn,
    LINT_MESSAGE
}

fn expr_check_owner(cx: &LateContext, expr: &Expr) -> bool {
    let expr_type = common::analysis::get_node_type_opt(cx, &expr.hir_id);
    if let Some(expr_type) = expr_type {
        expr_type
            .to_string()
            .contains("ink::ink_primitives::AccountId")
    } else {
        false
    }
}

fn expr_check_caller(expr: &Expr) -> bool {
    if let ExprKind::MethodCall(func, ..) = expr.kind {
        let function_name = func.ident.name.to_string();
        function_name.contains("caller")
    } else {
        false
    }
}

struct SetContractStorageVisitor<'tcx, 'a> {
    span: Option<Span>,
    cx: &'a LateContext<'tcx>,
    unprotected: bool,
    in_conditional: bool,
    has_caller_in_if: bool,
    has_owner_in_if: bool,
    has_set_contract: bool,
}

impl<'tcx, 'a> Visitor<'tcx> for SetContractStorageVisitor<'tcx, 'a> {
    fn visit_expr(&mut self, expr: &'tcx Expr<'_>) {
        if self.in_conditional {
            if let ExprKind::Binary(_, left, right) = &expr.kind {
                self.has_owner_in_if =
                    expr_check_owner(self.cx, right) || expr_check_owner(self.cx, left);
                self.has_caller_in_if = expr_check_caller(right) || expr_check_caller(left);
            }
        }
        if let ExprKind::If(..) = &expr.kind {
            self.in_conditional = true;
            walk_expr(self, expr);
            self.in_conditional = false;
        } else if let ExprKind::Call(callee, _) = expr.kind {
            if_chain! {
                if let ExprKind::Path(method_path) = &callee.kind;
                if let QPath::Resolved(None, path) = *method_path;
                if path.segments.len() == 2;
                if path.segments[0].ident.name.as_str() == "env";
                if path.segments[1].ident.name.as_str() == "set_contract_storage";
                then {
                    self.has_set_contract = true;
                    if !self.in_conditional && (!self.has_owner_in_if || !self.has_caller_in_if) {
                            self.unprotected = true;
                            self.span = Some(expr.span);
                    }
                }
            }
        }
        walk_expr(self, expr);
    }
}

impl<'tcx> LateLintPass<'tcx> for SetContractStorage {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: FnKind<'tcx>,
        _: &'tcx FnDecl<'_>,
        body: &'tcx Body<'_>,
        _: Span,
        _: LocalDefId,
    ) {
        let mut reentrant_storage = SetContractStorageVisitor {
            span: None,
            cx,
            unprotected: false,
            in_conditional: false,
            has_caller_in_if: false,
            has_owner_in_if: false,
            has_set_contract: false,
        };

        walk_expr(&mut reentrant_storage, body.value);

        if reentrant_storage.has_set_contract && reentrant_storage.unprotected {
            clippy_utils::diagnostics::span_lint_and_help(
                cx,
                SET_CONTRACT_STORAGE,
                reentrant_storage.span.unwrap(),
                LINT_MESSAGE,
                None,
                "Set access control and proper authorization validation for the set_contract_storage() function"
            );
        }
    }
}
