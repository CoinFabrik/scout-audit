#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_span;

use rustc_hir::{
    intravisit::{walk_expr, Visitor},
    Expr, ExprKind,
};
use rustc_lint::LateLintPass;
use rustc_span::Span;
use scout_audit_internal::Detector;

dylint_linting::declare_late_lint! {
    /// ### What it does
    /// Checks for usage of `unsafe` blocks.
    ///
    /// ### Why is this bad?
    /// `unsafe` blocks should not be used unless absolutely necessary.
    ///
    /// ### Example
    /// ```rust
    ///pub fn unsafe_function(n: u64) -> u64 {
    ///    unsafe {
    ///        let mut i = n as f64;
    ///        let mut y = i.to_bits();
    ///        y = 0x5fe6ec85e7de30da - (y >> 1);
    ///        i = f64::from_bits(y);
    ///        i *= 1.5 - 0.5 * n as f64 * i * i;
    ///        i *= 1.5 - 0.5 * n as f64 * i * i;
    ///
    ///        let result_ptr: *mut f64 = &mut i;
    ///        let result = *result_ptr;
    ///
    ///        result.to_bits()
    ///     }
    ///}
    /// Use instead:
    /// ```rust
    ///pub fn unsafe_function(n: u64) -> u64 {
    ///        let mut i = n as f64;
    ///        let mut y = i.to_bits();
    ///        y = 0x5fe6ec85e7de30da - (y >> 1);
    ///        i = f64::from_bits(y);
    ///        i *= 1.5 - 0.5 * n as f64 * i * i;
    ///        i *= 1.5 - 0.5 * n as f64 * i * i;
    ///        result.to_bits()
    ///}
    /// ```
    pub AVOID_UNSAFE_BLOCK,
    Warn,
    Detector::AvoidUnsafeBlock.get_lint_message()
}

impl<'tcx> LateLintPass<'tcx> for AvoidUnsafeBlock {
    fn check_fn(
        &mut self,
        cx: &rustc_lint::LateContext<'tcx>,
        _: rustc_hir::intravisit::FnKind<'tcx>,
        _: &'tcx rustc_hir::FnDecl<'tcx>,
        body: &'tcx rustc_hir::Body<'tcx>,
        _: rustc_span::Span,
        _: rustc_hir::def_id::LocalDefId,
    ) {
        struct UnsafeBlockVisitor {
            unsafe_blocks: Vec<Option<Span>>,
        }

        impl<'tcx> Visitor<'tcx> for UnsafeBlockVisitor {
            fn visit_expr(&mut self, expr: &'tcx Expr<'_>) {
                if let ExprKind::Block(block, _) = expr.kind {
                    if block.rules
                        == rustc_hir::BlockCheckMode::UnsafeBlock(
                            rustc_hir::UnsafeSource::UserProvided,
                        )
                    {
                        self.unsafe_blocks.push(Some(expr.span));
                    }
                }

                walk_expr(self, expr);
            }
        }

        let mut visitor = UnsafeBlockVisitor {
            unsafe_blocks: Vec::new(),
        };

        walk_expr(&mut visitor, body.value);

        visitor.unsafe_blocks.iter().for_each(|span| {
            if let Some(span) = span {
                Detector::AvoidUnsafeBlock.span_lint(cx, AVOID_UNSAFE_BLOCK, *span);
            }
        });
    }
}
