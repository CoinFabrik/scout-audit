#![feature(rustc_private)]

extern crate rustc_hir;
extern crate rustc_span;

use clippy_utils::diagnostics::span_lint_and_help;
use common::{
    analysis::{get_node_type_opt, is_soroban_function, is_soroban_map, is_soroban_vec},
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use rustc_hir::{intravisit::FnKind, Body, FnDecl, Param};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::{
    def_id::{DefId, LocalDefId},
    Span,
};
use std::collections::{HashMap, HashSet};

const LINT_MESSAGE: &str =
    "Avoid accepting soroban_sdk::Vec or Map parameters without validating their contents.";

#[expose_lint_info]
pub static AVOID_VEC_MAP_INPUT_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "Be careful of accepting Vec and Map<K, V> data types as functions’ inputs! \
        The Val is a raw value of the Soroban smart contract platform that types can be converted to and \
        from for storing, or passing between contracts. When the elements of a Vec or Map<K, V> are \
        transmitted to the Host environment, they are converted to Vals. However, there is no guarantee \
        that these values can be properly converted back to their expected types (T for vectors or K, V \
        for maps). Without proper validation, storing them in the contract can be dangerous, as attempting \
        to retrieve and use them later could halt contract execution.",
    severity: Severity::Medium,
    help: "https://coinfabrik.github.io/scout-audit/docs/detectors/soroban/avoid-vec-map-input",
    vulnerability_class: VulnerabilityClass::BestPractices,
};

dylint_linting::impl_late_lint! {
    pub AVOID_VEC_MAP_INPUT,
    Warn,
    LINT_MESSAGE,
    AvoidVecMapInput::default()
}

struct FlaggedParam {
    span: Span,
    ty_name: String,
}

#[derive(Default)]
struct AvoidVecMapInput {
    checked_functions: HashSet<String>,
    flagged_params: HashMap<DefId, Vec<FlaggedParam>>,
}

impl<'tcx> LateLintPass<'tcx> for AvoidVecMapInput {
    fn check_crate_post(&mut self, cx: &LateContext<'tcx>) {
        for (def_id, params) in &self.flagged_params {
            if is_soroban_function(cx, &self.checked_functions, def_id) {
                for param in params {
                    span_lint_and_help(
                        cx,
                        AVOID_VEC_MAP_INPUT,
                        param.span,
                        LINT_MESSAGE,
                        None,
                        format!(
                            "Validate `{}` contents or convert them to contract-defined types before using or storing them.",
                            param.ty_name
                        ),
                    );
                }
            }
        }
    }

    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: FnKind<'tcx>,
        _: &'tcx FnDecl<'tcx>,
        body: &'tcx Body<'tcx>,
        span: Span,
        local_def_id: LocalDefId,
    ) {
        let def_id = local_def_id.to_def_id();
        self.checked_functions.insert(cx.tcx.def_path_str(def_id));

        if span.from_expansion() {
            return;
        }

        let flagged_params: Vec<_> = body
            .params
            .iter()
            .filter_map(|param| detect_vec_or_map_param(cx, param))
            .collect();

        if !flagged_params.is_empty() {
            self.flagged_params.insert(def_id, flagged_params);
        }
    }
}

fn detect_vec_or_map_param<'tcx>(
    cx: &LateContext<'tcx>,
    param: &Param<'tcx>,
) -> Option<FlaggedParam> {
    let ty = get_node_type_opt(cx, &param.hir_id)?;

    if !(is_soroban_vec(cx, ty) || is_soroban_map(cx, ty)) {
        return None;
    }

    Some(FlaggedParam {
        span: param.span,
        ty_name: ty.to_string(),
    })
}
