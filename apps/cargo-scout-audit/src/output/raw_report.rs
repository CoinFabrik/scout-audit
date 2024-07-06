use anyhow::{Context, Result};
use serde_json::Value;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::{collections::HashMap, path::Path};

use super::report::{Category, Finding, Report, Severity, Summary, Vulnerability};
use crate::{startup::ProjectInfo, utils::detectors_info::LintInfo};

pub struct RawReport;

impl RawReport {
    #[tracing::instrument(name = "GENERATE FROM RAW REPORT", level = "debug", skip_all, fields(project = %info.name))]
    pub fn generate_report(
        scout_output: &str,
        info: &ProjectInfo,
        detector_info: &HashMap<String, LintInfo>,
    ) -> Result<Report> {
        let scout_findings =
            parse_scout_findings(scout_output).context("Failed to parse scout findings")?;
        let findings = process_findings(&scout_findings, info, detector_info)
            .context("Failed to process findings")?;
        let categories = generate_categories(detector_info, &findings)
            .context("Failed to generate categories")?;
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
    for line in scout_output.lines() {
        let value = serde_json::from_str::<Value>(line)
            .with_context(|| format!("Failed to parse JSON from line: {}", line))?;
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
) -> Result<Vec<Finding>> {
    let mut det_map: HashMap<String, u32> = HashMap::new();
    let mut findings: Vec<Finding> = Vec::new();

    for (id, finding) in scout_findings.iter().enumerate() {
        let category = parse_category(finding)
            .with_context(|| format!("Failed to parse category for finding id {}", id))?;
        if !detector_info.contains_key(&category) {
            continue;
        }

        let (file, file_path, package, file_name) =
            parse_file_details(finding, &info.workspace_root)
                .with_context(|| format!("Failed to parse file details for finding id {}", id))?;
        let span = parse_span(finding, &file_name);
        let code_snippet = extract_code_snippet(&file_path, finding)
            .with_context(|| format!("Failed to extract code snippet for finding id {}", id))?;

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
            file,
        });
    }

    Ok(findings)
}

fn parse_category(finding: &Value) -> Result<String> {
    let category = finding
        .get("message")
        .and_then(|message| message.get("code"))
        .and_then(|code| code.get("code"))
        .and_then(Value::as_str)
        .context("Category not found in finding")?
        .trim_matches('"')
        .to_string();
    Ok(category)
}

fn parse_file_details(
    finding: &Value,
    workspace_root: &Path,
) -> Result<(String, PathBuf, String, String)> {
    let file = finding
        .get("message")
        .and_then(|message| message.get("spans"))
        .and_then(|spans| spans.get(0))
        .and_then(|span| span.get("file_name"))
        .and_then(Value::as_str)
        .context("File name not found in finding")?
        .to_string()
        .replace('"', "");

    let file_path = workspace_root.join(&file);
    let (package, file_name) = file.split_once('/').unwrap_or(("", &file));
    Ok((
        file.clone(),
        file_path,
        package.to_string(),
        file_name.to_string(),
    ))
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
        .unwrap_or_else(|| "Span not valid".to_string())
}

fn extract_code_snippet(file_path: &PathBuf, finding: &Value) -> Result<String> {
    let sp = finding
        .get("message")
        .and_then(|message| message.get("spans"))
        .and_then(|spans| spans.get(0))
        .context("Span not found in finding")?;

    let byte_start = sp
        .get("byte_start")
        .and_then(Value::as_u64)
        .context("Byte start is missing in spans")?;
    let byte_end = sp
        .get("byte_end")
        .and_then(Value::as_u64)
        .context("Byte end is missing in spans")?;

    let file = std::fs::File::open(file_path)
        .with_context(|| format!("Failed to open file: {}", file_path.display()))?;
    let mut reader = BufReader::new(file);

    reader.seek(SeekFrom::Start(byte_start))?;
    let mut buffer = vec![0; (byte_end - byte_start) as usize];
    reader.read_exact(&mut buffer)?;

    String::from_utf8(buffer).with_context(|| "Failed to convert extracted bytes to UTF-8 string")
}

fn parse_error_message(finding: &Value) -> String {
    finding
        .get("message")
        .and_then(|message| message.get("message"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn generate_categories(
    detector_info: &HashMap<String, LintInfo>,
    findings: &[Finding],
) -> Result<Vec<Category>> {
    let mut categories: HashMap<String, Category> = HashMap::new();

    for finding in findings {
        if let Some(vuln_info) = detector_info.get(&finding.vulnerability_id) {
            let category = categories
                .entry(vuln_info.vulnerability_class.clone())
                .or_insert_with(|| Category {
                    id: vuln_info.vulnerability_class.clone(),
                    name: vuln_info.name.clone(),
                    vulnerabilities: Vec::new(),
                });

            if !category
                .vulnerabilities
                .iter()
                .any(|v| v.id == finding.vulnerability_id)
            {
                category
                    .vulnerabilities
                    .push(Vulnerability::from(vuln_info));
            }
        }
    }

    Ok(categories.into_values().collect())
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
