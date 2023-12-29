#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_span;

use std::{env, fs, io::Error, path::Path};

use rustc_ast::Crate;
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use scout_audit_internal::Detector;
use semver::*;

dylint_linting::declare_early_lint! {
    /// ### What it does
    /// Checks the soroban version of the contract
    ///
    /// ### Why is this bad?
    /// Using an outdated version of soroban could lead to security vulnerabilities, bugs, and other issues.
    pub CHECK_SOROBAN_VERSION,
    Warn,
    Detector::SorobanVersion.get_lint_message()
}

impl EarlyLintPass for CheckSorobanVersion {
    fn check_crate(&mut self, cx: &EarlyContext<'_>, _: &Crate) {
        let latest_version = match get_version() {
            Ok(version) => version,
            Err(e) => {
                cx.sess()
                    .struct_warn(format!("Failed to get the latest Soroban version: {}", e))
                    .emit();
                return;
            }
        };

        let manifest_dir = match env::var("CARGO_MANIFEST_DIR") {
            Ok(dir) => dir,
            Err(e) => {
                cx.sess()
                    .struct_warn(format!(
                        "Environment variable CARGO_MANIFEST_DIR not found: {}",
                        e
                    ))
                    .emit();
                return;
            }
        };

        let cargo_toml_path = Path::new(&manifest_dir).join("Cargo.toml");
        let cargo_toml = match fs::read_to_string(cargo_toml_path) {
            Ok(content) => content,
            Err(e) => {
                cx.sess()
                    .struct_warn(format!("Unable to read Cargo.toml: {}", e))
                    .emit();
                return;
            }
        };

        let toml: toml::Value = match toml::from_str(&cargo_toml) {
            Ok(value) => value,
            Err(e) => {
                cx.sess()
                    .struct_warn(format!("Error parsing Cargo.toml: {}", e))
                    .emit();
                return;
            }
        };

        let soroban_version = match toml
            .get("dependencies")
            .and_then(|d| d.get("soroban-sdk").and_then(|i| i.get("version")))
        {
            Some(version) => version.to_string(),
            None => {
                cx.sess()
                    .struct_warn("Soroban dependency not found in Cargo.toml")
                    .emit();
                return;
            }
        };

        let req = match Version::parse(&latest_version.replace('\"', "")) {
            Ok(version) => version,
            Err(e) => {
                cx.sess()
                    .struct_warn(format!("Error parsing latest Soroban version: {}", e))
                    .emit();
                return;
            }
        };

        let soroban_version = match VersionReq::parse(&soroban_version.replace('\"', "")) {
            Ok(version) => version,
            Err(e) => {
                cx.sess()
                    .struct_warn(format!("Error parsing project's Soroban version: {}", e))
                    .emit();
                return;
            }
        };

        if !soroban_version.matches(&req) {
            Detector::SorobanVersion.span_lint_and_help(
                cx,
                CHECK_SOROBAN_VERSION,
                rustc_span::DUMMY_SP,
                &format!(r#"The latest Soroban version is {latest_version}, and your version is "{soroban_version}""#),
            );
        }
    }
}

fn get_version() -> Result<String, String> {
    let response = ureq::get("https://crates.io/api/v1/crates/soroban-sdk")
        .set("User-Agent", "Scout/1.0")
        .call();

    match response {
        Ok(resp) => {
            let json: Result<serde_json::Value, Error> = resp.into_json();
            match json {
                Ok(json) => json
                    .get("crate")
                    .and_then(|c| c.get("max_stable_version"))
                    .map(|v| v.to_string())
                    .ok_or_else(|| "Failed to parse Soroban version from response".to_string()),
                Err(_) => Err("Failed to parse response from crates.io".to_string()),
            }
        }
        Err(_) => Err("Failed to get Soroban version from crates.io".to_string()),
    }
}
