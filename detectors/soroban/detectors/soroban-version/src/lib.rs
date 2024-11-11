#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_span;

use std::{io::Error, process::Command};

use clippy_wrappers::span_lint_and_help;
use rustc_ast::Crate;
use rustc_lint::{EarlyContext, EarlyLintPass, LintContext};
use rustc_span::DUMMY_SP;
use semver::Version;
use serde_json::Value;

const LINT_MESSAGE: &str = "Use the latest version of Soroban";

dylint_linting::declare_early_lint! {
    /// ### What it does
    /// Checks the soroban version of the contract
    ///
    /// ### Why is this bad?
    /// Using an outdated version of soroban could lead to security vulnerabilities, bugs, and other issues.
    pub SOROBAN_VERSION,
    Warn,
    LINT_MESSAGE,
    {
        name: "Check Soroban version",
        long_message: "Using a older version of Soroban can be dangerous, as it may have bugs or security issues. Use the latest version available.",
        severity: "Enhancement",
        help: "https://coinfabrik.github.io/scout-soroban/docs/detectors/soroban-version",
        vulnerability_class: "Best practices",
    }
}

impl EarlyLintPass for SorobanVersion {
    fn check_crate(&mut self, cx: &EarlyContext<'_>, _: &Crate) {
        let latest_soroban_version = match get_latest_soroban_version() {
            Ok(version) => version,
            Err(e) => {
                cx.sess()
                    .dcx()
                    .struct_warn(format!("Failed to get the latest Soroban version: {}", e))
                    .emit();
                return;
            }
        };

        let cargo_metadata = match get_cargo_metadata() {
            Ok(metadata) => metadata,
            Err(e) => {
                cx.sess()
                    .dcx()
                    .struct_warn(format!("Failed to get cargo metadata: {}", e))
                    .emit();
                return;
            }
        };

        let cargo_toml_package_opt = match cargo_metadata["packages"].as_array() {
            Some(packages) => packages.first(),
            None => {
                cx.sess()
                    .dcx()
                    .struct_warn("Error parsing cargo metadata: packages not found")
                    .emit();
                return;
            }
        };

        let cargo_toml_package = match cargo_toml_package_opt {
            Some(package) => package,
            None => {
                cx.sess()
                    .dcx()
                    .struct_warn("Error parsing cargo metadata: first package not found")
                    .emit();
                return;
            }
        };

        let dependencies = match cargo_toml_package["dependencies"].as_array() {
            Some(dependencies) => dependencies,
            None => {
                cx.sess()
                    .dcx()
                    .struct_warn("Error parsing cargo metadata: dependencies not found")
                    .emit();
                return;
            }
        };

        let current_dependency = match dependencies
            .iter()
            .find(|&dep| dep["name"].as_str().unwrap_or("") == "soroban-sdk")
        {
            Some(current_dependency) => current_dependency,
            None => {
                cx.sess()
                    .dcx()
                    .struct_warn("Soroban dependency not found in dependencies")
                    .emit();
                return;
            }
        };

        let current_dependency_version = match current_dependency["req"].as_str() {
            Some(version) => version,
            None => {
                cx.sess()
                    .dcx()
                    .struct_warn("Error parsing current Soroban version")
                    .emit();
                return;
            }
        };

        let req = match Version::parse(&latest_soroban_version.replace('\"', "")) {
            Ok(version) => version,
            Err(e) => {
                cx.sess()
                    .dcx()
                    .struct_warn(format!("Error parsing latest Soroban version: {}", e))
                    .emit();
                return;
            }
        };

        let cleaned_version = current_dependency_version.replace(&['=', '^'][..], "");

        let soroban_version = match Version::parse(&cleaned_version) {
            Ok(version) => version,
            Err(e) => {
                cx.sess()
                    .dcx()
                    .struct_warn(format!("Error parsing project's Soroban version: {}", e))
                    .emit();
                return;
            }
        };

        if !soroban_version.eq(&req) {
            span_lint_and_help(
                cx,
                SOROBAN_VERSION,
                DUMMY_SP,
                LINT_MESSAGE,
                None,
                format!(
                    r#"The latest Soroban version is {latest_soroban_version}, and your version is "{soroban_version}""#
                ),
            );
        }
    }
}

fn get_latest_soroban_version() -> Result<String, String> {
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

fn get_cargo_metadata() -> Result<Value, String> {
    let output = Command::new("cargo")
        .args(["metadata", "--format-version=1", "--no-deps"])
        .output();

    match output {
        Ok(output) if output.status.success() => serde_json::from_slice(&output.stdout)
            .map_err(|e| format!("Error parsing cargo metadata: {}", e)),
        Ok(output) => Err(String::from_utf8_lossy(&output.stderr).into_owned()),
        Err(e) => Err(e.to_string()),
    }
}
