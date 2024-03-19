use std::{path::PathBuf, process::Command};

use anyhow::{Context, Result};

use crate::output::{report::Report, utils::write_to_file};

use super::generator::{generate_body, generate_header, generate_summary};

const PDF_PATH: &str = "build/report.pdf";
const TEMP_HTML_PATH: &str = "build/temp_report.html";

// Generates a markdown report from a given `Report` object.
pub fn generate_pdf(report: &Report, path: Option<PathBuf>) -> Result<String> {
    let mut report_html = String::new();

    // Header
    report_html.push_str(&generate_header(report.date.clone()));

    // Summary
    report_html.push_str(&generate_summary(&report.categories, &report.findings));

    // Body
    report_html.push_str(&generate_body(&report.categories, &report.findings));

    // Write the generated HTML to a temporary file
    let html_output_path = PathBuf::from(TEMP_HTML_PATH);
    write_to_file(&html_output_path, report_html.as_bytes())
        .with_context(|| format!("Failed to write HTML to '{}'", html_output_path.display()))?;

    // Determine the output path for the PDF
    let pdf_output_path = get_output_path(path);

    // Generate PDF from HTML
    generate_pdf_from_html(TEMP_HTML_PATH, pdf_output_path.to_str().unwrap())?;

    Ok(pdf_output_path.to_string_lossy().into_owned())
}

fn get_output_path(path: Option<PathBuf>) -> PathBuf {
    path.unwrap_or_else(|| PathBuf::from(PDF_PATH))
}

fn generate_pdf_from_html(html_path: &str, pdf_path: &str) -> Result<()> {
    Command::new("wkhtmltopdf")
        .arg(html_path)
        .arg(pdf_path)
        .spawn()
        .expect("wkhtmltopdf command failed to start")
        .wait()
        .expect("Failed to wait on wkhtmltopdf");

    Ok(())
}
