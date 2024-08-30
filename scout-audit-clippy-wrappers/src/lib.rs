#![feature(rustc_private)]
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_span;

use rustc_errors::{Applicability, Diag, DiagMessage, MultiSpan, SubdiagMessage};
use rustc_hir::HirId;
use rustc_lint::{LateContext, Lint, LintContext};
use rustc_span::Span;
use clippy_wrapper_print_error::print_error;

pub fn span_lint<T: LintContext>(
    cx: &T,
    lint: &'static Lint,
    sp: impl Into<MultiSpan>,
    msg: impl Into<DiagMessage>,
) {
    print_error(|| {
        clippy_utils::diagnostics::span_lint(cx, lint, sp, msg);
    });
}

pub fn span_lint_and_help<T: LintContext>(
    cx: &T,
    lint: &'static Lint,
    span: impl Into<MultiSpan>,
    msg: impl Into<DiagMessage>,
    help_span: Option<Span>,
    help: impl Into<SubdiagMessage>,
) {
    print_error(|| {
        clippy_utils::diagnostics::span_lint_and_help(cx, lint, span, msg, help_span, help);
    });
}

pub fn span_lint_and_note<T: LintContext>(
    cx: &T,
    lint: &'static Lint,
    span: impl Into<MultiSpan>,
    msg: impl Into<DiagMessage>,
    note_span: Option<Span>,
    note: impl Into<SubdiagMessage>,
) {
    print_error(|| {
        clippy_utils::diagnostics::span_lint_and_note(cx, lint, span, msg, note_span, note);
    });
}

pub fn span_lint_and_then<C, S, M, F>(cx: &C, lint: &'static Lint, sp: S, msg: M, f: F)
where
    C: LintContext,
    S: Into<MultiSpan>,
    M: Into<DiagMessage>,
    F: FnOnce(&mut Diag<'_, ()>),
{
    print_error(|| {
        clippy_utils::diagnostics::span_lint_and_then(cx, lint, sp, msg, f);
    });
}

pub fn span_lint_hir(
    cx: &LateContext<'_>,
    lint: &'static Lint,
    hir_id: HirId,
    sp: Span,
    msg: impl Into<DiagMessage>,
) {
    print_error(|| {
        clippy_utils::diagnostics::span_lint_hir(cx, lint, hir_id, sp, msg);
    });
}

pub fn span_lint_hir_and_then(
    cx: &LateContext<'_>,
    lint: &'static Lint,
    hir_id: HirId,
    sp: impl Into<MultiSpan>,
    msg: impl Into<DiagMessage>,
    f: impl FnOnce(&mut Diag<'_, ()>),
) {
    print_error(|| {
        clippy_utils::diagnostics::span_lint_hir_and_then(cx, lint, hir_id, sp, msg, f);
    });
}

pub fn span_lint_and_sugg<T: LintContext>(
    cx: &T,
    lint: &'static Lint,
    sp: Span,
    msg: impl Into<DiagMessage>,
    help: impl Into<SubdiagMessage>,
    sugg: String,
    applicability: Applicability,
) {
    print_error(|| {
        clippy_utils::diagnostics::span_lint_and_sugg(cx, lint, sp, msg, help, sugg, applicability);
    });
}

pub fn multispan_sugg<I>(diag: &mut Diag<'_, ()>, help_msg: impl Into<SubdiagMessage>, sugg: I)
where
    I: IntoIterator<Item = (Span, String)>,
{
    print_error(|| {
        clippy_utils::diagnostics::multispan_sugg(diag, help_msg, sugg);
    });
}

pub fn multispan_sugg_with_applicability<I>(
    diag: &mut Diag<'_, ()>,
    help_msg: impl Into<SubdiagMessage>,
    applicability: Applicability,
    sugg: I,
) where
    I: IntoIterator<Item = (Span, String)>,
{
    print_error(|| {
        clippy_utils::diagnostics::multispan_sugg_with_applicability(
            diag,
            help_msg,
            applicability,
            sugg,
        );
    });
}
