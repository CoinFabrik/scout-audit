use super::generator::{generate_body, generate_header, generate_summary};
use crate::output::report::Report;
use crate::output::pdf::external::{
    build_library,
    call,
};
use anyhow::{Context, Result};
use std::io::Write;
use std::path::Path;
use tempfile::{Builder, NamedTempFile};

// Generates a HTML report from a given `Report` object.
fn generate_temp_html(report: &Report) -> Result<NamedTempFile> {
    let mut report_html = String::new();

    // Header
    report_html.push_str(&generate_header(report.date.clone()));

    // Summary
    report_html.push_str(&generate_summary(report));

    // Body
    report_html.push_str(&generate_body(&report.categories, &report.findings));

    let mut file = Builder::new()
        .suffix(".html")
        .tempfile()
        .with_context(|| ("Failed to create temporary HTML file"))?;
    file.write(report_html.as_bytes())
        .with_context(|| "Failed to write temporary HTML file")?;
    Ok(file)
}

pub fn generate_pdf(path: &Path, report: &Report) -> Result<()> {
    let temp_html = generate_temp_html(report)?;
    let library = build_library()?;
    let url = "file:///".to_string() + temp_html.path().to_str().unwrap();
    let output_path = path
        .as_os_str()
        .to_str()
        .with_context(|| "Failed to get output path for PDF")?;
    let result = call(&library, &url, output_path)?;
    let _ = temp_html.close();
    if result{
        Ok(())
    }else{
        None.with_context(|| "PDF generation failed")
    }
}
