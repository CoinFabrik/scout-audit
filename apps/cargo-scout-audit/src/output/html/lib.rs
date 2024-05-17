use crate::output::report::Report;

use super::{tera::HtmlEngine, utils};
use anyhow::{Context, Result};
use std::vec;

const BASE_TEMPLATE_NAME: &str = "base.html";

// Generates an HTML report from a given `Report` object.
pub fn generate_html(report: &Report) -> Result<String> {
    let tera = HtmlEngine::new()?;
    // Report context
    let report_context = tera.create_context("report", report);

    // Analytics context
    let report_analytics = utils::get_analytics(report);
    let analytics_context = tera.create_context("analytics", report_analytics);

    let html = tera
        .render_template(vec![report_context, analytics_context])
        .with_context(|| format!("Failed to render template '{}'", BASE_TEMPLATE_NAME))?;

    Ok(html)
}
