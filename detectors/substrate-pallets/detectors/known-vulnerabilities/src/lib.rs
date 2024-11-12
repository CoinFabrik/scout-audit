#![feature(rustc_private)]
extern crate rustc_span;
extern crate rustc_ast;

use cargo_metadata::MetadataCommand;
use rustsec::{
    report::Settings, repository::git::Repository, Database, Lockfile, Report, Vulnerability,
};
use scout_audit_dylint_linting::LintInfo;
use std::ffi::CString;
use std::path::PathBuf;
use rustc_span::DUMMY_SP;
use rustc_ast::Crate;
use rustc_lint::{EarlyContext, EarlyLintPass};
use clippy_wrappers::span_lint_and_help;

static NAME: &str = "KNOWN_VULNERABILITIES";
static DESC: &str =
    "This dependency has known vulnerabilities. Consider updating it or removing it.";
static LONG      : &str = "This dependency has known vulnerabilities. Consider updating it or removing it. A link to the full description of the vulnerability is provided.";
static SEVERITY: &str = "Medium";
static HELP: &str = "https://coinfabrik.github.io/scout/docs/vulnerabilities/known-vulnerabilities";
static VULN_CLASS: &str = "TBD";

fn string_to_c(s: &str) -> CString {
    CString::new(s.as_bytes()).unwrap()
}

fn construct_metadata() -> LintInfo {
    LintInfo {
        id: string_to_c(NAME.to_lowercase().as_str()),
        name: string_to_c(NAME),
        short_message: string_to_c(DESC),
        long_message: string_to_c(LONG),
        severity: string_to_c(SEVERITY),
        help: string_to_c(HELP),
        vulnerability_class: string_to_c(VULN_CLASS),
    }
}

scout_audit_dylint_linting::declare_early_lint! {
    /// ### What it does
    /// Checks the soroban version of the contract
    ///
    /// ### Why is this bad?
    /// Using an outdated version of soroban could lead to security vulnerabilities, bugs, and other issues.
    pub KNOWN_VULNERABILITIES,
    Warn,
    DESC,
    {
        name: NAME,
        long_message: LONG,
        severity: SEVERITY,
        help: HELP,
        vulnerability_class: VULN_CLASS,
    }
}

fn report_vuln(vuln: &Vulnerability) -> String{
    let mut message = format!(
        "Known vulnerability in {} version {} (advisory {}): {}",
        vuln.package.name, vuln.package.version, vuln.advisory.id, vuln.advisory.title
    );
    let url = vuln.advisory
        .url
        .clone()
        .map(|x| x.to_string());
    if let Some(url) = url{
        message = format!("{}\nRead the report at {} to see if this vulnerability is applicable to your use case.", message, url);
    }
    message
}

fn perform_checking_on_dir(
    cx: &EarlyContext<'_>,
    lock: &str,
    toml: &PathBuf,
    krate: Option<String>,
) -> Result<(), rustsec::Error> {
    let repo = Repository::fetch_default_repo()?;
    let database = Database::load_from_repo(&repo)?;
    let lockfile = Lockfile::load(lock)?;
    let settings = Settings::default();
    let report = Report::generate(&database, &lockfile, &settings);
    for vuln in report.vulnerabilities.list.iter() {
        span_lint_and_help(
            cx,
            KNOWN_VULNERABILITIES,
            DUMMY_SP,
            DESC,
            None,
            report_vuln(vuln),
        );
    }
    Ok(())
}

fn package_name_to_crate_name(s: &str) -> String {
    let mut ret = String::new();
    ret.reserve(s.len());
    for c in s.chars() {
        ret.push(if c == '-' { '_' } else { c });
    }
    ret
}

fn get_lock_path_and_crate(toml: &PathBuf) -> Result<(PathBuf, Option<String>), ()> {
    let mut metadata_command = MetadataCommand::new();
    metadata_command.manifest_path(toml);
    let metadata = metadata_command.exec().map_err(|_| ())?;
    let krate = metadata
        .workspace_default_packages()
        .first()
        .map(|x| package_name_to_crate_name(&x.name));
    Ok((metadata.workspace_root.into(), krate))
}

fn perform_checking(cx: &EarlyContext<'_>) -> Result<(), ()> {
    let mut toml = std::env::current_dir().map_err(|_| ())?;
    toml.push("Cargo.toml");
    let (mut lock, krate) = get_lock_path_and_crate(&toml)?;
    lock.push("Cargo.lock");
    let lock_str = lock.as_path().to_str().ok_or(())?;
    perform_checking_on_dir(cx, lock_str, &toml, krate).map_err(|_| ())
}

impl EarlyLintPass for KnownVulnerabilities {
    fn check_crate(&mut self, cx: &EarlyContext<'_>, _: &Crate) {
        let _ = perform_checking(cx);
    }
}
