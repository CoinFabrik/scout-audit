extern crate rustc_hir;

use analysis::{get_expr_hir_id_opt, ConstantAnalyzer};
use clippy_utils::consts::Constant;
use if_chain::if_chain;
use rustc_hir::{BinOpKind, Expr, ExprKind, HirId};
use std::collections::HashMap;

pub struct ArithmeticChecker<'a, 'tcx> {
    constant_analyzer: ConstantAnalyzer<'a, 'tcx>,
    arithmetic_context: HashMap<HirId, ArithmeticSafetyInfo>,
}

#[derive(Clone, Debug)]
pub struct ArithmeticSafetyInfo {
    max_value: u128,
    safe_hir_id: Option<HirId>,
    non_zero: bool,
}

impl<'a, 'tcx> ArithmeticChecker<'a, 'tcx> {
    pub fn new(constant_analyzer: ConstantAnalyzer<'a, 'tcx>) -> Self {
        Self {
            constant_analyzer,
            arithmetic_context: HashMap::new(),
        }
    }

    pub fn is_arithmetic_safe(&mut self, expr: &Expr<'tcx>) -> bool {
        if_chain! {
            if let ExprKind::MethodCall(path_segment, _, args, _) = &expr.kind;
            if path_segment.ident.name.as_str() == "checked_div";
            if let Some(arg_id) = get_expr_hir_id_opt(&args[0]);
            let arg_constant = self.constant_analyzer.get_constant(&args[0]);
            then {
                if let Some(arg_constant) = arg_constant {
                    if let Constant::Int(arg_int) = arg_constant {
                        return arg_int != 0;
                    }
                }

                if let Some(safety_info) = self.arithmetic_context.get(&arg_id) {
                    return safety_info.non_zero;
                }
                return false;
            }
        }

        if_chain! {
            if let ExprKind::MethodCall(path_segment, receiver, args, _) = &expr.kind;
            if path_segment.ident.name.as_str() == "checked_sub";
            if let Some(receiver_id) = get_expr_hir_id_opt(receiver);
            if let Some(arg_id) = get_expr_hir_id_opt(&args[0]);
            let receiver_constant = self.constant_analyzer.get_constant(receiver);
            let arg_constant = self.constant_analyzer.get_constant(&args[0]);
            then {
                match (receiver_constant, arg_constant) {
                    (Some(receiver_value), Some(arg_value)) => if_chain! {
                        if let Constant::Int(arg_int) = arg_value;
                        if let Constant::Int(receiver_int) = receiver_value;
                        then {
                            return arg_int <= receiver_int;
                        }
                    },
                    (Some(receiver_value), None) => if_chain! {
                        if let Some(safety_info) = self.arithmetic_context.get(&arg_id);
                        if let Constant::Int(receiver_int) = receiver_value;
                        then {
                            return safety_info.max_value <= receiver_int;
                        }
                    },
                    (None, Some(arg_value)) => if_chain! {
                        if let Some(safety_info) = self.arithmetic_context.get(&receiver_id);
                        if let Constant::Int(arg_int) = arg_value;
                        then {
                            return safety_info.max_value >= arg_int;
                        }
                    },
                    (None, None) => if_chain! {
                        if let Some(receiver_safety_info) = self.arithmetic_context.get(&receiver_id);
                        if let Some(arg_safety_info) = self.arithmetic_context.get(&arg_id);
                        then {
                            if let Some(safe_hir_id) = receiver_safety_info.safe_hir_id {
                                if arg_id == safe_hir_id {
                                    return true;
                                }
                            }
                            return arg_safety_info.max_value <= receiver_safety_info.max_value;
                        }
                    }
                }
            }
        }
        false
    }

    fn set_arithmetic_safety_info(
        &mut self,
        target_expr: &Expr,
        max_value: u128,
        non_zero: bool,
        safe_hir_id: Option<HirId>,
    ) {
        if let Some(target_id) = get_expr_hir_id_opt(target_expr) {
            self.arithmetic_context.insert(
                target_id,
                ArithmeticSafetyInfo {
                    max_value,
                    safe_hir_id,
                    non_zero,
                },
            );
        }
    }

    pub fn clear_context(&mut self) {
        self.arithmetic_context.clear();
    }

    pub fn analyze_condition(&mut self, condition: &Expr<'tcx>) {
        if let ExprKind::Binary(op, left, right) = &condition.kind {
            let left_constant = self.constant_analyzer.get_constant(left);
            let right_constant = self.constant_analyzer.get_constant(right);

            let mut handle_constant_compare =
                |constant: Constant, var_expr: &Expr, is_left_constant: bool| {
                    if let Constant::Int(value) = constant {
                        let (target, max_value, non_zero) = match op.node {
                            BinOpKind::Eq => (var_expr, 0, value != 0),
                            BinOpKind::Ne => (var_expr, 0, value == 0),
                            BinOpKind::Ge | BinOpKind::Gt if is_left_constant => {
                                (var_expr, value, value != 0)
                            }
                            BinOpKind::Le | BinOpKind::Lt if !is_left_constant => {
                                (var_expr, value, value != 0)
                            }
                            BinOpKind::Le | BinOpKind::Lt if is_left_constant => {
                                (var_expr, value, value != 0)
                            }
                            BinOpKind::Ge | BinOpKind::Gt => (var_expr, value, value != 0),
                            _ => return,
                        };
                        self.set_arithmetic_safety_info(target, max_value, non_zero, None);
                    }
                };

            match (left_constant, right_constant) {
                (Some(left_value), None) => handle_constant_compare(left_value, right, true),
                (None, Some(right_value)) => handle_constant_compare(right_value, left, false),
                (None, None) => match op.node {
                    BinOpKind::Ge | BinOpKind::Gt => {
                        self.set_arithmetic_safety_info(left, 0, false, get_expr_hir_id_opt(right));
                    }
                    BinOpKind::Le | BinOpKind::Lt => {
                        self.set_arithmetic_safety_info(right, 0, false, get_expr_hir_id_opt(left));
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}
