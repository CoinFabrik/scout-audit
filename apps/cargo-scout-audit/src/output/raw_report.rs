use super::report::{Category, Finding, Report, Severity, Summary, Vulnerability};
use crate::scout::project_info::ProjectInfo;
use crate::utils::detectors_info::LintStore;
use anyhow::{Context, Result};
use serde_json::Value;
use std::{
    collections::HashMap,
    io::{BufReader, Read, Seek, SeekFrom},
    path::{Path, PathBuf},
};

pub struct RawReport;

struct FileDetails {
    relative_path: String,
    absolute_path: PathBuf,
    package: String,
}

impl RawReport {
    #[tracing::instrument(name = "GENERATE FROM RAW REPORT", level = "debug", skip_all, fields(project = %info.name))]
    pub fn generate_report(
        json_findings: &[Value],
        crates: &HashMap<String, bool>,
        info: &ProjectInfo,
        detector_info: &LintStore,
    ) -> Result<Report> {
        let scout_findings = json_findings;
        let findings = process_findings(scout_findings, info, detector_info)
            .context("Failed to process findings")?;
        let categories = generate_categories(detector_info, &findings)
            .context("Failed to generate categories")?;
        let summary = create_summary(detector_info, info, &findings, json_findings, crates);
        Ok(Report::new(
            info.name.clone(),
            info.date.clone(),
            summary,
            categories,
            findings,
        ))
    }
}

pub(crate) fn json_to_string(s: &Value) -> String {
    if let Value::String(s) = s {
        s.clone()
    } else {
        s.to_string().trim_matches('"').to_string()
    }
}

pub(crate) fn json_to_string_opt(s: Option<&Value>) -> Option<String> {
    s.map(|s| {
        if let Value::String(s) = s {
            s.clone()
        } else {
            s.to_string().trim_matches('"').to_string()
        }
    })
}

fn process_findings(
    scout_findings: &[Value],
    info: &ProjectInfo,
    detector_info: &LintStore,
) -> Result<Vec<Finding>> {
    let mut det_map: HashMap<String, u32> = HashMap::new();
    let mut findings: Vec<Finding> = Vec::new();

    for (id, finding) in scout_findings.iter().enumerate() {
        let category = parse_category(finding).with_context(|| {
            format!("Failed to parse vulnerability category for finding {}", id)
        })?;
        if detector_info.find_by_id(&category).is_none() {
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
            category_id: detector_info
                .find_by_id(&category)
                .unwrap()
                .vulnerability_class
                .clone(),
            vulnerability_id: category,
            error_message,
            span,
            code_snippet,
            package,
            file_path: relative_path,
        });
    }

    Ok(findings)
}

fn parse_category(finding: &Value) -> Result<String> {
    let category = json_to_string(
        finding
            .get("code")
            .and_then(|code| code.get("code"))
            .with_context(|| "Category not found in finding structure")?,
    );
    Ok(category)
}

fn parse_file_details(finding: &Value, workspace_root: &Path) -> Result<FileDetails> {
    let relative_path = json_to_string(
        finding
            .get("spans")
            .and_then(|spans| spans.get(0))
            .and_then(|span| span.get("file_name"))
            .with_context(|| "File name not found in finding structure")?,
    );

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
        .get("spans")
        .map(|spans| match spans {
            Value::Array(spans) => {
                if spans.is_empty() {
                    None
                } else {
                    let span = &spans[0];
                    Some(format!(
                        "{}:{}:{} - {}:{}",
                        file_name,
                        span.get("line_start").unwrap_or(&Value::default()),
                        span.get("column_start").unwrap_or(&Value::default()),
                        span.get("line_end").unwrap_or(&Value::default()),
                        span.get("column_end").unwrap_or(&Value::default()),
                    ))
                }
            }
            _ => None,
        })
        .unwrap_or_else(|| None)
        .unwrap_or_else(|| "Span information not available".to_string())
}

fn extract_code_snippet(file_path: &Path, finding: &Value) -> Result<String> {
    let sp = finding
        .get("spans")
        .and_then(|spans| spans.get(0))
        .with_context(|| "Span information not found in finding structure")?;

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
        .and_then(Value::as_str)
        .unwrap_or("Error message not available")
        .to_string()
}

fn generate_categories(detector_info: &LintStore, findings: &[Finding]) -> Result<Vec<Category>> {
    let mut categories: HashMap<String, Category> = HashMap::new();

    for finding in findings {
        if let Some(vuln_info) = detector_info.find_by_id(&finding.vulnerability_id) {
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

fn create_summary(
    detector_info: &LintStore,
    info: &ProjectInfo,
    findings: &[Finding],
    json_findings: &[Value],
    crates: &HashMap<String, bool>,
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
        if let Some(lint_info) = detector_info.find_by_id(&finding.vulnerability_id) {
            match lint_info.severity.as_ref() {
                "Critical" => *by_severity.get_mut(&Severity::Critical).unwrap() += 1,
                "Medium" => *by_severity.get_mut(&Severity::Medium).unwrap() += 1,
                "Minor" => *by_severity.get_mut(&Severity::Minor).unwrap() += 1,
                "Enhancement" => *by_severity.get_mut(&Severity::Enhancement).unwrap() += 1,
                _ => continue,
            };
        }
    }

    let table = crate::output::table::construct_table(json_findings, crates, detector_info);

    Summary {
        executed_on: info.packages.clone(),
        total_vulnerabilities,
        by_severity,
        table,
    }
}
