use crate::output::{report::Report, utils::write_to_file};

use super::{
    tera::{create_context, render_template},
    utils,
};
use anyhow::{Context, Result};
use std::{path::PathBuf, vec};

const BASE_TEMPLATE: &str = "base.html";
const REPORT_HTML_PATH: &str = "src/output/html/build/report.html";
const OUTPUT_CSS_PATH: &str = "src/output/html/build/styles.css";
const STYLES_CSS: &[u8] = include_bytes!("templates/styles.css");

// Generates an HTML report from a given `Report` object.
pub fn generate_html(report: &Report) -> Result<&'static str> {
    // Report context
    let report_context = create_context("report", report);

    // Analytics context
    let report_analytics = utils::get_analytics(report);
    let analytics_context = create_context("analytics", report_analytics);

    let html = render_template(BASE_TEMPLATE, vec![report_context, analytics_context])
        .with_context(|| format!("Failed to render template '{}'", BASE_TEMPLATE))?;

    write_to_file(&PathBuf::from(REPORT_HTML_PATH), html.as_bytes())
        .with_context(|| format!("Failed to write HTML to '{}'", REPORT_HTML_PATH))?;

    write_to_file(&PathBuf::from(OUTPUT_CSS_PATH), STYLES_CSS)
        .with_context(|| format!("Failed to write CSS to '{}'", OUTPUT_CSS_PATH))?;

    Ok(REPORT_HTML_PATH)
}
