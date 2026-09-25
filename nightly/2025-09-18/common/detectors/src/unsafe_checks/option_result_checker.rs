extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_span;

use analysis::{get_node_type_opt, match_type_to_str, ConstantAnalyzer};
use if_chain::if_chain;
use rustc_hir::{def::Res, BinOpKind, Expr, ExprKind, HirId, LangItem, MatchSource, QPath, UnOp};
use rustc_lint::LateContext;
use rustc_span::Symbol;
use std::collections::HashSet;

const PANIC_INDUCING_FUNCTIONS: [&str; 2] = ["panic", "bail"];

pub struct OptionResultChecker<'a, 'tcx> {
    pub cx: &'a LateContext<'tcx>,
    pub conditional_checker: HashSet<ConditionalChecker>,
    pub checked_exprs: HashSet<HirId>,
}

/// Represents a conditional checker that is used to analyze `if` or `if let` expressions.
#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub struct ConditionalChecker {
    check_type: CheckType,
    checked_expr_hir_id: HirId,
}

impl ConditionalChecker {
    /// Handle te condition of the `if` or `if let` expression.
    pub fn handle_condition(condition: &Expr<'_>, inverse: bool) -> HashSet<Self> {
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
    pub fn from_expression(condition: &Expr<'_>) -> HashSet<Self> {
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

// Enum to represent the type being unwrapped
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum UnwrapType {
    Option,
    Result,
}

impl<'a, 'tcx> OptionResultChecker<'a, 'tcx> {
    pub fn new(cx: &'a LateContext<'tcx>) -> Self {
        Self {
            cx,
            conditional_checker: HashSet::new(),
            checked_exprs: HashSet::new(),
        }
    }

    pub fn is_method_call_unsafe(
        &self,
        receiver: &Expr<'tcx>,
        constant_analyzer: &ConstantAnalyzer<'a, 'tcx>,
    ) -> bool {
        if constant_analyzer.get_constant(receiver).is_some() {
            return false;
        }

        self.get_check_info(receiver)
            .map_or(true, |id| !self.checked_exprs.contains(&id))
    }

    pub fn get_check_info(&self, receiver: &Expr<'tcx>) -> Option<HirId> {
        if_chain! {
            if let ExprKind::Path(QPath::Resolved(_, path)) = &receiver.kind;
            if let Res::Local(hir_id) = path.res;
            then {
                return Some(hir_id);
            }
        }
        None
    }

    pub fn determine_unwrap_type(&self, receiver: &Expr<'tcx>) -> UnwrapType {
        let type_opt = get_node_type_opt(self.cx, &receiver.hir_id);
        if let Some(type_) = type_opt {
            if match_type_to_str(self.cx, type_, "Result") {
                return UnwrapType::Result;
            }
        }
        UnwrapType::Option
    }

    pub fn get_help_message(&self, unwrap_type: UnwrapType) -> &'static str {
        match unwrap_type {
            UnwrapType::Option => "Consider pattern matching or using `if let` instead of `unwrap`",
            UnwrapType::Result => {
                "Consider handling the error case explicitly or using `if let` instead of `unwrap`"
            }
        }
    }

    pub fn update_conditional_checker(
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

    pub fn handle_if_expressions(&mut self) {
        self.conditional_checker.iter().for_each(|checker| {
            if checker.check_type.should_halt_execution() {
                self.checked_exprs.insert(checker.checked_expr_hir_id);
            }
        });
    }

    pub fn is_panic_inducing_call(&self, func: &Expr<'tcx>) -> bool {
        if let ExprKind::Path(QPath::Resolved(_, path)) = &func.kind {
            return PANIC_INDUCING_FUNCTIONS.iter().any(|&func| {
                path.segments
                    .iter()
                    .any(|segment| segment.ident.name.as_str().contains(func))
            });
        }
        false
    }

    pub fn check_for_try(&mut self, expr: &Expr<'tcx>) {
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
