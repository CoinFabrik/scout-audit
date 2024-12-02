use super::print::print_warning;
use crate::scout::blockchain::BlockChain;
use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};
use std::{
    collections::HashSet,
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

pub struct ProfileConfig {
    blockchain: BlockChain,
    detector_names: Vec<String>,
}

impl ProfileConfig {
    pub fn new(blockchain: BlockChain, detector_names: Vec<String>) -> Self {
        Self {
            blockchain,
            detector_names,
        }
    }

    pub fn get_profile_detectors(&self, profile: &Option<String>) -> Result<Vec<String>> {
        match profile {
            Some(profile_name) => {
                let (config, config_path) = self
                    .open_config_and_sync_detectors()
                    .map_err(|err| anyhow!(
                        "Failed to open and synchronize configuration file.\n\n     → Caused by: {}",
                        err
                    ))?;

                print_warning(&format!(
                    "Using profile '{}' to filter detectors. To edit this profile, open the configuration file at: {}",
                    profile_name,
                    config_path.display()
                ));

                self.profile_enabled_detectors(&config, profile_name, &config_path)
            }
            None => Ok(self.detector_names.clone()),
        }
    }

    fn open_config_and_sync_detectors(&self) -> Result<(Value, PathBuf)> {
        let (mut config, config_path) = self.open_config_or_default()?;

        // Synchronize config with current detector names and sort
        self.sync_config_with_detectors(&mut config)?;

        // Save updated config
        self.save_config(&config, &config_path)?;

        Ok((config, config_path))
    }

    fn sync_config_with_detectors(&self, config: &mut Value) -> Result<()> {
        let default_detectors = config["default"]
            .as_array_mut()
            .with_context(|| "Default profile is missing or not an array")?;

        let current_detectors: HashSet<String> = default_detectors
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();

        let available_detectors: HashSet<String> = self.detector_names.iter().cloned().collect();

        // Add new detectors
        for detector in available_detectors.difference(&current_detectors) {
            default_detectors.push(json!(detector));
            print_warning(
            "Default profile synchronized with available detectors, do not edit default profile.",
        );
        }

        // Remove obsolete detectors
        default_detectors.retain(|d| {
            let keep = available_detectors.contains(d.as_str().unwrap_or(""));
            if !keep {
                print_warning(&format!(
                    "Obsolete detector removed from default profile: {}",
                    d
                ));
            }
            keep
        });

        // Sort default detectors
        self.sort_detectors(default_detectors);

        // Update and sort other profiles
        for (profile, detectors) in config.as_object_mut().unwrap() {
            if profile != "default" {
                let profile_detectors = detectors
                    .as_array_mut()
                    .with_context(|| format!("Profile '{}' is not an array", profile))?;

                profile_detectors.retain(|d| {
                    let keep = available_detectors.contains(d.as_str().unwrap_or(""));
                    if !keep {
                        print_warning(&format!(
                            "Obsolete detector removed from profile '{}': {}",
                            profile, d,
                        ));
                    }
                    keep
                });

                self.sort_detectors(profile_detectors);
            }
        }

        Ok(())
    }

    fn sort_detectors(&self, detectors: &mut [Value]) {
        detectors.sort_by(|a, b| {
            let a_str = a.as_str().unwrap_or("");
            let b_str = b.as_str().unwrap_or("");
            a_str.cmp(b_str)
        });
    }

    fn open_config_or_default(&self) -> Result<(Value, PathBuf)> {
        let config_file_path = self.get_config_file_path(self.blockchain)?;

        if !config_file_path.exists() {
            self.create_default_config(&config_file_path, &self.detector_names)
                .with_context(|| {
                    format!(
                        "Failed to create default config file: {}",
                        config_file_path.display()
                    )
                })?;
        }

        let config_str = self
            .read_file_to_string(&config_file_path)
            .with_context(|| format!("Failed to read config file: {:?}", config_file_path))?;

        let config = serde_json::from_str(&config_str)
            .with_context(|| format!("Failed to parse JSON config: {:?}", config_file_path))?;

        Ok((config, config_file_path))
    }

    fn get_config_file_path(&self, bc: BlockChain) -> Result<PathBuf> {
        let base_path =
            std::env::var("HOME").with_context(|| "Failed to get HOME environment variable")?;

        let config_path = PathBuf::from(base_path).join(".config/scout");

        fs::create_dir_all(&config_path)
            .with_context(|| format!("Failed to create config directory: {:?}", config_path))?;

        let file_path = config_path.join(match bc {
            BlockChain::Ink => "ink-config.json",
            BlockChain::Soroban => "soroban-config.json",
            BlockChain::SubstratePallets => "substrate-pallet-config.json",
        });

        Ok(file_path)
    }

    fn create_default_config(&self, file_path: &PathBuf, detectors: &[String]) -> Result<()> {
        let config = json!({
            "default": detectors,
        });

        let config_str = serde_json::to_string_pretty(&config)
            .with_context(|| "Failed to serialize config to JSON string")?;

        File::create(file_path)
            .with_context(|| format!("Failed to create file: {:?}", file_path))?
            .write_all(config_str.as_bytes())
            .with_context(|| "Failed to write config to file")?;

        Ok(())
    }

    fn read_file_to_string(&self, path: &Path) -> io::Result<String> {
        let mut file = File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(contents)
    }

    fn profile_enabled_detectors(
        &self,
        config: &Value,
        profile: &str,
        config_path: &Path,
    ) -> Result<Vec<String>> {
        let default_detectors: HashSet<String> = config["default"]
            .as_array()
            .context("Default profile is missing or not an array")?
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect();

        let profile_detectors = match config.get(profile).and_then(Value::as_array) {
            Some(detectors) => detectors,
            None => {
                print_warning(&format!(
                    "Profile '{}' does not exist, creating it with default detectors",
                    profile
                ));
                self.create_profile(
                    config_path,
                    &default_detectors.iter().cloned().collect::<Vec<_>>(),
                    profile,
                )
                .with_context(|| format!("Failed to create profile '{}'", profile))?;
                config.get("default").and_then(Value::as_array).unwrap()
            }
        };

        let enabled_detectors: Vec<String> = profile_detectors
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .filter(|detector| {
                default_detectors.contains(detector) && self.detector_names.contains(detector)
            })
            .collect();

        if enabled_detectors.is_empty() {
            Err(anyhow::anyhow!(
                "No enabled detectors found in profile '{}'",
                profile
            ))
        } else {
            Ok(enabled_detectors)
        }
    }

    fn create_profile(&self, file_path: &Path, detectors: &[String], profile: &str) -> Result<()> {
        let existing_profiles = self
            .read_file_to_string(file_path)
            .with_context(|| "Failed to read config file")?
            .parse::<Value>()
            .with_context(|| "Failed to parse JSON config")?;

        let mut new_profiles = existing_profiles.clone();
        new_profiles[profile] = detectors.iter().map(|d| Value::String(d.clone())).collect();

        let config_str = serde_json::to_string_pretty(&new_profiles)
            .with_context(|| "Failed to serialize config to JSON string")?;

        File::create(file_path)
            .with_context(|| format!("Failed to create file: {:?}", file_path))?
            .write_all(config_str.as_bytes())
            .with_context(|| "Failed to write config to file")?;

        Ok(())
    }

    fn save_config(&self, config: &Value, config_path: &Path) -> Result<()> {
        let config_str = serde_json::to_string_pretty(config)
            .with_context(|| "Failed to serialize config to JSON string")?;

        std::fs::write(config_path, config_str)
            .with_context(|| "Failed to write config to file")?;

        Ok(())
    }
}
