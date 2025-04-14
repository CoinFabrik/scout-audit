extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_span;

use std::collections::{HashMap, HashSet};

use if_chain::if_chain;
use rustc_hir::{
    intravisit::{walk_expr, Visitor},
    Expr, ExprKind,
};
use rustc_lint::LateContext;
use rustc_span::def_id::DefId;

/// `FunctionCallVisitor` is a visitor struct used to construct a call graph of functions within Rust code.
///
/// It records which functions are called by each function, helping in understanding
/// the relationships between different parts of the codebase.
pub struct FunctionCallVisitor<'a, 'tcx> {
    /// Context from the compiler's late lint pass, providing access to various compiler internals.
    cx: &'a LateContext<'tcx>,

    /// The `DefId` of the current function being visited. This acts as a key in the `call_graph`.
    current_fn: DefId,

    /// A mutable reference to a HashMap acting as the call graph. Each function (`DefId`) is mapped
    /// to a set of functions (`DefId`) it calls.
    call_graph: &'a mut HashMap<DefId, HashSet<DefId>>,
}

impl<'a, 'tcx> FunctionCallVisitor<'a, 'tcx> {
    /// Constructs a new `FunctionCallVisitor`.
    ///
    /// # Parameters
    /// - `cx`: The context from the compiler's late lint pass.
    /// - `current_fn`: The `DefId` of the current function being analyzed.
    /// - `call_graph`: A mutable reference to the call graph being constructed.
    ///
    /// # Returns
    /// A new instance of `FunctionCallVisitor`.
    pub fn new(
        cx: &'a LateContext<'tcx>,
        current_fn: DefId,
        call_graph: &'a mut HashMap<DefId, HashSet<DefId>>,
    ) -> Self {
        FunctionCallVisitor {
            cx,
            current_fn,
            call_graph,
        }
    }
}

impl<'a, 'tcx> Visitor<'tcx> for FunctionCallVisitor<'a, 'tcx> {
    /// Visits expressions within the code. If the expression is a function call, it is added to the call graph.
    ///
    /// # Parameters
    /// - `expr`: The expression being visited.
    fn visit_expr(&mut self, expr: &'tcx Expr<'tcx>) {
        // If the expression is a function call, record it in the call graph.
        if_chain! {
            if let ExprKind::Call(call_expr, _) = expr.kind; // Check if the expression is a call.
            if let ExprKind::Path(ref qpath) = call_expr.kind; // Ensure the call is a path expression.
            if let Some(def_id) = self.cx.qpath_res(qpath, call_expr.hir_id).opt_def_id(); // Resolve the path to a DefId.
            then {
                // Add the called function's DefId to the current function's entry in the call graph.
                self.call_graph
                    .entry(self.current_fn)
                    .or_default()
                    .insert(def_id);
            }
        }

        // Continue walking through the expression tree.
        walk_expr(self, expr);
    }
}
