use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::output::{report::Report, utils::write_to_file};

use super::generator::{generate_body, generate_header, generate_summary};

const REPORT_MD_PATH: &str = "build/report.md";

// Generates a markdown report from a given `Report` object.
pub fn generate_markdown(report: &Report, path: Option<PathBuf>) -> Result<String> {
    let mut report_markdown = String::new();

    // Header
    report_markdown.push_str(&generate_header(report.date.clone()));

    // Summary
    report_markdown.push_str(&generate_summary(&report.categories, &report.findings));

    // Body
    report_markdown.push_str(&generate_body(&report.categories, &report.findings));

    let output_path = get_output_path(path);

    write_to_file(&output_path, report_markdown.as_bytes())
        .with_context(|| format!("Failed to write markdown to '{}'", output_path.display()))?;

    Ok(output_path.to_string_lossy().into_owned())
}

fn get_output_path(path: Option<PathBuf>) -> PathBuf {
    path.map_or_else(|| PathBuf::from(REPORT_MD_PATH), |p| p.join("report.md"))
}
