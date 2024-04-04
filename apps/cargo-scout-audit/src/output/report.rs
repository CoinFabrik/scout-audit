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

impl From<&LintInfo> for Vulnerability {
    fn from(lint_info: &LintInfo) -> Self {
        Vulnerability {
            id: lint_info.id.clone(),
            name: lint_info.name.clone(),
            short_message: lint_info.short_message.clone(),
            long_message: lint_info.long_message.clone(),
            severity: lint_info.severity.clone(),
            help: lint_info.help.clone(),
        }
    }
}

use crate::startup::LintInfo;
use crate::startup::ProjectInfo;

pub fn generate_report(
    scout_output: String,
    info: ProjectInfo,
    detector_info: HashMap<String, LintInfo>,
) -> Report {
    let scout_findings = scout_output
        .lines()
        .map(|line| serde_json::from_str::<serde_json::Value>(line).unwrap())
        .filter(|finding: &Value| {
            finding
                .get("message")
                .and_then(|message| message.get("code"))
                .and_then(|code| code.get("code"))
                .is_some()
        })
        .collect::<Vec<Value>>();

    let mut id: u32 = 0;
    let mut det_map: HashMap<String, u32> = HashMap::new();

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
            .to_string()
            .trim_matches('"')
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
            .to_string()
            .trim_matches('"')
            .to_string();

        let v = det_map.entry(category.clone()).or_insert(0);
        *v += 1;

        let fndg = Finding {
            id,
            occurrence_index: *v,
            category_id: detector_info
                .get(&category)
                .map_or("Local detector".to_owned(), |f| {
                    f.vulnerability_class.clone()
                }),
            vulnerability_id: category,
            error_message,
            span,
            code_snippet,
            file: file
                .trim_start_matches(info.worspace_root.as_os_str().to_str().unwrap())
                .trim_start_matches('/')
                .to_string(),
        };
        id += 1;
        findings.push(fndg);
    }

    let summary_map = det_map
        .into_iter()
        .filter(|(_, v)| *v != 0)
        .collect::<HashMap<String, u32>>();

    let mut categories: Vec<Category> = Vec::new();

    for (vuln_id, _) in &summary_map {
        let info = detector_info.get::<String>(vuln_id);
        let vuln = match info {
            Some(lint_info) => Vulnerability::from(lint_info),
            None => Vulnerability {
                id: vuln_id.to_string(),
                name: "Local detector:".to_owned() + vuln_id,
                short_message: "".to_owned(),
                long_message: "".to_owned(),
                severity: "unknown".to_owned(),
                help: "".to_owned(),
            },
        };
        let id = detector_info
            .get::<String>(vuln_id)
            .map_or("Local detector".to_owned(), |f| {
                f.vulnerability_class.clone()
            });

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

    let mut by_severity: HashMap<String, u32> = vec![
        ("critical".to_string(), 0),
        ("medium".to_string(), 0),
        ("minor".to_string(), 0),
        ("enhancement".to_string(), 0),
        ("unknown".to_string(), 0),
    ]
    .iter()
    .cloned()
    .collect();

    for (vuln, count) in &summary_map {
        let severity = detector_info
            .get(vuln)
            .map_or("unknown".to_owned(), |f| f.severity.clone());
        let severity_count = by_severity.get_mut(&severity.to_lowercase()).unwrap();
        *severity_count += count;
    }

    let summary = Summary {
        total_vulnerabilities: findings.len() as u32,
        by_severity,
    };

    let date = format!(
        "{} {}",
        Local::now().naive_local().date(),
        Local::now().naive_local().time().format("%H:%M:%S")
    );

    Report::new(
        info.name,
        info.description,
        date,
        info.hash,
        summary,
        categories,
        findings,
    )
}

use crate::startup::BlockChain;

trait GetRawVulnerabilities {
    fn get_raw_vuln_from_name(&self, name: &str) -> RawVulnerability;
    fn get_array_of_vulnerability_names(&self) -> Vec<&'static str>;
}

impl GetRawVulnerabilities for BlockChain {
    fn get_raw_vuln_from_name(&self, name: &str) -> RawVulnerability {
        match &self {
            BlockChain::Ink => match name {
                "assert_violation" => INK_ASSERT_VIOLATION,
                "avoid_std_core_mem_forget" => INK_AVOID_STD_CORE_MEM_FORGET,
                "avoid_format_string" => INK_AVOID_FORMAT_STRING,
                "delegate_call" => INK_DELEGATE_CALL,
                "divide_before_multiply" => INK_DIVIDE_BEFORE_MULTIPLY,
                "dos_unbounded_operation" => INK_DOS_UNBOUNDED_OPERATION,
                "unexpected_revert_warn" => INK_UNEXPECTED_REVERT_WARN,
                "check_ink_version" => INK_CHECK_INK_VERSION,
                "insufficiently_random_values" => INK_INSUFFICIENTLY_RANDOM_VALUES,
                "integer_overflow_underflow" => INK_INTEGER_OVERFLOW_UNDERFLOW,
                "iterator_over_indexing" => INK_ITERATOR_OVER_INDEXING,
                "lazy_delegate" => INK_LAZY_DELEGATE,
                "panic_error" => INK_PANIC_ERROR,
                "reentrancy_1" => INK_REENTRANCY,
                "reentrancy_2" => INK_REENTRANCY,
                "unprotected_set_code_hash" => INK_UNPROTECTED_SET_CODE_HASH,
                "set_storage_warn" => INK_SET_STORAGE_WARN,
                "unprotected_mapping_operation" => INK_UNPROTECTED_MAPPING_OPERATION,
                "unprotected_self_destruct" => INK_UNPROTECTED_SELF_DESTRUCT,
                "unrestricted_transfer_from" => INK_UNRESTRICTED_TRANSFER_FROM,
                "unsafe_expect" => INK_UNSAFE_EXPECT,
                "unsafe_unwrap" => INK_UNSAFE_UNWRAP,
                "unused_return_enum" => INK_UNUSED_RETURN_ENUM,
                "zero_or_test_address" => INK_ZERO_OR_TEST_ADDRESS,
                _ => panic!("Unknown vulnerability name: {}", name),
            },
            BlockChain::Soroban => match name {
                "avoid_core_mem_forget" => SOROBAN_AVOID_CORE_MEM_FORGET,
                "avoid_panic_error" => SOROBAN_AVOID_PANIC_ERROR,
                "avoid_unsafe_block" => SOROBAN_AVOID_UNSAFE_BLOCK,
                "divide_before_multiply" => SOROBAN_DIVIDE_BEFORE_MULTIPLY,
                "dos_unbounded_operation" => SOROBAN_DOS_UNBOUNDED_OPERATION,
                "insufficiently_random_values" => SOROBAN_INSUFFICIENTLY_RANDOM_VALUES,
                "overflow_check" => SOROBAN_OVERFLOW_CHECK,
                "set_contract_storage" => SOROBAN_SET_CONTRACT_STORAGE,
                "soroban_version" => SOROBAN_SOROBAN_VERSION,
                "unprotected_update_current_contract_wasm" => {
                    SOROBAN_UNPROTECTED_UPDATE_CURRENT_CONTRACT_WASM
                }
                "unsafe_expect" => SOROBAN_UNSAFE_EXPECT,
                "unsafe_unwrap" => SOROBAN_UNSAFE_UNWRAP,
                "unused_return_enum" => SOROBAN_UNUSED_RETURN_ENUM,
                _ => panic!("Unknown vulnerability name: {}", name),
            },
        }
    }
    fn get_array_of_vulnerability_names(&self) -> std::vec::Vec<&'static str> {
        match &self {
            BlockChain::Ink => INK_DETECTORS.to_vec(),
            BlockChain::Soroban => SOROBAN_DETECTORS.to_vec(),
        }
    }
}
