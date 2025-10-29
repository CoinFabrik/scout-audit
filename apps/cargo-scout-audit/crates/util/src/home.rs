use std::{env, path::PathBuf};

#[cfg(windows)]
pub fn get_home_directory() -> PathBuf {
    PathBuf::from(env::var("USERPROFILE").unwrap_or_else(|e| {
        tracing::error!("Failed to get USERPROFILE: {}", e);
        ".".to_string()
    }))
}

#[cfg(unix)]
pub fn get_home_directory() -> PathBuf {
    PathBuf::from(env::var("HOME").unwrap_or_else(|e| {
        tracing::error!("Failed to get HOME: {}", e);
        ".".to_string()
    }))
}

#[cfg(not(any(windows, unix)))]
pub fn get_home_directory() -> PathBuf {
    tracing::warn!("Unsupported OS for home directory detection");
    PathBuf::from(".")
}

pub fn get_config_directory() -> PathBuf {
    get_home_directory().join(".scout-audit")
}
