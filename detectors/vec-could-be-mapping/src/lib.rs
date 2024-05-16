#![feature(rustc_private)]
#![recursion_limit = "256"]
#![feature(let_chains)]
extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;
extern crate rustc_type_ir;

use rustc_hir::{
    intravisit::{
        walk_expr,
        FnKind,
        Visitor,
    },
    def::Res,
    Body,
    Expr,
    Stmt,
    ExprKind,
    StmtKind,
    FnDecl,
    GenericArg,
    PathSegment,
    QPath,
    Ty,
    TyKind,
    HirId,
    Path,
    Local,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::ty::{Interner, TyCtxt, GenericArgKind};
use rustc_span::{def_id::LocalDefId, Span};
use scout_audit_clippy_utils::diagnostics::span_lint_and_help;

const LINT_MESSAGE: &str =
    "You are iterating over a vector of tuples using `find`. Consider using a mapping instead.";

dylint_linting::impl_late_lint! {
    pub VEC_COULD_BE_MAPPING,
    Warn,
    LINT_MESSAGE,
    VecCouldBeMapping::default(),
    {
        name: "Vec could be Mapping",
        long_message: "This vector could be a mapping. Consider changing it, because you are using `find` method in a vector of tuples",
        severity: "Enhancement",
        help: "https://coinfabrik.github.io/scout/docs/vulnerabilities/vec-could-be-mapping",
        vulnerability_class: "Gas Usage",
    }
}

#[derive(Default)]
pub struct VecCouldBeMapping {
}

impl<'tcx> LateLintPass<'tcx> for VecCouldBeMapping {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: FnKind<'tcx>,
        _: &'tcx FnDecl<'_>,
        body: &'tcx Body<'_>,
        _: Span,
        _: LocalDefId,
    ) {
        let mut vec_mapping_storage = FindIterations {
            cx,
            uses_as_hashmap: Vec::new(),
            function_body: body,
        };

        walk_expr(&mut vec_mapping_storage, body.value);

        vec_mapping_storage
            .uses_as_hashmap
            .iter()
            .for_each(|span| {
                span_lint_and_help(
                    cx,
                    VEC_COULD_BE_MAPPING,
                    span.clone(),
                    LINT_MESSAGE,
                    None,
                    "Change this to a `Key(...)`",
                );
            });
    }
}

struct FindIterations<'a, 'b, 'c> {
    cx: &'b LateContext<'a>,
    uses_as_hashmap: Vec<Span>,
    function_body: &'b Body<'c>,
}

impl<'a, 'b, 'c> FindIterations<'a, 'b, 'c> {
    fn vector_comes_from_local_set_from_storage<'hir>(&mut self, receiver: &'hir Expr<'hir>) -> bool{
        || -> Result<bool, ()>{
            let path = expr_to_path(&receiver.kind)?;
            let (_, object_path) = path_to_resolved(&path)?;
            let object_decl_hir_id = resolution_to_local(&object_path.res)?;
            let mut fst = FindStorageLocal{
                cx: &self.cx,
                result: false,
                id: &object_decl_hir_id,
            };
            walk_expr(&mut fst, self.function_body.value);

            Ok(fst.result)
        }().unwrap_or(false)
    }

    fn vector_comes_directly_from_storage<'hir>(&mut self, receiver: &'hir Expr<'hir>) -> bool{
        let mut fgse = FindGetStorageExpression{
            cx: &self.cx,
            result: false,
        };
        walk_expr(&mut fgse, receiver);
        fgse.result
    }

    fn visit_expr_internal<'d>(&mut self, expr: &'d Expr<'_>) -> Result<(), ()>{
        let (function_name, receiver, _, _) = expr_to_method_call(&expr.kind)?;
        if function_name.ident.as_str() != "find"{
            return Ok(());
        }
        let receiver_type = get_type_string(self.cx, &receiver.hir_id)?;
        if receiver_type != "soroban_sdk::iter::UnwrappedIter"{
            return Ok(());
        }
        
        let (function_name, receiver, _, _) = expr_to_method_call(&receiver.kind)?;
        if function_name.ident.as_str() != "iter"{
            return Ok(());
        }

        let (def, generic_args) = type_to_adt(get_node_type(self.cx, &receiver.hir_id).kind())?;
        if definition_to_string(self.cx, def.did()) != "soroban_sdk::vec::Vec"{
            return Ok(());
        }

        if generic_args.len() != 1{
            return Ok(());
        }

        let generic_arg = generic_args.first().unwrap().unpack();

        let type_string = {
            if let GenericArgKind::Type(x) = generic_arg{
                Ok(x)
            }else{
                Err(())
            }
        }?.to_string();
        let n = type_string.len();
        if !(n > 2 && type_string.chars().nth(0).unwrap() == '(' && type_string.chars().nth(n - 1).unwrap() == ')'){
            return Ok(());
        }

        //Iterating over a vector of tuples. Does it come from storage?
        if self.vector_comes_from_local_set_from_storage(receiver) || self.vector_comes_directly_from_storage(receiver){
            self.uses_as_hashmap.push(expr.span);
        }
        
        Ok(())
    }
}

struct FindStorageLocal<'a, 'b> {
    cx: &'b LateContext<'a>,
    result: bool,
    id: &'b HirId,
}

impl<'a, 'b> FindStorageLocal<'a, 'b> {
    fn visit_stmt_internal<'d>(&mut self, stmt: &'d Stmt<'_>) -> Result<bool, ()>{
        let let_struct = stmt_to_local(&stmt.kind)?;
        if let_struct.pat.hir_id != *self.id || !let_struct.init.is_some(){
            return Ok(false);
        }

        let init = let_struct.init.unwrap();

        let mut fgse = FindGetStorageExpression{
            cx: &self.cx,
            result: false,
        };
        walk_expr(&mut fgse, init);

        Ok(fgse.result)
    }
}

impl<'tcx, 'a, 'b> Visitor<'tcx> for FindStorageLocal<'a, 'b> {
    fn visit_stmt(&mut self, stmt: &'tcx Stmt<'_>) {
        if self.result{
            return;
        }
        if let Ok(r) = self.visit_stmt_internal(stmt){
            self.result = r;
        }
        //walk_expr(self, expr);
    }
}

struct FindGetStorageExpression<'a, 'b> {
    cx: &'b LateContext<'a>,
    result: bool,
}

impl<'tcx, 'a, 'b, 'c> Visitor<'tcx> for FindIterations<'a, 'b, 'c> {
    fn visit_expr(&mut self, expr: &'tcx Expr<'_>) {
        let _ = self.visit_expr_internal(expr);
        walk_expr(self, expr);
    }
}

fn get_node_type<'a>(cx: &LateContext<'a>, hir_id: &HirId) -> rustc_middle::ty::Ty<'a> {
    cx.typeck_results().node_type(*hir_id)
}

fn definition_to_string<'a>(cx: &LateContext<'a>, did: rustc_hir::def_id::DefId) -> String{
    cx
        .get_def_path(did)
        .iter()
        .map(|x| x.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

fn get_type_string<'a, 'hir>(cx: &LateContext<'a>, hir_id: &HirId) -> Result<String, ()>{
    let (def, _generic_args) = type_to_adt(get_node_type(cx, hir_id).kind())?;
    Ok(definition_to_string(cx, def.did()))
}

fn is_storage<'a>(cx: &LateContext<'a>, hir_id: &HirId) -> bool{
    let receiver_type = get_type_string(cx, hir_id);
    if let Ok(receiver_type) = receiver_type{
        receiver_type == "soroban_sdk::storage::Persistent" || receiver_type != "soroban_sdk::storage::Temporary" || receiver_type != "soroban_sdk::storage::Instance"
    }else{
        false
    }
}

impl<'a, 'b> FindGetStorageExpression<'a, 'b> {
    fn visit_expr_internal<'c>(&mut self, expr: &'c Expr<'_>) -> Result<(), ()>{
        let (function_name, receiver, _, _) = expr_to_method_call(&expr.kind)?;
        if function_name.ident.as_str() != "get"{
            return Ok(());
        }
        if !is_storage(self.cx, &receiver.hir_id){
            return Ok(());
        }
        let generic_args = {
            if let Some(x) = function_name.args{
                Ok(x)
            }else{
                Err(())
            }
        }?;
        let generic_args = generic_args.args;
        if generic_args.len() != 2{
            return Ok(());
        }
        let data_type = generic_args[1];
        if get_type_string(self.cx, &data_type.hir_id())? != "soroban_sdk::vec::Vec"{
            return Ok(());
        }
        let data_type = {
            if let GenericArg::Type(x) = data_type{
                Ok(x)
            }else{
                Err(())
            }
        }?;
        let path = {
            if let TyKind::Path(x) = data_type.kind && let QPath::Resolved(_, x) = x{
                Ok(x)
            }else{
                Err(())
            }
        }?;
        let last_segment = path.segments.last().ok_or(())?;
        let args = {
            if let Some(x) = last_segment.args{
                Ok(x)
            }else{
                Err(())
            }
        }?.args;
        if args.len() != 1{
            return Ok(())
        }
        let argument = args.first().unwrap();
        let argument = {
            if let GenericArg::Type(x) = argument{
                Ok(x)
            }else{
                Err(())
            }
        }?;
        
        let is_tuple = {
            if let TyKind::Tup(_) = argument.kind{
                true
            }else{
                false
            }
        };

        self.result = is_tuple;

        Ok(())
    }
}

fn resolution_to_local(resolution: &Res) -> Result<&HirId, ()> {
    if let Res::Local(a) = resolution {
        Ok(a)
    } else {
        Err(())
    }
}

fn path_to_resolved<'hir>(
    path: &'hir QPath<'hir>,
) -> Result<(&'hir Option<&'hir Ty<'hir>>, &'hir Path<'hir>), ()> {
    if let QPath::Resolved(a, b) = path {
        Ok((a, b))
    } else {
        Err(())
    }
}

fn expr_to_path<'hir>(kind: &'hir ExprKind<'hir>) -> Result<QPath<'hir>, ()> {
    if let ExprKind::Path(a) = kind {
        Ok(*a)
    } else {
        Err(())
    }
}

fn expr_to_method_call<'hir>(
    kind: &'hir ExprKind<'hir>,
) -> Result<
    (
        &'hir PathSegment<'hir>,
        &'hir Expr<'hir>,
        &'hir [Expr<'hir>],
        Span,
    ),
    (),
> {
    if let ExprKind::MethodCall(a, b, c, d) = kind {
        Ok((a, b, c, *d))
    } else {
        Err(())
    }
}

fn stmt_to_local<'hir>(
    kind: &'hir StmtKind<'hir>,
) -> Result<
    &'hir Local<'hir>,
    (),
> {
    if let StmtKind::Local(a) = kind {
        Ok(a)
    } else {
        Err(())
    }
}

fn type_to_adt<'hir>(
    kind: &'hir rustc_type_ir::TyKind<TyCtxt<'hir>>,
) -> Result<
    (
        &'hir <TyCtxt<'hir> as Interner>::AdtDef,
        &'hir <TyCtxt<'hir> as Interner>::GenericArgs,
    ),
    (),
> {
    if let rustc_type_ir::TyKind::Adt(a, b) = kind {
        Ok((&a, &b))
    } else {
        Err(())
    }
}

impl<'tcx, 'a, 'b> Visitor<'tcx> for FindGetStorageExpression<'a, 'b> {
    fn visit_expr(&mut self, expr: &'tcx Expr<'_>) {
        let _ = self.visit_expr_internal(expr);
        walk_expr(self, expr);
    }
}
