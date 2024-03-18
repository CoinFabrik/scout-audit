use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::PathBuf};
use strum_macros::{Display, EnumString};

use super::{html, markdown, pdf, raw_report};

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
    pub by_severity: HashMap<Severity, u32>,
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
    pub severity: Severity,
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

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Debug, EnumString, Display, Hash)]
#[strum(serialize_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Critical,
    Medium,
    Minor,
    Enhancement,
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

    pub fn from_raw(data: String) -> Result<Self> {
        raw_report::generate_report(data)
    }

    pub fn generate_html(&self, path: Option<PathBuf>) -> Result<String> {
        html::generate_html(self, path)
    }

    pub fn generate_markdown(&self, path: Option<PathBuf>) -> Result<String> {
        markdown::generate_markdown(self, path)
    }

    pub fn generate_pdf(&self, path: Option<PathBuf>) -> Result<String> {
        pdf::generate_pdf(self, path)
    }
}
