use super::tera::{create_context, render_template};
use anyhow::{Context, Result};
use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

const BASE_TEMPLATE: &str = "base.html";
const REPORT_HTML_PATH: &str = "build/report.html";
const OUTPUT_CSS_PATH: &str = "build/output.css";
const STYLES_CSS: &[u8] = include_bytes!("templates/styles.css");

// Generates an HTML report from a given `Report` object.
pub fn generate_html(report: impl serde::Serialize) -> Result<&'static str> {
    let context = create_context(report);
    let html = render_template(BASE_TEMPLATE, &context)
        .with_context(|| format!("Failed to render template '{}'", BASE_TEMPLATE))?;

    write_to_file(&PathBuf::from(REPORT_HTML_PATH), html.as_bytes())
        .with_context(|| format!("Failed to write HTML to '{}'", REPORT_HTML_PATH))?;

    write_to_file(&PathBuf::from(OUTPUT_CSS_PATH), STYLES_CSS)
        .with_context(|| format!("Failed to write CSS to '{}'", OUTPUT_CSS_PATH))?;

    Ok(REPORT_HTML_PATH)
}

// Writes data to a file at the specified path.
fn write_to_file(path: &PathBuf, data: &[u8]) -> Result<(), std::io::Error> {
    // Write to a temporary file first
    let temp_path = path.with_extension("tmp");
    let mut temp_file = File::create(&temp_path)?;
    temp_file.write_all(data)?;

    // Rename temporary file to the target path
    fs::rename(temp_path, path)?;

    Ok(())
}
