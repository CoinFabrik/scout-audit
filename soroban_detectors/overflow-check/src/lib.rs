#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_hir;
extern crate rustc_span;

use std::fs;

use rustc_lint::EarlyLintPass;
use scout_audit_internal::Detector;

dylint_linting::declare_early_lint! {
    /// ### What it does
    /// Checks that overflow-checks is enabled in Cargo.toml.
    ///
    /// ### Why is this bad?
    /// Integer overflow will trigger a panic in debug builds or will wrap in
    /// release mode. Division by zero will cause a panic in either mode. In some applications one
    /// wants explicitly checked, wrapping or saturating arithmetic.
    pub OVERFLOW_CHECK,
    Warn,
    Detector::OverflowCheck.get_lint_message()
}

impl EarlyLintPass for OverflowCheck {
    fn check_crate(&mut self, cx: &rustc_lint::EarlyContext<'_>, _: &rustc_ast::Crate) {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

        let cargo_toml_path = std::path::Path::new(&manifest_dir).join("Cargo.toml");

        let cargo_toml = fs::read_to_string(cargo_toml_path).expect("Unable to read Cargo.toml");

        let toml: toml::Value = toml::from_str(&cargo_toml).unwrap();

        if let Some(profiles) = toml.get("profile").and_then(|p| p.as_table()) {
            for profile in profiles {
                let profile_name = profile.0;
                let mut table = profile.1.as_table();
                let mut temp_table;
                if table.is_some() && table.unwrap().contains_key("inherits") {
                    let parent_name = table.unwrap().get("inherits").unwrap().as_str().unwrap();
                    if profiles.contains_key(parent_name) {
                        let parent_table = profiles.get(parent_name).unwrap().as_table().unwrap();
                        temp_table = parent_table.clone();
                        temp_table.extend(table.unwrap().clone().into_iter());
                        table = Some(&temp_table);
                    }
                }
                if table.is_some() && table.unwrap().contains_key("overflow-checks") {
                    let has_overflow_check = table
                        .unwrap()
                        .get("overflow-checks")
                        .is_some_and(|f| f.as_bool().unwrap_or(false));
                    if !has_overflow_check {
                        Detector::OverflowCheck.span_lint_and_help(
                            cx,
                            OVERFLOW_CHECK,
                            rustc_span::DUMMY_SP,
                            &format!("Enable overflow-checks on profile.{profile_name}"),
                        );
                    }
                } else {
                    Detector::OverflowCheck.span_lint_and_help(
                        cx,
                        OVERFLOW_CHECK,
                        rustc_span::DUMMY_SP,
                        &format!("Enable overflow-checks on profile.{profile_name}"),
                    );
                }
            }
        }
    }
}
