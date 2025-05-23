#![feature(rustc_private)]
#![feature(let_chains)]

extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;

use std::collections::HashSet;

use clippy_utils::diagnostics::span_lint_and_help;
use common::{
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use if_chain::if_chain;
use rustc_hir::{
    intravisit::{walk_expr, FnKind, Visitor},
    BinOpKind, Body, Expr, ExprKind, FnDecl,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::{
    mir::{
        BasicBlock, BasicBlockData, BasicBlocks, BinOp, Const, Operand, Place, Rvalue,
        StatementKind, TerminatorKind,
    },
    ty::TyKind,
};
use rustc_span::{
    def_id::{DefId, LocalDefId},
    Span,
};

const LINT_MESSAGE: &str = "Division before multiplication might result in a loss of precision";

#[expose_lint_info]
pub static DIVIDE_BEFORE_MULTIPLY_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "Division before multiplication might result in a loss of precision",
    severity: Severity::Medium,
    help: "https://coinfabrik.github.io/scout-audit/docs/detectors/rust/divide-before-multiply",
    vulnerability_class: VulnerabilityClass::Arithmetic,
};

dylint_linting::declare_late_lint! {
    pub DIVIDE_BEFORE_MULTIPLY,
    Warn,
    LINT_MESSAGE
}

fn get_divisions_inside_expr(expr: &Expr<'_>) -> Vec<Span> {
    struct DivisionsInsideExpr {
        divisions: Vec<Span>,
    }

    impl Visitor<'_> for DivisionsInsideExpr {
        fn visit_expr(&mut self, expr: &Expr<'_>) {
            if_chain! {
                if let ExprKind::Binary(op, _lexpr, _rexpr) = expr.kind;
                if BinOpKind::Div == op.node;
                then{
                    self.divisions.push(expr.span);
                }
            }
            walk_expr(self, expr);
        }
    }

    let mut visitor = DivisionsInsideExpr {
        divisions: Vec::default(),
    };

    walk_expr(&mut visitor, expr);

    visitor.divisions
}

struct DefIdFinder<'tcx, 'tcx_ref> {
    cx: &'tcx_ref LateContext<'tcx>,
    checked_div: Option<DefId>,
    checked_mul: Option<DefId>,
    saturating_div: Option<DefId>,
    saturating_mul: Option<DefId>,
}

impl Visitor<'_> for DefIdFinder<'_, '_> {
    fn visit_expr(&mut self, expr: &Expr<'_>) {
        if let ExprKind::MethodCall(path, ..) = expr.kind {
            let defid = self.cx.typeck_results().type_dependent_def_id(expr.hir_id);

            match path.ident.name.as_str() {
                "checked_div" => {
                    self.checked_div = defid;
                }
                "checked_mul" => {
                    self.checked_mul = defid;
                }
                "saturating_mul" => {
                    self.saturating_mul = defid;
                }
                "saturating_div" => {
                    self.saturating_div = defid;
                }
                _ => {}
            }
        }
        walk_expr(self, expr);
    }
}
fn check_operand<'tcx>(
    operand: &Operand,
    tainted_places: &mut Vec<Place<'tcx>>,
    place_to_taint: &Place<'tcx>,
) -> bool {
    match &operand {
        Operand::Copy(origplace) | Operand::Move(origplace) => {
            if tainted_places
                .clone()
                .into_iter()
                .any(|place| place.local == origplace.local)
            {
                tainted_places.push(*place_to_taint);
                true
            } else {
                false
            }
        }
        _ => false,
    }
}
fn navigate_trough_basicblocks<'tcx>(
    bb: BasicBlock,
    bbs: &BasicBlocks<'tcx>,
    def_ids: &DefIdFinder,
    tainted_places: &mut Vec<Place<'tcx>>,
    visited_bbs: &mut HashSet<BasicBlock>,
    spans: &mut Vec<Span>,
) {
    if visited_bbs.contains(&bb) {
        return;
    }
    visited_bbs.insert(bb);
    let bbdata: &BasicBlockData<'tcx> = &bbs[bb];

    for statement in &bbdata.statements {
        if let StatementKind::Assign(assign) = &statement.kind {
            match &assign.1 {
                Rvalue::Ref(_, _, origplace)
                | Rvalue::AddressOf(_, origplace)
                | Rvalue::Len(origplace)
                | Rvalue::CopyForDeref(origplace) => {
                    if tainted_places
                        .clone()
                        .into_iter()
                        .any(|place| place.local == origplace.local)
                    {
                        tainted_places.push(assign.0);
                    }
                }
                Rvalue::Use(operand) => {
                    check_operand(operand, tainted_places, &assign.0);
                }
                Rvalue::BinaryOp(op, operands) => {
                    if BinOp::Div == *op {
                        tainted_places.push(assign.0);
                    } else if BinOp::Mul == *op
                        || BinOp::MulUnchecked == *op
                            && (check_operand(&operands.0, tainted_places, &assign.0)
                                || check_operand(&operands.1, tainted_places, &assign.0))
                    {
                        spans.push(statement.source_info.span);
                    };
                }
                _ => {}
            }
        }
    }
    if bbdata.terminator.is_some() {
        let terminator = bbdata.terminator();
        match &terminator.kind {
            TerminatorKind::Call {
                func,
                args,
                destination,
                target,
                fn_span,
                ..
            } => {
                if let Operand::Constant(cst) = func
                    && let Const::Val(_, ty) = cst.const_
                    && let TyKind::FnDef(id, _) = ty.kind()
                {
                    if def_ids.checked_div.is_some_and(|f| f == *id)
                        || def_ids.saturating_div.is_some_and(|f| f == *id)
                    {
                        tainted_places.push(*destination);
                    } else {
                        for arg in args {
                            match arg {
                                Operand::Copy(place) | Operand::Move(place) => {
                                    if tainted_places.contains(place) {
                                        tainted_places.push(*destination);

                                        if def_ids.checked_mul.is_some_and(|f| f == *id)
                                            || def_ids.saturating_mul.is_some_and(|f| f == *id)
                                        {
                                            spans.push(*fn_span);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                if let Option::Some(next_bb) = target {
                    navigate_trough_basicblocks(
                        *next_bb,
                        bbs,
                        def_ids,
                        tainted_places,
                        visited_bbs,
                        spans,
                    );
                }
            }
            TerminatorKind::SwitchInt { targets, .. } => {
                for target in targets.all_targets() {
                    navigate_trough_basicblocks(
                        *target,
                        bbs,
                        def_ids,
                        tainted_places,
                        visited_bbs,
                        spans,
                    );
                }
            }
            TerminatorKind::Goto { target }
            | TerminatorKind::Drop { target, .. }
            | TerminatorKind::Assert { target, .. } => {
                navigate_trough_basicblocks(
                    *target,
                    bbs,
                    def_ids,
                    tainted_places,
                    visited_bbs,
                    spans,
                );
            }
            TerminatorKind::Yield { resume, drop, .. } => {
                navigate_trough_basicblocks(
                    *resume,
                    bbs,
                    def_ids,
                    tainted_places,
                    visited_bbs,
                    spans,
                );
                if let Option::Some(drop_target) = drop {
                    navigate_trough_basicblocks(
                        *drop_target,
                        bbs,
                        def_ids,
                        tainted_places,
                        visited_bbs,
                        spans,
                    );
                }
            }
            TerminatorKind::FalseEdge {
                real_target,
                imaginary_target,
            } => {
                navigate_trough_basicblocks(
                    *real_target,
                    bbs,
                    def_ids,
                    tainted_places,
                    visited_bbs,
                    spans,
                );
                navigate_trough_basicblocks(
                    *imaginary_target,
                    bbs,
                    def_ids,
                    tainted_places,
                    visited_bbs,
                    spans,
                );
            }
            TerminatorKind::FalseUnwind { real_target, .. } => {
                navigate_trough_basicblocks(
                    *real_target,
                    bbs,
                    def_ids,
                    tainted_places,
                    visited_bbs,
                    spans,
                );
            }
            TerminatorKind::InlineAsm {
                template: _,
                operands: _,
                options: _,
                line_spans: _,
                destination: Option::Some(dest),
                unwind: _,
            } => {
                navigate_trough_basicblocks(
                    *dest,
                    bbs,
                    def_ids,
                    tainted_places,
                    visited_bbs,
                    spans,
                );
            }

            _ => {}
        }
    }
}

impl<'tcx> LateLintPass<'tcx> for DivideBeforeMultiply {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: FnKind<'tcx>,
        _: &'tcx FnDecl<'tcx>,
        body: &'tcx Body<'tcx>,
        _: Span,
        localdef: LocalDefId,
    ) {
        let mut visitor = DefIdFinder {
            checked_div: None,
            checked_mul: None,
            saturating_div: None,
            saturating_mul: None,
            cx,
        };

        walk_expr(&mut visitor, body.value);

        let mir_body = cx.tcx.optimized_mir(localdef);
        if visitor.checked_div.is_some()
            || visitor.checked_mul.is_some()
            || visitor.saturating_mul.is_some()
        {
            let mut spans = vec![];
            navigate_trough_basicblocks(
                BasicBlock::from_u32(0),
                &mir_body.basic_blocks,
                &visitor,
                &mut vec![],
                &mut HashSet::<BasicBlock>::default(),
                &mut spans,
            );

            for span in spans {
                span_lint_and_help(
                    cx,
                    DIVIDE_BEFORE_MULTIPLY,
                    span,
                    LINT_MESSAGE,
                    None,
                    "Consider reversing the order of operations to reduce the loss of precision.",
                );
            }
        }
    }
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx Expr<'tcx>) {
        if_chain! {
            if let ExprKind::Binary(op, _lexpr, _rexpr) = expr.kind;
            if BinOpKind::Mul == op.node;
            then{
                for division in get_divisions_inside_expr(expr) {
                    span_lint_and_help(
                        cx,
                        DIVIDE_BEFORE_MULTIPLY,
                        division,
                        LINT_MESSAGE,
                        None,
                        "Consider reversing the order of operations to reduce the loss of precision.",
                    );
                }
            }
        }
    }
}
