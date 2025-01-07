#![feature(rustc_private)]
#![feature(let_chains)]

extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;

use common::{
    declarations::{ Severity, VulnerabilityClass },
    macros::expose_lint_info,
    analysis::{ get_node_type_opt, get_receiver_ident_name },
};
use if_chain::if_chain;
use rustc_hir::{
    def,
    intravisit::{ walk_expr, Visitor },
    Expr,
    ExprKind,
    BinOpKind,
    Param,
    PatKind,
};
use rustc_lint::{ LateContext, LateLintPass };
use rustc_middle::{
    mir::{
        BasicBlock,
        BasicBlockData,
        BasicBlocks,
        Const,
        Operand,
        Place,
        StatementKind,
        TerminatorKind,
    },
    ty::TyKind,
};
use rustc_span::{ def_id::DefId, Span };

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
        is_checked: bool
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
    caller_def_id: Option<DefId>,
    possible_terminate: Option<TerminateInfo>,
}

struct CallersAndTerminates<'tcx> {
    callers: Vec<(&'tcx BasicBlockData<'tcx>, BasicBlock)>,
    terminates: Vec<(&'tcx BasicBlockData<'tcx>, BasicBlock)>,
    terminates_info: Vec<TerminateInfo>,
}

impl<'tcx> Visitor<'tcx> for EqualAddressesFinder<'tcx, '_> {
    fn visit_expr(&mut self, expr: &'tcx Expr<'_>) {
        match expr.kind {
            ExprKind::Binary(op, rvalue, lvalue) => {
                if BinOpKind::Ne == op.node {
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
                            //dbg!("HOLA, {:?}", expr.hir_id);

                            self.possible_terminate = Some(TerminateInfo {
                                param_names: [rvalue.to_string(),lvalue.to_string()],
                                def_path: self.cx.tcx.def_path_str(expr.hir_id.owner),
                            }); 
                        }
                    );
                }
            }
            _ => {}
        }

        walk_expr(self, expr);
    }
}

fn find_caller_and_terminate_in_mir<'tcx>(
    bbs: &'tcx BasicBlocks<'tcx>,
    caller_def_id: Option<DefId>,
    terminate_def_id: Option<DefId>,
    possible_terminate: Option<TerminateInfo>
) -> CallersAndTerminates {
    let mut callers_vec = CallersAndTerminates {
        callers: vec![],
        terminates: vec![],
        terminates_info: vec![],
    };
    for (bb, bb_data) in bbs.iter().enumerate() {
        if bb_data.terminator.as_ref().is_none() {
            continue;
        }
        let terminator = bb_data.terminator.clone().unwrap();
        if let TerminatorKind::Call { func, .. } = &terminator.kind {
            if
                let Operand::Constant(fn_const) = func &&
                let Const::Val(_const_val, ty) = fn_const.const_ &&
                let TyKind::FnDef(def, _subs) = ty.kind()
            {
                if caller_def_id.is_some_and(|d| &d == def) {
                    callers_vec.callers.push((bb_data, BasicBlock::from_usize(bb)));
                }
                if terminate_def_id.is_some_and(|d| &d == def) {
                    if possible_terminate.is_some() {
                        let mut terminate_info = possible_terminate.clone().unwrap();
                        /*terminate_info.is_checked = true; */
                        callers_vec.terminates_info.push(terminate_info);
                    }
                    callers_vec.terminates.push((bb_data, BasicBlock::from_usize(bb)));
                }
            }
        }
    }
    callers_vec
}

fn navigate_trough_basicblocks<'tcx>(
    bbs: &'tcx BasicBlocks<'tcx>,
    bb: BasicBlock,
    caller_and_terminate: &CallersAndTerminates<'tcx>,
    after_comparison: bool,
    tainted_places: &mut Vec<Place<'tcx>>
) -> Vec<(Place<'tcx>, Span)> {
    let mut ret_vec = Vec::<(Place, Span)>::new();
    if bbs[bb].terminator.is_none() {
        return ret_vec;
    }
    for statement in &bbs[bb].statements {
        if let StatementKind::Assign(assign) = &statement.kind {
            match &assign.1 {
                | rustc_middle::mir::Rvalue::Ref(_, _, origplace)
                | rustc_middle::mir::Rvalue::AddressOf(_, origplace)
                | rustc_middle::mir::Rvalue::Len(origplace)
                | rustc_middle::mir::Rvalue::CopyForDeref(origplace) => {
                    if
                        tainted_places
                            .clone()
                            .into_iter()
                            .any(|place| place == *origplace)
                    {
                        tainted_places.push(assign.0);
                    }
                }
                rustc_middle::mir::Rvalue::Use(
                    Operand::Copy(origplace) | Operand::Move(origplace),
                ) => {
                    if
                        tainted_places
                            .clone()
                            .into_iter()
                            .any(|place| place == *origplace)
                    {
                        tainted_places.push(assign.0);
                    }
                }
                _ => {}
            }
        }
    }
    let kind = &bbs[bb].terminator().kind;
    match kind {
        TerminatorKind::SwitchInt { discr, targets } => {
            let comparison_with_caller = match discr {
                Operand::Copy(place) | Operand::Move(place) => {
                    after_comparison ||
                        tainted_places.iter().any(|tainted_place| tainted_place == place)
                }
                Operand::Constant(_cons) => after_comparison,
            };
            for target in targets.all_targets() {
                ret_vec.append(
                    &mut navigate_trough_basicblocks(
                        bbs,
                        *target,
                        caller_and_terminate,
                        comparison_with_caller,
                        tainted_places
                    )
                );
            }
            return ret_vec;
        }
        TerminatorKind::Call { destination, args, target, fn_span, .. } => {
            for arg in args {
                match arg.node {
                    Operand::Copy(origplace) | Operand::Move(origplace) => {
                        if
                            tainted_places
                                .clone()
                                .into_iter()
                                .any(|place| place == origplace)
                        {
                            tainted_places.push(*destination);
                        }
                    }
                    Operand::Constant(_) => {}
                }
            }
            for caller in &caller_and_terminate.callers {
                if caller.1 == bb {
                    tainted_places.push(*destination);
                }
            }
            for terminate in &caller_and_terminate.terminates {
                if terminate.1 == bb && !after_comparison {
                    ret_vec.push((*destination, *fn_span));
                }
            }
            if target.is_some() {
                ret_vec.append(
                    &mut navigate_trough_basicblocks(
                        bbs,
                        target.unwrap(),
                        caller_and_terminate,
                        after_comparison,
                        tainted_places
                    )
                );
            }
        }
        | TerminatorKind::Assert { target, .. }
        | TerminatorKind::Goto { target, .. }
        | TerminatorKind::Drop { target, .. } => {
            ret_vec.append(
                &mut navigate_trough_basicblocks(
                    bbs,
                    *target,
                    caller_and_terminate,
                    after_comparison,
                    tainted_places
                )
            );
        }
        TerminatorKind::Yield { resume, .. } => {
            ret_vec.append(
                &mut navigate_trough_basicblocks(
                    bbs,
                    *resume,
                    caller_and_terminate,
                    after_comparison,
                    tainted_places
                )
            );
        }
        TerminatorKind::FalseEdge { real_target, .. } => {
            ret_vec.append(
                &mut navigate_trough_basicblocks(
                    bbs,
                    *real_target,
                    caller_and_terminate,
                    after_comparison,
                    tainted_places
                )
            );
        }
        TerminatorKind::FalseUnwind { real_target, .. } => {
            ret_vec.append(
                &mut navigate_trough_basicblocks(
                    bbs,
                    *real_target,
                    caller_and_terminate,
                    after_comparison,
                    tainted_places
                )
            );
        }
        TerminatorKind::InlineAsm { targets, .. } => {
            targets.iter().for_each(|target| {
                ret_vec.append(
                    &mut navigate_trough_basicblocks(
                        bbs,
                        *target,
                        caller_and_terminate,
                        after_comparison,
                        tainted_places
                    )
                );
            });
        }
        _ => {}
    }
    ret_vec
}

impl<'tcx> LateLintPass<'tcx> for EqualAddresses {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: rustc_hir::intravisit::FnKind<'tcx>,
        _: &'tcx rustc_hir::FnDecl<'tcx>,
        body: &'tcx rustc_hir::Body<'tcx>,
        _: Span,
        localdef: rustc_span::def_id::LocalDefId
    ) {
        let mut utf_storage = EqualAddressesFinder {
            cx,
            terminate_contract_def_id: None,
            terminate_contract_span: None,
            caller_def_id: None,
            possible_terminate: None,
        };

        // Look for function params with AccountId type
        let mir_body = cx.tcx.optimized_mir(localdef);
        for (arg, hir_param) in mir_body.args_iter().zip(body.params.iter()) {
            if
                mir_body.local_decls[arg].ty.to_string() ==
                "<T as frame_system::Config>::AccountId" /* ||
                mir_body.local_decls[arg].ty.to_string() ==
                    "<T as frame_system::Config>::RuntimeOrigin" */
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
                    false
                );
            }
        }

        walk_expr(&mut utf_storage, body.value);
        let possible_term: Option<TerminateInfo> = utf_storage.possible_terminate.clone();
        //dbg!("{:?}", possible_term);
        //dbg!("{:?},{:?}", localdef, body.value.hir_id);
        let func_hir_id = utf_storage.cx.tcx.def_path_str(body.value.hir_id.owner);

        let caller_and_terminate = find_caller_and_terminate_in_mir(
            &mir_body.basic_blocks,
            utf_storage.caller_def_id,
            utf_storage.terminate_contract_def_id,
            utf_storage.possible_terminate
        );
        if !caller_and_terminate.terminates.is_empty() {
            //dbg!("{:?}", localdef);
            //dbg!("{:?}", &caller_and_terminate.terminates);
            //dbg!("{:?}", &caller_and_terminate.terminates_info);
            //dbg!("{:?}", &self.param_infos);

            if caller_and_terminate.callers.is_empty() {
                for terminate in caller_and_terminate.terminates {
                    //dbg!("{:?}", cx.tcx.def_path_str(localdef));
                    if
                        !self.param_infos.is_empty() &&
                        self.param_infos
                            .iter()
                            .any(|param_info| { func_hir_id.to_string() == param_info.def_path })
                    {
                        let mut span = self.param_infos[0].span;
                        let mut def_path = self.param_infos[0].def_path.clone();

                        //dbg!("{:?}", &caller_and_terminate.terminates_info);

                        let res = self.param_infos.iter().all(|param_info| {
                            caller_and_terminate.terminates_info.iter().any(|terminate_info| {
                                // Verifica si el param_name está en param_names
                                let equal_name = terminate_info.param_names.contains(
                                    &param_info.param_name
                                );
                                // Verifica si el def_path coincide
                                let equal_func = terminate_info.def_path == param_info.def_path;
                                if equal_func && !equal_name {
                                    span = param_info.span;
                                    def_path = terminate_info.def_path.clone();
                                }
                                equal_func && equal_name
                            })
                        });
                        //dbg!("{:?}", res);
                        if res {
                            self.update_param_info(def_path);
                        }
                    }
                }
            }
        } else if
            //If there is no terminator and the function has more than two parameters of type address, indicates that the check is not performed.
            self.param_infos.len() >= 2 &&
            self.param_infos
                .iter()
                //The function name must be different to allow analysis of the parameters once the function analysis is complete
                .any(|param_info| { func_hir_id.to_string() != param_info.def_path })
        {
            self.param_infos.iter().for_each(|p| {
                dbg!("AAAAAAA {:?}, {:?}", p.is_checked, &p.def_path);

                if !p.is_checked && p.def_path.contains("Pallet") {
                    clippy_utils::diagnostics::span_lint(cx, EQUAL_ADDRESSES, p.span, LINT_MESSAGE)
                }
            });
        }
    }
}
