#![feature(rustc_private)]

extern crate rustc_hir;
extern crate rustc_span;

use clippy_wrappers::span_lint_and_help;
use common::expose_lint_info;
use rustc_hir::{
    intravisit::{walk_expr, FnKind, Visitor},
    BinOpKind, Body, Expr, ExprKind, FnDecl, UnOp,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::{def_id::LocalDefId, Span, Symbol};
use std::collections::HashSet;
use utils::ConstantAnalyzer;

pub const LINT_MESSAGE: &str = "Potential for integer arithmetic overflow/underflow. Consider checked, wrapping or saturating arithmetic.";

#[expose_lint_info]
pub static INTEGER_OVERFLOW_OR_UNDERFLOW_INFO: LintInfo = LintInfo {
    name: "Integer Overflow/Underflow",
    short_message: LINT_MESSAGE,
    long_message: "An overflow/underflow is typically caught and generates an error. When it is not caught, the operation will result in an inexact result which could lead to serious problems.",
    severity: "Critical",
    help: "https://coinfabrik.github.io/scout-soroban/docs/vulnerabilities/integer-overflow-or-underflow",
    vulnerability_class: "Arithmetic",
};

dylint_linting::declare_late_lint! {
    pub INTEGER_OVERFLOW_OR_UNDERFLOW,
    Warn,
    LINT_MESSAGE
}
enum Type {
    Overflow,
    Underflow,
    OverflowAndUnderflow,
}

impl Type {
    fn message(&self) -> &'static str {
        match self {
            Type::Overflow => "overflow",
            Type::Underflow => "underflow",
            Type::OverflowAndUnderflow => "overflow or underflow",
        }
    }
}

enum Cause {
    Add,
    Sub,
    Mul,
    Pow,
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
    constant_analyzer: ConstantAnalyzer<'a, 'tcx>,
}

impl<'tcx> IntegerOverflowOrUnderflowVisitor<'_, 'tcx> {
    pub fn check_pow(&mut self, expr: &Expr<'tcx>, base: &Expr<'tcx>, exponent: &Expr<'tcx>) {
        if self.constant_analyzer.is_constant(base) && self.constant_analyzer.is_constant(exponent)
        {
            return;
        }

        let base_type = self.cx.typeck_results().expr_ty(base);
        if base_type.is_integral() {
            self.findings
                .push(Finding::new(expr.span, Type::Overflow, Cause::Pow));
        }
    }

    pub fn check_negate(&mut self, expr: &Expr<'tcx>, operand: &Expr<'tcx>) {
        if self.constant_analyzer.is_constant(operand) {
            return;
        }

        let operand_type = self.cx.typeck_results().expr_ty(operand);
        if operand_type.is_integral() && operand_type.is_signed() {
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
            self.cx.typeck_results().expr_ty(left).peel_refs(),
            self.cx.typeck_results().expr_ty(right).peel_refs(),
        );
        if !left_type.is_integral() || !right_type.is_integral() {
            return;
        }

        let (finding_type, cause) = if self.is_complex_operation {
            (Type::OverflowAndUnderflow, Cause::Multiple)
        } else {
            match op {
                BinOpKind::Add => (Type::Overflow, Cause::Add),
                BinOpKind::Sub => (Type::Underflow, Cause::Sub),
                BinOpKind::Mul => (Type::Overflow, Cause::Mul),
                _ => return,
            }
        };

        self.findings
            .push(Finding::new(expr.span, finding_type, cause));
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
            ExprKind::MethodCall(method_name, receiver, args, ..) => {
                if method_name.ident.name == Symbol::intern("pow") {
                    self.check_pow(expr, receiver, &args[0]);
                }
            }
            _ => (),
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
        let mut constant_analyzer = ConstantAnalyzer {
            cx,
            constants: HashSet::new(),
        };
        constant_analyzer.visit_body(body);

        // Analyze the function for integer overflow/underflow
        let mut visitor = IntegerOverflowOrUnderflowVisitor {
            cx,
            findings: Vec::new(),
            is_complex_operation: false,
            constant_analyzer,
        };
        visitor.visit_body(body);

        // Report any findings
        for finding in visitor.findings {
            span_lint_and_help(
                cx,
                INTEGER_OVERFLOW_OR_UNDERFLOW,
                finding.span,
                finding.generate_message(),
                None,
                "Consider using the checked version of this operation/s",
            )
        }
    }
}
