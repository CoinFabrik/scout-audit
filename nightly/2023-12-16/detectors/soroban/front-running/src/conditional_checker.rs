use rustc_hir::{def::Res, BinOpKind, Expr, ExprKind, HirId, QPath, UnOp};

const PANIC_INDUCING_FUNCTIONS: [&str; 2] = ["panic", "bail"];

#[derive(Clone, Copy, Hash, Eq, PartialEq, Default, Debug)]
pub struct ConditionalChecker {
    pub greater_expr: Option<HirId>,
    pub lesser_expr: Option<HirId>,
}

impl ConditionalChecker {
    fn handle_binary_op(&mut self, op: BinOpKind, left: &Expr, right: &Expr) -> bool {
        match op {
            BinOpKind::Ge | BinOpKind::Gt => {
                self.greater_expr = get_res_hir_id(left);
                self.lesser_expr = get_res_hir_id(right);
                true
            }
            BinOpKind::Le | BinOpKind::Lt => {
                self.lesser_expr = get_res_hir_id(left);
                self.greater_expr = get_res_hir_id(right);
                true
            }
            _ => false,
        }
    }

    pub fn handle_condition(&mut self, condition: &Expr) -> bool {
        match &condition.kind {
            ExprKind::Binary(op, left, right) => self.handle_binary_op(op.node, left, right),
            ExprKind::Unary(UnOp::Not, unary_expr) => {
                if let ExprKind::Binary(op, left, right) = &unary_expr.kind {
                    self.handle_binary_op(op.node, left, right)
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}

pub fn get_res_hir_id(expr: &Expr) -> Option<HirId> {
    if let ExprKind::Path(QPath::Resolved(_, path)) = &expr.kind {
        if let Res::Local(hir_id) = path.res {
            Some(hir_id)
        } else {
            None
        }
    } else {
        None
    }
}

pub fn is_panic_inducing_call(func: &Expr<'_>) -> bool {
    if let ExprKind::Path(QPath::Resolved(_, path)) = &func.kind {
        return PANIC_INDUCING_FUNCTIONS.iter().any(|&func| {
            path.segments
                .iter()
                .any(|segment| segment.ident.name.as_str().contains(func))
        });
    }
    false
}
