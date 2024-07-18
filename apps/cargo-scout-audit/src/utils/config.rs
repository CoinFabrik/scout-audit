use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use crate::scout::blockchain::BlockChain;

use super::print::print_warning;

pub fn open_config_or_default(bc: BlockChain, detectors: &[String]) -> Result<(Value, PathBuf)> {
    let config_path = get_config_path()?;

    let file_path = config_path.join(match bc {
        BlockChain::Ink => "ink-config.json",
        BlockChain::Soroban => "soroban-config.json",
    });

    if !file_path.exists() {
        create_default_config(&file_path, detectors)
            .with_context(|| format!("Failed to create default config file: {:?}", file_path))?;
    }

    let config_str = read_file_to_string(&file_path)
        .with_context(|| format!("Failed to read config file: {:?}", file_path))?;

    let config = serde_json::from_str(&config_str)
        .with_context(|| format!("Failed to parse JSON config: {:?}", file_path))?;

    Ok((config, file_path))
}

fn get_config_path() -> Result<PathBuf> {
    let base_path = if cfg!(windows) {
        std::env::var("USERPROFILE")
            .with_context(|| "Failed to get USERPROFILE environment variable")?
    } else {
        std::env::var("HOME").with_context(|| "Failed to get HOME environment variable")?
    };

    let config_path = PathBuf::from(base_path).join(if cfg!(windows) {
        "scout"
    } else {
        ".config/scout"
    });

    fs::create_dir_all(&config_path)
        .with_context(|| format!("Failed to create config directory: {:?}", config_path))?;

    Ok(config_path)
}

fn create_default_config(file_path: &PathBuf, detectors: &[String]) -> Result<()> {
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

fn read_file_to_string(path: &Path) -> io::Result<String> {
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

pub fn profile_enabled_detectors(
    config: &Value,
    profile: &str,
    config_path: &Path,
) -> Result<Vec<String>> {
    let default_detectors: HashSet<String> = config
        .get("default")
        .and_then(Value::as_array)
        .with_context(|| "Default profile is missing or not an array")?
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
            create_profile(
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
        .filter(|detector| default_detectors.contains(detector))
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

fn create_profile(file_path: &Path, detectors: &[String], profile: &str) -> Result<()> {
    let existing_profiles = read_file_to_string(file_path)
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
