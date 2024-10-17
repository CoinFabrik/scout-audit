#![feature(rustc_private)]
extern crate rustc_session;
extern crate rustc_lint;

use scout_audit_dylint_linting::LintInfo;
use std::ffi::CString;
use std::path::PathBuf;
use rustsec::{
    report::Settings, repository::git::Repository, Database, Lockfile, Report, Vulnerability
};
use cargo_metadata::MetadataCommand;
use colored::Colorize;

static NAME      : &str = "KNOWN_VULNERABILITIES";
static DESC      : &str = "This dependency has known vulnerabilities. Consider updating it or removing it.";
static LONG      : &str = "This dependency has known vulnerabilities. Consider updating it or removing it. A link to the full description of the vulnerability is provided.";
static SEVERITY  : &str = "Medium";
static HELP      : &str = "https://coinfabrik.github.io/scout/docs/vulnerabilities/known-vulnerabilities";
static VULN_CLASS: &str = "TBD";

fn string_to_c(s: &str) -> CString{
    CString::new(s.as_bytes()).unwrap()
}

fn construct_metadata() -> LintInfo{
    LintInfo{
        id:                  string_to_c(NAME.to_lowercase().as_str()),
        name:                string_to_c(NAME),
        short_message:       string_to_c(DESC),
        long_message:        string_to_c(LONG),
        severity:            string_to_c(SEVERITY),
        help:                string_to_c(HELP),
        vulnerability_class: string_to_c(VULN_CLASS),
    }
}

#[no_mangle]
pub fn lint_info(info: &mut LintInfo){
    *info = construct_metadata();
}

#[doc(hidden)]
#[no_mangle]
pub extern "C" fn dylint_version() -> *mut std::os::raw::c_char {
    std::ffi::CString::new("0.1.0")
        .unwrap()
        .into_raw()
}

#[no_mangle]
pub fn register_lints(_: &rustc_session::Session, _: &mut rustc_lint::LintStore){}

#[no_mangle]
pub fn custom_detector(){
    let _ = perform_checking();
}

fn report_vuln(vuln: &Vulnerability, toml: &PathBuf, krate: &Option<String>) -> Result<(), ()>{
    let port = std::env::var("SCOUT_PORT_NUMBER").map_err(|_| ())?;

    let short_message = format!("Known vulnerability in {} version {}", vuln.package.name, vuln.package.version);
    let first_line = format!("{}: {}", "warning".yellow(), short_message).bold().to_string();
    let rendered = format!(
        concat!(
            "{}\n",
            "    {}: {}\n",
            "    {}: {}\n",
            "    {}: {}\n",
            "    Read the report to see if this vulnerability is applicable to your use case.\n",
            "\n",
        ),
        first_line,
        "Advisory".bold(),
        vuln.advisory.id,
        "Description".bold(),
        vuln.advisory.title,
        "URL".bold(),
        vuln.advisory.url.clone().and_then(|x| Some(x.to_string())).unwrap_or("N/A".to_string()),
    );
    let body = serde_json::json!({
            "crate": krate,
            "message": {
                "code": {
                    "code": NAME.to_lowercase(),
                    "explanation": null,
                },
                "message": short_message,
                "rendered": rendered,
                "spans": [
                    {
                        "byte_start": 0,
                        "byte_end": 0,
                        "line_start": 0,
                        "line_end": 0,
                        "column_start": 0,
                        "column_end": 0,
                        "expansion": null,
                        "file_name": toml,
                        "is_primary": true,
                        "suggested_replacement": null,
                        "suggestion_applicability": null,
                        "text": null,
                    }
                ],
            },
        })
        .to_string();

    let _ = reqwest::blocking::Client::new()
        .post(format!("http://127.0.0.1:{port}/vuln"))
        .body(body)
        .send();

    Ok(())
}

fn perform_checking_on_dir(lock: &str, toml: &PathBuf, krate: Option<String>) -> Result<(), rustsec::Error>{
    let repo = Repository::fetch_default_repo()?;
    let database = Database::load_from_repo(&repo)?;
    let lockfile = Lockfile::load(lock)?;
    let settings = Settings::default();
    let report = Report::generate(&database, &lockfile, &settings);
    for vuln in report.vulnerabilities.list.iter(){
        let _ = report_vuln(vuln, toml, &krate);
    }
    Ok(())
}

fn package_name_to_crate_name(s: &String) -> String {
    let mut ret = String::new();
    ret.reserve(s.len());
    for c in s.chars() {
        ret.push(if c == '-' { '_' } else { c });
    }
    ret
}

fn get_lock_path_and_crate(toml: &PathBuf) -> Result<(PathBuf, Option<String>), ()>{
    let mut metadata_command = MetadataCommand::new();
    metadata_command.manifest_path(toml);
    let metadata = metadata_command.exec().map_err(|_| ())?;
    let krate = metadata.workspace_default_packages().first().and_then(|x| Some(package_name_to_crate_name(&x.name)));
    Ok((metadata.workspace_root.into(), krate))
}

fn perform_checking() -> Result<(), ()>{
    let mut toml = std::env::current_dir().map_err(|_| ())?;
    toml.push("Cargo.toml");
    let (mut lock, krate) = get_lock_path_and_crate(&toml)?;
    lock.push("Cargo.lock");
    let lock_str = lock.as_path().to_str().ok_or(())?;
    perform_checking_on_dir(lock_str, &toml, krate).map_err(|_| ())
}
