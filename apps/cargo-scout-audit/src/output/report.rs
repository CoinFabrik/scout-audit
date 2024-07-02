use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::utils::detectors_info::LintInfo;

use super::{html, markdown, pdf, utils};

#[derive(Serialize, Deserialize, Debug)]
pub struct Report {
    pub name: String,
    pub date: String,
    pub summary: Summary,
    pub categories: Vec<Category>,
    pub findings: Vec<Finding>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    Medium,
    Minor,
    Enhancement,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Summary {
    pub executed_on: Vec<Package>,
    pub total_vulnerabilities: u32,
    pub by_severity: HashMap<Severity, u32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Package {
    pub name: String,
    pub relative_path: PathBuf,
    pub absolute_path: PathBuf,
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
    pub package: String,
    pub file_path: String,
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

impl Report {
    pub fn new(
        name: String,
        date: String,
        summary: Summary,
        categories: Vec<Category>,
        findings: Vec<Finding>,
    ) -> Self {
        Report {
            name,
            date,
            summary,
            categories,
            findings,
        }
    }

    #[tracing::instrument(name = "SAVING REPORT TO FILE", level = "debug", skip_all, fields(path = %path.display()))]
    pub fn save_to_file(&self, path: &PathBuf, content: String) -> Result<()> {
        utils::write_to_file(path, content.as_bytes())?;
        Ok(())
    }

    #[tracing::instrument(name = "GENERATING HTML FROM REPORT", level = "debug", skip_all)]
    pub fn generate_html(&self) -> Result<String> {
        html::generate_html(self)
    }

    #[tracing::instrument(name = "GENERATING MARKDOWN FROM REPORT", level = "debug", skip_all)]
    pub fn generate_markdown(&self, render_styles: bool) -> Result<String> {
        markdown::generate_markdown(self, render_styles)
    }

    #[tracing::instrument(name = "GENERATING JSON FROM REPORT", level = "debug", skip_all)]
    pub fn generate_json(&self) -> Result<String> {
        let json = serde_json::to_string_pretty(self)?;
        Ok(json)
    }

    #[tracing::instrument(name = "GENERATING PDF FROM REPORT", level = "debug", skip_all)]
    pub fn generate_pdf(&self, path: &Path) -> Result<()> {
        let temp_html = pdf::generate_pdf(self)?;

        std::process::Command::new("which")
            .arg("wkhtmltopdf")
            .output()
            .expect("Please, install wkhtmltopdf to generate pdf reports.");

        let mut child = std::process::Command::new("wkhtmltopdf")
            .arg(temp_html)
            .arg(path.to_str().unwrap())
            .spawn()?;

        child.wait()?;

        std::fs::remove_file(temp_html)?;

        Ok(())
    }
}
