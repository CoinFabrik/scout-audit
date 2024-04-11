use anyhow::Result;
use chrono::offset::Local;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::path::Path;
use std::{collections::HashMap, os::unix::process::CommandExt};

use super::{html, markdown, pdf};

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

    pub fn generate_pdf(&self, path: &Path) -> Result<()> {
        let temp_html = pdf::generate_pdf(self)?;

        std::process::Command::new("wkhtmltopdf")
            .arg(temp_html)
            .arg(path.to_str().unwrap())
            .exec();

        std::fs::remove_file(temp_html)?;

        Ok(())
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

use crate::startup::ProjectInfo;
use crate::utils::detectors_info::LintInfo;

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

    for (id, finding) in scout_findings.iter().enumerate() {
        let category: String = finding
            .get("message")
            .and_then(|message| message.get("code"))
            .and_then(|code| code.get("code"))
            .unwrap()
            .to_string()
            .trim_matches('"')
            .to_string();

        let sp = finding
            .get("message")
            .and_then(|message| message.get("spans"))
            .unwrap();
        let file = sp[0]
            .get("file_name")
            .unwrap_or(&Value::default())
            .to_string()
            .replace("\"", "");
        let file_path = info.workspace_root.join(&file);

        let span = if ["check_ink_version"].contains(&category.as_str()) {
            "Cargo.toml".to_string()
        } else {
            format!(
                "{}:{}:{} - {}:{}",
                sp[0]
                    .get("file_name")
                    .unwrap_or(&Value::default())
                    .to_string()
                    .trim_matches('"'),
                sp[0].get("line_start").unwrap_or(&Value::default()),
                sp[0].get("column_start").unwrap_or(&Value::default()),
                sp[0].get("line_end").unwrap_or(&Value::default()),
                sp[0].get("column_end").unwrap_or(&Value::default())
            )
        };

        //given a byte_start and byte_end, we can extract the code snippet from the file
        let byte_start = sp[0].get("byte_start").unwrap().as_u64().unwrap() as usize;
        let byte_end = sp[0].get("byte_end").unwrap().as_u64().unwrap() as usize;

        let code_snippet: String =
            std::fs::read_to_string(&file_path).unwrap()[byte_start..byte_end].to_string();

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
            id: id as u32,
            occurrence_index: *v,
            category_id: detector_info
                .get(&category)
                .map_or("Local detectors".to_owned(), |f| {
                    f.vulnerability_class.clone()
                }),
            vulnerability_id: category,
            error_message,
            span,
            code_snippet,
            file,
        };
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
                name: vuln_id.to_string(),
                short_message: "".to_owned(),
                long_message: "".to_owned(),
                severity: "unknown".to_owned(),
                help: "".to_owned(),
            },
        };
        let id = detector_info
            .get::<String>(vuln_id)
            .map_or("Local detectors".to_owned(), |f| {
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

    let mut by_severity: HashMap<String, u32> = [
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
