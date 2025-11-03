use crate::scout::output::report::Report;

use super::{tera::HtmlEngine, utils};
use anyhow::Result;
use std::{error::Error, fmt::Write, vec};

// Generates an HTML report from a given `Report` object.
pub fn generate_html(report: &Report) -> Result<String> {
    let tera = HtmlEngine::new()?;

    // Report context
    let report_context = tera.create_context("report", report);

    // Analytics context
    let report_analytics = utils::get_analytics(report);
    let analytics_context = tera.create_context("analytics", report_analytics);
    tera.render_template(vec![report_context, analytics_context])
        .map_err(|err: tera::Error| {
            let mut error_msg = format!("Error rendering HTML report:\n -> {}", err);
            if let Some(source) = err.source() {
                write!(error_msg, "\n -> Caused by: {}", source).unwrap();
            }
            anyhow::anyhow!(error_msg)
        })
}
