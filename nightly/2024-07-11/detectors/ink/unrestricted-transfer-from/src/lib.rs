#![feature(rustc_private)]
#![feature(let_chains)]

extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;

use common::{
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use if_chain::if_chain;
use rustc_ast::{ast::UintTy, LitIntType, LitKind};
use rustc_hir::{
    def::Res,
    intravisit::{walk_expr, Visitor},
    Body, Expr, ExprKind, FnRetTy, PatKind, QPath,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::mir::{BasicBlock, BasicBlocks, Local, Operand, StatementKind, TerminatorKind};
use rustc_span::Span;

const LINT_MESSAGE: &str = "This argument comes from a user-supplied argument";

#[expose_lint_info]
pub static UNRESTRICTED_TRANSFER_FROM_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "In an ink! Substrate smart contract, allowing unrestricted transfer_from operations poses a significant vulnerability. When from arguments for that function is provided directly by the user, this might enable the withdrawal of funds from any actor with token approval on the contract. This could result in unauthorized transfers and loss of funds. To mitigate this vulnerability, instead of allowing an arbitrary from address, the from address should be restricted, ideally to the address of the caller (self.env().caller()), ensuring that the sender can initiate a transfer only with their own tokens.    ",
    severity: Severity::Critical,
    help: "https://coinfabrik.github.io/scout-audit/docs/detectors/ink/unrestricted-transfer-from",
    vulnerability_class: VulnerabilityClass::ErrorHandling,
};

dylint_linting::impl_late_lint! {
    pub UNRESTRICTED_TRANSFER_FROM,
    Warn,
    LINT_MESSAGE,
    UnrestrictedTransferFrom::default()
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
        fn_decl: &'tcx rustc_hir::FnDecl<'tcx>,
        body: &'tcx rustc_hir::Body<'tcx>,
        _: Span,
        localdef: rustc_span::def_id::LocalDefId,
    ) {
        struct UnrestrictedTransferFromFinder<'tcx, 'tcx_ref> {
            cx: &'tcx_ref LateContext<'tcx>,
            def_id: Option<rustc_span::def_id::DefId>,
            pusharg_def_id: Option<rustc_span::def_id::DefId>,
            span: Option<Span>,
            from_ref: bool,
            the_body: &'tcx Body<'tcx>,
        }

        impl<'tcx> Visitor<'tcx> for UnrestrictedTransferFromFinder<'tcx, '_> {
            fn visit_expr(&mut self, expr: &'tcx Expr<'_>) {
                match expr.kind {
                    ExprKind::Call(path, args) => {
                        // Look for Selector::new([0x54, 0xb3, 0xc7, 0x6e])
                        if let ExprKind::Path(qpath) = &path.kind
                            && let rustc_hir::QPath::TypeRelative(ty, path_segment) = qpath
                            && path_segment.ident.name.to_string() == "new"
                            && let rustc_hir::TyKind::Path(qpath_2) = &ty.kind
                            && let rustc_hir::QPath::Resolved(_, path_2) = qpath_2
                            && path_2
                                .segments
                                .iter()
                                .any(|s| s.ident.name.to_string() == "Selector")
                            && args.len() == 1
                            && let ExprKind::Array(sel_arr) = args.first().unwrap().kind
                            && sel_arr.len() == 4
                        {
                            let transfer_from_selector = [0x54, 0xb3, 0xc7, 0x6e];
                            let mut is_tranfer_from = true;

                            for (id, expr) in sel_arr.iter().enumerate() {
                                if let ExprKind::Lit(byte) = expr.kind
                                    && let LitKind::Int(value, int_ty) = byte.node
                                    && let LitIntType::Unsigned(u_ty) = int_ty
                                    && u_ty == UintTy::U8
                                    && value == transfer_from_selector[id]
                                {
                                    is_tranfer_from &= true;
                                }
                            }
                            if is_tranfer_from {
                                self.def_id =
                                    self.cx.typeck_results().type_dependent_def_id(path.hir_id);
                            }
                        }

                        if_chain! {
                            if let ExprKind::Path(qpath) = &path.kind;
                            if let rustc_hir::QPath::TypeRelative(ty, path_segment) = qpath;
                            if path_segment.ident.name.to_string() == "transfer_from"
                            || path_segment.ident.name.to_string() == "transfer_from_builder";
                            if let rustc_hir::TyKind::Path(qpath_2) = &ty.kind;
                            if let rustc_hir::QPath::Resolved(_, path_2) = qpath_2;
                            if path_2.segments.iter().any(|s|s.ident.name.to_string() == "PSP22Ref");
                            if args.len() == 5;
                            then {
                                let mut param_hir_ids = Vec::new();
                                let mut arg_hir_ids = Vec::new();

                                for i in 0..self.the_body.params.len() {
                                    if let PatKind::Binding(_, hir_id, _, _) = self.the_body.params[i].pat.kind {
                                        param_hir_ids.push(hir_id);
                                    }
                                }

                                arg_hir_ids.push(args[1].hir_id);

                                if let ExprKind::Path(QPath::Resolved(_, path)) = &args[1].kind {

                                    if let Res::Local(hir_id) = path.res {
                                        arg_hir_ids.push(hir_id);
                                    }
                                    for j in 0..path.segments.len() {
                                        arg_hir_ids.push(path.segments[j].hir_id);
                                    }

                                }


                                for param_id in param_hir_ids {
                                    if arg_hir_ids.contains(&param_id) {
                                        self.span = Some(args[1].span);
                                        self.from_ref = true;

                                    }
                                }
                            }

                        }
                    }
                    ExprKind::MethodCall(path, ..) => {
                        if path.ident.name.to_string() == "push_arg" {
                            self.pusharg_def_id =
                                self.cx.typeck_results().type_dependent_def_id(expr.hir_id);
                        }
                    }
                    _ => {}
                }

                walk_expr(self, expr);
            }
        }

        let mut utf_storage = UnrestrictedTransferFromFinder {
            cx,
            def_id: None,
            pusharg_def_id: None,
            span: None,
            from_ref: false,
            the_body: body,
        };

        if let FnRetTy::Return(ret_ty) = fn_decl.output
            && let rustc_hir::TyKind::Path(qpath) = &ret_ty.kind
            && let rustc_hir::QPath::Resolved(_, path) = qpath
            && path.segments.last().map_or(false, |s| {
                s.ident.name.to_string() == "CallBuilder"
                    || s.ident.name.to_string() == "CreateBuilder"
            })
        {
            return;
        }

        let mir_body = cx.tcx.optimized_mir(localdef.to_def_id());

        walk_expr(&mut utf_storage, body.value);

        if utf_storage.from_ref {
            clippy_utils::diagnostics::span_lint(
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
                        rustc_middle::mir::Rvalue::Use(
                            Operand::Copy(origplace) | Operand::Move(origplace),
                        ) => {
                            if tainted_locals
                                .clone()
                                .into_iter()
                                .any(|local| local == origplace.local)
                            {
                                tainted_locals.push(assign.0.local);
                            }
                        }
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
                        cx: &LateContext,
                        bb: &BasicBlock,
                        bbs: &BasicBlocks,
                        tainted_locals: &Vec<Local>,
                        _tainted_selector_places: &mut Vec<Local>,
                        utf_storage: &UnrestrictedTransferFromFinder,
                    ) {
                        if let TerminatorKind::Call {
                            func,
                            args,
                            fn_span,
                            target,
                            ..
                        } = &bbs[*bb].terminator().kind
                            && let Operand::Constant(cst) = func
                            && let rustc_middle::mir::Const::Val(_, val_type) = &cst.const_
                            && let rustc_middle::ty::TyKind::FnDef(def, _) = val_type.kind()
                        {
                            if utf_storage.pusharg_def_id.is_some_and(|id| &id == def) {
                                for arg in args {
                                    if arg.node.place().map_or(false, |place| {
                                        tainted_locals.iter().any(|l| l == &place.local)
                                    }) {
                                        clippy_utils::diagnostics::span_lint(
                                            cx,
                                            UNRESTRICTED_TRANSFER_FROM,
                                            *fn_span,
                                            LINT_MESSAGE,
                                        );
                                    }
                                }
                            }
                            if target.is_some() {
                                navigate_trough_bbs(
                                    cx,
                                    &target.unwrap(),
                                    bbs,
                                    tainted_locals,
                                    _tainted_selector_places,
                                    utf_storage,
                                );
                            }
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
