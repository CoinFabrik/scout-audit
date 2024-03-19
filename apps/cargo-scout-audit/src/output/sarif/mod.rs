use super::{report::Report, utils::write_to_file};
use anyhow::Result;
use std::path::PathBuf;

const SARIF_PATH: &str = "build/report.sarif";

pub fn generate_sarif(report: &Report, path: Option<PathBuf>) -> Result<String> {
    let sarif = serde_json::to_string_pretty(&report)?;
    let output_path = get_output_path(path);
    write_to_file(&output_path, sarif.as_bytes())?;
    Ok(output_path.to_string_lossy().into_owned())
}

fn get_output_path(path: Option<PathBuf>) -> PathBuf {
    path.map_or_else(|| PathBuf::from(SARIF_PATH), |p| p.join("report.sarif"))
}
