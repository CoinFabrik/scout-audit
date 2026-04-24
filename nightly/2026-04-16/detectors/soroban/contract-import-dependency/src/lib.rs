#![feature(rustc_private)]

extern crate rustc_ast;
extern crate rustc_span;

use clippy_utils::diagnostics::span_lint_and_help;
use common::{
    declarations::{Severity, VulnerabilityClass},
    macros::expose_lint_info,
};
use if_chain::if_chain;
use rustc_ast::{
    token::{LitKind, TokenKind},
    tokenstream::TokenTree,
    AttrArgs, AttrKind, Item, MacCall,
};
use rustc_lint::{EarlyContext, EarlyLintPass};
use rustc_span::{sym, Span};
use std::{collections::HashSet, fs, path::PathBuf};

const LINT_MESSAGE: &str =
    "The `contractimport!` macro embeds another contract's WASM without tracking it as a dependency.";

#[expose_lint_info]
pub static CONTRACT_IMPORT_DEPENDENCY_INFO: LintInfo = LintInfo {
    name: env!("CARGO_PKG_NAME"),
    short_message: LINT_MESSAGE,
    long_message: "Using Soroban's `contractimport!` macro to import a contract binary does not require the imported contract to be declared as a dependency. This makes it easy to deploy a contract with an outdated version of a dependency, since tests will still pass against the stale WASM. Prefer mechanisms that keep dependencies explicit so upgrades are caught by the toolchain.",
    severity: Severity::Medium,
    help: "https://coinfabrik.github.io/scout-audit/docs/detectors/soroban/contract-import-dependency",
    vulnerability_class: VulnerabilityClass::BestPractices,
};

dylint_linting::impl_pre_expansion_lint! {
    pub CONTRACT_IMPORT_DEPENDENCY,
    Warn,
    LINT_MESSAGE,
    ContractImportDependency::default()
}

#[derive(Default)]
pub struct ContractImportDependency {
    test_spans: Vec<Span>,
    cargo_dependencies: Option<HashSet<String>>,
}

impl ContractImportDependency {
    fn is_within_test(&self, span: Span) -> bool {
        self.test_spans
            .iter()
            .any(|test_span| test_span.contains(span))
    }

    fn is_test_token_present(args: &AttrArgs) -> bool {
        matches!(
            args,
            AttrArgs::Delimited(delim_args)
                if delim_args.tokens.iter().any(|tree| {
                    matches!(tree, TokenTree::Token(token, _) if token.is_ident_named(sym::test))
                })
        )
    }

    fn is_test_item(item: &Item) -> bool {
        item.attrs.iter().any(|attr| {
            attr.has_name(sym::test)
                || (attr.has_name(sym::cfg)
                    && attr.meta_item_list().is_some_and(|list| {
                        list.iter().any(|item| item.has_name(sym::test))
                    }))
                || matches!(&attr.kind, AttrKind::Normal(n) if Self::is_test_token_present(&n.item.args))
        })
    }

    fn is_contractimport_macro(mac: &MacCall) -> bool {
        mac.path
            .segments
            .last()
            .is_some_and(|seg| matches!(seg.ident.name.as_str(), "contractimport"))
    }

    fn load_cargo_dependencies(&mut self) {
        let mut deps = HashSet::new();
        if_chain! {
            if self.cargo_dependencies.is_none();
            if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR").map(PathBuf::from);
            if let Ok(contents) = fs::read_to_string(dir.join("Cargo.toml"));
            if let Ok(manifest) = toml::from_str::<toml::Value>(&contents);
            then {
                for dep_type in ["dependencies", "build-dependencies"] {
                    if let Some(table) = manifest.get(dep_type).and_then(|d| d.as_table()) {
                        for (key, value) in table {
                            let package_name = if let Some(dep_table) = value.as_table() {
                                dep_table
                                    .get("package")
                                    .and_then(|p| p.as_str())
                                    .unwrap_or(key)
                            } else {
                                key.as_str()
                            };

                            deps.insert(package_name.to_string());
                        }
                    }
                }
            }
        }

        self.cargo_dependencies = Some(deps);
    }

    fn extract_contract_info_from_macro(mac: &MacCall) -> Option<(String, bool)> {
        for token in mac.args.tokens.iter() {
            if_chain! {
                if let TokenTree::Token(token, _) = token;
                if let TokenKind::Literal(lit) = &token.kind;
                if let LitKind::Str = lit.kind;
                let path = lit.symbol.as_str();
                if path.contains(".wasm");
                if let Some(filename) = path.rsplit('/').next();
                let contract_name = filename.trim_end_matches(".wasm");
                if !contract_name.is_empty();
                then {
                    let uses_deps_path = path.contains("/release/deps/");
                    return Some((contract_name.to_string(), uses_deps_path));
                }
            }
        }
        None
    }

    fn is_contract_in_dependencies(&self, contract_name: &str) -> bool {
        let normalized = contract_name.replace('_', "-");
        self.cargo_dependencies
            .as_ref()
            .is_some_and(|deps| deps.contains(contract_name) || deps.contains(&normalized))
    }
}

impl EarlyLintPass for ContractImportDependency {
    fn check_crate(&mut self, _: &EarlyContext<'_>, _: &rustc_ast::Crate) {
        self.load_cargo_dependencies();
    }

    fn check_item(&mut self, _: &EarlyContext<'_>, item: &Item) {
        if Self::is_test_item(item) {
            self.test_spans.push(item.span);
        }
    }

    fn check_mac(&mut self, cx: &EarlyContext<'_>, mac: &MacCall) {
        if_chain! {
            if Self::is_contractimport_macro(mac);
            if !self.is_within_test(mac.span());
            if let Some((contract_name, uses_deps_path)) = Self::extract_contract_info_from_macro(mac);
            then {
                let is_in_deps = self.is_contract_in_dependencies(&contract_name);

                // Check 1: Contract must be in Cargo.toml
                if !is_in_deps {
                    let help_msg = format!(
                        "Add '{contract_name}' as a dependency in Cargo.toml: \
                         `{contract_name} = {{ path = \"../path/to/{contract_name}\" }}`"
                    );

                    span_lint_and_help(
                        cx,
                        CONTRACT_IMPORT_DEPENDENCY,
                        mac.span(),
                        LINT_MESSAGE,
                        None,
                        help_msg,
                    );
                }

                // Check 2: Must use /release/deps/ path
                if !uses_deps_path {
                    let help_msg = format!(
                        "Use the correct path pattern for contractimport: \
                         '/release/deps/{contract_name}.wasm'"
                    );

                    span_lint_and_help(
                        cx,
                        CONTRACT_IMPORT_DEPENDENCY,
                        mac.span(),
                        "The `contractimport!` macro should use the /release/deps/ path pattern.",
                        None,
                        help_msg,
                    );
                }
            }
        }
    }
}
