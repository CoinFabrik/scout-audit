use anyhow::Result;
use std::path::{Path, PathBuf};
use tempfile::NamedTempFile;

pub mod build_and_run;
pub mod cargo;
pub mod command;
pub mod dependencies;
pub mod detectors;
pub mod detectors_info;
pub mod env;
pub mod git;
pub mod json;
pub mod library;
pub mod logger;
pub mod print;
pub mod home;

pub fn paths_to_strings(paths: &[PathBuf]) -> Vec<String> {
    paths
        .iter()
        .filter_map(|x| x.to_str())
        .map(|x| x.to_owned())
        .collect()
}

pub fn path_to_string(path: &Path) -> String {
    path.to_str().map(|x| x.to_owned()).unwrap_or_default()
}

pub fn temp_path() -> Result<(NamedTempFile, String)> {
    let mut file = NamedTempFile::new()?;
    file.disable_cleanup(true);
    let path = path_to_string(file.path());
    Ok((file, path))
}
