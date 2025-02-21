#![feature(rustc_private)]
#![feature(let_chains)]

extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;
extern crate rustc_ast;

use common::{
    analysis::{
        decomposers::{ expr_to_call, expr_to_path, path_to_resolved, resolved_to_def },
        get_receiver_ident_name,
    },
    declarations::{ Severity, VulnerabilityClass },
    macros::expose_lint_info,
};
use rustc_lint::{ EarlyContext, LateContext, LateLintPass, EarlyLintPass };
use rustc_ast::{
    token::TokenKind,
    tokenstream::TokenTree,
    AssocItemKind,
    HasTokens,
    Item,
    ItemKind,
};
use rustc_hir::{ intravisit::{ walk_expr, Visitor }, BinOpKind, Expr, ExprKind, PatKind };
use rustc_middle::ty::TyKind;
use rustc_span::{ def_id::DefId, Span };
use std::{ collections::HashMap, ops::Deref, sync::{ Arc, Mutex } };
const LINT_MESSAGE: &str =
    "Not checking for a zero value in the parameters may result in redundant operations";

#[expose_lint_info]
pub static MISSING_ZERO_CHECK_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "Failing to check for a zero value in the parameters may lead to unnecessary operations, potentially increasing resource usage and reducing the efficiency of the function. Consider checking the parameter to not be zero",
    severity: Severity::Minor,
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/missing-zero-check",
    vulnerability_class: VulnerabilityClass::BestPractices,
};

common_utils::declare_pre_expansion_and_late_lint! {
    pub MISSING_ZERO_CHECK,
    Warn,
    LINT_MESSAGE
}

#[derive(Default)]
pub struct MissingZeroCheckState {
    pub param_infos: Vec<ParamInfo>,
    pub extrinsic_functions: Vec<String>,
}

impl MissingZeroCheckState {
    pub fn new() -> Self {
        Self {
            param_infos: Vec::new(),
            extrinsic_functions: Vec::new(),
        }
    }
    fn get_state() -> Arc<Mutex<MissingZeroCheckState>> {
        let mut gs = MISSING_ZERO_CHECK_STATE.lock().unwrap();
        match gs.deref() {
            None => {
                let ret = Arc::<Mutex<MissingZeroCheckState>>::new(
                    Mutex::<MissingZeroCheckState>::new(MissingZeroCheckState::new())
                );
                *gs = Some(ret.clone());
                ret
            }
            Some(p) => p.clone(),
        }
    }
    pub fn add_param_info(param_name: &str, def_path: &str, span: Span, is_checked: bool) {
        let gs = Self::get_state();
        let mut lock = gs.lock().unwrap();
        lock.param_infos.push(ParamInfo {
            param_name: param_name.to_string(),
            def_path: def_path.to_string(),
            span,
            is_checked,
        });
    }
    pub fn add_extrinsic_function(fn_name: String) {
        let gs = Self::get_state();
        let mut lock = gs.lock().unwrap();
        lock.extrinsic_functions.push(fn_name);
    }
}
static MISSING_ZERO_CHECK_STATE: Mutex<Option<Arc<Mutex<MissingZeroCheckState>>>> =
    Mutex::new(None);

struct ExtrinsicFunctionValidator {
    pub function_name: String,
    pub collected_text: Vec<String>,
}
impl ExtrinsicFunctionValidator {
    fn new(function_name: String) -> Self {
        Self {
            function_name,
            collected_text: Vec::new(),
        }
    }

    fn process_token_trees(&mut self, trees: &[TokenTree]) {
        for tree in trees {
            match tree {
                TokenTree::Delimited(.., token_stream) => {
                    self.process_token_trees(&token_stream.trees().cloned().collect::<Vec<_>>());
                }
                TokenTree::Token(token, _) => {
                    if let TokenKind::Ident(symbol, _) = token.kind {
                        self.collected_text.push(symbol.to_string());
                    }
                }
            }
        }
    }
}
#[derive(Debug, Clone)]
pub struct ParamInfo {
    pub param_name: String,
    pub def_path: String,
    pub span: Span,
    pub is_checked: bool,
}
struct MissingZeroCheckFinder<'tcx, 'tcx_ref> {
    cx: &'tcx_ref LateContext<'tcx>,
    pub param_infos: Vec<ParamInfo>,
}

fn expr_to_def_id<'hir>(kind: &'hir ExprKind<'hir>) -> Option<DefId> {
    let (call, _) = expr_to_call(kind)?;
    let qpath = expr_to_path(&call.kind)?;
    let (_, path) = path_to_resolved(&qpath)?;
    let (_, def_id) = resolved_to_def(&path.res)?;
    Some(*def_id)
}

impl<'tcx> Visitor<'tcx> for MissingZeroCheckFinder<'tcx, '_> {
    fn visit_expr(&mut self, expr: &'tcx Expr<'_>) {
        if let ExprKind::Binary(op, rvalue, lvalue) = expr.kind {
            if matches!(op.node, BinOpKind::Ne | BinOpKind::Eq) {
                let rvalue_name = get_receiver_ident_name(rvalue);
                let lvalue_name = get_receiver_ident_name(lvalue);
                // If one of the values in the binary operation is a function parameter, verify if the other value is zero.
                self.param_infos.iter_mut().for_each(|param| {
                    let actual_fn = self.cx.tcx.def_path_str(expr.hir_id.owner);
                    if param.def_path == actual_fn {
                        if
                            param.param_name == lvalue_name.to_string() ||
                            param.param_name == rvalue_name.to_string()
                        {
                            if let Some(def_id) = expr_to_def_id(&lvalue.kind) {
                                let name = self.cx.tcx.def_path_str(def_id);
                                if name.to_string().contains("zero") {
                                    param.is_checked = true;
                                }
                            }
                        }
                    }
                });
            }
        }
        walk_expr(self, expr);
    }
}

impl EarlyLintPass for MissingZeroCheck {
    fn check_item(&mut self, _: &EarlyContext<'_>, item: &Item) {
        if let ItemKind::Impl(impl_) = &item.kind {
            for impl_item in &impl_.items {
                if let AssocItemKind::Fn(..) = &impl_item.kind {
                    let fn_name = impl_item.ident.name.to_string();

                    for attr in &impl_item.attrs {
                        if attr.is_doc_comment() {
                            continue;
                        }

                        if let Some(token_stream) = attr.tokens() {
                            let mut validator = ExtrinsicFunctionValidator::new(fn_name.clone());

                            let attr_token_stream = token_stream.to_attr_token_stream();
                            let attr_token_trees = attr_token_stream.to_token_trees();

                            validator.process_token_trees(&attr_token_trees);
                            if
                                validator.collected_text.starts_with(
                                    &["pallet".to_string(), "call_index".to_string()]
                                ) ||
                                validator.collected_text.starts_with(
                                    &["pallet".to_string(), "weight".to_string()]
                                )
                            {
                                MissingZeroCheckState::add_extrinsic_function(fn_name.clone());
                            }
                        }
                    }
                }
            }
        }
    }
}

impl<'tcx> LateLintPass<'tcx> for MissingZeroCheck {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: rustc_hir::intravisit::FnKind<'tcx>,
        _: &'tcx rustc_hir::FnDecl<'tcx>,
        body: &'tcx rustc_hir::Body<'tcx>,
        _: Span,
        localdef: rustc_span::def_id::LocalDefId
    ) {
        let mut utf_storage = MissingZeroCheckFinder {
            cx,
            param_infos: Vec::default(),
        };
        let gs = MissingZeroCheckState::get_state();
        let extrinsic_functions = {
            let lock = gs.lock().unwrap();
            utf_storage.param_infos = lock.param_infos.clone();
            lock.extrinsic_functions.clone()
        };
        // Look for function params with AccountId type
        let mir_body = cx.tcx.optimized_mir(localdef);
        let fn_name = &cx.tcx.def_path_str(localdef);
        if extrinsic_functions.iter().any(|func| fn_name.ends_with(func.as_str())) {
            for (arg, hir_param) in mir_body.args_iter().zip(body.params.iter()) {
                let fn_name = &cx.tcx.def_path_str(localdef);
                if let TyKind::Alias(_alias_kind, alias_ty) = mir_body.local_decls[arg].ty.kind() {
                    let def_id = &cx.tcx.def_path_str(alias_ty.def_id);
                    if
                        def_id.contains("traits::Currency::Balance") ||
                        def_id.contains("Config::Balance")
                    {
                        let mut param_name = "";
                        if let PatKind::Binding(_, _, ident, _) = &hir_param.pat.kind {
                            param_name = ident.name.as_str();
                        }

                        if !fn_name.contains("new_call_variant") {
                            MissingZeroCheckState::add_param_info(
                                param_name,
                                fn_name,
                                mir_body.local_decls[arg].source_info.span,
                                false
                            );
                        }
                    }
                }
            }
        }
        walk_expr(&mut utf_storage, body.value);
        let mut lock = gs.lock().unwrap();

        utf_storage.param_infos
            .clone()
            .iter()
            .for_each(|p| {
                if p.is_checked {
                    lock.param_infos.retain(|param| {
                        param.param_name != p.param_name || param.def_path != p.def_path
                    });
                }
            });
    }

    fn check_crate_post(&mut self, cx: &LateContext<'tcx>) {
        let gs = MissingZeroCheckState::get_state();
        let lock = gs.lock().unwrap();
        lock.param_infos
            .iter()
            .for_each(|p| {
                clippy_utils::diagnostics::span_lint(cx, MISSING_ZERO_CHECK, p.span, LINT_MESSAGE)
            });
    }
}
