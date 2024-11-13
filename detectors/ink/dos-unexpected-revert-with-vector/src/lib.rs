#![feature(rustc_private)]
#![feature(let_chains)]

extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;

use std::collections::HashSet;

use clippy_wrappers::span_lint;
use common::{
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use if_chain::if_chain;
use rustc_hir::{
    intravisit::{walk_expr, FnKind, Visitor},
    Body, Expr, ExprKind, FnDecl, QPath,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::{
    mir::{
        BasicBlock, BasicBlockData, BasicBlocks, Const, Operand, Place, Rvalue, StatementKind,
        TerminatorKind,
    },
    ty::{Ty, TyKind},
};
use rustc_span::{def_id::DefId, def_id::LocalDefId, Span};

const LINT_MESSAGE: &str = "This vector operation is called without access control";

#[expose_lint_info]
pub static DOS_UNEXPECTED_REVERT_WITH_VECTOR_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "It occurs by preventing transactions by other users from being successfully executed forcing the blockchain state to revert to its original state.",
    severity: Severity::Medium,
    help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/dos-unexpected-revert-with-vector",
    vulnerability_class: VulnerabilityClass::DoS,
};

dylint_linting::declare_late_lint! {
    pub DOS_UNEXPECTED_REVERT_WITH_VECTOR,
    Warn,
    LINT_MESSAGE
}

struct UnprotectedVectorFinder<'tcx, 'tcx_ref> {
    cx: &'tcx_ref LateContext<'tcx>,
    callers_def_id: HashSet<DefId>,
    push_def_id: Option<DefId>,
}

const VEC_NAME: &str = "ink::ink_prelude::vec::Vec";

impl<'tcx> Visitor<'tcx> for UnprotectedVectorFinder<'tcx, '_> {
    fn visit_expr(&mut self, expr: &'tcx Expr<'_>) {
        if let ExprKind::MethodCall(path, receiver, ..) = expr.kind {
            let Some(defid) = self.cx.typeck_results().type_dependent_def_id(expr.hir_id) else {
                return;
            };
            let ty = Ty::new_foreign(self.cx.tcx, defid);

            if ty.to_string().contains(VEC_NAME) && path.ident.name.to_string() == "push" {
                self.push_def_id = Some(defid);
                return;
            }

            if_chain! {
                if let ExprKind::MethodCall(rec_path, receiver2, ..) = receiver.kind;
                if rec_path.ident.name.to_string() == "env";
                if let ExprKind::Path(QPath::Resolved(qualifier, rec2_path)) = &receiver2.kind;
                if rec2_path.segments.first().map_or(false, |seg| {
                    seg.ident.to_string() == "self" && qualifier.is_none()
                });
                if path.ident.name.to_string() == "caller";
                if let Some(caller_def_id) = self.cx.typeck_results().type_dependent_def_id(expr.hir_id);
                then {
                    self.callers_def_id.insert(caller_def_id);
                    return;
                }
            }

            if_chain! {
                if let ExprKind::Call(receiver2, ..) = receiver.kind;
                if let ExprKind::Path(QPath::TypeRelative(ty2, rec2_path)) = &receiver2.kind;
                if rec2_path.ident.name.to_string() == "env";
                if let rustc_hir::TyKind::Path(QPath::Resolved(_, rec3_path)) = &ty2.kind;
                if rec3_path.segments[0].ident.to_string() == "Self";
                if let Some(caller_def_id) = self.cx.typeck_results().type_dependent_def_id(expr.hir_id);
                then {
                    self.callers_def_id.insert(caller_def_id);
                    return;
                }
            }
        }
        walk_expr(self, expr);
    }
}

#[derive(Debug)]
struct CallersAndVecOps<'tcx> {
    callers: Vec<(&'tcx BasicBlockData<'tcx>, BasicBlock)>,
    vec_ops: Vec<(&'tcx BasicBlockData<'tcx>, BasicBlock)>,
}

impl<'tcx> CallersAndVecOps<'tcx> {
    fn find_caller_and_vec_ops_in_mir(
        bbs: &'tcx BasicBlocks<'tcx>,
        callers_def_id: HashSet<DefId>,
        push_def_id: Option<DefId>,
    ) -> Self {
        let mut callers_vec = Self {
            callers: vec![],
            vec_ops: vec![],
        };

        for (bb, bb_data) in bbs.iter().enumerate() {
            if bb_data.terminator.as_ref().is_none() {
                continue;
            }
            let terminator = bb_data.terminator.clone().unwrap();
            if let TerminatorKind::Call { func, .. } = terminator.kind {
                if let Operand::Constant(fn_const) = func
                    && let Const::Val(_const_val, ty) = fn_const.const_
                    && let TyKind::FnDef(def, _subs) = ty.kind()
                {
                    if !callers_def_id.is_empty() {
                        for caller in &callers_def_id {
                            if caller == def {
                                callers_vec
                                    .callers
                                    .push((bb_data, BasicBlock::from_usize(bb)));
                            }
                        }
                    } else if let Some(op) = push_def_id {
                        if op == *def {
                            callers_vec
                                .vec_ops
                                .push((bb_data, BasicBlock::from_usize(bb)));
                        }
                    }
                }
            }
        }
        callers_vec
    }
}

impl DosUnexpectedRevertWithVector {
    fn navigate_trough_basicblocks<'tcx>(
        bbs: &'tcx BasicBlocks<'tcx>,
        bb: BasicBlock,
        caller_and_vec_ops: &CallersAndVecOps<'tcx>,
        after_comparison: bool,
        tainted_places: &mut Vec<Place<'tcx>>,
        visited_bbs: &mut HashSet<BasicBlock>,
    ) -> Vec<(Place<'tcx>, Span)> {
        let mut ret_vec = Vec::<(Place, Span)>::new();
        if visited_bbs.contains(&bb) {
            return ret_vec;
        } else {
            visited_bbs.insert(bb);
        }
        if bbs[bb].terminator.is_none() {
            return ret_vec;
        }
        for statement in &bbs[bb].statements {
            if let StatementKind::Assign(assign) = &statement.kind {
                match &assign.1 {
                    Rvalue::Ref(_, _, origplace)
                    | Rvalue::AddressOf(_, origplace)
                    | Rvalue::Len(origplace)
                    | Rvalue::CopyForDeref(origplace) => {
                        if tainted_places
                            .clone()
                            .into_iter()
                            .any(|place| place == *origplace)
                        {
                            tainted_places.push(assign.0);
                        }
                    }
                    Rvalue::Use(Operand::Copy(origplace) | Operand::Move(origplace)) => {
                        if tainted_places
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
        match &bbs[bb].terminator().kind {
            TerminatorKind::SwitchInt { discr, targets } => {
                let comparison_with_caller = match discr {
                    Operand::Copy(place) | Operand::Move(place) => {
                        tainted_places
                            .iter()
                            .any(|tainted_place| tainted_place == place)
                            || after_comparison
                    }
                    Operand::Constant(_cons) => after_comparison,
                };
                for target in targets.all_targets() {
                    ret_vec.append(
                        &mut DosUnexpectedRevertWithVector::navigate_trough_basicblocks(
                            bbs,
                            *target,
                            caller_and_vec_ops,
                            comparison_with_caller,
                            tainted_places,
                            visited_bbs,
                        ),
                    );
                }
                return ret_vec;
            }
            TerminatorKind::Call {
                destination,
                args,
                target,
                fn_span,
                ..
            } => {
                for arg in args {
                    match arg.node {
                        Operand::Copy(origplace) | Operand::Move(origplace) => {
                            if tainted_places
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
                for caller in &caller_and_vec_ops.callers {
                    if caller.1 == bb {
                        tainted_places.push(*destination);
                    }
                }
                for map_op in &caller_and_vec_ops.vec_ops {
                    if map_op.1 == bb
                        && !after_comparison
                        && args.get(1).map_or(true, |f| {
                            f.node.place().is_some_and(|f| !tainted_places.contains(&f))
                        })
                    {
                        ret_vec.push((*destination, *fn_span))
                    }
                }
                if target.is_some() {
                    ret_vec.append(
                        &mut DosUnexpectedRevertWithVector::navigate_trough_basicblocks(
                            bbs,
                            target.unwrap(),
                            caller_and_vec_ops,
                            after_comparison,
                            tainted_places,
                            visited_bbs,
                        ),
                    );
                }
            }
            TerminatorKind::Assert { target, .. }
            | TerminatorKind::Goto { target, .. }
            | TerminatorKind::Drop { target, .. } => {
                ret_vec.append(
                    &mut DosUnexpectedRevertWithVector::navigate_trough_basicblocks(
                        bbs,
                        *target,
                        caller_and_vec_ops,
                        after_comparison,
                        tainted_places,
                        visited_bbs,
                    ),
                );
            }
            TerminatorKind::Yield { resume, .. } => {
                ret_vec.append(
                    &mut DosUnexpectedRevertWithVector::navigate_trough_basicblocks(
                        bbs,
                        *resume,
                        caller_and_vec_ops,
                        after_comparison,
                        tainted_places,
                        visited_bbs,
                    ),
                );
            }
            TerminatorKind::FalseEdge { real_target, .. }
            | TerminatorKind::FalseUnwind { real_target, .. } => {
                ret_vec.append(
                    &mut DosUnexpectedRevertWithVector::navigate_trough_basicblocks(
                        bbs,
                        *real_target,
                        caller_and_vec_ops,
                        after_comparison,
                        tainted_places,
                        visited_bbs,
                    ),
                );
            }
            TerminatorKind::InlineAsm { targets, .. } => {
                targets.iter().for_each(|target| {
                    ret_vec.append(
                        &mut DosUnexpectedRevertWithVector::navigate_trough_basicblocks(
                            bbs,
                            *target,
                            caller_and_vec_ops,
                            after_comparison,
                            tainted_places,
                            visited_bbs,
                        ),
                    );
                });
            }
            _ => {}
        }
        ret_vec
    }
}

impl<'tcx> LateLintPass<'tcx> for DosUnexpectedRevertWithVector {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: FnKind<'tcx>,
        _: &'tcx FnDecl<'tcx>,
        body: &'tcx Body<'tcx>,
        _: Span,
        localdef: LocalDefId,
    ) {
        let mut uvf_storage = UnprotectedVectorFinder {
            cx,
            callers_def_id: HashSet::default(),
            push_def_id: None,
        };

        walk_expr(&mut uvf_storage, body.value);

        let mir_body = cx.tcx.optimized_mir(localdef);

        let caller_and_vec_ops = CallersAndVecOps::find_caller_and_vec_ops_in_mir(
            &mir_body.basic_blocks,
            uvf_storage.callers_def_id,
            uvf_storage.push_def_id,
        );

        if !caller_and_vec_ops.vec_ops.is_empty() {
            let unchecked_places = DosUnexpectedRevertWithVector::navigate_trough_basicblocks(
                &mir_body.basic_blocks,
                BasicBlock::from_u32(0),
                &caller_and_vec_ops,
                false,
                &mut vec![],
                &mut HashSet::<BasicBlock>::default(),
            );

            for place in unchecked_places {
                span_lint(cx, DOS_UNEXPECTED_REVERT_WITH_VECTOR, place.1, LINT_MESSAGE);
            }
        }
    }
}
