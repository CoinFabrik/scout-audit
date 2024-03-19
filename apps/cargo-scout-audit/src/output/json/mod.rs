use std::path::PathBuf;

use super::{report::Report, utils::write_to_file};

const JSON_PATH: &str = "build/report.json";

pub fn generate_json(report: &Report, path: Option<PathBuf>) -> anyhow::Result<String> {
    let json = serde_json::to_string_pretty(report)?;
    let output_path = get_json_output_path(path);
    write_to_file(&output_path, json.as_bytes())?;
    Ok(output_path.to_string_lossy().into_owned())
}

fn get_json_output_path(path: Option<PathBuf>) -> PathBuf {
    path.map_or_else(|| PathBuf::from(JSON_PATH), |p| p.join("report.json"))
}
