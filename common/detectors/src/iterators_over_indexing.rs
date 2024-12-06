extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_middle;
extern crate rustc_span;
extern crate rustc_type_ir;

use std::collections::HashSet;

use rustc_hir::{
    intravisit::{walk_expr, Visitor},
    Expr, ExprKind, HirId, LangItem, LoopSource, MatchSource,
    QPath
};
use rustc_lint::LateContext;
use rustc_span::Span;
use analysis::decomposers::*;

pub struct IteratorsOverIndexingConfig{
    pub check_get: bool,
    pub check_index: bool,
    pub relevant_object_types: HashSet<String>,
}

struct ForLoopVisitor<'a, 'b, 'c> {
    config: &'c IteratorsOverIndexingConfig,
    span_constant: HashSet<Span>,
    cx: &'b LateContext<'a>,
}

struct VectorAccessVisitor<'a, 'b, 'c> {
    config: &'c IteratorsOverIndexingConfig,
    index_id: HirId,
    has_vector_access: bool,
    cx: &'b LateContext<'a>,
}

pub fn get_node_type<'a>(
    cx: &rustc_lint::LateContext<'a>,
    hir_id: &HirId,
) -> rustc_middle::ty::Ty<'a> {
    cx.typeck_results().node_type(*hir_id)
}

#[allow(dead_code)]
impl<'a, 'b, 'c> VectorAccessVisitor<'a, 'b, 'c> {
    fn match_expression(&mut self, kind: &'a ExprKind<'a>) -> Option<()> {
        match kind {
            ExprKind::MethodCall(function_name, object, arguments, _) => {
                if !self.config.check_get {
                    None?;
                }
                let name = function_name.ident.name.as_str();
                if name == "get" || name == "get_unchecked" {
                    self.match_method_call_obj(&object.kind, arguments)
                }else{
                    None
                }
            },
            ExprKind::Index(arr, index, _) => {
                if !self.config.check_index{
                    None
                }else{
                    self.handle_index(&arr.hir_id, &arr.kind, &index.kind)
                }
            }
            _ => None,
        }
    }

    fn handle_index(
        &mut self,
        expr_id: &HirId,
        arr: &'a ExprKind<'a>,
        index: &'a ExprKind<'a>,
    ) -> Option<()> {
        match arr {
            ExprKind::Field(_, _) | ExprKind::Path(_) => {
                let type_name = self.get_id_type(expr_id)?;
                self.final_check(&type_name, index)
            }
            _ => Some(()),
        }
    }

    fn match_method_call_obj(
        &mut self,
        kind: &'a ExprKind<'a>,
        arguments: &'a [Expr<'a>],
    ) -> Option<()> {
        match kind {
            ExprKind::Path(object_path) => {
                if arguments.len() != 1 {
                    None?;
                }
                let index = &arguments.first().unwrap().kind;

                self.handle_path(object_path, index)
            }
            _ => None,
        }
    }

    fn get_id_type(&mut self, id: &HirId) -> Option<String> {
        analysis::ty_to_string(self.cx, &get_node_type(self.cx, id))
    }

    fn get_path_type(&mut self, object_path: &'_ QPath<'_>) -> Option<String> {
        let (_, object_path) = path_to_resolved(object_path)?;
        let object_decl_hir_id = resolution_to_local(&object_path.res)?;

        self.get_id_type(object_decl_hir_id)
    }

    fn get_expression_id(&mut self, expr: &'a ExprKind<'a>) -> Option<HirId> {
        let index_qpath = expr_to_path(expr)?;
        let (_, index_path) = path_to_resolved(&index_qpath)?;
        Some(*resolution_to_local(&index_path.res)?)
    }

    fn handle_path(
        &mut self,
        object_path: &'a QPath<'a>,
        index: &'a ExprKind<'a>,
    ) -> Option<()> {
        let type_name = self.get_path_type(object_path)?;
        self.final_check(&type_name, index)
    }

    fn final_check(&mut self, object_type: &String, index: &'a ExprKind<'a>) -> Option<()> {
        if !self.config.relevant_object_types.contains(object_type) {
            None?;
        }

        if self.get_expression_id(index)? == self.index_id {
            self.has_vector_access = true;
        }
        None
    }
}

impl<'a, 'b, 'c> Visitor<'a> for VectorAccessVisitor<'a, 'b, 'c> {
    fn visit_expr(&mut self, expr: &'a Expr<'a>) {
        let _ = self.match_expression(&expr.kind);
        walk_expr(self, expr);
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

fn handle_expr<'a>(me: &mut ForLoopVisitor<'a, '_, '_>, expr: &'a Expr<'a>) -> Option<()> {
    //Ignore DropTemps()
    let expr = expr_to_drop_temps(&expr.kind).or(Some(expr))?;

    let (match_expr, arms, source) = expr_to_match(&expr.kind)?;
    if source != MatchSource::ForLoopDesugar {
        return Some(());
    }
    let (func, args) = expr_to_call(&match_expr.kind)?;
    let qpath = expr_to_path(&func.kind)?;
    let (item, _) = path_to_lang_item(&qpath)?;
    if item != LangItem::IntoIterIntoIter {
        return Some(());
    }
    if args.first().is_none() {
        return Some(());
    }
    let (qpath, fields, _) = expr_to_struct(&args.first().unwrap().kind)?;
    let (langitem, _) = path_to_lang_item(qpath)?;
    if !is_range(langitem) {
        return Some(());
    }
    if fields.last().is_none() {
        return Some(());
    }
    let lit = expr_to_lit(&fields.last().unwrap().expr.kind)?;
    let _ = lit_to_int(&lit.node)?;
    if arms.first().is_none() {
        return Some(());
    }
    let (block, _, loopsource, _) = expr_to_loop(&arms.first().unwrap().body.kind)?;
    if loopsource != LoopSource::ForLoop {
        return Some(());
    }
    if block.stmts.first().is_none() {
        return Some(());
    }
    let stmtexpr = stmt_to_expr(&block.stmts.first().unwrap().kind)?;
    let (_, some_none_arms, match_source) = expr_to_match(&stmtexpr.kind)?;
    if match_source != MatchSource::ForLoopDesugar {
        return Some(());
    }

    let mut visitor = VectorAccessVisitor {
        config: me.config,
        has_vector_access: false,
        index_id: expr.hir_id,
        cx: me.cx,
    };
    for arm in some_none_arms {
        let hir_id = (|| -> Option<HirId> {
            let (qpath, pats, _) = pattern_to_struct(&arm.pat.kind)?;
            let (item_type, _) = path_to_lang_item(qpath)?;
            if item_type != LangItem::OptionSome {
                return None;
            }
            if pats.last().is_none() {
                return None;
            }
            let (_, hir_id, _ident, _) = pattern_to_binding(&pats.last().unwrap().pat.kind)?;
            Some(*hir_id)
        })();

        if let Some(hir_id) = hir_id {
            visitor.index_id = hir_id;
            walk_expr(&mut visitor, arm.body);
        }
    }

    if visitor.has_vector_access {
        me.span_constant.insert(expr.span);
    }

    Some(())
}

impl<'a, 'b, 'c> Visitor<'a> for ForLoopVisitor<'a, 'b, 'c> {
    fn visit_expr(&mut self, expr: &'a rustc_hir::Expr<'a>) {
        let _ = handle_expr(self, expr);
        walk_expr(self, expr);
    }
}

pub fn check_expr<'a, 'b>(cx: &LateContext<'a>, expr: &'a Expr<'_>, config: &'b IteratorsOverIndexingConfig) -> HashSet<Span> {
    let mut visitor = ForLoopVisitor {
        config,
        span_constant: HashSet::new(),
        cx,
    };
    walk_expr(&mut visitor, expr);
    visitor.span_constant
}
