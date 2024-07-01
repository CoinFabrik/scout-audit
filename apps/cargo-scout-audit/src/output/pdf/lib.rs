use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::output::{report::Report, utils::write_to_file};

use super::generator::{generate_body, generate_header, generate_summary};

const REPORT_PATH: &str = "temp_report.html";

// Generates a HTML report from a given `Report` object.
pub fn generate_pdf(report: &Report) -> Result<&'static str> {
    let mut report_html = String::new();

    // Header
    report_html.push_str(&generate_header(report.date.clone()));

    // Summary
    report_html.push_str(&generate_summary(report));

    // Body
    report_html.push_str(&generate_body(&report.categories, &report.findings));

    write_to_file(&PathBuf::from(REPORT_PATH), report_html.as_bytes())
        .with_context(|| format!("Failed to write html for pdf to '{}'", REPORT_PATH))?;

    Ok(REPORT_PATH)
}
