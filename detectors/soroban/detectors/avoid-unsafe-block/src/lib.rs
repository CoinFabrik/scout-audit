#![feature(rustc_private)]

extern crate rustc_hir;
extern crate rustc_span;

use clippy_wrappers::span_lint;
use common::macros::expose_lint_info;
use rustc_hir::{
    def_id::LocalDefId,
    intravisit::{walk_expr, FnKind, Visitor},
    BlockCheckMode, Body, Expr, ExprKind, FnDecl, UnsafeSource,
};
use rustc_lint::{LateContext, LateLintPass};
use rustc_span::Span;

const LINT_MESSAGE: &str = "Avoid using unsafe blocks as it may lead to undefined behavior";

#[expose_lint_info]
pub static AVOID_UNSAFE_BLOCK_INFO: LintInfo = LintInfo {
    name: "Avoid unsafe block",
    short_message: LINT_MESSAGE,
    long_message: "The unsafe block is used to bypass Rust's safety checks. It is recommended to avoid using unsafe blocks as much as possible, and to use them only when necessary.    ",
    severity: "Enhancement",
    help: "https://coinfabrik.github.io/scout-soroban/docs/detectors/avoid-unsafe-block",
    vulnerability_class: "Best practices",
};

dylint_linting::declare_late_lint! {
    pub AVOID_UNSAFE_BLOCK,
    Warn,
    LINT_MESSAGE
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
