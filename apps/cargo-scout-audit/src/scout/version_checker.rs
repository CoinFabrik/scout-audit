use anyhow::{Context, Result};
use colored::Colorize;
use reqwest::blocking::Client;
use semver::Version;
use serde_json::Value;
use std::env;
use thiserror::Error;

use crate::utils::telemetry::TracedError;

const CRATE_NAME: &str = env!("CARGO_PKG_NAME");
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Default)]
pub struct VersionChecker {
    client: Client,
}

#[derive(Error, Debug)]
pub enum VersionCheckerError {
    #[error("Failed to connect to crates.io API. Please check your internet connection")]
    RequestFailed,

    #[error("Failed to parse JSON response from crates.io")]
    JsonParseFailed,

    #[error("Failed to extract version string from response")]
    VersionStringExtractionFailed,

    #[error("Failed to parse version string from crates.io")]
    VersionParseFailed,
}

impl VersionChecker {
    pub fn new() -> Self {
        VersionChecker {
            client: Client::new(),
        }
    }

    pub fn check_for_updates(&self) -> Result<()> {
        let current_version = Version::parse(CURRENT_VERSION)
            .map_err(VersionCheckerError::VersionParseFailed.traced())?;
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
            .map_err(VersionCheckerError::RequestFailed.traced())?
            .json::<Value>()
            .map_err(VersionCheckerError::JsonParseFailed.traced())?;

        let version_str = response["crate"]["max_stable_version"]
            .as_str()
            .with_context(|| VersionCheckerError::VersionStringExtractionFailed)?;

        Version::parse(version_str).map_err(VersionCheckerError::VersionParseFailed.traced())
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

        eprintln!("{}", message.yellow());
    }
}
