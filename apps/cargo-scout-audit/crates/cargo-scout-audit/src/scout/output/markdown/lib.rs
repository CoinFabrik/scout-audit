use super::{generator::generate_summary_context, tera::MdEngine};
use crate::scout::output::{report::Report, table::register_functions_for_tera_md};
use anyhow::{Context, Result};

// Generates an Markdown report from a given `Report` object.
pub fn generate_markdown(report: &Report, render_styles: bool) -> Result<String> {
    let mut tera = MdEngine::new()?;

    let (summary, table) = generate_summary_context(report);

    let report_context = tera.create_context("report", report);
    let summary_context = tera.create_context("summary", summary);
    let style_context = tera.create_context("render_styles", render_styles);

    let summary_table_context = tera.create_context("summary_table", table);
    register_functions_for_tera_md(tera.get_tera_mut());

    // Render the template with the contexts
    let html = tera
        .render_template(vec![
            report_context,
            summary_context,
            summary_table_context,
            style_context,
        ])
        .with_context(|| "Failed to render template 'base_template'")?;

    Ok(html)
}
