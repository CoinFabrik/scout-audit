#![feature(internal_output_capture)]
#![feature(rustc_private)]
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_span;

use capture_stdio::Capture;
use rustc_errors::{Applicability, Diagnostic, MultiSpan};
use rustc_hir::HirId;
use rustc_lint::{LateContext, Lint, LintContext};
use rustc_span::Span;
use std::io::BufRead;

fn print_error<F: FnOnce()>(cb: F) {
    let old = std::io::set_output_capture(None);
    let mut piped_stderr = capture_stdio::PipedStderr::capture().unwrap();

    let port = std::env::var("SCOUT_PORT_NUMBER");

    if port.is_err() {
        cb();
        return;
    }

    let port = port.unwrap();

    //let _ = reqwest::blocking::Client::new()
    //    .post(format!("http://127.0.0.1:{port}/print"))
    //    .body("A")
    //    .send();

    cb();

    //let _ = reqwest::blocking::Client::new()
    //    .post(format!("http://127.0.0.1:{port}/print"))
    //    .body("B")
    //    .send();

    let _ = std::io::set_output_capture(old);
    let mut captured = String::new();
    let mut buf_reader = std::io::BufReader::new(piped_stderr.get_reader());
    let _ = buf_reader.read_line(&mut captured);

    let krate = std::env::var("CARGO_PKG_NAME");
    let krate = if let Ok(krate2) = krate {
        krate2
    } else {
        String::new()
    };

    let body = {
        let json = serde_json::from_str::<serde_json::Value>(&captured);
        if let Ok(json) = json {
            serde_json::json!({
                "crate": krate,
                "message": json,
            })
            .to_string()
        } else {
            captured
        }
    };

    let _ = reqwest::blocking::Client::new()
        .post(format!("http://127.0.0.1:{port}/vuln"))
        .body(body)
        .send();
}

pub fn span_lint<T: LintContext>(cx: &T, lint: &'static Lint, sp: impl Into<MultiSpan>, msg: &str) {
    print_error(|| {
        clippy_utils::diagnostics::span_lint(cx, lint, sp, msg);
    });
}

pub fn span_lint_and_help<T: LintContext>(
    cx: &T,
    lint: &'static Lint,
    span: impl Into<MultiSpan>,
    msg: &str,
    help_span: Option<Span>,
    help: &str,
) {
    print_error(|| {
        clippy_utils::diagnostics::span_lint_and_help(cx, lint, span, msg, help_span, help);
    });
}

pub fn span_lint_and_note<T: LintContext>(
    cx: &T,
    lint: &'static Lint,
    span: impl Into<MultiSpan>,
    msg: &str,
    note_span: Option<Span>,
    note: &str,
) {
    print_error(|| {
        clippy_utils::diagnostics::span_lint_and_note(cx, lint, span, msg, note_span, note);
    });
}

pub fn span_lint_and_then<C, S, F>(cx: &C, lint: &'static Lint, sp: S, msg: &str, f: F)
where
    C: LintContext,
    S: Into<MultiSpan>,
    F: FnOnce(&mut Diagnostic),
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
    msg: &str,
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
    msg: &str,
    f: impl FnOnce(&mut Diagnostic),
) {
    print_error(|| {
        clippy_utils::diagnostics::span_lint_hir_and_then(cx, lint, hir_id, sp, msg, f);
    });
}

pub fn span_lint_and_sugg<T: LintContext>(
    cx: &T,
    lint: &'static Lint,
    sp: Span,
    msg: &str,
    help: &str,
    sugg: String,
    applicability: Applicability,
) {
    print_error(|| {
        clippy_utils::diagnostics::span_lint_and_sugg(cx, lint, sp, msg, help, sugg, applicability);
    });
}

pub fn multispan_sugg<I>(diag: &mut Diagnostic, help_msg: &str, sugg: I)
where
    I: IntoIterator<Item = (Span, String)>,
{
    print_error(|| {
        clippy_utils::diagnostics::multispan_sugg(diag, help_msg, sugg);
    });
}

pub fn multispan_sugg_with_applicability<I>(
    diag: &mut Diagnostic,
    help_msg: &str,
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
