use std::path::PathBuf;

use anyhow::{Context, Result};

use crate::output::{report::Report, utils::write_to_file};

use super::generator::{generate_body, generate_header, generate_summary};

const REPORT_MD_PATH: &str = "build/report.html";

// Generates a markdown report from a given `Report` object.
pub fn generate_pdf(report: &Report) -> Result<&'static str> {
    let mut report_md = String::new();

    // Header
    report_md.push_str(&generate_header(report.date.clone()));

    // Summary
    report_md.push_str(&generate_summary(&report.categories, &report.findings));

    // Body
    report_md.push_str(&generate_body(&report.categories, &report.findings));

    write_to_file(&PathBuf::from(REPORT_MD_PATH), report_md.as_bytes())
        .with_context(|| format!("Failed to write html for pdf to '{}'", REPORT_MD_PATH))?;

    Ok(REPORT_MD_PATH)
}
