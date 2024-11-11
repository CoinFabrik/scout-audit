#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_span;

use clippy_wrappers::span_lint_and_help;
use rustc_ast::{
    ptr::P,
    tokenstream::TokenTree,
    visit::{walk_expr, Visitor},
    AssocItemKind, AttrArgs, AttrKind, Block, Expr, ExprKind, FnRetTy, Item, ItemKind, MacCall,
    ModKind, TyKind,
};
use rustc_lint::{EarlyContext, EarlyLintPass};
use rustc_span::{sym, Span};

const LINT_MESSAGE: &str = "The panic! macro is used in a function that returns Result. \
    Consider using the ? operator or return Err() instead.";

dylint_linting::impl_pre_expansion_lint! {
        /// ### What it does
    /// The panic! macro is used to stop execution when a condition is not met.
    /// This is useful for testing and prototyping, but should be avoided in production code
    ///
    /// ### Why is this bad?
    /// The usage of panic! is not recommended because it will stop the execution of the caller contract.
    ///
    /// ### Known problems
    /// While this linter detects explicit calls to panic!, there are some ways to raise a panic such as unwrap() or expect().
    ///
    /// ### Example
    /// ```rust
    /// pub fn add(env: Env, value: u32) -> u32 {
    ///     let storage = env.storage().instance();
    ///     let mut count: u32 = storage.get(&COUNTER).unwrap_or(0);
    ///     match count.checked_add(value) {
    ///         Some(value) => count = value,
    ///         None => panic!("Overflow error"),
    ///     }
    ///     storage.set(&COUNTER, &count);
    ///     storage.extend_ttl(100, 100);
    ///     count
    /// }
    /// ```
    /// Use instead:
    /// ```rust
    /// pub fn add(env: Env, value: u32) -> Result<u32, Error> {
    ///     let storage = env.storage().instance();
    ///     let mut count: u32 = storage.get(&COUNTER).unwrap_or(0);
    ///     match count.checked_add(value) {
    ///         Some(value) => count = value,
    ///         None => return Err(Error::OverflowError),
    ///     }
    ///     storage.set(&COUNTER, &count);
    ///     storage.extend_ttl(100, 100);
    ///     Ok(count)
    /// }
    /// ```
    pub AVOID_PANIC_ERROR,
    Warn,
    LINT_MESSAGE,
    AvoidPanicError::default(),
    {
        name: "Avoid panic! macro",
        long_message: "Using panic! in functions that return Result defeats the purpose of error handling. \
            Consider propagating the error using ? or return Err() instead.",
        severity: "Enhancement",
        help: "https://coinfabrik.github.io/scout-soroban/docs/detectors/avoid-panic-error",
        vulnerability_class: "Validations and error handling",
    }
}

#[derive(Default)]
pub struct AvoidPanicError {
    in_test_span: Option<Span>,
}

impl EarlyLintPass for AvoidPanicError {
    fn check_item(&mut self, cx: &EarlyContext, item: &Item) {
        if is_test_item(item) {
            self.in_test_span = Some(item.span);
            return;
        }

        if let Some(test_span) = self.in_test_span {
            if !test_span.contains(item.span) {
                self.in_test_span = None;
            } else {
                return;
            }
        }

        match &item.kind {
            ItemKind::Impl(impl_item) => {
                for assoc_item in &impl_item.items {
                    if let AssocItemKind::Fn(fn_item) = &assoc_item.kind {
                        self.check_function(cx, &fn_item.sig.decl.output, &fn_item.body);
                    }
                }
            }
            ItemKind::Fn(fn_item) => {
                self.check_function(cx, &fn_item.sig.decl.output, &fn_item.body);
            }
            ItemKind::Mod(_, ModKind::Loaded(items, _, _)) => {
                for item in items {
                    self.check_item(cx, item);
                }
            }
            ItemKind::Trait(trait_item) => {
                for item in &trait_item.items {
                    if let AssocItemKind::Fn(fn_item) = &item.kind {
                        self.check_function(cx, &fn_item.sig.decl.output, &fn_item.body);
                    }
                }
            }
            _ => {}
        }
    }
}

impl AvoidPanicError {
    fn check_function(&self, cx: &EarlyContext, output: &FnRetTy, body: &Option<P<Block>>) {
        if let Some(body) = body {
            if is_result_type(output) {
                let mut visitor = PanicVisitor { cx };
                visitor.visit_block(body);
            }
        }
    }
}

struct PanicVisitor<'a, 'tcx> {
    cx: &'a EarlyContext<'tcx>,
}

impl<'a, 'tcx> Visitor<'tcx> for PanicVisitor<'a, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx Expr) {
        if let ExprKind::MacCall(mac) = &expr.kind {
            check_macro_call(self.cx, expr.span, mac);
        }
        walk_expr(self, expr);
    }
}

fn check_macro_call(cx: &EarlyContext, span: Span, mac: &MacCall) {
    if mac.path == sym::panic {
        let suggestion = "Consider using '?' to propagate errors or 'return Err()' to return early with an error";
        span_lint_and_help(cx, AVOID_PANIC_ERROR, span, LINT_MESSAGE, None, suggestion);
    }
}

fn is_test_item(item: &Item) -> bool {
    item.attrs.iter().any(|attr| {
        attr.has_name(sym::test)
            || (attr.has_name(sym::cfg)
                && attr.meta_item_list().map_or(false, |list| {
                    list.iter().any(|item| item.has_name(sym::test))
                }))
            || matches!(
                &attr.kind,
                AttrKind::Normal(normal) if is_test_token_present(&normal.item.args)
            )
    })
}

fn is_test_token_present(args: &AttrArgs) -> bool {
    if let AttrArgs::Delimited(delim_args) = args {
        delim_args.tokens.trees().any(
            |tree| matches!(tree, TokenTree::Token(token, _) if token.is_ident_named(sym::test)),
        )
    } else {
        false
    }
}

fn is_result_type(output: &FnRetTy) -> bool {
    match output {
        FnRetTy::Default(_) => false,
        FnRetTy::Ty(ty) => {
            if let TyKind::Path(None, path) = &ty.kind {
                path.segments
                    .last()
                    .map_or(false, |seg| seg.ident.name == sym::Result)
            } else {
                false
            }
        }
    }
}
