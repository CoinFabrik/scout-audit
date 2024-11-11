#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;
extern crate rustc_target;

use std::collections::{HashMap, HashSet};

use clippy_wrappers::span_lint_and_help;
use common::expose_lint_info;
use if_chain::if_chain;
use rustc_ast::ast::LitKind;
use rustc_hir::{
    def::Res,
    intravisit::{walk_expr, walk_local, FnKind, Visitor},
    Body, Expr, ExprKind, FnDecl, HirId, LetStmt, PatKind, QPath,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::ty::TyKind;
use rustc_span::{def_id::LocalDefId, Span, Symbol};
use rustc_target::abi::VariantIdx;

const LINT_MESSAGE:&str = "External calls could open the opportunity for a malicious contract to execute any arbitrary code";

#[expose_lint_info]
pub static REENTRANCY_2_INFO: LintInfo = LintInfo {
    name: "Reentrancy",
    short_message: LINT_MESSAGE,
    long_message: "An ink! smart contract can interact with other smart contracts. These operations imply (external) calls where control flow is passed to the called contract until the execution of the called code is over, then the control is delivered back to the caller. A reentrancy vulnerability may happen when a user calls a function, this function calls a malicious contract which again calls this same function, and this 'reentrancy' has unexpected reprecussions to the contract.",
    severity: "Critical",
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/reentrancy",
    vulnerability_class: "Reentrancy",
};

dylint_linting::declare_late_lint! {
    /// ### What it does
    /// This linting rule checks whether the 'check-effect' interaction pattern has been properly followed by code that invokes a contract that may call back the original one.
    /// ### Why is this bad?
    /// If state modifications are made after a contract call, reentrant calls may not detect these modifications, potentially leading to unexpected behaviors such as double spending.
    /// ### Known problems
    /// If called method does not perform a malicious reentrancy (i.e. known method from known contract) false positives will arise.
    /// If the usage of set_allow_reentry(true) or later state changes are performed in an auxiliary function, this detector will not detect the reentrancy.
    ///
    /// ### Example
    /// ```rust
    /// let caller_addr = self.env().caller();
    /// let caller_balance = self.balance(caller_addr);
    ///
    /// if amount > caller_balance {
    ///     return Ok(caller_balance);
    /// }
    ///
    /// let call = build_call::<ink::env::DefaultEnvironment>()
    ///     .call(address)
    ///     .transferred_value(amount)
    ///     .exec_input(ink::env::call::ExecutionInput::new(Selector::new(
    ///         selector.to_be_bytes(),
    ///     )))
    ///     .call_flags(ink::env::CallFlags::default().set_allow_reentry(true))
    ///     .returns::<()>()
    ///     .params();
    /// self.env()
    ///     .invoke_contract(&call)
    ///     .map_err(|_| Error::ContractInvokeFailed)?
    ///     .map_err(|_| Error::ContractInvokeFailed)?;
    ///
    /// let new_balance = caller_balance.checked_sub(amount).ok_or(Error::Underflow)?;
    /// self.balances.insert(caller_addr, &new_balance);
    /// ```
    /// Use instead:
    /// ```rust
    /// let caller_addr = self.env().caller();
    /// let caller_balance = self.balances.get(caller_addr).unwrap_or(0);
    /// if amount <= caller_balance {
    ///     //The balance is updated before the contract call
    ///     self.balances
    ///         .insert(caller_addr, &(caller_balance - amount));
    ///     let call = build_call::<ink::env::DefaultEnvironment>()
    ///         .call(address)
    ///         .transferred_value(amount)
    ///         .exec_input(ink::env::call::ExecutionInput::new(Selector::new(
    ///             selector.to_be_bytes(),
    ///         )))
    ///         .call_flags(ink::env::CallFlags::default().set_allow_reentry(true))
    ///         .returns::<()>()
    ///         .params();
    ///     self.env()
    ///         .invoke_contract(&call)
    ///         .unwrap_or_else(|err| panic!("Err {:?}", err))
    ///         .unwrap_or_else(|err| panic!("LangErr {:?}", err));
    ///
    ///     return caller_balance - amount;
    /// } else {
    ///     return caller_balance;
    /// }
    /// ```
    pub REENTRANCY_2,
    Warn,
    LINT_MESSAGE
}

const SET_ALLOW_REENTRY: &str = "set_allow_reentry";
const INVOKE_CONTRACT: &str = "invoke_contract";
const INSERT: &str = "insert";
const MAPPING: &str = "Mapping";
const ACCOUNT_ID: &str = "AccountId";
const U128: &str = "u128";
const CALL_FLAGS: &str = "call_flags";
const ALLOW_REENTRY: &str = "ALLOW_REENTRY";

struct ReentrancyVisitor<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    tainted_contracts: HashSet<Symbol>,
    current_method: Option<Symbol>,
    bool_values: HashMap<HirId, bool>,
    reentrancy_spans: Vec<Span>,
    looking_for_insert: bool,
    found_insert: bool,
}

impl<'a, 'tcx> ReentrancyVisitor<'a, 'tcx> {
    fn mark_current_as_tainted(&mut self) {
        if let Some(method) = self.current_method.take() {
            self.tainted_contracts.insert(method);
        }
    }

    fn handle_set_allow_reentry(&mut self, args: &[Expr<'_>]) {
        let is_reentry_enabled = match &args[0].kind {
            ExprKind::Lit(lit) => matches!(lit.node, LitKind::Bool(true)),
            ExprKind::Path(qpath) => {
                if_chain! {
                    if let res = self.cx.qpath_res(qpath, args[0].hir_id);
                    if let Res::Local(_) = res;
                    if let QPath::Resolved(_, path) = qpath;
                    then {
                        path.segments.iter().any(|segment| {
                            if let Res::Local(hir_id) = segment.res {
                                self.bool_values.get(&hir_id).copied().unwrap_or(true)
                            } else {
                                false
                            }
                        })
                    } else {
                        false
                    }
                }
            }
            _ => false,
        };

        if is_reentry_enabled {
            self.mark_current_as_tainted();
        }
    }

    fn handle_invoke_contract(&mut self, args: &[Expr<'_>], expr: &Expr<'_>) {
        if_chain! {
            if let ExprKind::AddrOf(_, _, invoke_expr) = &args[0].kind;
            if let ExprKind::Path(qpath) = &invoke_expr.kind;
            if let QPath::Resolved(_, path) = qpath;
            then {
                for segment in path.segments.iter() {
                    if self.tainted_contracts.contains(&segment.ident.name) {
                        self.looking_for_insert = true;
                        self.reentrancy_spans.push(expr.span);
                    }
                }
            }
        }
    }

    fn handle_call_flags(&mut self, args: &[Expr<'_>]) {
        if_chain! {
            if let ExprKind::Path(qpath) = &args[0].kind;
            if let QPath::TypeRelative(_, segment) = qpath;
            if segment.ident.name.as_str() == ALLOW_REENTRY;
            then {
                self.mark_current_as_tainted();
            }
        }
    }

    fn handle_insert(&mut self, expr: &Expr<'_>) {
        if_chain! {
            if let ExprKind::MethodCall(_, receiver, _, _) = &expr.kind;
            if let object_type = self.cx.typeck_results().expr_ty(receiver);
            if let TyKind::Adt(adt_def, substs) = object_type.kind();
            if let Some(variant) = adt_def.variants().get(VariantIdx::from_u32(0));
            if variant.name.as_str() == MAPPING;
            then {
                let mut has_account_id = false;
                let mut has_u128 = false;

                substs.types().for_each(|ty| {
                    let type_str = ty.to_string();
                    has_account_id |= type_str.contains(ACCOUNT_ID);
                    has_u128 |= type_str.contains(U128);
                });

                self.found_insert = has_account_id && has_u128;
            }
        }
    }
}

impl<'a, 'tcx> Visitor<'tcx> for ReentrancyVisitor<'a, 'tcx> {
    fn visit_local(&mut self, local: &'tcx LetStmt<'tcx>) {
        if let Some(init) = &local.init {
            if let PatKind::Binding(_, _, ident, _) = &local.pat.kind {
                match &init.kind {
                    ExprKind::Lit(lit) => {
                        if let LitKind::Bool(value) = lit.node {
                            self.bool_values.insert(local.pat.hir_id, value);
                        }
                    }
                    ExprKind::MethodCall(_, _, _, _) => {
                        self.current_method = Some(ident.name);
                    }
                    ExprKind::Path(QPath::Resolved(_, path)) => {
                        if let Some(segment) = path.segments.last() {
                            if let Res::Local(hir_id) = segment.res {
                                if let Some(&value) = self.bool_values.get(&hir_id) {
                                    self.bool_values.insert(local.pat.hir_id, value);
                                }
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
        walk_local(self, local);
    }

    fn visit_expr(&mut self, expr: &'tcx Expr<'tcx>) {
        if let ExprKind::MethodCall(func, _, args, _) = &expr.kind {
            match func.ident.name.as_str() {
                SET_ALLOW_REENTRY => self.handle_set_allow_reentry(args),
                CALL_FLAGS => self.handle_call_flags(args),
                INVOKE_CONTRACT => self.handle_invoke_contract(args, expr),
                INSERT if self.looking_for_insert => self.handle_insert(expr),
                _ => (),
            }
        }
        walk_expr(self, expr);
    }
}

impl<'tcx> LateLintPass<'tcx> for Reentrancy2 {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: FnKind<'tcx>,
        _: &'tcx FnDecl<'_>,
        body: &'tcx Body<'_>,
        _: Span,
        _: LocalDefId,
    ) {
        let mut visitor = ReentrancyVisitor {
            cx,
            tainted_contracts: HashSet::new(),
            current_method: None,
            bool_values: HashMap::new(),
            reentrancy_spans: Vec::new(),
            looking_for_insert: false,
            found_insert: false,
        };
        walk_expr(&mut visitor, body.value);

        if visitor.found_insert {
            for span in visitor.reentrancy_spans {
                span_lint_and_help(
                    cx,
                    REENTRANCY_2,
                    span,
                    LINT_MESSAGE,
                    None,
                    "This statement seems to call another contract after the flag \
                     set_allow_reentry was enabled [todo: check state changes after this statement]",
                );
            }
        }
    }
}
