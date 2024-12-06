#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_span;
extern crate rustc_middle;
extern crate rustc_errors;

use clippy_utils::diagnostics::{
    span_lint_and_sugg,
    span_lint_and_help,
};
use common::{
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use rustc_hir::{
    def_id::LocalDefId,
    intravisit::{walk_expr, FnKind, Visitor},
    Body, Expr, FnDecl,
};
use rustc_errors::Applicability;
use rustc_lint::{
    LateContext,
    LateLintPass,
};
use rustc_span::Span;
use common::analysis::{
    get_node_type,
    decomposers::*,
};
use std::{
    collections::{
        HashSet,
        HashMap,
    },
    ops::Deref,
    sync::{
        Arc,
        Mutex,
    },
};

const LINT_MESSAGE: &str = "Saturating arithmetic may silently generate incorrect results.";
const F: bool = false;
const T: bool = true;
const RELEVANT_FUNCTIONS: [(&str, bool, bool); 12] = [
    ("saturating_add",        T, T),
    ("saturating_add_signed", T, F),
    ("saturating_sub",        T, T),
    ("saturating_mul",        T, T),
    ("saturating_div",        T, F),
    ("saturating_pow",        T, T),
    ("saturating_less_one",   F, T),
    ("saturating_plus_one",   F, T),
    ("saturating_inc",        F, T),
    ("saturating_dec",        F, T),
    ("saturating_accrue",     F, T),
    ("saturating_reduce",     F, T),
];
const IGNORED_FUNCTIONS: [&str; 1] = [
    "size_hint",
];
const IGNORED_TYPES: [&str; 1] = [
    "sp_weights::weight_v2::Weight",
];

#[expose_lint_info]
pub static SATURATING_ARITHMETIC_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: LINT_MESSAGE,
    severity: Severity::Critical,
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/incorrect-exponentiation",
    vulnerability_class: VulnerabilityClass::Arithmetic,
};

dylint_linting::declare_late_lint! {
    pub SATURATING_ARITHMETIC,
    Warn,
    LINT_MESSAGE
}

#[derive(Clone)]
struct FunctionAvailability{
    pub available_on_std: bool,
    pub available_on_substrate: bool,
}

impl FunctionAvailability{
    pub fn new(available_on_std: bool, available_on_substrate: bool) -> Self{
        Self{
            available_on_std,
            available_on_substrate,
        }
    }
}

struct GlobalState{
    relevant_functions: HashMap<String, FunctionAvailability>,
    ignored_functions: HashSet<String>,
    ignored_types: HashSet<String>,
}

impl GlobalState{
    pub fn new() -> Self{
        Self{
            relevant_functions: RELEVANT_FUNCTIONS
                .iter()
                .map(|(x, y, z)| (x.to_string(), FunctionAvailability::new(*y, *z)))
                .collect(),
            ignored_functions: Self::to_hash_set(&IGNORED_FUNCTIONS[..]),
            ignored_types: Self::to_hash_set(&IGNORED_TYPES[..]),
        }
    }
    fn to_hash_set(strings: &[&str]) -> HashSet<String>{
        strings
            .iter()
            .map(|x| x.to_string())
            .collect::<HashSet<_>>()
    }
    fn get_state() -> Arc<Mutex<GlobalState>>{
        let mut gs = GLOBAL_STATE.lock().unwrap();
        match gs.deref(){
            None => {
                let ret = Arc::<Mutex<GlobalState>>::new(Mutex::<GlobalState>::new(GlobalState::new()));
                *gs = Some(ret.clone());
                ret
            },
            Some(p) => p.clone(),
        }
    }
    pub fn method_is_relevant(method_name: &str) -> Option<FunctionAvailability>{
        let gs = Self::get_state();
        let lock = gs.lock().unwrap();
        Some(lock.relevant_functions.get(method_name)?.clone())
    }
    pub fn type_is_ignored(name: &Option<String>) -> bool{
        if let Some(name) = name{
            let gs = Self::get_state();
            let lock = gs.lock().unwrap();
            lock.ignored_types.contains(name.as_str())
        }else{
            false
        }
    }
    pub fn function_is_ignored(name: &String) -> bool{
        let gs = Self::get_state();
        let lock = gs.lock().unwrap();
        lock.ignored_functions.contains(name.as_str())
    }
}

static GLOBAL_STATE: Mutex<Option<Arc<Mutex<GlobalState>>>> = Mutex::new(None);

fn detect_saturating_call<'tcx>(cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) -> Option<()>{
    let (method_name, receiver, _args, _span) = expr_to_method_call(&expr.kind)?;
    let method_name_str = method_name.ident.name.as_str();
    let _availability = GlobalState::method_is_relevant(method_name_str)?;
    let node_type = get_node_type(cx, &receiver.hir_id);
    if GlobalState::type_is_ignored(&common::analysis::ty_to_string(cx, &node_type)){
        None?;
    }
    span_lint_and_sugg(
        cx,
        SATURATING_ARITHMETIC,
        method_name.ident.span,
        LINT_MESSAGE,
        "Instead of overflowing, saturating arithmetic clamps the result to the representation limit for the data type. Consider checked arithmetic instead",
        format!("checked_{}", &method_name_str[11..]),
        Applicability::MaybeIncorrect,
    );
    None
}

fn ident<'a>(f: &FnKind<'a>) -> String{
    match f{
        FnKind::ItemFn(id, _, _) => id.name.to_ident_string(),
        FnKind::Method(id, _) => id.name.to_ident_string(),
        FnKind::Closure => "<closure>".to_string()
    }
}

struct SaturatingFinder<'a, 'b>{
    cx: &'b LateContext<'a>,
}

impl<'a, 'b> Visitor<'a> for SaturatingFinder<'a, 'b> {
    fn visit_expr(&mut self, expr: &'a rustc_hir::Expr<'a>) {
        let _ = detect_saturating_call(self.cx, expr);
        walk_expr(self, expr);
    }
}

impl<'tcx> LateLintPass<'tcx> for SaturatingArithmetic {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        kind: FnKind<'tcx>,
        _: &'tcx FnDecl<'tcx>,
        body: &'tcx Body<'tcx>,
        _: Span,
        _: LocalDefId,
    ){
        if GlobalState::function_is_ignored(&ident(&kind)){
            return;
        }

        SaturatingFinder{
            cx,
        }.visit_expr(body.value);
    }
}
