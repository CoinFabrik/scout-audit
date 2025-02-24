#![feature(rustc_private)]
#![feature(let_chains)]

extern crate rustc_hir;
extern crate rustc_span;

use common::{
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use rustc_hir::{
    intravisit::{walk_expr, FnKind, Visitor},
    Expr, ExprKind, QPath, TyKind,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::Span;

const LINT_MESSAGE: &str = "Unused return enum";

#[expose_lint_info]
pub static UNUSED_RETURN_ENUM_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "Ink! messages can return a Result enum with a custom error type. This is useful for the caller to know what went wrong when the message fails. The definition of the Result type enum consists of two variants: Ok and Err. If any of the variants is not used, the code could be simplified or it could imply a bug.    ",
    severity: Severity::Minor,
    help: "https://coinfabrik.github.io/scout-audit/docs/detectors/ink/unused-return-enum",
    vulnerability_class: VulnerabilityClass::ErrorHandling,
};

dylint_linting::declare_late_lint! {
    pub UNUSED_RETURN_ENUM,
    Warn,
    LINT_MESSAGE
}

struct CounterVisitor {
    count_err: u32,
    count_ok: u32,
    found_try: bool,
    found_return: bool,
    span: Vec<Span>,
}

impl<'tcx> Visitor<'tcx> for CounterVisitor {
    fn visit_expr(&mut self, expr: &'tcx Expr) {
        if let ExprKind::Call(func, _args) = expr.kind
            && let ExprKind::Path(qpath) = &func.kind
            && let QPath::Resolved(_ty, path) = qpath
        {
            let vec: Vec<String> = path.segments.iter().map(|f| f.ident.to_string()).collect();
            let fun_path = vec.join("::");
            if fun_path.ends_with("Ok") {
                self.count_ok += 1;
                self.span.push(func.span);
            } else if fun_path.ends_with("Err") {
                self.count_err += 1;
                self.span.push(func.span);
            }
        }
        match expr.kind {
            ExprKind::Ret(retval) => {
                if retval.is_some()
                    && let ExprKind::Call(func, _args) = retval.unwrap().kind
                    && let ExprKind::Path(qpath) = &func.kind
                    && let QPath::Resolved(_, path) = qpath
                    && let Some(last_segment) = path.segments.last()
                {
                    match last_segment.ident.as_str() {
                        "Err" | "Ok" => {}
                        _ => {
                            self.found_return = true;
                        }
                    }
                }
            }
            ExprKind::Match(_expr, _arms, rustc_hir::MatchSource::TryDesugar(_)) => {
                self.found_try = true;
            }
            _ => {}
        }
        walk_expr(self, expr);
    }
}
impl<'tcx> LateLintPass<'tcx> for UnusedReturnEnum {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        fnkind: rustc_hir::intravisit::FnKind<'tcx>,
        decl: &'tcx rustc_hir::FnDecl<'tcx>,
        body: &'tcx rustc_hir::Body<'tcx>,
        _: Span,
        _: rustc_span::def_id::LocalDefId,
    ) {
        if let FnKind::Method(_ident, _fnsig) = fnkind {
        } else {
            return;
        }

        let mut expression_return: bool = false;
        //if the function uses expression return (not using ; at the end),
        //the base expression of the function is a block and the return value is stored in block.expr
        if let ExprKind::Block(block, _label) = body.value.kind
            && block.expr.is_some()
            && let ExprKind::Call(func, _args) = block.expr.unwrap().kind
            && let ExprKind::Path(qpath) = &func.kind
            && let QPath::Resolved(_, path) = qpath
        {
            if let Some(last_segment) = path.segments.last() {
                match last_segment.ident.as_str() {
                    "Err" | "Ok" => {}
                    _ => {
                        expression_return = true;
                    }
                }
            }
            //if to ignore some automatically generated functions.
            // this is provisional i will improve it when i know how
            if let Some(first) = path.segments.first()
                && first.ident.as_str() == "{{root}}"
            {
                expression_return = true;
            }
        }

        match decl.output {
            rustc_hir::FnRetTy::Return(ret) => {
                if let TyKind::Path(qpath) = &ret.kind
                    && let QPath::Resolved(_ty, path) = qpath
                {
                    //ignore function if not returns a Result type
                    if path
                        .segments
                        .last()
                        .is_some_and(|f| f.ident.to_string() != "Result")
                    {
                        return;
                    }
                };
            }
            _ => return,
        };

        let mut visitor = CounterVisitor {
            count_ok: 0,
            count_err: 0,
            found_try: false,
            found_return: false,
            span: Vec::new(),
        };

        walk_expr(&mut visitor, body.value);

        if !visitor.found_try
            && !visitor.found_return
            && !expression_return
            && (visitor.count_err == 0 || visitor.count_ok == 0)
        {
            visitor.span.iter().for_each(|span| {
                clippy_utils::diagnostics::span_lint_and_help(
                    cx,
                    UNUSED_RETURN_ENUM,
                    *span,
                    LINT_MESSAGE,
                    None,
                    "If any of the variants (Ok/Err) is not used, the code could be simplified or it could imply a bug"
                );
            });
        }
    }
}
