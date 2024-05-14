use super::{generator::generate_summary_context, tera::MdEngine};
use crate::output::report::Report;
use anyhow::{Context, Result};

// Generates an Markdown report from a given `Report` object.
pub fn generate_markdown(report: &Report, render_styles: bool) -> Result<String> {
    let tera = MdEngine::new()?;

    let summary_context = generate_summary_context(report);

    let report_context = tera.create_context("report", report);
    let summary_context = tera.create_context("summary", summary_context);
    let style_context = tera.create_context("render_styles", render_styles);

    // Render the template with the contexts
    let html = tera
        .render_template(vec![report_context, summary_context, style_context])
        .with_context(|| "Failed to render template 'base_template'")?;

    Ok(html)
}
