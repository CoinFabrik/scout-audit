use crate::{
    cli_args::{BlockChain, OutputFormat},
    util::print::print_info,
};
use anyhow::{Context, Result, anyhow};
use cargo_metadata::{Metadata, camino::Utf8PathBuf};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Serialize, Deserialize, Default)]
struct ScoutConfig {
    #[serde(default)]
    pub exclude: Vec<String>,
    #[serde(default)]
    pub output_format: Vec<OutputFormat>,
}

pub struct ProfileConfig {
    pub blockchain: BlockChain,
    pub detector_names: Vec<String>,
    pub output_format: Vec<OutputFormat>,
}

impl ProfileConfig {
    pub fn new(
        blockchain: BlockChain,
        detector_names: Vec<String>,
        output_format: Vec<OutputFormat>,
    ) -> Self {
        Self {
            blockchain,
            detector_names,
            output_format,
        }
    }

    pub fn get_config(&self, metadata: &Metadata) -> Result<ProfileConfig> {
        match self.load_project_config(metadata)? {
            Some(config) => {
                print_info("Using project configuration file, please check it carefully.");
                Ok(ProfileConfig {
                    blockchain: self.blockchain,
                    detector_names: self
                        .detector_names
                        .iter()
                        .filter(|name| !config.exclude.contains(name))
                        .cloned()
                        .collect(),
                    output_format: config.output_format,
                })
            }
            None => Ok(ProfileConfig {
                blockchain: self.blockchain,
                detector_names: self.detector_names.clone(),
                output_format: self.output_format.clone(),
            }),
        }
    }

    fn load_project_config(&self, metadata: &Metadata) -> Result<Option<ScoutConfig>> {
        let Some(config_path) = self.find_config_path(metadata) else {
            return Ok(None);
        };

        let contents = fs::read_to_string(&config_path)
            .with_context(|| format!("Failed to read config file: {:?}", config_path))?;

        let config: ScoutConfig = serde_yaml::from_str(&contents)
            .with_context(|| format!("Failed to parse config file: {:?}", config_path))?;

        // Validate config structure
        self.validate_config(&config)?;

        Ok(Some(config))
    }

    fn find_config_path(&self, metadata: &Metadata) -> Option<Utf8PathBuf> {
        let config_locations = [
            // First check package-level config
            metadata
                .root_package()
                .and_then(|package| package.manifest_path.parent())
                .map(|parent| parent.join(".scout-audit").join("config.yaml")),
            // Then fallback to workspace-level config
            metadata
                .workspace_root
                .join(".scout-audit")
                .join("config.yaml")
                .into(),
        ];

        config_locations
            .into_iter()
            .flatten()
            .find(|path| path.exists())
    }

    fn validate_config(&self, config: &ScoutConfig) -> Result<()> {
        // Validate that all excluded detectors exist in the available detectors
        for excluded in &config.exclude {
            if !self.detector_names.contains(excluded) {
                return Err(anyhow!(
                    "Configuration error: Unknown detector '{}' in exclude list",
                    excluded
                ));
            }
        }

        Ok(())
    }
}
