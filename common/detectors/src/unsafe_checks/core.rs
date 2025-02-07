extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_span;

use crate::unsafe_checks::option_result_checker::ConditionalChecker;
use analysis::ConstantAnalyzer;
use clippy_utils::{diagnostics::span_lint_and_help, higher::IfOrIfLet, is_from_proc_macro};
use if_chain::if_chain;
use rustc_hir::{
    intravisit::{walk_expr, Visitor},
    Expr, ExprKind,
};
use rustc_lint::{LateContext, Lint};
use rustc_span::{sym, Symbol};

use super::{arithmetic_checker::ArithmeticChecker, option_result_checker::OptionResultChecker};

pub struct UnsafeChecks<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    lint: &'static Lint,
    checks_symbol: Symbol,
    arithmetic_checker: ArithmeticChecker<'a, 'tcx>,
    option_result_checker: OptionResultChecker<'a, 'tcx>,
}

impl<'a, 'tcx> UnsafeChecks<'a, 'tcx> {
    pub fn new(
        cx: &'a LateContext<'tcx>,
        lint: &'static Lint,
        constant_analyzer: ConstantAnalyzer<'a, 'tcx>,
        checks_symbol: Symbol,
    ) -> Self {
        Self {
            cx,
            lint,
            checks_symbol,
            arithmetic_checker: ArithmeticChecker::new(constant_analyzer),
            option_result_checker: OptionResultChecker::new(cx),
        }
    }

    fn check_expr_for_unsafe_method(&mut self, expr: &Expr<'tcx>) {
        if_chain! {
            if !is_from_proc_macro(self.cx, expr);
            if let ExprKind::MethodCall(path_segment, receiver, _, _) = &expr.kind;
            if path_segment.ident.name == self.checks_symbol;
            if !self.arithmetic_checker.is_arithmetic_safe(receiver);
            if self.option_result_checker.is_method_call_unsafe(receiver);
            then {
                // TODO: Implement this
                let help_message = if self.checks_symbol == sym::expect {
                    "Please, use a custom error instead of `expect`"
                } else {
                    self.option_result_checker.get_help_message(self.option_result_checker.determine_unwrap_type(receiver))
                };

                span_lint_and_help(
                    self.cx,
                    self.lint,
                    expr.span,
                    self.lint.desc,
                    None,
                    help_message,
                );
            }
        }
    }
}

impl<'a, 'tcx> Visitor<'tcx> for UnsafeChecks<'a, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx Expr<'_>) {
        // Check for try desugaring '?'
        self.option_result_checker.check_for_try(expr);

        // If we are inside an `if` or `if let` expression, we analyze its body
        if !self.option_result_checker.conditional_checker.is_empty() {
            match &expr.kind {
                ExprKind::Ret(..) => self.option_result_checker.handle_if_expressions(),
                ExprKind::Call(func, _)
                    if self.option_result_checker.is_panic_inducing_call(func) =>
                {
                    self.option_result_checker.handle_if_expressions()
                }
                _ => {}
            }
        }

        // Find `if` or `if let` expressions
        if let Some(IfOrIfLet {
            cond,
            then: if_expr,
            r#else: _,
        }) = IfOrIfLet::hir(expr)
        {
            // Left TOOD: handle returns in arithmetic context.
            self.arithmetic_checker.analyze_condition(cond);

            // If we are interested in the condition (if it is a CheckType) we traverse the body.
            let conditional_checker = ConditionalChecker::from_expression(cond);
            self.option_result_checker
                .update_conditional_checker(&conditional_checker, true);
            walk_expr(self, if_expr);
            self.option_result_checker
                .update_conditional_checker(&conditional_checker, false);

            self.arithmetic_checker.clear_context();
            return;
        }

        // If we find an unsafe `expect`, we raise a warning
        self.check_expr_for_unsafe_method(expr);

        walk_expr(self, expr);
    }
}
