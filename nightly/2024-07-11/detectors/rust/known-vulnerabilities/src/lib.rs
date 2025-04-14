#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_span;

use clippy_utils::diagnostics::span_lint_and_help;
use common::{
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use rustc_ast::Crate;
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_span::DUMMY_SP;
use rustsec::{report::Settings, repository::git::Repository, Database, Lockfile, Report};

const LINT_MESSAGE: &str =
    "This dependency has known vulnerabilities. Consider updating it or removing it.";

#[expose_lint_info]
pub static KNOWN_VULNERABILITIES_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message:
        "Using dependencies with known vulnerabilities can expose your project to security risks",
    severity: Severity::Medium,
    help: "https://coinfabrik.github.io/scout-audit/docs/detectors/rust/known-vulnerabilities",
    vulnerability_class: VulnerabilityClass::KnownBugs,
};

dylint_linting::declare_early_lint! {
    pub KNOWN_VULNERABILITIES,
    Warn,
    LINT_MESSAGE
}

impl EarlyLintPass for KnownVulnerabilities {
    fn check_crate(&mut self, cx: &EarlyContext<'_>, _: &Crate) {
        // Get the workspace root path
        let workspace_path = match std::env::current_dir() {
            Ok(path) => path,
            Err(e) => {
                cx.sess()
                    .dcx()
                    .struct_warn(format!("Failed to get current directory: {}", e))
                    .emit();
                return;
            }
        };

        // Get the path to the Cargo.lock file and its content
        let lock_path = workspace_path.join("Cargo.lock");
        let lock_str = match lock_path.to_str() {
            Some(s) => s,
            None => {
                cx.sess()
                    .dcx()
                    .struct_warn("Invalid path to Cargo.lock")
                    .emit();
                return;
            }
        };

        // Fetch and check vulnerabilities
        let repo = match Repository::fetch_default_repo() {
            Ok(repo) => repo,
            Err(e) => {
                cx.sess()
                    .dcx()
                    .struct_warn(format!("Failed to fetch vulnerability database: {}", e))
                    .emit();
                return;
            }
        };

        let database = match Database::load_from_repo(&repo) {
            Ok(db) => db,
            Err(e) => {
                cx.sess()
                    .dcx()
                    .struct_warn(format!("Failed to load vulnerability database: {}", e))
                    .emit();
                return;
            }
        };

        let lockfile = match Lockfile::load(lock_str) {
            Ok(lock) => lock,
            Err(e) => {
                cx.sess()
                    .dcx()
                    .struct_warn(format!("Failed to load Cargo.lock: {}", e))
                    .emit();
                return;
            }
        };

        let settings = Settings::default();
        let report = Report::generate(&database, &lockfile, &settings);

        // Report each vulnerability
        for vuln in report.vulnerabilities.list {
            let message = format!(
                "Known vulnerability in {} version {}",
                vuln.package.name, vuln.package.version
            );

            let help = format!(
                "Advisory: {}\nDescription: {}\nURL: {}",
                vuln.advisory.id,
                vuln.advisory.title,
                vuln.advisory
                    .url
                    .map_or("N/A".to_string(), |u| u.to_string())
            );

            span_lint_and_help(cx, KNOWN_VULNERABILITIES, DUMMY_SP, message, None, help);
        }
    }
}
