use anyhow::{Context, Result};
use colored::Colorize;
use reqwest::blocking::Client;
use semver::Version;
use serde_json::Value;
use std::env;

const CRATE_NAME: &str = env!("CARGO_PKG_NAME");
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct VersionChecker {
    client: Client,
}

impl VersionChecker {
    pub fn new() -> Self {
        VersionChecker {
            client: Client::new(),
        }
    }

    pub fn check_for_updates(&self) -> Result<()> {
        let current_version =
            Version::parse(CURRENT_VERSION).with_context(|| "Failed to parse current version")?;
        let latest_version = self.get_latest_version()?;

        if latest_version > current_version {
            self.print_update_warning(&current_version, &latest_version);
        }

        Ok(())
    }

    fn get_latest_version(&self) -> Result<Version> {
        let url = format!("https://crates.io/api/v1/crates/{}", CRATE_NAME);
        let response = self
            .client
            .get(&url)
            .header("User-Agent", "scout-version-checker/1.0")
            .send()
            .with_context(|| "Failed to send request to crates.io")?
            .json::<Value>()
            .with_context(|| "Failed to parse JSON response from crates.io")?;

        let version_str = response["crate"]["max_stable_version"]
            .as_str()
            .with_context(|| "Failed to extract version string from response")?;

        Version::parse(version_str).with_context(|| "Failed to parse version string from crates.io")
    }

    fn print_update_warning(&self, current_version: &Version, latest_version: &Version) {
        let title = "A new version of Scout is available!";
        let command = "cargo install cargo-scout-audit";
        let message = &format!(
            r#"
                ╔════════════════════════════════════════════════════════════════╗
                ║{:^64}║
                ║                                                                ║
                ║  Current version: {:<44} ║
                ║  Latest version:  {:<44} ║
                ║                                                                ║
                ║  Update available! Run the following command to update:        ║
                ║{:^64}║
                ╚════════════════════════════════════════════════════════════════╝
                "#,
            title.bold(),
            current_version.to_string().yellow(),
            latest_version.to_string().green(),
            command.cyan()
        );

        println!("{}", message.yellow());
    }
}
