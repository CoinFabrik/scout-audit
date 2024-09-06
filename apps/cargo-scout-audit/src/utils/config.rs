use super::print::print_warning;
use crate::scout::blockchain::BlockChain;
use anyhow::{Context, Result};
use serde_json::{json, Value};
use std::{
    collections::HashSet,
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
};

pub fn open_config_and_sync_detectors(
    blockchain: BlockChain,
    detector_names: &[String],
) -> Result<(Value, PathBuf)> {
    let (mut config, config_path) = open_config_or_default(blockchain, detector_names)?;

    // Synchronize config with current detector names and sort
    sync_config_with_detectors(&mut config, detector_names)?;

    // Save updated config
    save_config(&config, &config_path)?;

    Ok((config, config_path))
}

fn sync_config_with_detectors(config: &mut Value, detector_names: &[String]) -> Result<()> {
    let default_detectors = config["default"]
        .as_array_mut()
        .with_context(|| "Default profile is missing or not an array")?;

    let current_detectors: HashSet<String> = default_detectors
        .iter()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();

    let available_detectors: HashSet<String> = detector_names.iter().cloned().collect();

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
    sort_detectors(default_detectors);

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

            sort_detectors(profile_detectors);
        }
    }

    Ok(())
}

fn sort_detectors(detectors: &mut [Value]) {
    detectors.sort_by(|a, b| {
        let a_str = a.as_str().unwrap_or("");
        let b_str = b.as_str().unwrap_or("");
        a_str.cmp(b_str)
    });
}

fn open_config_or_default(bc: BlockChain, detectors: &[String]) -> Result<(Value, PathBuf)> {
    let config_file_path = get_config_file_path(bc)?;

    if !config_file_path.exists() {
        create_default_config(&config_file_path, detectors).with_context(|| {
            format!(
                "Failed to create default config file: {:?}",
                config_file_path
            )
        })?;
    }

    let config_str = read_file_to_string(&config_file_path)
        .with_context(|| format!("Failed to read config file: {:?}", config_file_path))?;

    let config = serde_json::from_str(&config_str)
        .with_context(|| format!("Failed to parse JSON config: {:?}", config_file_path))?;

    Ok((config, config_file_path))
}

fn get_config_file_path(bc: BlockChain) -> Result<PathBuf> {
    let base_path =
        std::env::var("HOME").with_context(|| "Failed to get HOME environment variable")?;

    let config_path = PathBuf::from(base_path).join(".config/scout");

    fs::create_dir_all(&config_path)
        .with_context(|| format!("Failed to create config directory: {:?}", config_path))?;

    let file_path = config_path.join(match bc {
        BlockChain::Ink => "ink-config.json",
        BlockChain::Soroban => "soroban-config.json",
    });

    Ok(file_path)
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
    detector_names: &[String],
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
        .filter(|detector| {
            default_detectors.contains(detector) && detector_names.contains(detector)
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

fn save_config(config: &Value, config_path: &Path) -> Result<()> {
    let config_str = serde_json::to_string_pretty(config)
        .context("Failed to serialize config to JSON string")?;

    std::fs::write(config_path, config_str).context("Failed to write config to file")?;

    Ok(())
}
