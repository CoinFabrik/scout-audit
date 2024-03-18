use std::collections::HashMap;
use std::fs;

use super::{
    report::{Category, Finding, Report, Severity, Summary, Vulnerability},
    vulnerabilities::*,
};

use anyhow::{Context, Result};
use chrono::Local;
use serde_json::Value;

pub fn generate_report(scout_output: String) -> Result<Report> {
    let config = Config::load().context("Failed to load the configuration")?;
    let scout_findings = parse_scout_output(&scout_output, &config.detectors)?;

    let mut findings: Vec<Finding> = Vec::new();
    let mut det_map: HashMap<String, u32> = HashMap::new();

    for finding in scout_findings {
        process_finding(
            &finding,
            &mut findings,
            &mut det_map,
            &config.vulnerabilities,
        )?;
    }

    let summary = create_summary(&det_map, &config.vulnerabilities);
    let categories = categorize_findings(&findings, &config.vulnerabilities);

    let date = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    Ok(Report::new(
        "name".into(),
        "description".into(),
        date,
        "source".into(),
        summary,
        categories,
        findings,
    ))
}

fn parse_scout_output(scout_output: &str, detectors: &[String]) -> Result<Vec<Value>> {
    scout_output
        .lines()
        .map(|line| serde_json::from_str::<Value>(line).context("Failed to parse line into JSON"))
        .filter(|result| {
            if let Ok(finding) = result {
                if let Some(code) = finding
                    .get("message")
                    .and_then(|m| m.get("code"))
                    .and_then(|c| c.get("code"))
                    .and_then(|c| c.as_str())
                {
                    return detectors.contains(&code.to_string());
                }
            }
            false
        })
        .collect()
}

fn process_finding(
    finding: &Value,
    findings: &mut Vec<Finding>,
    det_map: &mut HashMap<String, u32>,
    vulnerabilities: &[RawVulnerability],
) -> Result<()> {
    // Assuming 'finding' is a type that supports nested key access, similar to a JSON object.
    let category = finding
        .get("message")
        .and_then(|m| m.get("code"))
        .and_then(|c| c.get("code"))
        .and_then(|c| c.as_str())
        .ok_or_else(|| anyhow::Error::msg("Category not found in finding"))
        .map(|s| s.trim_matches('"').to_string())?;

    let file = finding
        .get("target")
        .and_then(|t| t.get("src_path"))
        .and_then(|s| s.as_str())
        .ok_or_else(|| anyhow::Error::msg("File path not found in finding"))
        .map(|f| f.to_string())?;

    let span = extract_span(finding)?;
    let code_snippet = extract_code_snippet(&file, finding)?;
    let error_message = extract_error_message(finding)?;

    let occurrence_index = det_map.entry(category.clone()).or_insert(0);
    *occurrence_index += 1;

    let fndg = Finding {
        id: findings.len() as u32,
        occurrence_index: *occurrence_index,
        category_id: vulnerabilities
            .iter()
            .find(|&v| v.id == category)
            .ok_or_else(|| anyhow::Error::msg("Vulnerability not found for the category"))?
            .vulnerability_class
            .clone(),
        vulnerability_id: category,
        error_message,
        span,
        code_snippet,
        file,
    };

    findings.push(fndg);

    Ok(())
}

fn extract_span(finding: &Value) -> Result<String> {
    // TODO: this should be improved to handle no-span detectors
    finding
        .get("message")
        .and_then(|m| m.get("spans"))
        .and_then(|s| s.get(0))
        .map(|span| {
            format!(
                "{}:{}:{} - {}:{}",
                span.get("file_name")
                    .unwrap_or(&Value::default())
                    .to_string()
                    .trim_matches('"'),
                span.get("line_start").unwrap_or(&Value::default()),
                span.get("column_start").unwrap_or(&Value::default()),
                span.get("line_end").unwrap_or(&Value::default()),
                span.get("column_end").unwrap_or(&Value::default())
            )
        })
        .ok_or_else(|| anyhow::Error::msg("Failed to extract span"))
}

fn extract_code_snippet(file: &str, finding: &Value) -> Result<String> {
    let content = fs::read_to_string(file.trim_matches('"'))
        .context("Failed to read source file for code snippet")?;
    let byte_start = finding
        .get("message")
        .and_then(|m| m.get("spans"))
        .and_then(|s| s.get(0))
        .and_then(|span| span.get("byte_start").and_then(Value::as_u64))
        .context("Failed to get byte_start for code snippet")? as usize;
    let byte_end = finding
        .get("message")
        .and_then(|m| m.get("spans"))
        .and_then(|s| s.get(0))
        .and_then(|span| span.get("byte_end").and_then(Value::as_u64))
        .context("Failed to get byte_end for code snippet")? as usize;

    Ok(content[byte_start..byte_end].to_string())
}

fn extract_error_message(finding: &Value) -> Result<String> {
    finding
        .get("message")
        .and_then(|m| m.get("message"))
        .map(|v| v.to_string())
        .ok_or_else(|| anyhow::Error::msg("Failed to extract error message"))
}

fn create_summary(det_map: &HashMap<String, u32>, vulnerabilities: &[RawVulnerability]) -> Summary {
    let total_vulnerabilities: u32 = det_map.values().sum();
    let mut by_severity: HashMap<Severity, u32> = HashMap::new();

    for vuln in vulnerabilities {
        *by_severity.entry(vuln.severity.clone()).or_insert(0) +=
            det_map.get(&vuln.id).cloned().unwrap_or_default();
    }

    Summary {
        total_vulnerabilities,
        by_severity,
    }
}

fn categorize_findings(
    findings: &[Finding],
    vulnerabilities: &[RawVulnerability],
) -> Vec<Category> {
    let mut categories = HashMap::new();
    for finding in findings {
        if let Some(vuln) = vulnerabilities
            .iter()
            .find(|&v| v.id == finding.vulnerability_id)
        {
            let category = categories
                .entry(vuln.vulnerability_class.clone())
                .or_insert_with(|| Category {
                    id: vuln.vulnerability_class.clone(),
                    name: vuln.name.clone(),
                    vulnerabilities: vec![],
                });

            if category
                .vulnerabilities
                .iter()
                .all(|v: &Vulnerability| v.id != vuln.id)
            {
                category.vulnerabilities.push(Vulnerability {
                    id: vuln.id.clone(),
                    name: vuln.name.clone(),
                    short_message: vuln.short_message.clone(),
                    long_message: vuln.long_message.clone(),
                    severity: vuln.severity.clone(),
                    help: vuln.help.clone(),
                });
            }
        }
    }

    categories.into_values().collect()
}
