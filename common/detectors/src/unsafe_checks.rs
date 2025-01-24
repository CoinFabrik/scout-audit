extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_span;

use analysis::{get_node_type_opt, match_type_to_str, ConstantAnalyzer};
use clippy_utils::{diagnostics::span_lint_and_help, higher::IfOrIfLet, is_from_proc_macro};
use if_chain::if_chain;
use rustc_hir::{
    def::Res,
    intravisit::{walk_expr, Visitor},
    BinOpKind, Expr, ExprKind, HirId, LangItem, MatchSource, PathSegment, QPath, UnOp,
};
use rustc_lint::{LateContext, Lint};
use rustc_span::{sym, Symbol};
use std::collections::HashSet;

const PANIC_INDUCING_FUNCTIONS: [&str; 2] = ["panic", "bail"];

/// Represents the type of check performed on method call expressions to determine their safety or behavior.
#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub enum CheckType {
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

    /// Determines if it is safe to unwrap the value without further checks, i.e., the value is present.
    fn is_safe(self) -> bool {
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

// Enum to represent the type being unwrapped
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum UnwrapType {
    Option,
    Result,
}

pub struct UnsafeChecks<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    lint: &'static Lint,
    constant_analyzer: ConstantAnalyzer<'a, 'tcx>,
    conditional_checker: HashSet<ConditionalChecker>,
    checked_exprs: HashSet<HirId>,
    returns_result_or_option: bool,
    // Only sym::expect or sym::unwrap are supported, DO NOT USE OTHER SYMBOLS
    checks_symbol: Symbol,
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
            constant_analyzer,
            conditional_checker: HashSet::new(),
            checked_exprs: HashSet::new(),
            returns_result_or_option: false,
        }
    }

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

    fn update_conditional_checker(
        &mut self,
        conditional_checkers: &HashSet<ConditionalChecker>,
        set: bool,
    ) {
        for checker in conditional_checkers {
            if set {
                self.conditional_checker.insert(*checker);
                if checker.check_type.is_safe() {
                    self.checked_exprs.insert(checker.checked_expr_hir_id);
                }
            } else {
                if checker.check_type.is_safe() {
                    self.checked_exprs.remove(&checker.checked_expr_hir_id);
                }
                self.conditional_checker.remove(checker);
            }
        }
    }

    fn get_check_info(&self, receiver: &Expr<'tcx>) -> Option<HirId> {
        if_chain! {
            if let ExprKind::Path(QPath::Resolved(_, path)) = &receiver.kind;
            if let Res::Local(hir_id) = path.res;
            then {
                return Some(hir_id);
            }
        }
        None
    }

    fn is_method_call_unsafe(&self, path_segment: &PathSegment, receiver: &Expr<'tcx>) -> bool {
        if path_segment.ident.name == self.checks_symbol {
            if self.constant_analyzer.is_constant(receiver) {
                return false;
            }

            return self
                .get_check_info(receiver)
                .map_or(true, |id| !self.checked_exprs.contains(&id));
        }
        false
    }

    /// Process conditional expressions to determine if they should halt execution.
    fn handle_if_expressions(&mut self) {
        self.conditional_checker.iter().for_each(|checker| {
            if checker.check_type.should_halt_execution() {
                self.checked_exprs.insert(checker.checked_expr_hir_id);
            }
        });
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

    fn check_expr_for_unsafe_method(&mut self, expr: &Expr<'tcx>) {
        if_chain! {
            if !is_from_proc_macro(self.cx, expr);
            if let ExprKind::MethodCall(path_segment, receiver, _, _) = &expr.kind;
            if self.is_method_call_unsafe(path_segment, receiver);
            then {
                let help_message = if self.checks_symbol == sym::expect {
                    "Please, use a custom error instead of `expect`"
                } else {
                    self.get_help_message(self.determine_unwrap_type(receiver))
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

    fn determine_unwrap_type(&self, receiver: &Expr<'tcx>) -> UnwrapType {
        let type_opt = get_node_type_opt(self.cx, &receiver.hir_id);
        if let Some(type_) = type_opt {
            if match_type_to_str(self.cx, type_, "Result") {
                return UnwrapType::Result;
            }
        }
        UnwrapType::Option
    }

    fn get_help_message(&self, unwrap_type: UnwrapType) -> &'static str {
        match (self.returns_result_or_option, unwrap_type) {
            (true, UnwrapType::Option) => "Consider using `ok_or` to convert Option to Result",
            (true, UnwrapType::Result) => "Consider using the `?` operator for error propagation",
            (false, UnwrapType::Option) => {
                "Consider pattern matching or using `if let` instead of `unwrap`"
            }
            (false, UnwrapType::Result) => {
                "Consider handling the error case explicitly or using `if let` instead of `unwrap`"
            }
        }
    }
}

impl<'a, 'tcx> Visitor<'tcx> for UnsafeChecks<'a, 'tcx> {
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
        self.check_expr_for_unsafe_method(expr);

        walk_expr(self, expr);
    }
}
