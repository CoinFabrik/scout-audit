use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use super::report::{Category, Finding, Report, Severity, Summary, Vulnerability};
use crate::{scout::project_info::ProjectInfo, utils::detectors_info::LintInfo};

pub struct RawReport;

struct FileDetails {
    relative_path: String,
    absolute_path: PathBuf,
    package: String,
}

impl RawReport {
    #[tracing::instrument(name = "GENERATE FROM RAW REPORT", level = "debug", skip_all, fields(project = %info.name))]
    pub fn generate_report(
        scout_output: &str,
        info: &ProjectInfo,
        detector_info: &HashMap<String, LintInfo>,
    ) -> Result<Report> {
        let scout_findings = parse_scout_findings(scout_output).with_context(|| {
            format!("Failed to parse scout findings for project '{}'", info.name)
        })?;
        let (findings, det_map) = process_findings(&scout_findings, info, detector_info)
            .with_context(|| format!("Failed to process findings for project '{}'", info.name))?;
        let categories = generate_categories(&det_map, detector_info)
            .with_context(|| "Failed to generate vulnerability categories")?;
        let summary = create_summary(detector_info, info, &findings);
        Ok(Report::new(
            info.name.clone(),
            info.date.clone(),
            summary,
            categories,
            findings,
        ))
    }
}

fn parse_scout_findings(scout_output: &str) -> Result<Vec<Value>> {
    let mut results = Vec::new();
    for (line_number, line) in scout_output.lines().enumerate() {
        let value = serde_json::from_str::<Value>(line).with_context(|| {
            format!(
                "Failed to parse JSON from line {}: '{}'",
                line_number + 1,
                line
            )
        })?;
        let has_code = value
            .get("message")
            .and_then(|message| message.get("code"))
            .and_then(|code| code.get("code"))
            .is_some();
        if has_code {
            results.push(value);
        }
    }
    Ok(results)
}

fn process_findings(
    scout_findings: &[Value],
    info: &ProjectInfo,
    detector_info: &HashMap<String, LintInfo>,
) -> Result<(Vec<Finding>, HashMap<String, u32>)> {
    let mut det_map: HashMap<String, u32> = HashMap::new();
    let mut findings: Vec<Finding> = Vec::new();

    for (id, finding) in scout_findings.iter().enumerate() {
        let category = parse_category(finding).with_context(|| {
            format!("Failed to parse vulnerability category for finding {}", id)
        })?;
        if !detector_info.contains_key(&category) {
            continue;
        }

        let FileDetails {
            relative_path,
            absolute_path,
            package,
        } = parse_file_details(finding, &info.workspace_root)
            .with_context(|| format!("Failed to parse file details for finding {}", id))?;

        let file_name = absolute_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_string();

        let span = parse_span(finding, &file_name);
        let code_snippet = extract_code_snippet(&absolute_path, finding).with_context(|| {
            format!(
                "Failed to extract code snippet for finding {} in file '{}'",
                id, relative_path
            )
        })?;

        let error_message = parse_error_message(finding);

        let occurrence_index = det_map.entry(category.clone()).or_insert(0);
        *occurrence_index += 1;

        findings.push(Finding {
            id: id as u32,
            occurrence_index: *occurrence_index,
            category_id: detector_info[&category].vulnerability_class.clone(),
            vulnerability_id: category,
            error_message,
            span,
            code_snippet,
            package,
            file_path: relative_path,
        });
    }

    Ok((findings, det_map))
}

fn parse_category(finding: &Value) -> Result<String> {
    let category = finding
        .get("message")
        .and_then(|message| message.get("code"))
        .and_then(|code| code.get("code"))
        .and_then(Value::as_str)
        .with_context(|| "Category not found in finding structure")?
        .trim_matches('"')
        .to_string();
    Ok(category)
}

fn parse_file_details(finding: &Value, workspace_root: &Path) -> Result<FileDetails> {
    let relative_path = finding
        .get("message")
        .and_then(|message| message.get("spans"))
        .and_then(|spans| spans.get(0))
        .and_then(|span| span.get("file_name"))
        .and_then(Value::as_str)
        .with_context(|| "File name not found in finding structure")?
        .trim_matches('"')
        .to_string();

    let absolute_path = workspace_root.join(&relative_path);

    let path = Path::new(&relative_path);
    let package = path
        .components()
        .next()
        .and_then(|comp| comp.as_os_str().to_str())
        .unwrap_or("")
        .to_string();

    Ok(FileDetails {
        relative_path,
        absolute_path,
        package,
    })
}

fn parse_span(finding: &Value, file_name: &str) -> String {
    finding
        .get("message")
        .and_then(|message| message.get("spans"))
        .map(|spans| {
            format!(
                "{}:{}:{} - {}:{}",
                file_name,
                spans[0].get("line_start").unwrap_or(&Value::default()),
                spans[0].get("column_start").unwrap_or(&Value::default()),
                spans[0].get("line_end").unwrap_or(&Value::default()),
                spans[0].get("column_end").unwrap_or(&Value::default())
            )
        })
        .unwrap_or_else(|| "Span information not available".to_string())
}

fn extract_code_snippet(file_path: &Path, finding: &Value) -> Result<String> {
    let sp = finding
        .get("message")
        .and_then(|message| message.get("spans"))
        .and_then(|spans| spans.get(0))
        .with_context(|| "Span information not found in finding structure")?;

    let byte_start = sp
        .get("byte_start")
        .and_then(Value::as_u64)
        .with_context(|| "Byte start information missing in span")? as usize;
    let byte_end = sp
        .get("byte_end")
        .and_then(Value::as_u64)
        .with_context(|| "Byte end information missing in span")? as usize;

    let file_content = std::fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file content from '{}'", file_path.display()))?;

    Ok(file_content[byte_start..byte_end].to_string())
}

fn parse_error_message(finding: &Value) -> String {
    finding
        .get("message")
        .and_then(|message| message.get("message"))
        .and_then(Value::as_str)
        .unwrap_or("Error message not available")
        .to_string()
}

fn generate_categories(
    det_map: &HashMap<String, u32>,
    detector_info: &HashMap<String, LintInfo>,
) -> Result<Vec<Category>> {
    let mut categories: Vec<Category> = Vec::new();
    for (id, count) in det_map {
        if *count == 0 {
            continue;
        }
        let vuln_info = detector_info
            .get(id)
            .with_context(|| format!("Vulnerability info not found for detector ID: {}", id))?;
        let category = Category {
            id: vuln_info.vulnerability_class.clone(),
            name: vuln_info.name.clone(),
            vulnerabilities: vec![Vulnerability::from(vuln_info)],
        };
        categories.push(category);
    }
    Ok(categories)
}

#[tracing::instrument(name = "CREATE SUMMARY", level = "trace", skip_all)]
fn create_summary(
    detector_info: &HashMap<String, LintInfo>,
    info: &ProjectInfo,
    findings: &[Finding],
) -> Summary {
    let total_vulnerabilities = findings.len() as u32;

    let mut by_severity: HashMap<Severity, u32> = [
        (Severity::Critical, 0),
        (Severity::Medium, 0),
        (Severity::Minor, 0),
        (Severity::Enhancement, 0),
    ]
    .iter()
    .cloned()
    .collect();

    for finding in findings {
        if let Some(lint_info) = detector_info.get(&finding.vulnerability_id) {
            match lint_info.severity.as_ref() {
                "Critical" => *by_severity.get_mut(&Severity::Critical).unwrap() += 1,
                "Medium" => *by_severity.get_mut(&Severity::Medium).unwrap() += 1,
                "Minor" => *by_severity.get_mut(&Severity::Minor).unwrap() += 1,
                "Enhancement" => *by_severity.get_mut(&Severity::Enhancement).unwrap() += 1,
                _ => continue,
            };
        }
    }

    Summary {
        executed_on: info.packages.clone(),
        total_vulnerabilities,
        by_severity,
    }
}
