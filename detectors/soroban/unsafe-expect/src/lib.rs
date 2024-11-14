#![feature(rustc_private)]
#![allow(clippy::enum_variant_names)]

extern crate rustc_hir;
extern crate rustc_span;

use clippy_utils::higher::IfOrIfLet;
use clippy_utils::diagnostics::span_lint_and_help;
use common::{
    analysis::{fn_returns, ConstantAnalyzer},
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use if_chain::if_chain;
use rustc_hir::{
    def::Res,
    def_id::LocalDefId,
    intravisit::{walk_expr, FnKind, Visitor},
    BinOpKind, Body, Expr, ExprKind, FnDecl, HirId, LangItem, MatchSource, PathSegment, QPath,
    UnOp,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::{sym, Span, Symbol};
use std::{collections::HashSet, hash::Hash};

const LINT_MESSAGE: &str = "Unsafe usage of `expect`";
const PANIC_INDUCING_FUNCTIONS: [&str; 2] = ["panic", "bail"];

#[expose_lint_info]
pub static UNSAFE_EXPECT_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "In Rust, the expect method is commonly used for error handling. It retrieves the value from a Result or Option and panics with a specified error message if an error occurs. However, using expect can lead to unexpected program crashes.    ",
    severity: Severity::Medium,
    help: "https://coinfabrik.github.io/scout-soroban/docs/detectors/unsafe-expect",
    vulnerability_class: VulnerabilityClass::ErrorHandling,
};

dylint_linting::declare_late_lint! {
    pub UNSAFE_EXPECT,
    Warn,
    LINT_MESSAGE
}

/// Represents the type of check performed on method call expressions to determine their safety or behavior.
#[derive(Clone, Copy, Hash, Eq, PartialEq)]
enum CheckType {
    IsSome,
    IsNone,
    IsOk,
    IsErr,
}

impl CheckType {
    fn from_method_name(name: Symbol) -> Option<Self> {
        match name.as_str() {
            "is_some" => Some(Self::IsSome),
            "is_none" => Some(Self::IsNone),
            "is_ok" => Some(Self::IsOk),
            "is_err" => Some(Self::IsErr),
            _ => None,
        }
    }

    fn inverse(self) -> Self {
        match self {
            Self::IsSome => Self::IsNone,
            Self::IsNone => Self::IsSome,
            Self::IsOk => Self::IsErr,
            Self::IsErr => Self::IsOk,
        }
    }

    /// Determines if the check type implies execution should halt, such as in error conditions.
    fn should_halt_execution(self) -> bool {
        matches!(self, Self::IsNone | Self::IsErr)
    }

    /// Determines if it is safe to expect the value without further checks, i.e., the value is present.
    fn is_safe_to_expect(self) -> bool {
        matches!(self, Self::IsSome | Self::IsOk)
    }
}

/// Represents a conditional checker that is used to analyze `if` or `if let` expressions.
#[derive(Clone, Copy, Hash, Eq, PartialEq)]
struct ConditionalChecker {
    check_type: CheckType,
    checked_expr_hir_id: HirId,
}

impl ConditionalChecker {
    /// Handle te condition of the `if` or `if let` expression.
    fn handle_condition(condition: &Expr<'_>, inverse: bool) -> HashSet<Self> {
        if_chain! {
            if let ExprKind::MethodCall(path_segment, receiver, _, _) = condition.kind;
            if let Some(check_type) = CheckType::from_method_name(path_segment.ident.name);
            if let ExprKind::Path(QPath::Resolved(_, checked_expr_path)) = receiver.kind;
            if let Res::Local(checked_expr_hir_id) = checked_expr_path.res;
            then {
                let check_type = if inverse { check_type.inverse() } else { check_type };
                return std::iter::once(Self { check_type, checked_expr_hir_id }).collect();
            }
        }
        HashSet::new()
    }

    /// Constructs a ConditionalChecker from an expression if it matches a method call with a valid CheckType.
    fn from_expression(condition: &Expr<'_>) -> HashSet<Self> {
        match condition.kind {
            // Single `not` expressions are supported
            ExprKind::Unary(op, condition) => Self::handle_condition(condition, op == UnOp::Not),
            // Multiple `or` expressions are supported
            ExprKind::Binary(op, left_condition, right_condition) if op.node == BinOpKind::Or => {
                let mut result = Self::from_expression(left_condition);
                result.extend(Self::from_expression(right_condition));
                result
            }
            ExprKind::MethodCall(..) => Self::handle_condition(condition, false),
            _ => HashSet::new(),
        }
    }
}

/// Main unsafe-expect visitor
struct UnsafeExpectVisitor<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    constant_analyzer: ConstantAnalyzer<'a, 'tcx>,
    conditional_checker: HashSet<ConditionalChecker>,
    checked_exprs: HashSet<HirId>,
    linted_spans: HashSet<Span>,
}

impl<'a, 'tcx> UnsafeExpectVisitor<'a, 'tcx> {
    fn is_panic_inducing_call(&self, func: &Expr<'tcx>) -> bool {
        if let ExprKind::Path(QPath::Resolved(_, path)) = &func.kind {
            return PANIC_INDUCING_FUNCTIONS.iter().any(|&func| {
                path.segments
                    .iter()
                    .any(|segment| segment.ident.name.as_str().contains(func))
            });
        }
        false
    }

    fn get_expect_info(&self, receiver: &Expr<'tcx>) -> Option<HirId> {
        if_chain! {
            if let ExprKind::Path(QPath::Resolved(_, path)) = &receiver.kind;
            if let Res::Local(hir_id) = path.res;
            then {
                return Some(hir_id);
            }
        }
        None
    }

    fn update_conditional_checker(
        &mut self,
        conditional_checkers: &HashSet<ConditionalChecker>,
        set: bool,
    ) {
        for checker in conditional_checkers {
            if set {
                self.conditional_checker.insert(*checker);
                if checker.check_type.is_safe_to_expect() {
                    self.checked_exprs.insert(checker.checked_expr_hir_id);
                }
            } else {
                if checker.check_type.is_safe_to_expect() {
                    self.checked_exprs.remove(&checker.checked_expr_hir_id);
                }
                self.conditional_checker.remove(checker);
            }
        }
    }

    /// Process conditional expressions to determine if they should halt execution.
    fn handle_if_expressions(&mut self) {
        self.conditional_checker.iter().for_each(|checker| {
            if checker.check_type.should_halt_execution() {
                self.checked_exprs.insert(checker.checked_expr_hir_id);
            }
        });
    }

    fn is_method_call_unsafe(&self, path_segment: &PathSegment, receiver: &Expr<'tcx>) -> bool {
        if path_segment.ident.name == sym::expect {
            if self.constant_analyzer.is_constant(receiver) {
                return false;
            }

            return self
                .get_expect_info(receiver)
                .map_or(true, |id| !self.checked_exprs.contains(&id));
        }
        false
    }

    fn check_expr_for_unsafe_expect(&mut self, expr: &Expr<'tcx>) {
        if let ExprKind::MethodCall(path_segment, receiver, _, _) = &expr.kind {
            if self.is_method_call_unsafe(path_segment, receiver)
                && !self.linted_spans.contains(&expr.span)
            {
                span_lint_and_help(
                    self.cx,
                    UNSAFE_EXPECT,
                    expr.span,
                    LINT_MESSAGE,
                    None,
                    "Please, use a custom error instead of `expect`",
                );
                self.linted_spans.insert(expr.span);
            }
        }
    }

    fn check_for_try(&mut self, expr: &Expr<'tcx>) {
        if_chain! {
            // Check for match expressions desugared from try
            if let ExprKind::Match(expr, _, MatchSource::TryDesugar(_)) = &expr.kind;
            if let ExprKind::Call(func, args) = &expr.kind;
            // Check for the try trait branch lang item
            if let ExprKind::Path(QPath::LangItem(LangItem::TryTraitBranch, _)) = &func.kind;
            if let ExprKind::Path(QPath::Resolved(_, path)) = &args[0].kind;
            // Get the HirId of the expression that is being checked
            if let Res::Local(hir_id) = path.res;
            then {
                self.checked_exprs.insert(hir_id);
            }
        }
    }
}

impl<'a, 'tcx> Visitor<'tcx> for UnsafeExpectVisitor<'a, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx Expr<'_>) {
        // Check for try desugaring '?'
        self.check_for_try(expr);

        // If we are inside an `if` or `if let` expression, we analyze its body
        if !self.conditional_checker.is_empty() {
            match &expr.kind {
                ExprKind::Ret(..) => self.handle_if_expressions(),
                ExprKind::Call(func, _) if self.is_panic_inducing_call(func) => {
                    self.handle_if_expressions()
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
            // If we are interested in the condition (if it is a CheckType) we traverse the body.
            let conditional_checker = ConditionalChecker::from_expression(cond);
            self.update_conditional_checker(&conditional_checker, true);
            walk_expr(self, if_expr);
            self.update_conditional_checker(&conditional_checker, false);
            return;
        }

        // If we find an unsafe `expect`, we raise a warning
        self.check_expr_for_unsafe_expect(expr);

        walk_expr(self, expr);
    }
}

impl<'tcx> LateLintPass<'tcx> for UnsafeExpect {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: FnKind<'tcx>,
        fn_decl: &'tcx FnDecl<'tcx>,
        body: &'tcx Body<'tcx>,
        span: Span,
        _: LocalDefId,
    ) {
        // If the function comes from a macro expansion or does not return a Result<(), ()> or Option<()>, we don't want to analyze it.
        if span.from_expansion()
            || !fn_returns(fn_decl, sym::Result) && !fn_returns(fn_decl, sym::Option)
        {
            return;
        }

        let mut constant_analyzer = ConstantAnalyzer {
            cx,
            constants: HashSet::new(),
        };
        constant_analyzer.visit_body(body);

        let mut visitor = UnsafeExpectVisitor {
            cx,
            constant_analyzer,
            checked_exprs: HashSet::new(),
            conditional_checker: HashSet::new(),
            linted_spans: HashSet::new(),
        };

        walk_expr(&mut visitor, body.value);
    }
}
