#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_span;

use clippy_utils::diagnostics::span_lint_and_help;
use common::{
    analysis::decomposers::{
        expr_to_block, expr_to_match, expr_to_path, expr_to_unary, path_to_resolved,
        path_to_type_relative, pattern_to_binding, pattern_to_struct, resolution_to_self_ty_alias,
        stmt_to_let, type_to_path,
    },
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use rustc_ast::{
    token::TokenKind, tokenstream::TokenTree, AssocItemKind, HasTokens, Item, ItemKind,
};
use rustc_hir::{Arm, Expr, ExprKind, LetStmt, Stmt};
use rustc_lint::{EarlyContext, EarlyLintPass, LateContext, LateLintPass};
use rustc_span::Span;
use std::{
    collections::HashMap,
    ops::Deref,
    sync::{Arc, Mutex},
};

const LINT_MESSAGE: &str =
    "Inconsistent weight attribute name: Each extrinsic must use its own unique weight calculation function.";

#[expose_lint_info]
pub static INCONSISTENT_EXTRINSIC_WEIGHT_NAME_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "The weight attribute is using a weight calculation function that doesn't match the extrinsic name. \
                    Each extrinsic must have its own dedicated weight calculation to accurately reflect its resource consumption. \
                    Reusing weight calculations from other functions can lead to incorrect resource estimation and potential issues in production.",
    severity: Severity::Enhancement,
    help: "https://coinfabrik.github.io/scout-audit/docs/detectors/substrate/invalid-weight-info",
    vulnerability_class: VulnerabilityClass::KnownBugs,
};

common_utils::declare_pre_expansion_and_late_lint! {
//dylint_linting::declare_pre_expansion_lint! {
    pub INCONSISTENT_EXTRINSIC_WEIGHT_NAME,
    Warn,
    LINT_MESSAGE
}
struct GlobalState {
    functions_to_reconsider: HashMap<String, (Span, Span)>,
}

impl GlobalState {
    pub fn new() -> Self {
        Self {
            functions_to_reconsider: HashMap::new(),
        }
    }
    fn get_state() -> Arc<Mutex<GlobalState>> {
        let mut gs = GLOBAL_STATE.lock().unwrap();
        match gs.deref() {
            None => {
                let ret =
                    Arc::<Mutex<GlobalState>>::new(Mutex::<GlobalState>::new(GlobalState::new()));
                *gs = Some(ret.clone());
                ret
            }
            Some(p) => p.clone(),
        }
    }
    pub fn add_function_to_be_reconsidered(function_name: String, span1: Span, span2: Span) {
        let gs = Self::get_state();
        let mut lock = gs.lock().unwrap();
        lock.functions_to_reconsider
            .insert(function_name, (span1, span2));
    }
    pub fn any_functions_to_reconsider() -> bool {
        let gs = Self::get_state();
        let lock = gs.lock().unwrap();
        !lock.functions_to_reconsider.is_empty()
    }
    pub fn functions_needs_reconsideration(s: &str) -> Option<(Span, Span)> {
        let gs = Self::get_state();
        let lock = gs.lock().unwrap();
        Some(*(lock.functions_to_reconsider.get(s)?))
    }
}

static GLOBAL_STATE: Mutex<Option<Arc<Mutex<GlobalState>>>> = Mutex::new(None);
struct WeightInfoValidator {
    pub function_name: String,
    pub collected_text: Vec<String>,
}

impl WeightInfoValidator {
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

    fn is_valid_weight_info(&self) -> bool {
        // Less than 4 elements means we are not in the weight attribute
        if self.collected_text.len() < 4 {
            return true;
        }

        // We are looking for the following pattern: #[pallet::weight(...::WeightInfo::FUNCTION_NAME)]
        // First two items should be "pallet" and "weight"
        let is_pallet_attr = self.collected_text[0] == "pallet";
        let is_weight_attr = self.collected_text[1] == "weight";

        // Last item should be the function name
        let function_name = &self.collected_text[self.collected_text.len() - 1];
        let is_function_attr = function_name == &self.function_name;

        // Second to last should be "WeightInfo"
        let second_last = &self.collected_text[self.collected_text.len() - 2];
        let is_weight_info = second_last == "WeightInfo";

        is_pallet_attr && is_weight_attr && is_function_attr && is_weight_info
    }
}

impl EarlyLintPass for InconsistentExtrinsicWeightName {
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
                            let mut validator = WeightInfoValidator::new(fn_name.clone());

                            let attr_token_stream = token_stream.to_attr_token_stream();
                            let attr_token_trees = attr_token_stream.to_token_trees();

                            validator.process_token_trees(&attr_token_trees);

                            if !validator.is_valid_weight_info() {
                                GlobalState::add_function_to_be_reconsidered(
                                    validator.function_name,
                                    impl_item.span,
                                    attr.span,
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

//Returns true if the expression has the form "*self"
fn is_star_self(expr: &'_ rustc_hir::Expr<'_>) -> bool {
    (|| -> Option<()> {
        let (f, op) = expr_to_unary(&expr.kind)?;
        if f != rustc_ast::UnOp::Deref {
            return None;
        }

        let path = expr_to_path(&op.kind)?;
        let (_, path) = path_to_resolved(&path)?;
        if path.segments.len() != 1 {
            return None;
        }

        let segment = path.segments.first().unwrap();
        if segment.ident.name.as_str() != "self" {
            None
        } else {
            Some(())
        }
    })()
    .is_some()
}

//Returns Some(identifier) if the pattern has the form "Self::" + identifier + "{}", otherwise returns None.
fn get_self_member(pattern: &'_ rustc_hir::Pat<'_>) -> Option<String> {
    let (path, fields, ellipsis) = pattern_to_struct(&pattern.kind)?;
    if ellipsis || !fields.is_empty() {
        return None;
    }

    let (ty, segment) = path_to_type_relative(path)?;
    let path = type_to_path(&ty.kind)?;
    let (_, path) = path_to_resolved(&path)?;
    let _ = resolution_to_self_ty_alias(&path.res)?;

    Some(segment.ident.name.to_ident_string())
}

//Returns Some(expr) iff the let statement has the form "let __pallet_base_weight = " expr.
fn get_pallet_base_weight_init<'a>(let_stmt: &'a LetStmt<'a>) -> Option<&'a Expr<'a>> {
    let (_, _, id, _) = pattern_to_binding(&let_stmt.pat.kind)?;
    if id.name.as_str() != "__pallet_base_weight" {
        return None;
    }

    let_stmt.init
}

fn process_init_call(function: &'_ Expr<'_>, function_being_analyzed: &str) -> Option<bool> {
    let path = expr_to_path(&function.kind)?;
    let (_, segment) = path_to_type_relative(&path)?;

    Some(segment.ident.name.as_str() == function_being_analyzed)
}

fn process_init(init: &'_ Expr<'_>, function_being_analyzed: &str) -> Option<bool> {
    match &init.kind {
        ExprKind::Call(function, _) => process_init_call(function, function_being_analyzed),
        ExprKind::Block(block, _) => {
            if !block.stmts.is_empty() {
                return None;
            }
            if let Some(ret) = block.expr {
                process_init(ret, function_being_analyzed)
            } else {
                None
            }
        }
        _ => None,
    }
}

fn process_statement<'a>(stmt: &'a Stmt<'a>) -> Option<&'a Expr<'a>> {
    let let_stmt = stmt_to_let(&stmt.kind)?;
    get_pallet_base_weight_init(let_stmt)
}

fn find_pallet_base_weight<'a>(expr: &'a rustc_hir::Expr<'a>) -> Option<&'a rustc_hir::Expr<'a>> {
    let (statements, _) = expr_to_block(&expr.kind)?;

    for stmt in statements.stmts.iter() {
        let init = process_statement(stmt);
        if init.is_some() {
            return init;
        }
    }

    None
}

fn process_branch(branch: &'_ Arm<'_>, cx: &LateContext<'_>) -> Option<()> {
    let function_name = get_self_member(branch.pat)?;
    let (span1, span2) = GlobalState::functions_needs_reconsideration(&function_name)?;
    let pallet_base_weight_expr = find_pallet_base_weight(branch.body)?;
    let valid_function = process_init(pallet_base_weight_expr, &function_name)?;
    if valid_function {
        return None;
    }
    span_lint_and_help(
        cx,
        INCONSISTENT_EXTRINSIC_WEIGHT_NAME,
        span1,
        LINT_MESSAGE,
        Some(span2),
        "used here",
    );
    None
}

fn analyze_get_dispatch_info(body: &'_ rustc_hir::Body<'_>, cx: &LateContext<'_>) -> Option<()> {
    let (block, _) = expr_to_block(&body.value.kind)?;
    let expr = block.expr?;
    let (match_expr, match_branches, match_source) = expr_to_match(&expr.kind)?;
    if match_source != rustc_hir::MatchSource::Normal || !is_star_self(match_expr) {
        return None;
    }

    for branch in match_branches.iter() {
        let _ = process_branch(branch, cx);
    }

    None
}

impl<'tcx> LateLintPass<'tcx> for InconsistentExtrinsicWeightName {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: rustc_hir::intravisit::FnKind<'tcx>,
        _: &'tcx rustc_hir::FnDecl<'tcx>,
        body: &'tcx rustc_hir::Body<'tcx>,
        _: Span,
        localdef: rustc_span::def_id::LocalDefId,
    ) {
        if !GlobalState::any_functions_to_reconsider() {
            return;
        }

        if cx.tcx.def_path_str(localdef)
            != "<pallet::Call<T> as frame_support::dispatch::GetDispatchInfo>::get_dispatch_info"
        {
            return;
        }

        let _ = analyze_get_dispatch_info(body, cx);
    }
}
