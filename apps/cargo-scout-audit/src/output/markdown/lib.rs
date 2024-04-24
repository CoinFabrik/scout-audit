use anyhow::Result;

use crate::output::report::Report;

use super::generator::{generate_body, generate_header, generate_summary};

// Generates a markdown report from a given `Report` object.
pub fn generate_markdown(report: &Report) -> Result<String> {
    let mut report_markdown = String::new();

    // Header
    report_markdown.push_str(&generate_header(report.date.clone()));

    // Summary
    report_markdown.push_str(&generate_summary(&report.categories, &report.findings));

    // Body
    report_markdown.push_str(&generate_body(&report.categories, &report.findings));

    Ok(report_markdown)
}
