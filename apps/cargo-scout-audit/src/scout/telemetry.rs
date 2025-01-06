use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::{env, fs, path::PathBuf};
use strum::EnumIter;

use super::blockchain::BlockChain;

// TODO: Change to the new telemetry endpoint
// const SCOUT_TELEMETRY_URL: &str = "https://telemetry.coinfabrik.com/scout";
const SCOUT_TELEMETRY_URL: &str = "http://190.104.235.53:38522";

pub struct TelemetryClient {
    report: ReportDto,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReportDto {
    pub user_id: String,
    pub scout_version: String,
    pub crate_type: BlockChain,
    pub client_type: ClientType,
    pub os: Os,
}

#[derive(Debug, Deserialize, Serialize, EnumIter, strum::Display, Clone)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum ClientType {
    Vscode,
    Cicd,
    Cli,
}

#[derive(Debug, Deserialize, Serialize, EnumIter, strum::Display, Clone)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum Os {
    Linux,
    Macos,
    Windows,
    Other,
}

impl From<&str> for Os {
    fn from(value: &str) -> Self {
        match value {
            "linux" => Os::Linux,
            "macos" => Os::Macos,
            "windows" => Os::Windows,
            _ => Os::Other,
        }
    }
}

#[derive(Debug, Deserialize)]
struct NewUserResponse {
    user_id: String,
}

impl TelemetryClient {
    pub fn new(blockchain: BlockChain, client_type: ClientType) -> Self {
        let user_id = Self::get_user_id();

        Self {
            report: ReportDto {
                user_id,
                scout_version: env!("CARGO_PKG_VERSION").to_string(),
                crate_type: blockchain,
                client_type,
                os: Os::from(env::consts::OS),
            },
        }
    }

    fn get_user_id() -> String {
        let home_dir = get_home_directory();
        let user_id_path = home_dir
            .join(".scout-audit")
            .join("telemetry")
            .join("user_id.txt");

        // Read user ID from file
        if let Ok(user_id) = fs::read_to_string(&user_id_path) {
            if !user_id.trim().is_empty() {
                return user_id;
            }
        }

        // Create parent directory if it doesn't exist
        if let Some(parent) = user_id_path.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                tracing::error!("Failed to create telemetry directory: {}", e);
                return "DONOTTRACK".to_string();
            }
        }

        // Request new user ID from server
        match Self::request_new_user_id() {
            Ok(user_id) => {
                if let Err(e) = fs::write(&user_id_path, &user_id) {
                    tracing::error!("Failed to cache user ID: {}", e);
                }
                user_id
            }
            Err(e) => {
                tracing::warn!("Failed to get user ID: {}", e);
                "DONOTTRACK".to_string()
            }
        }
    }

    fn request_new_user_id() -> Result<String> {
        let client = Client::new();
        let response = client
            .post(format!("{}/user/new", SCOUT_TELEMETRY_URL))
            .send()
            .context("Failed to send request to telemetry server")?;

        let new_user: NewUserResponse = response
            .json()
            .context("Failed to parse response from telemetry server")?;

        Ok(new_user.user_id)
    }

    pub fn detect_client_type(args: &[String]) -> ClientType {
        if args.contains(&"--message-format=json".to_string()) {
            ClientType::Vscode
        } else if args.contains(&"--cicd".to_string()) {
            ClientType::Cicd
        } else {
            ClientType::Cli
        }
    }

    pub fn send_report(&self) -> Result<()> {
        if self.report.user_id.is_empty() || self.report.user_id.eq("DONOTTRACK") {
            tracing::info!("Telemetry is disabled");
            return Ok(());
        }

        let client = Client::new();
        client
            .post(format!("{}/report/new", SCOUT_TELEMETRY_URL))
            .json(&self.report)
            .send()
            .context("Failed to send telemetry report")?;

        Ok(())
    }
}

#[cfg(windows)]
fn get_home_directory() -> PathBuf {
    PathBuf::from(env::var("USERPROFILE").unwrap_or_else(|e| {
        tracing::error!("Failed to get USERPROFILE: {}", e);
        ".".to_string()
    }))
}

#[cfg(unix)]
fn get_home_directory() -> PathBuf {
    PathBuf::from(env::var("HOME").unwrap_or_else(|e| {
        tracing::error!("Failed to get HOME: {}", e);
        ".".to_string()
    }))
}

#[cfg(not(any(windows, unix)))]
fn get_home_directory() -> PathBuf {
    tracing::warn!("Unsupported OS for home directory detection");
    PathBuf::from(".")
}
