use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;

use super::report::Severity;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub detectors: Vec<String>,
    pub vulnerabilities: Vec<RawVulnerability>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RawVulnerability {
    pub id: String,
    pub name: String,
    pub short_message: String,
    pub long_message: String,
    pub severity: Severity,
    pub help: String,
    pub vulnerability_class: String,
}

impl Config {
    pub fn load() -> Result<Config> {
        let data = fs::read_to_string("/Users/josegarcia/Desktop/coinfabrik/scout-audit/apps/cargo-scout-audit/src/output/vulnerabilities.json")
            .context("Failed to read vulnerabilities.json")?;
        let config: Config =
            serde_json::from_str(&data).context("Failed to parse vulnerabilities.json")?;

        config.validate()?;

        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        let vulnerability_ids: HashSet<_> =
            self.vulnerabilities.iter().map(|v| v.id.clone()).collect();

        for detector in &self.detectors {
            if !vulnerability_ids.contains(detector) {
                anyhow::bail!(
                    "Detector '{}' is not listed in the vulnerabilities",
                    detector
                );
            }
        }

        Ok(())
    }
}
