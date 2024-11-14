use super::{html, markdown, pdf, utils};
use crate::output::table::Table;
use crate::startup::OutputFormat;
use crate::utils::detectors_info::LintInfo;
use crate::finding::Finding as JsonFinding;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};

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
    pub table: Table,
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
        pdf::generate_pdf(path, self)
    }

    fn write_single_json(file: &mut File, findings: &Vec<JsonFinding>) -> Result<()>{
        let bytes = findings
            .iter()
            .map(|x| x.json().to_string().as_bytes().to_vec())
            .collect::<Vec<_>>();

        let mut w = |x| std::io::Write::write(file, x);

        w(b"[")?;
        let mut first = true;
        for finding in bytes.iter() {
            let s: &[u8] = if first{
                first = false;
                b"\n"
            }else{
                b",\n"
            };
            w(s)?;
            w(finding.as_slice())?;
        }
        w(b"\n]")?;

        Ok(())
    }

    pub fn write_out(
        &self,
        findings: &Vec<JsonFinding>,
        raw_findings: &Vec<JsonFinding>,
        output_path: Option<PathBuf>,
        output_format: &OutputFormat,
    ) -> Result<Option<PathBuf>> {
        match output_format {
            OutputFormat::Html => {
                // Generate HTML report
                let html = self.generate_html()?;

                // Save to file
                let html_path = output_path.unwrap_or_else(|| PathBuf::from("report.html"));
                self.save_to_file(&html_path, html)?;

                // Open the HTML report in the default web browser
                webbrowser::open(
                    html_path
                        .to_str()
                        .with_context(|| "Path conversion to string failed")?,
                )
                .with_context(|| "Failed to open HTML report")?;

                Ok(Some(html_path))
            }
            OutputFormat::Json => {
                // Generate JSON report
                let json = self.generate_json()?;

                // Save to file
                let json_path = output_path.unwrap_or_else(|| PathBuf::from("report.json"));
                self.save_to_file(&json_path, json)?;

                Ok(Some(json_path))
            }
            OutputFormat::RawJson => {
                let json_path = output_path.unwrap_or_else(|| PathBuf::from("raw-report.json"));
                let mut json_file = File::create(&json_path)?;

                for finding in findings.iter() {
                    std::io::Write::write(&mut json_file, finding.json().to_string().as_bytes())?;
                    std::io::Write::write(&mut json_file, b"\n")?;
                }

                Ok(Some(json_path))
            }
            OutputFormat::RawSingleJson => {
                let json_path = output_path.unwrap_or_else(|| PathBuf::from("raw-report.json"));
                let mut json_file = File::create(&json_path)?;
                Self::write_single_json(&mut json_file, findings)?;
                Ok(Some(json_path))
            }
            OutputFormat::UnfilteredJson => {
                let json_path = output_path.unwrap_or_else(|| PathBuf::from("raw-report.json"));
                let mut json_file = File::create(&json_path)?;
                Self::write_single_json(&mut json_file, raw_findings)?;
                Ok(Some(json_path))
            }
            OutputFormat::Markdown => {
                // Generate Markdown
                let markdown = self.generate_markdown(true)?;

                // Save to file
                let md_path = output_path.unwrap_or_else(|| PathBuf::from("report.md"));
                self.save_to_file(&md_path, markdown)?;

                Ok(Some(md_path))
            }
            OutputFormat::MarkdownGithub => {
                // Generate Markdown
                let markdown = self.generate_markdown(false)?;

                // Save to file
                let md_path = output_path.unwrap_or_else(|| PathBuf::from("report.md"));
                self.save_to_file(&md_path, markdown)?;

                Ok(Some(md_path))
            }
            OutputFormat::Sarif => {
                let sarif_path = output_path.unwrap_or_else(|| PathBuf::from("report.sarif"));

                let mut sarif_file = File::create(&sarif_path)?;

                let child = std::process::Command::new("clippy-sarif")
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .spawn()?;

                for finding in findings {
                    std::io::Write::write_all(
                        &mut child.stdin.as_ref().unwrap(),
                        finding.rendered().as_bytes(),
                    )?;
                }

                std::io::Write::write_all(&mut sarif_file, &child.wait_with_output()?.stdout)?;

                Ok(Some(sarif_path))
            }
            OutputFormat::Pdf => {
                let pdf_path = output_path.unwrap_or_else(|| PathBuf::from("report.pdf"));
                self.generate_pdf(&pdf_path)?;
                Ok(Some(pdf_path))
            }
        }
    }
}
