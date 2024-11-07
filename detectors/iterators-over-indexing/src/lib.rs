#![feature(rustc_private)]
#![feature(let_chains)]

extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;
extern crate rustc_type_ir;

use std::collections::HashSet;

use clippy_wrappers::span_lint_and_help;
use rustc_ast::{Label, LitIntType, LitKind};
use rustc_hir::{
    def::Res,
    def_id::LocalDefId,
    intravisit::{walk_expr, FnKind, Visitor},
    BindingAnnotation, Block, Expr, ExprField, ExprKind, HirId, LangItem, LoopSource, MatchSource,
    Pat, PatField, PatKind, Path, QPath, StmtKind, Ty,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::ty::{TyCtxt, TyKind};
use rustc_span::{symbol::Ident, Span};
use rustc_type_ir::Interner;

const LINT_MESSAGE: &str =
    "Hardcoding an index could lead to panic if the top bound is out of bounds.";

scout_audit_dylint_linting::declare_late_lint! {
    pub ITERATORS_OVER_INDEXING,
    Warn,
    LINT_MESSAGE,
    {
        name: "Iterators Over Indexing",
        long_message: "Instead, use an iterator or index to `.len()`.",
        severity: "Medium",
        help: "https://coinfabrik.github.io/scout-soroban/docs/detectors/iterators-over-indexing",
        vulnerability_class: "Incorrect Use of Indexing",
    }
}

pub fn get_node_type<'a>(
    cx: &rustc_lint::LateContext<'a>,
    hir_id: &HirId,
) -> rustc_middle::ty::Ty<'a> {
    cx.typeck_results().node_type(*hir_id)
}

struct ForLoopVisitor<'a, 'b> {
    span_constant: HashSet<Span>,
    cx: &'b LateContext<'a>,
}
struct VectorAccessVisitor<'a, 'b> {
    index_id: HirId,
    has_vector_access: bool,
    cx: &'b LateContext<'a>,
}

#[allow(dead_code)]
impl<'a, 'b> VectorAccessVisitor<'a, 'b> {
    fn match_expression(&mut self, kind: &'a ExprKind<'a>) -> Result<(), ()> {
        match kind {
            //get() is not unsafe in Ink!
            /*
            ExprKind::MethodCall(function_name, object, arguments, _) => {
                let name = function_name.ident.name.as_str();
                if name == "get" || name == "get_unchecked" {
                    self.match_method_call_obj(&object.kind, arguments)
                }else{
                    Ok(())
                }
            },
            */
            ExprKind::Index(arr, index, _) => {
                self.handle_index(&arr.hir_id, &arr.kind, &index.kind)
            }
            _ => Ok(()),
        }
    }

    fn handle_index(
        &mut self,
        expr_id: &HirId,
        arr: &'a ExprKind<'a>,
        index: &'a ExprKind<'a>,
    ) -> Result<(), ()> {
        match arr {
            ExprKind::Field(_, _) => {
                let type_name = self.get_id_type(expr_id)?;
                self.final_check(&type_name, index)
            }
            _ => Ok(()),
        }
    }

    fn match_method_call_obj(
        &mut self,
        kind: &'a ExprKind<'a>,
        arguments: &'a [Expr<'a>],
    ) -> Result<(), ()> {
        match kind {
            ExprKind::Path(object_path) => {
                if arguments.len() != 1 {
                    return Ok(());
                }
                let index = &arguments.first().unwrap().kind;

                self.handle_path(object_path, index)
            }
            _ => Ok(()),
        }
    }

    fn get_id_type(&mut self, id: &HirId) -> Result<String, ()> {
        let object_type = get_node_type(self.cx, id);
        let kind = object_type.kind();
        let (def, _generic_args) = type_to_adt(kind)?;
        let type_name = self
            .cx
            .get_def_path(def.did())
            .iter()
            .map(|x| x.to_string())
            .collect::<Vec<_>>()
            .join("::");

        Ok(type_name)
    }

    fn get_path_type(&mut self, object_path: &'_ QPath<'_>) -> Result<String, ()> {
        let (_, object_path) = path_to_resolved(object_path)?;
        let object_decl_hir_id = resolution_to_local(&object_path.res)?;

        self.get_id_type(object_decl_hir_id)
    }

    fn get_expression_id(&mut self, expr: &'a ExprKind<'a>) -> Result<HirId, ()> {
        let index_qpath = expr_to_path(expr)?;
        let (_, index_path) = path_to_resolved(&index_qpath)?;
        Ok(*resolution_to_local(&index_path.res)?)
    }

    fn handle_path(
        &mut self,
        object_path: &'a QPath<'a>,
        index: &'a ExprKind<'a>,
    ) -> Result<(), ()> {
        let type_name = self.get_path_type(object_path)?;
        self.final_check(&type_name, index)
    }

    fn final_check(&mut self, object_type: &String, index: &'a ExprKind<'a>) -> Result<(), ()> {
        if object_type != "alloc::vec::Vec" {
            return Ok(());
        }

        if self.get_expression_id(index)? == self.index_id {
            self.has_vector_access = true;
        }
        Ok(())
    }
}

impl<'a, 'b> Visitor<'a> for VectorAccessVisitor<'a, 'b> {
    fn visit_expr(&mut self, expr: &'a Expr<'a>) {
        let _ = self.match_expression(&expr.kind);
        walk_expr(self, expr);
    }
}

//---------------------------------------------------------------------

fn type_to_adt<'hir>(
    kind: &'hir rustc_type_ir::TyKind<TyCtxt<'hir>>,
) -> Result<
    (
        &'hir <TyCtxt<'hir> as Interner>::AdtDef,
        &'hir <TyCtxt<'hir> as Interner>::GenericArgs,
    ),
    (),
> {
    if let TyKind::Adt(a, b) = kind {
        Ok((a, b))
    } else {
        Err(())
    }
}

//---------------------------------------------------------------------

fn stmt_to_expr<'hir>(kind: &'hir StmtKind<'hir>) -> Result<&'hir Expr<'hir>, ()> {
    if let StmtKind::Expr(a) = kind {
        Ok(a)
    } else {
        Err(())
    }
}

//---------------------------------------------------------------------

fn expr_to_drop_temps<'hir>(kind: &'hir ExprKind<'hir>) -> Result<&'hir Expr<'hir>, ()> {
    if let ExprKind::DropTemps(a) = kind {
        Ok(a)
    } else {
        Err(())
    }
}

fn expr_to_match<'hir>(
    kind: &'hir ExprKind<'hir>,
) -> Result<(&'hir Expr<'hir>, &'hir [rustc_hir::Arm<'hir>], MatchSource), ()> {
    if let ExprKind::Match(a, b, c) = kind {
        Ok((a, b, *c))
    } else {
        Err(())
    }
}

fn expr_to_call<'hir>(
    kind: &'hir ExprKind<'hir>,
) -> Result<(&'hir Expr<'hir>, &'hir [Expr<'hir>]), ()> {
    if let ExprKind::Call(a, b) = kind {
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

fn expr_to_struct<'hir>(
    kind: &'hir ExprKind<'hir>,
) -> Result<
    (
        &'hir QPath<'hir>,
        &'hir [ExprField<'hir>],
        Option<&'hir Expr<'hir>>,
    ),
    (),
> {
    if let ExprKind::Struct(a, b, c) = kind {
        Ok((a, b, *c))
    } else {
        Err(())
    }
}

fn expr_to_lit<'hir>(kind: &'hir ExprKind<'hir>) -> Result<&'hir rustc_hir::Lit, ()> {
    if let ExprKind::Lit(a) = kind {
        Ok(a)
    } else {
        Err(())
    }
}

fn expr_to_loop<'hir>(
    kind: &'hir ExprKind<'hir>,
) -> Result<(&'hir Block<'hir>, &Option<Label>, LoopSource, &Span), ()> {
    if let ExprKind::Loop(a, b, c, d) = kind {
        Ok((a, b, *c, d))
    } else {
        Err(())
    }
}

//---------------------------------------------------------------------

fn path_to_lang_item(path: &QPath) -> Result<(LangItem, Span), ()> {
    if let QPath::LangItem(a, b) = path {
        Ok((*a, *b))
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

//---------------------------------------------------------------------

fn resolution_to_local(resolution: &Res) -> Result<&HirId, ()> {
    if let Res::Local(a) = resolution {
        Ok(a)
    } else {
        Err(())
    }
}

//---------------------------------------------------------------------

fn lit_to_int(kind: &LitKind) -> Result<(u128, LitIntType), ()> {
    if let LitKind::Int(a, b) = kind {
        Ok((*a, *b))
    } else {
        Err(())
    }
}

//---------------------------------------------------------------------

fn pattern_to_struct<'hir>(
    pat: &'hir PatKind<'hir>,
) -> Result<(&QPath<'hir>, &'hir [PatField<'hir>], bool), ()> {
    if let PatKind::Struct(a, b, c) = pat {
        Ok((a, b, *c))
    } else {
        Err(())
    }
}

fn pattern_to_binding<'hir>(
    pat: &'hir PatKind<'hir>,
) -> Result<(&BindingAnnotation, &HirId, &Ident, &Option<&'hir Pat<'hir>>), ()> {
    if let PatKind::Binding(a, b, c, d) = pat {
        Ok((a, b, c, d))
    } else {
        Err(())
    }
}

//---------------------------------------------------------------------

fn is_range(item: LangItem) -> bool {
    matches!(
        item,
        LangItem::Range | LangItem::RangeInclusiveStruct | LangItem::RangeInclusiveNew
    )
}

//---------------------------------------------------------------------

fn handle_expr<'a>(me: &mut ForLoopVisitor<'a, '_>, expr: &'a Expr<'a>) -> Result<(), ()> {
    //Ignore DropTemps()
    let expr = expr_to_drop_temps(&expr.kind).or(Ok(expr))?;

    let (match_expr, arms, source) = expr_to_match(&expr.kind)?;
    if source != MatchSource::ForLoopDesugar {
        return Ok(());
    }
    let (func, args) = expr_to_call(&match_expr.kind)?;
    let qpath = expr_to_path(&func.kind)?;
    let (item, _) = path_to_lang_item(&qpath)?;
    if item != LangItem::IntoIterIntoIter {
        return Ok(());
    }
    if args.first().is_none() {
        return Ok(());
    }
    let (qpath, fields, _) = expr_to_struct(&args.first().unwrap().kind)?;
    let (langitem, _) = path_to_lang_item(qpath)?;
    if !is_range(langitem) {
        return Ok(());
    }
    if fields.last().is_none() {
        return Ok(());
    }
    let lit = expr_to_lit(&fields.last().unwrap().expr.kind)?;
    let _ = lit_to_int(&lit.node)?;
    if arms.first().is_none() {
        return Ok(());
    }
    let (block, _, loopsource, _) = expr_to_loop(&arms.first().unwrap().body.kind)?;
    if loopsource != LoopSource::ForLoop {
        return Ok(());
    }
    if block.stmts.first().is_none() {
        return Ok(());
    }
    let stmtexpr = stmt_to_expr(&block.stmts.first().unwrap().kind)?;
    let (_, some_none_arms, match_source) = expr_to_match(&stmtexpr.kind)?;
    if match_source != MatchSource::ForLoopDesugar {
        return Ok(());
    }

    let mut visitor = VectorAccessVisitor {
        has_vector_access: false,
        index_id: expr.hir_id,
        cx: me.cx,
    };
    for arm in some_none_arms {
        let hir_id = (|| -> Result<HirId, ()> {
            let (qpath, pats, _) = pattern_to_struct(&arm.pat.kind)?;
            let (item_type, _) = path_to_lang_item(qpath)?;
            if item_type != LangItem::OptionSome {
                return Err(());
            }
            if pats.last().is_none() {
                return Err(());
            }
            let (_, hir_id, _ident, _) = pattern_to_binding(&pats.last().unwrap().pat.kind)?;
            Ok(*hir_id)
        })();

        if let Ok(hir_id) = hir_id {
            visitor.index_id = hir_id;
            walk_expr(&mut visitor, arm.body);
        }
    }

    if visitor.has_vector_access {
        me.span_constant.insert(expr.span);
    }

    Ok(())
}

impl<'a, 'b> Visitor<'a> for ForLoopVisitor<'a, 'b> {
    fn visit_expr(&mut self, expr: &'a rustc_hir::Expr<'a>) {
        let _ = handle_expr(self, expr);
        walk_expr(self, expr);
    }
}

impl<'tcx> LateLintPass<'tcx> for IteratorsOverIndexing {
    fn check_fn(
        &mut self,
        cx: &rustc_lint::LateContext<'tcx>,
        kind: rustc_hir::intravisit::FnKind<'tcx>,
        _decl: &'tcx rustc_hir::FnDecl<'tcx>,
        body: &'tcx rustc_hir::Body<'tcx>,
        _: Span,
        _: LocalDefId,
    ) {
        if let FnKind::Method(_ident, _sig) = kind {
            let span_constant = {
                let mut visitor = ForLoopVisitor {
                    span_constant: HashSet::new(),
                    cx,
                };
                walk_expr(&mut visitor, body.value);
                visitor.span_constant
            };

            for span in span_constant {
                span_lint_and_help(
                    cx,
                    ITERATORS_OVER_INDEXING,
                    span,
                    LINT_MESSAGE,
                    None,
                    "Instead, use an iterator or index to `.len()`.",
                );
            }
        }
    }
}
