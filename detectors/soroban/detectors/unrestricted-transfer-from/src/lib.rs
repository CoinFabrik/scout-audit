#![feature(rustc_private)]
#![feature(let_chains)]

extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;

use rustc_hir::PatKind;
use rustc_hir::{
    intravisit::{walk_expr, Visitor},
    Body, Expr, ExprKind,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::mir::{BasicBlock, BasicBlocks, Local, Operand, StatementKind, TerminatorKind};
use rustc_span::Span;

const LINT_MESSAGE: &str = "This argument comes from a user-supplied argument";

dylint_linting::impl_late_lint! {
    pub UNRESTRICTED_TRANSFER_FROM,
    Warn,
    LINT_MESSAGE,
    UnrestrictedTransferFrom::default(),
    {
        name: "Unrestricted Transfer From",
        long_message: "In an smart contract, allowing unrestricted transfer_from operations poses a significant vulnerability. When from arguments for that function is provided directly by the user, this might enable the withdrawal of funds from any actor with token approval on the contract. This could result in unauthorized transfers and loss of funds. To mitigate this vulnerability, instead of allowing an arbitrary from address, the from address should be restricted.",
        severity: "Critical",
        help: "https://coinfabrik.github.io/scout-soroban/docs/detectors/unrestricted-transfer-from",
        vulnerability_class: "Validations and error handling",
    }
}

#[derive(Default)]
pub struct UnrestrictedTransferFrom {}
impl UnrestrictedTransferFrom {
    pub fn new() -> Self {
        Self {}
    }
}

impl<'tcx> LateLintPass<'tcx> for UnrestrictedTransferFrom {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: rustc_hir::intravisit::FnKind<'tcx>,
        _fn_decl: &'tcx rustc_hir::FnDecl<'tcx>,
        body: &'tcx rustc_hir::Body<'tcx>,
        _: Span,
        localdef: rustc_span::def_id::LocalDefId,
    ) {
        struct UnrestrictedTransferFromFinder<'tcx, 'tcx_ref> {
            cx: &'tcx_ref LateContext<'tcx>,
            def_id: Option<rustc_span::def_id::DefId>,
            //pusharg_def_id: Option<rustc_span::def_id::DefId>,
            span: Option<Span>,
            from_ref: bool,
            the_body: &'tcx Body<'tcx>,
        }

        impl<'tcx> Visitor<'tcx> for UnrestrictedTransferFromFinder<'tcx, '_> {
            fn visit_expr(&mut self, expr: &'tcx Expr<'_>) {
                if let ExprKind::MethodCall(path_segment, _, methodargs, ..) = expr.kind {
                    if path_segment.ident.name.to_string() == "transfer_from" {
                        self.def_id = self
                            .cx
                            .typeck_results()
                            .type_dependent_def_id(path_segment.hir_id);
                        let mut possible_params = Vec::new();
                        for i in 0..self.the_body.params.len() {
                            if let PatKind::Binding(_, _, name, _) =
                                self.the_body.params[i].pat.kind
                            {
                                possible_params.push(name.to_string());
                            }
                        }
                        let from_param = methodargs[1];
                        if let ExprKind::AddrOf(_, _, new_exp, ..) = from_param.kind {
                            if let ExprKind::Path(
                                rustc_hir::QPath::Resolved(_, rustc_hir::Path { segments, .. }),
                                ..,
                            ) = new_exp.kind
                            {
                                let from_ref_param = segments.first();
                                let from_addr;
                                if from_ref_param.is_some() {
                                    from_addr = from_ref_param.unwrap();
                                    if possible_params.contains(&from_addr.ident.name.to_string()) {
                                        self.span = Some(from_addr.ident.span);
                                        self.from_ref = true;
                                    }
                                }
                            }
                        }
                    }
                    self.def_id = self
                        .cx
                        .typeck_results()
                        .type_dependent_def_id(path_segment.hir_id);
                }
                walk_expr(self, expr);
            }
        }

        let mut utf_storage = UnrestrictedTransferFromFinder {
            cx,
            def_id: None,
            span: None,
            from_ref: false,
            the_body: body,
        };

        let mir_body = cx.tcx.optimized_mir(localdef.to_def_id());

        walk_expr(&mut utf_storage, body.value);

        if utf_storage.from_ref {
            clippy_wrappers::span_lint(
                cx,
                UNRESTRICTED_TRANSFER_FROM,
                utf_storage.span.unwrap(),
                LINT_MESSAGE,
            );
        }

        if utf_storage.def_id.is_none() {
            return;
        }
        //vector with function args and variables derived from those args
        let mut tainted_locals: Vec<Local> = mir_body.args_iter().collect();

        for bb in mir_body.basic_blocks.iter() {
            for statement in &bb.statements {
                if let StatementKind::Assign(assign) = &statement.kind {
                    match &assign.1 {
                        rustc_middle::mir::Rvalue::Ref(_, _, origplace)
                        | rustc_middle::mir::Rvalue::AddressOf(_, origplace)
                        | rustc_middle::mir::Rvalue::Len(origplace)
                        | rustc_middle::mir::Rvalue::CopyForDeref(origplace) => {
                            if tainted_locals
                                .clone()
                                .into_iter()
                                .any(|local| local == origplace.local)
                            {
                                tainted_locals.push(assign.0.local);
                            }
                        }
                        rustc_middle::mir::Rvalue::Use(operand) => match &operand {
                            Operand::Copy(origplace) | Operand::Move(origplace) => {
                                if tainted_locals
                                    .clone()
                                    .into_iter()
                                    .any(|local| local == origplace.local)
                                {
                                    tainted_locals.push(assign.0.local);
                                }
                            }
                            Operand::Constant(_) => todo!(),
                        },
                        _ => {}
                    }
                }
            }
        }
        for bb in mir_body.basic_blocks.iter() {
            if let TerminatorKind::Call {
                func,
                args: _,
                destination,
                target,
                unwind: _,
                fn_span: _,
                ..
            } = &bb.terminator().kind
            {
                if let Operand::Constant(cont) = func
                    && let rustc_middle::mir::Const::Val(_, val_type) = &cont.const_
                    && let rustc_middle::ty::TyKind::FnDef(def, _) = val_type.kind()
                    && utf_storage.def_id.is_some_and(|id| &id == def)
                    && target.is_some()
                {
                    //here the terminator is the call to new, the destination has the place with the selector
                    //from here on, what I do is look for where the selector is used and where user given args are pushed to it
                    let mut tainted_selector_places: Vec<Local> = vec![destination.local];
                    fn navigate_trough_bbs(
                        _cx: &LateContext,
                        bb: &BasicBlock,
                        bbs: &BasicBlocks,
                        _tainted_locals: &Vec<Local>,
                        _tainted_selector_places: &mut Vec<Local>,
                        _utf_storage: &UnrestrictedTransferFromFinder,
                    ) {
                        if let TerminatorKind::Call {
                            func,
                            args: _,
                            fn_span: _,
                            target,
                            ..
                        } = &bbs[*bb].terminator().kind
                            && let Operand::Constant(cst) = func
                            && let rustc_middle::mir::Const::Val(_, val_type) = &cst.const_
                            && let rustc_middle::ty::TyKind::FnDef(_def, _) = val_type.kind()
                            && target.is_some()
                        {
                            navigate_trough_bbs(
                                _cx,
                                &target.unwrap(),
                                bbs,
                                _tainted_locals,
                                _tainted_selector_places,
                                _utf_storage,
                            );
                        }
                    }
                    navigate_trough_bbs(
                        cx,
                        &target.unwrap(),
                        &mir_body.basic_blocks,
                        &tainted_locals,
                        &mut tainted_selector_places,
                        &utf_storage,
                    );
                }
            }
        }
    }
}
