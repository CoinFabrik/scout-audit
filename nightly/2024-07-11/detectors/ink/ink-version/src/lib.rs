#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_span;

use std::fs;

use common::{
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use rustc_lint::EarlyLintPass;
use semver::*;

const LINT_MESSAGE: &str = "Use the latest version of ink!";

#[expose_lint_info]
pub static INK_VERSION_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "Using a older version of ink! can be dangerous, as it may have bugs or security issues. Use the latest version available.",
    severity: Severity::Enhancement,
    help: "https://coinfabrik.github.io/scout-audit/docs/detectors/ink/ink-version",
    vulnerability_class: VulnerabilityClass::BestPractices,
};

dylint_linting::declare_early_lint! {
    pub INK_VERSION,
    Warn,
    LINT_MESSAGE
}

impl EarlyLintPass for InkVersion {
    fn check_crate(&mut self, cx: &rustc_lint::EarlyContext<'_>, _: &rustc_ast::Crate) {
        let latest_version = get_version();

        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

        let cargo_toml_path = std::path::Path::new(&manifest_dir).join("Cargo.toml");

        let cargo_toml = fs::read_to_string(cargo_toml_path).expect("Unable to read Cargo.toml");

        let toml: toml::Value = toml::from_str(&cargo_toml).unwrap();

        let ink_version = match toml
            .get("dependencies")
            .and_then(|d| d.get("ink").and_then(|i| i.get("version")))
        {
            Some(version) => version.to_string(),
            None => return,
        };

        let req = Version::parse(&latest_version.replace('\"', "")).unwrap();
        let ink_version = VersionReq::parse(&ink_version.replace('\"', "")).unwrap();

        if !ink_version.matches(&req) {
            clippy_utils::diagnostics::span_lint_and_help(
                cx,
                INK_VERSION,
                rustc_span::DUMMY_SP,
                LINT_MESSAGE,
                None,
                format!("The latest ink! version is {latest_version}, and your version is {ink_version}")
            );
        }
    }
}

fn get_version() -> String {
    let resp: serde_json::Value = ureq::get("https://crates.io/api/v1/crates/ink")
        .set("User-Agent", "Scout/1.0")
        .call()
        .expect("Failed to get ink! version from crates.io")
        .into_json()
        .expect("Failed to parse ink! version from crates.io");
    let version = resp
        .get("crate")
        .unwrap()
        .get("max_stable_version")
        .unwrap()
        .to_string();
    version
}
