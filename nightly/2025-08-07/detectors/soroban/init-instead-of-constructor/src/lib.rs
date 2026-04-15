#![feature(rustc_private)]

extern crate rustc_hir;
extern crate rustc_span;

use clippy_utils::diagnostics::span_lint;
use common::{
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::Span;
use std::vec;

const LINT_MESSAGE: &str = "Use the constructor pattern to initialize the contract";

#[expose_lint_info]
pub static TOKEN_INTERFACE_EVENTS_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "Initialization functions may be sniped by attackers before the deployer has a chance to call them, whereas constructors are guaranteed to be called at constract deployment",
    severity: Severity::Medium,
    help: "https://coinfabrik.github.io/scout-audit/docs/detectors/soroban/init-instead-of-constructor",
    vulnerability_class: VulnerabilityClass::BestPractices,
};

dylint_linting::impl_late_lint! {
    pub INIT_INSTEAD_OF_CONSTRUCTOR,
    Warn,
    LINT_MESSAGE,
    InitInsteadOfConstructor::default()
}

static INITIALIZATION_FUNCTION: [&str; 2] = ["init", "initialize"];

#[derive(Default)]
struct InitInsteadOfConstructor {}

fn is_relevant_function(name: &str) -> bool {
    for s in INITIALIZATION_FUNCTION {
        if name.ends_with(&format!("::{s}")) && !name.ends_with(&format!("Client::<'a>::{s}")) {
            return true;
        }
    }
    false
}

impl<'tcx> LateLintPass<'tcx> for InitInsteadOfConstructor {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: rustc_hir::intravisit::FnKind<'tcx>,
        _: &'tcx rustc_hir::FnDecl<'tcx>,
        _: &'tcx rustc_hir::Body<'tcx>,
        span: Span,
        local_def_id: rustc_span::def_id::LocalDefId,
    ) {
        let def_id = local_def_id.to_def_id();
        let fn_name = cx.tcx.def_path_str(def_id);
        if !is_relevant_function(&fn_name) {
            return;
        }

        //TODO: This can be improved by getting the span of the function name
        //      and suggesting replacing with __constructor. Unfortunately this
        //      cannot be done at this time because FnKind doesn't expose that
        //      span. We can look into this again after updating the nightly
        //      version.
        span_lint(cx, INIT_INSTEAD_OF_CONSTRUCTOR, span, LINT_MESSAGE);
    }
}
