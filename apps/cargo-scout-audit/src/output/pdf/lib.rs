use super::generator::{generate_body, generate_header, generate_summary};
use crate::output::report::Report;
use anyhow::{Context, Result};
use headless_chrome::{Browser, LaunchOptionsBuilder};
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
    let browser = Browser::new(LaunchOptionsBuilder::default().headless(true).build()?)?;
    let tab = browser.new_tab()?;
    let url = "file:///".to_string() + temp_html.path().to_str().unwrap();
    tab.navigate_to(url.as_str())?;
    tab.wait_until_navigated()?;
    std::fs::write(path, tab.print_to_pdf(None)?)?;
    let _ = temp_html.close();
    Ok(())
}
