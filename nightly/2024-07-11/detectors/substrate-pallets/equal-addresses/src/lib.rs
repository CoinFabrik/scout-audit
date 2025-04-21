#![feature(rustc_private)]
#![feature(let_chains)]

extern crate rustc_error_messages;
extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;

use common::{
    analysis::{get_node_type_opt, get_receiver_ident_name},
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use if_chain::if_chain;
use rustc_error_messages::MultiSpan;
use rustc_hir::{
    intravisit::{walk_expr, Visitor},
    BinOpKind, Expr, ExprKind, PatKind,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::{
    mir::{BasicBlock, BasicBlockData, BasicBlocks, Const, Operand, TerminatorKind},
    ty::TyKind,
};
use rustc_span::{def_id::DefId, Span};

const LINT_MESSAGE: &str =
    "Not checking for a difference in the addresses could lead to unexpected behavior or security vulnerabilities";

#[expose_lint_info]
pub static EQUAL_ADDRESSES_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "Functions that receive two addresses as parameters should include a check to ensure they are not the same. Failing to verify this condition could lead to unexpected behavior. It is recommended to add an explicit check to verify that the addresses are different before proceeding with further logic.",
    severity: Severity::Minor,
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/equal-addresses",
    vulnerability_class: VulnerabilityClass::ErrorHandling,
};

dylint_linting::impl_late_lint! {
    pub EQUAL_ADDRESSES,
    Warn,
    LINT_MESSAGE,
    EqualAddresses::default()
}

#[derive(Default)]
pub struct EqualAddresses {
    pub param_infos: Vec<ParamInfo>,
    pub term_infos: Vec<TerminateInfo>,
}
impl EqualAddresses {
    pub fn new() -> Self {
        Self {
            param_infos: Vec::new(),
            term_infos: Vec::new(),
        }
    }
    pub fn add_param_info(
        &mut self,
        param_name: &str,
        def_path: &str,
        span: Span,
        is_checked: bool,
    ) {
        self.param_infos.push(ParamInfo {
            param_name: param_name.to_string(),
            def_path: def_path.to_string(),
            span,
            is_checked,
        });
    }
    pub fn add_term_info(&mut self, param_names: [String; 2], def_path: String) {
        self.term_infos.push(TerminateInfo {
            param_names,
            def_path,
        });
    }
    pub fn update_param_info(&mut self, def_path: String) {
        for param_info in self.param_infos.iter_mut() {
            if param_info.def_path == def_path {
                param_info.is_checked = true;
            }
        }
    }
}
#[derive(Debug)]
pub struct ParamInfo {
    pub param_name: String,
    pub def_path: String,
    pub span: Span,
    pub is_checked: bool,
}
#[derive(Debug, Clone)]
pub struct TerminateInfo {
    pub param_names: [String; 2],
    pub def_path: String,
}
struct EqualAddressesFinder<'tcx, 'tcx_ref> {
    cx: &'tcx_ref LateContext<'tcx>,
    terminate_contract_span: Option<Span>,
    terminate_contract_def_id: Option<DefId>,
    possible_terminate: Vec<Option<TerminateInfo>>,
}

struct Terminates<'tcx> {
    terminates: Vec<(&'tcx BasicBlockData<'tcx>, BasicBlock)>,
    terminates_info: Vec<TerminateInfo>,
}

impl<'tcx> Visitor<'tcx> for EqualAddressesFinder<'tcx, '_> {
    fn visit_expr(&mut self, expr: &'tcx Expr<'_>) {
        if let ExprKind::Binary(op, rvalue, lvalue) = expr.kind {
            if BinOpKind::Ne == op.node || BinOpKind::Eq == op.node {
                let rtype = get_node_type_opt(self.cx, &rvalue.hir_id).unwrap();
                let ltype = get_node_type_opt(self.cx, &lvalue.hir_id).unwrap();

                if_chain!(
                    if rtype.to_string() == "<T as frame_system::Config>::AccountId";
                    if ltype.to_string() == "<T as frame_system::Config>::AccountId";
                    then {
                        self.terminate_contract_span = Some(expr.span);
                        self.terminate_contract_def_id = self.cx.typeck_results().type_dependent_def_id(expr.hir_id);
                        let rvalue = get_receiver_ident_name(rvalue);
                        let lvalue = get_receiver_ident_name(lvalue);
                        self.possible_terminate.push(Some(TerminateInfo {
                            param_names: [rvalue.to_string(),lvalue.to_string()],
                            def_path: self.cx.tcx.def_path_str(expr.hir_id.owner),
                        }));
                    }
                );
            }
        }

        walk_expr(self, expr);
    }
}

fn find_terminate_in_mir<'tcx>(
    bbs: &'tcx BasicBlocks<'tcx>,
    terminate_def_id: Option<DefId>,
    possible_terminate: Vec<Option<TerminateInfo>>,
) -> Terminates {
    let mut terminates_vec = Terminates {
        terminates: vec![],
        terminates_info: vec![],
    };
    for (bb, bb_data) in bbs.iter().enumerate() {
        if bb_data.terminator.as_ref().is_none() {
            continue;
        }
        let terminator = bb_data.terminator.clone().unwrap();
        if let TerminatorKind::Call { func, .. } = &terminator.kind {
            if let Operand::Constant(fn_const) = func
                && let Const::Val(_const_val, ty) = fn_const.const_
                && let TyKind::FnDef(def, _subs) = ty.kind()
                && terminate_def_id.is_some_and(|d| &d == def)
            {
                if !possible_terminate.is_empty() {
                    possible_terminate.iter().for_each(|terminate_info| {
                        if let Some(ref info) = terminate_info {
                            terminates_vec.terminates_info.push(info.clone());
                        }
                    });
                }
                terminates_vec
                    .terminates
                    .push((bb_data, BasicBlock::from_usize(bb)));
            }
        }
    }
    terminates_vec
}

impl<'tcx> LateLintPass<'tcx> for EqualAddresses {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: rustc_hir::intravisit::FnKind<'tcx>,
        _: &'tcx rustc_hir::FnDecl<'tcx>,
        body: &'tcx rustc_hir::Body<'tcx>,
        _: Span,
        localdef: rustc_span::def_id::LocalDefId,
    ) {
        let mut utf_storage = EqualAddressesFinder {
            cx,
            terminate_contract_def_id: None,
            terminate_contract_span: None,
            possible_terminate: Vec::default(),
        };

        // Look for function params with AccountId type
        let mir_body = cx.tcx.optimized_mir(localdef);
        for (arg, hir_param) in mir_body.args_iter().zip(body.params.iter()) {
            if mir_body.local_decls[arg].ty.to_string() == "<T as frame_system::Config>::AccountId"
                || mir_body.local_decls[arg].ty.to_string()
                    == "<T as frame_system::Config>::RuntimeOrigin"
            {
                let fn_name = &cx.tcx.def_path_str(localdef);
                let mut param_name = "";
                if let PatKind::Binding(_, _, ident, _) = &hir_param.pat.kind {
                    param_name = ident.name.as_str();
                }
                self.add_param_info(
                    param_name,
                    fn_name,
                    mir_body.local_decls[arg].source_info.span,
                    false,
                );
            }
        }

        walk_expr(&mut utf_storage, body.value);
        let func_hir_id = utf_storage.cx.tcx.def_path_str(body.value.hir_id.owner);

        let terminate = find_terminate_in_mir(
            &mir_body.basic_blocks,
            utf_storage.terminate_contract_def_id,
            utf_storage.possible_terminate,
        );
        if !terminate.terminates.is_empty()
            && !self.param_infos.is_empty()
            && self
                .param_infos
                .iter()
                .any(|param_info| func_hir_id == param_info.def_path)
        {
            let mut span = self.param_infos[0].span;
            let mut def_path = self.param_infos[0].def_path.clone();

            let res = self.param_infos.iter().all(|param_info| {
                terminate.terminates_info.iter().any(|terminate_info| {
                    // Checks if param_name is in param_names
                    let equal_name = terminate_info.param_names.contains(&param_info.param_name);
                    // Checks if def_path is the same
                    let equal_func = terminate_info.def_path == param_info.def_path;
                    if equal_func && !equal_name {
                        span = param_info.span;
                        def_path = terminate_info.def_path.clone();
                    }
                    equal_func && equal_name
                })
            });
            if res {
                self.update_param_info(def_path);
            }
        } else if
        //If there is no terminator and the function has more than two parameters of type address, indicates that the check is not performed.
        self.param_infos.len() >= 2
            && self
                .param_infos
                .iter()
                //The function name must be different to allow analysis of the parameters once the function analysis is complete
                .any(|param_info| func_hir_id != param_info.def_path)
        {
            let mut spans = vec![];

            let first_def_path = self.param_infos.first().unwrap().def_path.clone();

            // Once the warning of the function is emmited, the element of the array must be deleted, if not, it would be appearing in others functions
            self.param_infos.retain(|p| {
                if p.def_path.contains("new_call_variant_") {
                    false
                } else if !p.is_checked && p.def_path == *first_def_path {
                    spans.push(p.span);
                    false
                } else {
                    !p.is_checked
                }
            });
            if !spans.is_empty() {
                clippy_utils::diagnostics::span_lint(
                    cx,
                    EQUAL_ADDRESSES,
                    MultiSpan::from_spans(spans),
                    LINT_MESSAGE,
                )
            }
            // If len == 1, there is only one parameter in the function that is an address
        } else if self.param_infos.len() == 1 {
            self.param_infos.clear();
        }
    }
}
