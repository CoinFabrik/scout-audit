#![feature(rustc_private)]

extern crate rustc_hir;
extern crate rustc_middle;
extern crate rustc_span;

mod safety_context;

use clippy_utils::consts::Constant;
use clippy_utils::diagnostics::span_lint_and_help;
use common::{
    analysis::{match_type_to_str, ConstantAnalyzer},
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use rustc_hir::{
    intravisit::{walk_expr, FnKind, Visitor},
    BinOpKind, Body, Expr, ExprKind, FnDecl, StmtKind, UnOp,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_middle::ty::Ty;
use rustc_span::{def_id::LocalDefId, Span, Symbol};
use safety_context::SafetyContext;

pub const LINT_MESSAGE: &str = "Potential for integer arithmetic overflow/underflow. Consider checked, wrapping or saturating arithmetic.";

#[expose_lint_info]
pub static INTEGER_OVERFLOW_OR_UNDERFLOW_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "An overflow/underflow is typically caught and generates an error. When it is not caught, the operation will result in an inexact result which could lead to serious problems.",
    severity: Severity::Critical,
    help: "https://coinfabrik.github.io/scout-audit/docs/detectors/substrate/integer-overflow-or-underflow",
    vulnerability_class: VulnerabilityClass::ErrorHandling,
};

dylint_linting::declare_late_lint! {
    pub INTEGER_OVERFLOW_OR_UNDERFLOW,
    Warn,
    LINT_MESSAGE
}
enum Type {
    Overflow,
    Underflow,
    OverflowUnderflow,
}

impl Type {
    fn message(&self) -> &'static str {
        match self {
            Type::Overflow => "overflow",
            Type::Underflow => "underflow",
            Type::OverflowUnderflow => "overflow or underflow",
        }
    }
}

enum Cause {
    Add,
    Sub,
    Mul,
    Pow,
    Div,
    Negate,
    Multiple,
}

impl Cause {
    fn message(&self) -> &'static str {
        match self {
            Cause::Add => "addition operation",
            Cause::Sub => "subtraction operation",
            Cause::Mul => "multiplication operation",
            Cause::Pow => "exponentiation operation",
            Cause::Div => "division operation",
            Cause::Negate => "negation operation",
            Cause::Multiple => "operation",
        }
    }
}

pub struct Finding {
    span: Span,
    type_: Type,
    cause: Cause,
}

impl Finding {
    fn new(span: Span, type_: Type, cause: Cause) -> Self {
        Finding { span, type_, cause }
    }

    fn generate_message(&self) -> String {
        format!(
            "This {} could {}.",
            self.cause.message(),
            self.type_.message()
        )
    }
}
pub struct IntegerOverflowOrUnderflowVisitor<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    findings: Vec<Finding>,
    is_complex_operation: bool,
    constant_analyzer: &'a mut ConstantAnalyzer<'a, 'tcx>,
    safety_context: SafetyContext,
}

impl<'a, 'tcx> IntegerOverflowOrUnderflowVisitor<'a, 'tcx> {
    pub fn check_pow(&mut self, expr: &Expr<'tcx>, base: &Expr<'tcx>, exponent: &Expr<'tcx>) {
        if self.constant_analyzer.is_constant(base) && self.constant_analyzer.is_constant(exponent)
        {
            return;
        }

        let base_type = self.cx.typeck_results().expr_ty(base);
        if self.is_overflow_susceptible_type(base_type) {
            self.findings
                .push(Finding::new(expr.span, Type::Overflow, Cause::Pow));
        }
    }

    pub fn check_negate(&mut self, expr: &Expr<'tcx>, operand: &Expr<'tcx>) {
        if self.constant_analyzer.is_constant(operand) {
            return;
        }

        let operand_type = self.cx.typeck_results().expr_ty(operand);
        if self.is_overflow_susceptible_type(operand_type) && operand_type.is_signed() {
            self.findings
                .push(Finding::new(expr.span, Type::Overflow, Cause::Negate));
        }
    }

    pub fn check_binary(
        &mut self,
        expr: &Expr<'tcx>,
        op: BinOpKind,
        left: &Expr<'tcx>,
        right: &Expr<'tcx>,
    ) {
        if self.constant_analyzer.is_constant(left) && self.constant_analyzer.is_constant(right) {
            return;
        }

        let (left_type, right_type) = (
            self.cx.typeck_results().expr_ty(left),
            self.cx.typeck_results().expr_ty(right),
        );

        if !self.is_overflow_susceptible_type(left_type)
            || !self.is_overflow_susceptible_type(right_type)
        {
            return;
        }

        let (finding_type, cause) = if self.is_complex_operation {
            (Type::OverflowUnderflow, Cause::Multiple)
        } else {
            match op {
                BinOpKind::Add => (Type::Overflow, Cause::Add),
                BinOpKind::Sub => {
                    if self
                        .safety_context
                        .is_subtraction_safe(left, right, self.constant_analyzer)
                    {
                        return;
                    }
                    (Type::Underflow, Cause::Sub)
                }
                BinOpKind::Mul => (Type::Overflow, Cause::Mul),
                BinOpKind::Div => {
                    if self
                        .safety_context
                        .is_division_safe(right, self.constant_analyzer)
                    {
                        return;
                    }
                    (Type::Overflow, Cause::Div)
                }
                _ => return,
            }
        };

        self.findings
            .push(Finding::new(expr.span, finding_type, cause));
    }

    fn is_overflow_susceptible_type(&self, ty: Ty<'_>) -> bool {
        let ty = ty.peel_refs();
        ty.is_integral() || match_type_to_str(self.cx, ty, "Balance")
    }
}

impl<'a, 'tcx> Visitor<'tcx> for IntegerOverflowOrUnderflowVisitor<'a, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx Expr<'tcx>) {
        match expr.kind {
            ExprKind::Binary(op, lhs, rhs) | ExprKind::AssignOp(op, lhs, rhs) => {
                self.is_complex_operation = matches!(lhs.kind, ExprKind::Binary(..))
                    || matches!(rhs.kind, ExprKind::Binary(..));
                self.check_binary(expr, op.node, lhs, rhs);
                if self.is_complex_operation {
                    return;
                }
            }
            ExprKind::Unary(UnOp::Neg, arg) => {
                self.check_negate(expr, arg);
            }
            ExprKind::Block(block, ..) => {
                block.stmts.iter().for_each(|stmt| {
                    if let StmtKind::Local(let_expr) = stmt.kind {
                        if let Some(init) = let_expr.init {
                            self.visit_expr(init);
                            self.safety_context
                                .check_operation(init, let_expr.pat.hir_id);
                        }
                    }
                });
            }
            ExprKind::Let(let_expr) => {
                self.safety_context
                    .check_operation(let_expr.init, let_expr.pat.hir_id);
            }
            ExprKind::If(cond, then_expr, _) => {
                if let ExprKind::DropTemps(cond) = cond.kind {
                    if let ExprKind::Binary(op, lhs, rhs) = cond.kind {
                        let (left, right, is_zero_comparison) = match op.node {
                            BinOpKind::Gt | BinOpKind::Ge => (rhs, lhs, false),
                            BinOpKind::Lt | BinOpKind::Le => (lhs, rhs, false),
                            BinOpKind::Eq | BinOpKind::Ne => {
                                if let Some(Constant::Int(0)) =
                                    self.constant_analyzer.get_constant(lhs)
                                {
                                    (rhs, lhs, true)
                                } else if let Some(Constant::Int(0)) =
                                    self.constant_analyzer.get_constant(rhs)
                                {
                                    (lhs, rhs, true)
                                } else {
                                    self.visit_expr(then_expr);
                                    return;
                                }
                            }
                            _ => {
                                self.visit_expr(then_expr);
                                return;
                            }
                        };

                        self.safety_context.enter_scope();
                        if is_zero_comparison {
                            self.safety_context
                                .track_zero_comparison(left, matches!(op.node, BinOpKind::Ne));
                        } else {
                            self.safety_context.track_comparison(
                                left,
                                right,
                                self.constant_analyzer,
                            );
                        }
                        self.visit_expr(then_expr);
                        self.safety_context.exit_scope();
                        return;
                    }
                }
            }
            ExprKind::MethodCall(method_name, receiver, args, ..) => {
                if method_name.ident.name == Symbol::intern("pow") {
                    self.check_pow(expr, receiver, &args[0]);
                }
            }
            _ => {}
        }

        walk_expr(self, expr);
    }
}

impl<'tcx> LateLintPass<'tcx> for IntegerOverflowOrUnderflow {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: FnKind<'tcx>,
        _: &'tcx FnDecl<'tcx>,
        body: &'tcx Body<'tcx>,
        span: Span,
        _: LocalDefId,
    ) {
        // If the function comes from a macro expansion, we ignore it
        if span.from_expansion() {
            return;
        }

        // Gather all compile-time variables in the function
        let mut constant_analyzer = ConstantAnalyzer::new(cx);
        constant_analyzer.visit_body(body);

        // Analyze the function for integer overflow/underflow
        let mut visitor = IntegerOverflowOrUnderflowVisitor {
            cx,
            findings: Vec::new(),
            is_complex_operation: false,
            constant_analyzer: &mut constant_analyzer,
            safety_context: SafetyContext::new(),
        };
        visitor.visit_expr(body.value);

        // Report any findings
        for finding in visitor.findings {
            span_lint_and_help(
                cx,
                INTEGER_OVERFLOW_OR_UNDERFLOW,
                finding.span,
                &finding.generate_message(),
                None,
                "Consider using the checked version of this operation/s",
            )
        }
    }
}
