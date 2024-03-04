use anyhow::Result;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::html;

#[derive(Serialize, Deserialize, Debug)]
pub struct Report {
    name: String,
    description: String,
    date: NaiveDate,
    source_url: String,
    summary: Summary,
    categories: Vec<Category>,
    findings: Vec<Finding>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Summary {
    total_vulnerabilities: u32,
    by_severity: HashMap<String, u32>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Category {
    id: String,
    name: String,
    vulnerabilities: Vec<Vulnerability>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Vulnerability {
    id: String,
    name: String,
    short_message: String,
    long_message: String,
    severity: String,
    help: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Finding {
    id: u32,
    occurrence_index: u32,
    category_id: String,
    vulnerability_id: String,
    error_message: String,
    span: String,
    code_snippet: String,
    file: String,
}

impl Report {
    pub fn new(
        name: String,
        description: String,
        date: NaiveDate,
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

    pub fn generate_html(&self) -> Result<&'static str> {
        html::generate_html(self)
    }
}
