use clippy_utils::consts::Constant;
use common::analysis::{get_expr_hir_id_opt, ConstantAnalyzer};
use if_chain::if_chain;
use rustc_hir::{Expr, ExprKind, HirId};
use std::collections::{HashMap, HashSet};

// Main safety context for tracking arithmetic safety
pub struct SafetyContext {
    subtraction_context: SubtractionContext,
    division_context: DivisionContext,
}

impl SafetyContext {
    pub fn new() -> Self {
        Self {
            subtraction_context: SubtractionContext::new(),
            division_context: DivisionContext::new(),
        }
    }

    pub fn check_operation(&mut self, init: &Expr<'_>, pat_hir_id: HirId) {
        self.subtraction_context.check_operation(init, pat_hir_id);
        // TODO: In the future we might want to have a 'global' context to track safe divisions
    }

    pub fn is_subtraction_safe<'tcx>(
        &self,
        minuend: &Expr<'tcx>,
        subtrahend: &Expr<'tcx>,
        constant_analyzer: &ConstantAnalyzer<'_, 'tcx>,
    ) -> bool {
        self.subtraction_context
            .is_subtraction_safe(minuend, subtrahend, constant_analyzer)
    }

    pub fn is_division_safe<'tcx>(
        &self,
        divisor: &Expr<'tcx>,
        constant_analyzer: &ConstantAnalyzer<'_, 'tcx>,
    ) -> bool {
        self.division_context
            .is_division_safe(divisor, constant_analyzer)
    }

    pub fn enter_scope(&mut self) {
        self.subtraction_context.enter_scope();
        self.division_context.enter_scope();
    }

    pub fn exit_scope(&mut self) {
        self.subtraction_context.exit_scope();
        self.division_context.exit_scope();
    }

    pub fn track_comparison<'tcx>(
        &mut self,
        less_eq: &Expr<'tcx>,
        greater_eq: &Expr<'tcx>,
        constant_analyzer: &ConstantAnalyzer<'_, 'tcx>,
    ) {
        self.subtraction_context
            .track_comparison(less_eq, greater_eq, constant_analyzer);
        self.division_context
            .track_comparison(less_eq, greater_eq, constant_analyzer);
    }

    pub fn track_zero_comparison(&mut self, expr: &Expr<'_>, is_not_equal: bool) {
        self.division_context
            .track_zero_comparison(expr, is_not_equal);
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Debug)]
enum SafetyValue {
    Variable(HirId),
    Constant(i128),
}

pub struct SubtractionContext {
    safe_subtractions: HashMap<HirId, HashSet<SafetyValue>>,
    current_scope: Option<HashMap<HirId, HashSet<SafetyValue>>>,
}

impl SubtractionContext {
    pub fn new() -> Self {
        Self {
            safe_subtractions: HashMap::new(),
            current_scope: None,
        }
    }

    pub fn enter_scope(&mut self) {
        self.current_scope = Some(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        self.current_scope = None;
    }

    pub fn track_comparison<'tcx>(
        &mut self,
        less_eq: &Expr<'tcx>,
        greater_eq: &Expr<'tcx>,
        constant_analyzer: &ConstantAnalyzer<'_, 'tcx>,
    ) {
        // Analyze if less_eq is a constant
        if let Some(const_val) = constant_analyzer.get_constant(less_eq) {
            if_chain! {
                if let Some(greater_hir_id) = get_expr_hir_id_opt(greater_eq);
                if let Some(scope) = &mut self.current_scope;
                if let Constant::Int(val) = const_val;
                then {
                    scope
                    .entry(greater_hir_id)
                    .or_default()
                    .insert(SafetyValue::Constant(val as i128));
                }
            }
            return;
        }

        // Analyze if greater_eq is a constant
        if let Some(const_val) = constant_analyzer.get_constant(greater_eq) {
            if_chain! {
                if let Some(less_hir_id) = get_expr_hir_id_opt(less_eq);
                if let Some(scope) = &mut self.current_scope;
                if let Constant::Int(val) = const_val;
                then {
                    scope
                    .entry(less_hir_id)
                    .or_default()
                    .insert(SafetyValue::Constant(val as i128));
                }
            }
            return;
        }

        // Handle variable-variable comparison
        if_chain! {
            if let Some(less_hir_id) = get_expr_hir_id_opt(less_eq);
            if let Some(greater_hir_id) = get_expr_hir_id_opt(greater_eq);
            if let Some(scope) = &mut self.current_scope;
            then {
                scope
                    .entry(greater_hir_id)
                    .or_default()
                    .insert(SafetyValue::Variable(less_hir_id));
            }
        }
    }

    pub fn check_operation(&mut self, init: &Expr, pat_hir_id: HirId) {
        if let ExprKind::MethodCall(method_name, receiver, args, ..) = init.kind {
            match method_name.ident.name.as_str() {
                "min" => self.track_min_operation(pat_hir_id, receiver, &args[0]),
                "max" => self.track_max_operation(pat_hir_id, receiver, &args[0]),
                "clamp" if args.len() == 2 => {
                    self.track_clamp_operation(pat_hir_id, &args[0], &args[1])
                }
                _ => {}
            }
        }
    }

    pub fn track_min_operation(&mut self, result_hir_id: HirId, a: &Expr, b: &Expr) {
        if let Some(a_hir_id) = get_expr_hir_id_opt(a) {
            self.record_safe_subtraction(a_hir_id, SafetyValue::Variable(result_hir_id));
        }
        if let Some(b_hir_id) = get_expr_hir_id_opt(b) {
            self.record_safe_subtraction(b_hir_id, SafetyValue::Variable(result_hir_id));
        }
    }

    pub fn track_max_operation(&mut self, result_hir_id: HirId, a: &Expr, b: &Expr) {
        if let Some(a_hir_id) = get_expr_hir_id_opt(a) {
            self.record_safe_subtraction(result_hir_id, SafetyValue::Variable(a_hir_id));
        }
        if let Some(b_hir_id) = get_expr_hir_id_opt(b) {
            self.record_safe_subtraction(result_hir_id, SafetyValue::Variable(b_hir_id));
        }
    }

    pub fn track_clamp_operation(&mut self, result_hir_id: HirId, min_val: &Expr, max_val: &Expr) {
        if let Some(min_hir_id) = get_expr_hir_id_opt(min_val) {
            self.record_safe_subtraction(result_hir_id, SafetyValue::Variable(min_hir_id));
        }
        if let Some(max_hir_id) = get_expr_hir_id_opt(max_val) {
            self.record_safe_subtraction(max_hir_id, SafetyValue::Variable(result_hir_id));
        }
    }

    pub fn is_subtraction_safe<'tcx>(
        &self,
        minuend: &Expr<'tcx>,
        subtrahend: &Expr<'tcx>,
        constant_analyzer: &ConstantAnalyzer<'_, 'tcx>,
    ) -> bool {
        // 1. Both minuend and subtrahend are constants
        if_chain! {
            if let Some(minuend_const) = constant_analyzer.get_constant(minuend);
            if let Some(subtrahend_const) = constant_analyzer.get_constant(subtrahend);
            if let (Constant::Int(m), Constant::Int(s)) = (minuend_const, subtrahend_const);
            then {
                if (m as i128) >= (s as i128) {
                    return true;
                }
            }
        }

        // 2. Subtrahend is a constant, minuend is a variable
        if_chain! {
            if let Some(subtrahend_const) = constant_analyzer.get_constant(subtrahend);
            if let Constant::Int(s) = subtrahend_const;
            if let Some(minuend_hir_id) = get_expr_hir_id_opt(minuend);
            if let Some(scope) = &self.current_scope;
            if let Some(safe_values) = scope.get(&minuend_hir_id);
            then {
                let subtrahend_value = SafetyValue::Constant(s as i128);
                if safe_values.contains(&subtrahend_value) {
                    return true;
                }

                return self
                    .safe_subtractions
                    .get(&minuend_hir_id)
                    .map(|safe_values| safe_values.contains(&subtrahend_value))
                    .unwrap_or(false);
            }
        }

        // 3. Subtrahend is a variable, minuend is a constant
        if_chain! {
            if let Some(subtrahend_hir_id) = get_expr_hir_id_opt(subtrahend);
            if let Some(scope) = &self.current_scope;
            if let Some(safe_values) = scope.get(&subtrahend_hir_id);
            if let Some(minuend_const) = constant_analyzer.get_constant(minuend);
            if let Constant::Int(m) = minuend_const;
            then {
                let minuend_value = SafetyValue::Constant(m as i128);
                if safe_values.contains(&minuend_value) {
                    return true;
                }

                return self
                    .safe_subtractions
                    .get(&subtrahend_hir_id)
                    .map(|safe_values| safe_values.contains(&minuend_value))
                    .unwrap_or(false);
            }
        }

        // 4. Both minuend and subtrahend are variables
        if_chain! {
            if let Some(minuend_hir_id) = get_expr_hir_id_opt(minuend);
            if let Some(subtrahend_hir_id) = get_expr_hir_id_opt(subtrahend);
            if let Some(scope) = &self.current_scope;
            if let Some(safe_values) = scope.get(&minuend_hir_id);
            then {
                let subtrahend_value = SafetyValue::Variable(subtrahend_hir_id);
                if safe_values.contains(&subtrahend_value) {
                    return true;
                }

                return self
                    .safe_subtractions
                    .get(&minuend_hir_id)
                    .map(|safe_values| safe_values.contains(&subtrahend_value))
                    .unwrap_or(false);
            }
        }

        false
    }

    fn record_safe_subtraction(&mut self, minuend: HirId, subtrahend: SafetyValue) {
        if let Some(scope) = &mut self.current_scope {
            scope.entry(minuend).or_default().insert(subtrahend.clone());
        } else {
            self.safe_subtractions
                .entry(minuend)
                .or_default()
                .insert(subtrahend);
        }
    }
}

pub struct DivisionContext {
    current_scope: Option<HashSet<HirId>>,
}

impl DivisionContext {
    pub fn new() -> Self {
        Self {
            current_scope: None,
        }
    }

    pub fn enter_scope(&mut self) {
        self.current_scope = Some(HashSet::new());
    }

    pub fn exit_scope(&mut self) {
        self.current_scope = None;
    }

    fn record_non_zero(&mut self, hir_id: HirId) {
        if let Some(scope) = &mut self.current_scope {
            scope.insert(hir_id);
        }
    }

    pub fn track_comparison<'tcx>(
        &mut self,
        less_eq: &Expr<'tcx>,
        greater_eq: &Expr<'tcx>,
        constant_analyzer: &ConstantAnalyzer<'_, 'tcx>,
    ) {
        if let Some(Constant::Int(val)) = constant_analyzer.get_constant(less_eq) {
            if_chain! {
                if let Some(greater_hir_id) = get_expr_hir_id_opt(greater_eq);
                if val != 0;
                then {
                    self.record_non_zero(greater_hir_id);
                }
            }
            return;
        }

        if let Some(Constant::Int(val)) = constant_analyzer.get_constant(greater_eq) {
            if_chain! {
                if let Some(less_hir_id) = get_expr_hir_id_opt(less_eq);
                if val != 0;
                then {
                    self.record_non_zero(less_hir_id);
                }
            }
        }
    }

    // Track equality comparisons with zero
    pub fn track_zero_comparison(&mut self, expr: &Expr<'_>, is_not_equal: bool) {
        if is_not_equal {
            if let Some(expr_hir_id) = get_expr_hir_id_opt(expr) {
                self.record_non_zero(expr_hir_id);
            }
        }
    }

    pub fn is_division_safe<'tcx>(
        &self,
        divisor: &Expr<'tcx>,
        constant_analyzer: &ConstantAnalyzer<'_, 'tcx>,
    ) -> bool {
        // Check if it's a constant first
        if let Some(Constant::Int(val)) = constant_analyzer.get_constant(divisor) {
            return val != 0;
        }

        // Then check if we know it's non-zero from comparisons
        if let Some(divisor_hir_id) = get_expr_hir_id_opt(divisor) {
            if let Some(scope) = &self.current_scope {
                return scope.contains(&divisor_hir_id);
            }
        }

        false
    }
}
