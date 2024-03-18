use crate::output::{report::Report, utils::write_to_file};

use super::{
    tera::{create_context, render_template},
    utils,
};
use anyhow::{Context, Result};
use std::{fs, io::Read, path::PathBuf, vec};

const BASE_TEMPLATE: &str = "base.html";
const BUILD_DIR: &str = "src/output/html/build";
const REPORT_HTML_PATH: &str = "src/output/html/build/report.html";

// Generates an HTML report from a given `Report` object.
pub fn generate_html(report: &Report, path: Option<PathBuf>) -> Result<String> {
    // Report context
    let report_context = create_context("report", report);

    // Analytics context
    let report_analytics = utils::get_analytics(report);
    let analytics_context = create_context("analytics", report_analytics);

    let html = render_template(BASE_TEMPLATE, vec![report_context, analytics_context])
        .with_context(|| format!("Failed to render template '{}'", BASE_TEMPLATE))?;

    let combined_html = combine_html(html, BUILD_DIR)?;

    let output_path = get_output_path(path);

    write_to_file(&output_path, combined_html.as_bytes())
        .with_context(|| format!("Failed to write HTML to '{}'", output_path.display()))?;

    Ok(output_path.to_string_lossy().into_owned())
}

fn get_output_path(path: Option<PathBuf>) -> PathBuf {
    path.map_or_else(
        || PathBuf::from(REPORT_HTML_PATH),
        |p| p.join("report.html"),
    )
}

fn combine_html(html: String, build_dir: &str) -> Result<String> {
    let mut combined_html = html;

    let build_files = fs::read_dir(build_dir)
        .with_context(|| format!("Failed to read build directory {}", build_dir))?;

    for entry in build_files {
        let path = entry?.path();
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("js") => {
                let mut content = String::new();
                let mut file = fs::File::open(&path)
                    .with_context(|| format!("Failed to open JS file {:?}", path))?;
                file.read_to_string(&mut content)
                    .with_context(|| format!("Failed to read JS file {:?}", path))?;
                combined_html.push_str(&format!("<script>{}</script>\n", content));
            }
            Some("css") => {
                let css_content = fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read CSS file {:?}", path))?;
                combined_html.push_str(&format!("<style>{}</style>\n", css_content));
            }
            _ => {}
        }
    }

    Ok(combined_html)
}
