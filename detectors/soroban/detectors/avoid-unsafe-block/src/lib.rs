#![feature(rustc_private)]

extern crate rustc_hir;
extern crate rustc_span;

use clippy_wrappers::span_lint;
use rustc_hir::{
    def_id::LocalDefId,
    intravisit::{walk_expr, FnKind, Visitor},
    BlockCheckMode, Body, Expr, ExprKind, FnDecl, UnsafeSource,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::Span;

const LINT_MESSAGE: &str = "Avoid using unsafe blocks as it may lead to undefined behavior";

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
    LINT_MESSAGE,
    {
        name: "Avoid unsafe block",
        long_message: "The unsafe block is used to bypass Rust's safety checks. It is recommended to avoid using unsafe blocks as much as possible, and to use them only when necessary.    ",
        severity: "Enhancement",
        help: "https://coinfabrik.github.io/scout-soroban/docs/detectors/avoid-unsafe-block",
        vulnerability_class: "Best practices",
    }
}

impl<'tcx> LateLintPass<'tcx> for AvoidUnsafeBlock {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _: FnKind<'tcx>,
        _: &'tcx FnDecl<'tcx>,
        body: &'tcx Body<'tcx>,
        _: Span,
        _: LocalDefId,
    ) {
        struct UnsafeBlockVisitor {
            unsafe_blocks: Vec<Option<Span>>,
        }

        impl<'tcx> Visitor<'tcx> for UnsafeBlockVisitor {
            fn visit_expr(&mut self, expr: &'tcx Expr<'_>) {
                if let ExprKind::Block(block, _) = expr.kind {
                    if block.rules == BlockCheckMode::UnsafeBlock(UnsafeSource::UserProvided) {
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
                span_lint(cx, AVOID_UNSAFE_BLOCK, *span, LINT_MESSAGE);
            }
        });
    }
}
