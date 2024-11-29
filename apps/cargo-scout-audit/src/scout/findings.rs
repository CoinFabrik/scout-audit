use crate::finding::Finding;
use anyhow::Result;
use serde_json::{from_str, Value};
use std::collections::HashMap;
use tempfile::NamedTempFile;

pub fn get_crates(
    findings: &Vec<Finding>,
    packages: &[crate::output::report::Package],
) -> HashMap<String, bool> {
    let mut ret = HashMap::<String, bool>::new();
    for package in packages.iter() {
        ret.insert(normalize_crate_name(&package.name), true);
    }
    for (name, ok) in get_crates_from_output(findings).iter() {
        if ret.contains_key(name) {
            ret.insert(name.clone(), *ok);
        }
    }

    ret
}

pub fn split_findings(
    findings: &[Finding],
    crates: &HashMap<String, bool>,
) -> (Vec<Finding>, Vec<Finding>) {
    let mut successful_findings = Vec::<Finding>::new();
    let mut failed_findings = Vec::<Finding>::new();

    for finding in findings.iter() {
        let krate = finding.krate();
        if krate.is_empty() {
            continue;
        }
        if *crates.get(&krate).unwrap_or(&true) {
            &mut successful_findings
        } else {
            &mut failed_findings
        }
        .push(finding.clone());
    }

    (successful_findings, failed_findings)
}

//In some cases, rustc (or dylint, or clipply, or whoever) has returned the
//package name where it should be returning the crate name. If you run into
//problems in the future, try removing the call to this function.
fn normalize_crate_name(s: &str) -> String {
    s.replace("-", "_")
}

fn get_crates_from_output(output: &Vec<Finding>) -> HashMap<String, bool> {
    let mut ret = HashMap::<String, bool>::new();

    for finding in output {
        let reason = finding.reason();
        if reason != "compiler-message" {
            continue;
        }
        let name = finding.package();
        if name.is_empty() {
            continue;
        }
        if let Some(previous) = ret.get(&name) {
            if !previous {
                continue;
            }
        }
        ret.insert(name, !finding.is_compiler_error());
    }

    ret
}

pub fn temp_file_to_string(mut file: NamedTempFile) -> Result<String> {
    let mut ret = String::new();
    std::io::Read::read_to_string(&mut file, &mut ret)?;
    let _ = file.close();
    Ok(ret)
}

pub fn output_to_json(output: &str) -> Vec<Value> {
    output
        .lines()
        .map(|line| from_str::<Value>(line).unwrap())
        .collect::<Vec<Value>>()
}
