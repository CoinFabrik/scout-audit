use crate::startup::OutputFormat;

use anyhow::Result;

use super::{html::generate_html, report::Report};

pub fn get_output(report: Report, output_format: OutputFormat) -> Result<&'static str> {
    match output_format {
        OutputFormat::Html => generate_html(report),
        _ => unimplemented!(),
        // OutputType::Sarif => generate_sarif(report),
        // OutputType::Json => generate_json(report),
        // OutputType::Markdown => generate_markdown(report),
    }
}
