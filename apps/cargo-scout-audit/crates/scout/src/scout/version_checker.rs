use anyhow::{Context, Result};
use colored::Colorize;
use reqwest::blocking::Client;
use semver::Version;
use serde_json::Value;
use std::{env, path::PathBuf};
use thiserror::Error;
use util::logger::TracedError;
use serde::{Serialize, Deserialize};
use util::home::get_config_directory;
use std::fs::read_to_string;
use chrono::{
    DateTime,
    Utc,
};

const CRATES_IO_URL: &str = "https://crates.io/api/v1/crates/cargo-scout-audit";
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

#[derive(Serialize, Deserialize, Debug)]
struct CachedVersionCheck{
    last_check: Option<i64>,
    pub latest_version: Option<Version>,
}

impl CachedVersionCheck{
    pub fn load() -> Option<CachedVersionCheck>{
        let path = Self::get_path();
        if !std::fs::exists(&path).ok()?{
            None
        }else{
            serde_json::from_str::<CachedVersionCheck>(&read_to_string(path).ok()?).ok()
        }
    }
    pub fn save(ver: Version) -> Result<()>{
        let mut vci = Self{
            last_check: None,
            latest_version: Some(ver),
        };
        vci.set_last_check();

        std::fs::write(Self::get_path(), serde_json::to_string(&vci)?)?;
        Ok(())
    }
    fn get_path() -> PathBuf{
        get_config_directory().join("version.json")
    }
    pub fn get_last_check(&self) -> Option<DateTime<Utc>>{
        DateTime::from_timestamp(self.last_check?, 0)
    }
    pub fn set_last_check(&mut self){
        self.last_check = Some(Utc::now().timestamp());
    }
}

impl VersionChecker {
    pub fn new() -> Self {
        VersionChecker {
            client: Client::new(),
        }
    }

    pub fn check_for_updates(&self) -> Result<()> {
        let cached = CachedVersionCheck::load();
        
        let mut latest_version = None;
        if let Some(last_check) = cached{
            if let Some(t) = last_check.get_last_check(){
                if (Utc::now() - t).num_hours() < 24{
                    if let Some(lv) = last_check.latest_version{
                        latest_version = Some(lv)
                    }
                }
            }
        };
        let latest_version = if let Some(latest_version) = latest_version{
            latest_version
        }else{
            let latest_version = self.get_latest_version()?;
            let _ = CachedVersionCheck::save(latest_version.clone());
            latest_version
        };

        let current_version = Version::parse(CURRENT_VERSION)
            .map_err(VersionCheckerError::VersionParseFailed.traced())?;

        if latest_version > current_version {
            self.print_update_warning(&current_version, &latest_version);
        }

        Ok(())
    }

    fn get_latest_version(&self) -> Result<Version> {
        let response = self
            .client
            .get(CRATES_IO_URL)
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
