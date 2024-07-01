use anyhow::{Error, Result};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::{env, fs};

use crate::scout::blockchain::BlockChain;

const INK_DEFAULT_CONFIG: &str = include_str!("./ink_default_config.toml");

fn create_default_config(
    file_path: PathBuf,
    detectors: Vec<String>,
    bc: BlockChain,
) -> Result<File> {
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(file_path)?;
    let str = match bc {
        BlockChain::Ink => INK_DEFAULT_CONFIG.to_string(),
        BlockChain::Soroban => {
            let mut default_config = toml::Table::new();
            let mut dev_table = toml::Table::new();
            let mut auditor_table = toml::Table::new();
            for detector in detectors {
                let mut detector_config = toml::Table::new();
                detector_config.insert("enabled".to_string(), toml::Value::Boolean(true));
                dev_table.insert(
                    detector.clone(),
                    toml::Value::Table(detector_config.clone()),
                );
                auditor_table.insert(detector, toml::Value::Table(detector_config));
            }
            default_config.insert("auditor".to_string(), toml::Value::Table(auditor_table));
            default_config.insert("dev".to_string(), toml::Value::Table(dev_table));
            toml::to_string_pretty::<toml::Table>(&default_config)?
        }
    };
    file.write_all(str.as_bytes())?;
    Ok(file)
}

pub fn open_config_or_default(bc: BlockChain, detectors: Vec<String>) -> Result<toml::Table> {
    let base_path = match std::env::consts::OS {
        "windows" => env::var("USERPROFILE")? + "/scout/",
        _ => env::var("HOME")? + "/.config/scout/",
    };
    let path = PathBuf::from(base_path);
    if let Err(_metadata) = fs::metadata(&path) {
        fs::create_dir_all(&path)?;
    }
    let file_path = path.as_path().join(match bc {
        BlockChain::Ink => "ink-config.toml",
        BlockChain::Soroban => "soroban-config.toml",
    });
    if !file_path.as_path().exists() {
        create_default_config(file_path.clone(), detectors, bc)?;
    }
    let mut file = File::open(&file_path)?;
    let mut toml_str = String::new();
    let _len = file.read_to_string(&mut toml_str)?;
    let config: toml::Table = toml::from_str(&toml_str)?;

    Ok(config)
}

pub fn profile_enabled_detectors(config: toml::Table, profile: String) -> Result<Vec<String>> {
    let profile = config.get(&profile);
    if profile.is_none() {
        return Err(Error::msg(format!(
            "Profile \"{:?}\" does not exist",
            profile
        )));
    }
    let mut ret_vec = Vec::<String>::new();
    for (detector, config) in profile.unwrap().as_table().unwrap().into_iter() {
        if let Some(val) = config.as_table().unwrap().get("enabled") {
            if val.is_bool() && val.as_bool().unwrap() {
                ret_vec.push(detector.clone());
            }
        }
    }
    Ok(ret_vec)
}
