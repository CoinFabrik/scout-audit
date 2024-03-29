use std::collections::HashMap;

use crate::output::report::{Category, Finding};

use super::utils;

// Generate the header for the report
pub fn generate_header(date: String) -> String {
    format!(
        "<!DOCTYPE html>\n<html>\n<head>\n\
        <title>Scout Report - {}</title>\n\
        <style>\n\
        body {{ font-family: 'Arial', sans-serif; line-height: 1.6; }}\n\
        img.banner {{ width: 100%; height: auto; }}\n\
        h1, h2, h3 {{ color: #333; }}\n\
        table {{ width: 100%; border-collapse: collapse; background-color: #f8f8f8; }}\n\
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}\n\
        th {{ background-color: #f0f0f0; color: #333; }}\n\
        td {{ word-wrap: break-word; }}\n\
        ul.summary {{ list-style: none; padding: 0; }}\n\
        ul.summary li a {{ text-decoration: none; color: #333; }}\n\
        </style>\n\
        </head>\n<body>\n\
        <h1>Scout Report - {}</h1>\n",
        date, date
    )
}

// Generate the summary for the report
pub fn generate_summary(categories: &[Category], findings: &[Finding]) -> String {
    let mut summary_markdown = String::from("<h2>Summary</h2>");
    let findings_summary = summarize_findings(categories, findings);

    for category in categories {
        if let Some((count, severity)) = findings_summary.get(&category.id) {
            summary_markdown.push_str(&format!(
                " - <a href=\"{}\">{}</a> ({} results) ({})<br>",
                utils::sanitize_category_name(&category.name),
                category.name,
                count,
                severity
            ));
        }
    }

    summary_markdown.push_str("<br>");
    summary_markdown
}

// This function summarizes the findings by category
fn summarize_findings(
    categories: &[Category],
    findings: &[Finding],
) -> HashMap<String, (usize, String)> {
    let mut summary = HashMap::new();

    for finding in findings {
        if let Some(category) = categories.iter().find(|c| c.id == finding.category_id) {
            let severity = category
                .vulnerabilities
                .first()
                .map(|v| utils::capitalize(&v.severity))
                .unwrap_or_default();
            let entry = summary.entry(category.id.clone()).or_insert((0, severity));
            entry.0 += 1;
        }
    }

    summary
}

// Generate the body for the report
pub fn generate_body(categories: &[Category], findings: &[Finding]) -> String {
    categories
        .iter()
        .map(|category| {
            let category_markdown = generate_category(category);
            let table = generate_table_for_category(category, findings);
            format!("{}{}", category_markdown, table)
        })
        .collect::<Vec<_>>()
        .join("<br>")
}

// Function to generate HTML for a category
fn generate_category(category: &Category) -> String {
    let mut category_markdown = format!("<h2>{}</h2><br>", category.name);
    for vulnerability in &category.vulnerabilities {
        category_markdown.push_str(&format!("<h3>{}</h3><br>", vulnerability.name));
        category_markdown.push_str(&format!(
            "<strong>Impact:</strong> <span style=\"font-weight: bold\">{}</span><br><br>",
            utils::capitalize(&vulnerability.severity)
        ));
        category_markdown.push_str(&format!(
            "<strong>Description:</strong> {}<br><br>",
            vulnerability.short_message
        ));
        category_markdown.push_str(&format!(
            "<strong>More about:</strong> <a href=\"{}\">here</a><br><br>",
            vulnerability.help
        ));
    }
    category_markdown
}

// Function to generate a table for a category
fn generate_table_for_category(category: &Category, findings: &[Finding]) -> String {
    let table_header = "<table style=\"width: 100%; table-layout: fixed;\"><thead><tr>\
                        <th style=\"width: 20%;\">ID</th>\
                        <th style=\"width: 60%;\">Detection</th>\
                        <th style=\"width: 20%;\">Status</th>\
                        </tr></thead><tbody>\n";
    let table_body: String = findings
        .iter()
        .filter(|finding| finding.category_id == category.id)
        .map(generate_finding)
        .collect();
    format!(
        "{}{}{}</tbody></table><br><br>",
        table_header, table_body, "</tbody></table><br><br>"
    )
}

// Function to generate Markdown for a finding
fn generate_finding(finding: &Finding) -> String {
    let status_options = "<ul><li>- [ ] False Positive </li>\
                          <li>- [ ] Acknowledged</li>\
                          <li>- [ ] Resolved</li></ul>";

    format!(
        "<tr><td>{}</td><td><a href=\"{}\">{}</a></td><td>{}</td></tr>\n",
        finding.id, "link-to-github", finding.span, status_options
    )
}
