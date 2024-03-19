use anyhow::Result;
use chrono::offset::Local;
use core::panic;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::HashMap, os::unix::process::CommandExt, path::PathBuf};

use super::{html, markdown, pdf, vulnerabilities::*};

#[derive(Serialize, Deserialize, Debug)]
pub struct Report {
    pub name: String,
    pub description: String,
    pub date: String,
    pub source_url: String,
    pub summary: Summary,
    pub categories: Vec<Category>,
    pub findings: Vec<Finding>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Summary {
    pub total_vulnerabilities: u32,
    pub by_severity: HashMap<String, u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub vulnerabilities: Vec<Vulnerability>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Vulnerability {
    pub id: String,
    pub name: String,
    pub short_message: String,
    pub long_message: String,
    pub severity: String,
    pub help: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Finding {
    pub id: u32,
    pub occurrence_index: u32,
    pub category_id: String,
    pub vulnerability_id: String,
    pub error_message: String,
    pub span: String,
    pub code_snippet: String,
    pub file: String,
}

impl Report {
    pub fn new(
        name: String,
        description: String,
        date: String,
        source_url: String,
        summary: Summary,
        categories: Vec<Category>,
        findings: Vec<Finding>,
    ) -> Self {
        Report {
            name,
            description,
            date,
            source_url,
            summary,
            categories,
            findings,
        }
    }

    pub fn generate_html(&self) -> Result<String> {
        html::generate_html(self)
    }

    pub fn generate_markdown(&self) -> Result<&'static str> {
        markdown::generate_markdown(self)
    }

    pub fn generate_pdf(&self, path: &PathBuf) -> Result<()> {
        let temp_html = pdf::generate_pdf(self)?;

        std::process::Command::new("wkhtmltopdf")
            .arg(temp_html)
            .arg(path.to_str().unwrap())
            .exec();

        std::fs::remove_file(temp_html)?;

        Ok(())
    }
}

pub struct RawVulnerability {
    pub id: &'static str,
    pub name: &'static str,
    pub short_message: &'static str,
    pub long_message: &'static str,
    pub severity: &'static str,
    pub help: &'static str,
    pub vulnerability_class: &'static str,
}

impl From<RawVulnerability> for Vulnerability {
    fn from(finding: RawVulnerability) -> Self {
        Vulnerability {
            id: finding.id.to_string(),
            name: finding.name.to_string(),
            short_message: finding.short_message.to_string(),
            long_message: finding.long_message.to_string(),
            severity: finding.severity.to_string(),
            help: finding.help.to_string(),
        }
    }
}

pub fn generate_report(scout_output: String) -> Report {
    let scout_findings = scout_output
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .filter(|finding: &Value| {
            finding
                .get("message")
                .and_then(|message| message.get("code"))
                .and_then(|code| code.get("code"))
                .and_then(|code| code.as_str())
                .filter(|code| DETECTORS.contains(code))
                .is_some()
        })
        .collect::<Vec<Value>>();

    let mut id: u32 = 0;
    let mut det_map: HashMap<_, _> = DETECTORS
        .iter()
        .map(|&detector| (detector.to_string(), 0))
        .collect();

    let mut findings: Vec<Finding> = Vec::new();

    for finding in &scout_findings {
        let category: String = finding
            .get("message")
            .and_then(|message| message.get("code"))
            .and_then(|code| code.get("code"))
            .unwrap()
            .to_string()
            .trim_matches('"')
            .to_string();

        let file = finding
            .get("target")
            .and_then(|target| target.get("src_path"))
            .unwrap()
            .to_string();

        let sp = finding
            .get("message")
            .and_then(|message| message.get("spans"))
            .unwrap();

        let span = if ["check_ink_version"].contains(&category.as_str()) {
            "Cargo.toml".to_string()
        } else {
            format!(
                "{}:{}:{} - {}:{}",
                sp[0]
                    .get("file_name")
                    .unwrap_or(&Value::default())
                    .to_string()
                    .trim_matches('"')
                    .to_string(),
                sp[0].get("line_start").unwrap_or(&Value::default()),
                sp[0].get("column_start").unwrap_or(&Value::default()),
                sp[0].get("line_end").unwrap_or(&Value::default()),
                sp[0].get("column_end").unwrap_or(&Value::default())
            )
        };

        //given a byte_start and byte_end, we can extract the code snippet from the file
        let byte_start = sp[0].get("byte_start").unwrap().as_u64().unwrap() as usize;
        let byte_end = sp[0].get("byte_end").unwrap().as_u64().unwrap() as usize;

        let code_snippet: String = std::fs::read_to_string(&(file.trim_matches('"'))).unwrap()
            [byte_start..byte_end]
            .to_string();

        let error_message = finding
            .get("message")
            .and_then(|message| message.get("message"))
            .unwrap()
            .to_string();

        let v = det_map.entry(category.clone()).or_insert(0);
        *v += 1;

        let fndg = Finding {
            id,
            occurrence_index: *v,
            category_id: get_raw_vuln_from_name(&category)
                .vulnerability_class
                .to_string(),
            vulnerability_id: category,
            error_message,
            span,
            code_snippet,
            file,
        };
        id += 1;
        findings.push(fndg);
    }

    let summary_map = det_map
        .into_iter()
        .filter(|(_, v)| *v != 0)
        .collect::<HashMap<String, u32>>();

    let mut categories: Vec<Category> = Vec::new();

    for (vuln, _) in &summary_map {
        let raw_vuln = get_raw_vuln_from_name(vuln);
        let id = raw_vuln.vulnerability_class.to_string();
        let vuln = Vulnerability::from(raw_vuln);

        if categories.iter().any(|cat| cat.id == id) {
            let cat = categories.iter_mut().find(|cat| cat.id == id).unwrap();
            if findings.iter().any(|f| f.vulnerability_id == vuln.id) {
                cat.vulnerabilities.push(vuln);
            }
            continue;
        } else {
            let cat = Category {
                id,
                name: vuln.name.clone(),
                vulnerabilities: vec![vuln],
            };
            categories.push(cat);
        }
    }

    let mut vulns_by_severity = vec![
        ("critical".to_string(), 0),
        ("medium".to_string(), 0),
        ("minor".to_string(), 0),
        ("enhancement".to_string(), 0),
    ];

    for (vuln, count) in &summary_map {
        let severity = get_raw_vuln_from_name(vuln).severity.to_string();
        let severity_count = vulns_by_severity
            .iter_mut()
            .find(|(s, _)| s.to_lowercase() == severity.to_lowercase())
            .unwrap();
        severity_count.1 += count;
    }

    let summary = Summary {
        total_vulnerabilities: findings.len() as u32,
        by_severity: vulns_by_severity.into_iter().collect(),
    };

    let date = format!(
        "{} {}",
        Local::now().naive_local().date(),
        Local::now().naive_local().time().format("%H:%M:%S")
    );

    Report::new(
        "name".into(),
        "description".into(),
        date,
        "source".into(),
        summary,
        categories,
        findings,
    )
}

fn get_raw_vuln_from_name(name: &str) -> RawVulnerability {
    match name {
        "assert_violation" => ASSERT_VIOLATION,
        "avoid_std_core_mem_forget" => AVOID_STD_CORE_MEM_FORGET,
        "avoid_format_string" => AVOID_FORMAT_STRING,
        "delegate_call" => DELEGATE_CALL,
        "divide_before_multiply" => DIVIDE_BEFORE_MULTIPLY,
        "dos_unbounded_operation" => DOS_UNBOUNDED_OPERATION,
        "unexpected_revert_warn" => UNEXPECTED_REVERT_WARN,
        "check_ink_version" => CHECK_INK_VERSION,
        "insufficiently_random_values" => INSUFFICIENTLY_RANDOM_VALUES,
        "integer_overflow_underflow" => INTEGER_OVERFLOW_UNDERFLOW,
        "iterator_over_indexing" => ITERATOR_OVER_INDEXING,
        "lazy_delegate" => LAZY_DELEGATE,
        "panic_error" => PANIC_ERROR,
        "reentrancy_1" => REENTRANCY,
        "reentrancy_2" => REENTRANCY,
        "unprotected_set_code_hash" => UNPROTECTED_SET_CODE_HASH,
        "set_storage_warn" => SET_STORAGE_WARN,
        "unprotected_mapping_operation" => UNPROTECTED_MAPPING_OPERATION,
        "unprotected_self_destruct" => UNPROTECTED_SELF_DESTRUCT,
        "unrestricted_transfer_from" => UNRESTRICTED_TRANSFER_FROM,
        "unsafe_expect" => UNSAFE_EXPECT,
        "unsafe_unwrap" => UNSAFE_UNWRAP,
        "unused_return_enum" => UNUSED_RETURN_ENUM,
        "zero_or_test_address" => ZERO_OR_TEST_ADDRESS,
        _ => panic!("Unknown vulnerability name: {}", name),
    }
}
