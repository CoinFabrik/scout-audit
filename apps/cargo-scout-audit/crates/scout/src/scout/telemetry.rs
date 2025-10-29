use super::blockchain::BlockChain;
use anyhow::{Context, Result};
use cli_args::Scout;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::{
    env, fs,
    time::{SystemTime, UNIX_EPOCH},
};
use strum::EnumIter;
use util::home::get_config_directory;

const SCOUT_TELEMETRY_URL: &str = "https://scout-api.coinfabrik.com";

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
        let user_id_path = get_config_directory().join("telemetry").join("user_id.txt");

        // Read user ID from file
        if let Ok(content) = fs::read_to_string(&user_id_path)
            && let Some(last_line) = content.lines().last()
            && !last_line.trim().is_empty()
        {
            return last_line.to_string();
        }

        // Create parent directory if it doesn't exist
        if let Some(parent) = user_id_path.parent()
            && let Err(e) = fs::create_dir_all(parent)
        {
            tracing::error!("Failed to create telemetry directory: {}", e);
            return "DONOTTRACK".to_string();
        }

        // Request new user ID from server
        match Self::request_new_user_id() {
            Ok(user_id) => {
                let file_content = format!(
                    "# This file is used by Scout to gather anonymous usage data. You can see the\n\
                     # last few reports that have been sent in the reports/ subdirectory.\n\
                     #\n\
                     # View your data at: https://scout-api.coinfabrik.com/report/data/{}\n\
                     # Delete your data at: https://scout-api.coinfabrik.com/user/delete/{}\n\
                     # To opt-out of telemetry, replace the following line with DONOTTRACK\n\
                     {}",
                    user_id, user_id, user_id
                );

                if let Err(e) = fs::write(&user_id_path, file_content) {
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

    pub fn detect_client_type(opts: &Scout) -> ClientType {
        if opts.cicd.is_some() {
            ClientType::Cicd
        } else if opts.args.contains(&"--message-format=json".to_string()) {
            ClientType::Vscode
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

        let reports_dir = get_config_directory().join("telemetry").join("reports");
        fs::create_dir_all(&reports_dir)?;

        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let report_path = reports_dir.join(format!("report_{}.json", timestamp));
        let json = serde_json::to_string_pretty(&self.report)?;
        fs::write(&report_path, json)?;

        Ok(())
    }
}
