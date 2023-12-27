#![feature(rustc_private)]
#![feature(let_chains)]

extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;

use rustc_hir::{
    intravisit::{walk_expr, Visitor},
    Expr, ExprKind,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::mir::{BasicBlock, BasicBlocks, Const, Operand, TerminatorKind};
use rustc_middle::ty::TyKind;
use rustc_span::def_id::DefId;
use rustc_span::Span;
use scout_audit_internal::Detector;

dylint_linting::impl_late_lint! {
    pub UNPROTECTED_UPDATE_CURRENT_CONTRACT_WASM,
    Warn,
    Detector::UnprotectedUpdateCurrentContractWasm.get_lint_message(),
    UnprotectedUpdateCurrentContractWasm::default()
}

#[derive(Default)]
pub struct UnprotectedUpdateCurrentContractWasm {}
impl UnprotectedUpdateCurrentContractWasm {
    pub fn new() -> Self {
        Self {}
    }
}

impl<'tcx> LateLintPass<'tcx> for UnprotectedUpdateCurrentContractWasm {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: rustc_hir::intravisit::FnKind<'tcx>,
        _: &'tcx rustc_hir::FnDecl<'tcx>,
        body: &'tcx rustc_hir::Body<'tcx>,
        _: Span,
        localdef: rustc_span::def_id::LocalDefId,
    ) {
        struct UnprotectedUpdateFinder<'tcx, 'tcx_ref> {
            cx: &'tcx_ref LateContext<'tcx>,
            require_auth_def_id: Option<DefId>,
            update_contract_def_id: Option<DefId>,
        }

        impl<'tcx> Visitor<'tcx> for UnprotectedUpdateFinder<'tcx, '_> {
            fn visit_expr(&mut self, expr: &'tcx Expr<'_>) {
                if let ExprKind::MethodCall(path, receiver, ..) = expr.kind {
                    if path.ident.name.to_string() == "require_auth" {
                        self.require_auth_def_id =
                            self.cx.typeck_results().type_dependent_def_id(expr.hir_id);
                    } else if path.ident.name.to_string() == "update_current_contract_wasm"
                        && let ExprKind::MethodCall(path2, ..) = receiver.kind
                        && path2.ident.name.to_string() == "deployer"
                    {
                        self.update_contract_def_id =
                            self.cx.typeck_results().type_dependent_def_id(expr.hir_id);
                    }
                }

                walk_expr(self, expr);
            }
        }

        let mut uuf_storage = UnprotectedUpdateFinder {
            cx,
            require_auth_def_id: None,
            update_contract_def_id: None,
        };

        walk_expr(&mut uuf_storage, body.value);

        let mir_body = cx.tcx.optimized_mir(localdef);

        let spans = navigate_trough_basicblocks(
            &mir_body.basic_blocks,
            BasicBlock::from_u32(0),
            false,
            &uuf_storage,
        );

        for span in spans {
            Detector::UnprotectedUpdateCurrentContractWasm.span_lint(
                cx,
                UNPROTECTED_UPDATE_CURRENT_CONTRACT_WASM,
                span,
            )
        }

        fn navigate_trough_basicblocks<'tcx>(
            bbs: &'tcx BasicBlocks<'tcx>,
            bb: BasicBlock,
            auth_checked: bool,
            uuf_storage: &UnprotectedUpdateFinder,
        ) -> Vec<Span> {
            let mut ret_vec: Vec<Span> = Vec::<Span>::new();
            if bbs[bb].terminator.is_none() {
                return ret_vec;
            }
            let mut checked = auth_checked;
            match &bbs[bb].terminator().kind {
                TerminatorKind::Call {
                    func,
                    target,
                    fn_span,
                    ..
                } => {
                    if let Operand::Constant(fn_const) = func
                        && let Const::Val(_const, ty) = fn_const.const_
                        && let TyKind::FnDef(def, _) = ty.kind()
                    {
                        if uuf_storage.require_auth_def_id.is_some_and(|f| f == *def) {
                            checked = true;
                        } else if uuf_storage
                            .update_contract_def_id
                            .is_some_and(|f| f == *def)
                            && !checked
                        {
                            ret_vec.push(*fn_span);
                        }
                    }
                    if let Some(utarget) = target {
                        ret_vec.append(&mut navigate_trough_basicblocks(
                            bbs,
                            *utarget,
                            checked,
                            uuf_storage,
                        ));
                    }
                }
                TerminatorKind::SwitchInt { targets, .. } => {
                    for target in targets.all_targets() {
                        ret_vec.append(&mut navigate_trough_basicblocks(
                            bbs,
                            *target,
                            checked,
                            uuf_storage,
                        ));
                    }
                }
                TerminatorKind::Assert { target, .. }
                | TerminatorKind::Goto { target, .. }
                | TerminatorKind::Drop { target, .. } => {
                    ret_vec.append(&mut navigate_trough_basicblocks(
                        bbs,
                        *target,
                        checked,
                        uuf_storage,
                    ));
                }
                TerminatorKind::Yield { resume, .. } => {
                    ret_vec.append(&mut navigate_trough_basicblocks(
                        bbs,
                        *resume,
                        checked,
                        uuf_storage,
                    ));
                }
                TerminatorKind::FalseEdge { real_target, .. }
                | TerminatorKind::FalseUnwind { real_target, .. } => {
                    ret_vec.append(&mut navigate_trough_basicblocks(
                        bbs,
                        *real_target,
                        checked,
                        uuf_storage,
                    ));
                }
                TerminatorKind::InlineAsm { destination, .. } => {
                    if let Some(udestination) = destination {
                        ret_vec.append(&mut navigate_trough_basicblocks(
                            bbs,
                            *udestination,
                            checked,
                            uuf_storage,
                        ));
                    }
                }
                TerminatorKind::Return
                | TerminatorKind::Unreachable
                | TerminatorKind::GeneratorDrop
                | TerminatorKind::UnwindResume
                | TerminatorKind::UnwindTerminate(_) => {}
            }
            ret_vec
        }
    }
}
