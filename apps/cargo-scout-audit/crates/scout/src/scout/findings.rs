use crate::finding::Finding;
use util::dependencies::DependencyGraph;
use anyhow::Result;
use cargo_metadata::Metadata;
use serde_json::{from_str, Value};
use std::{
    collections::HashMap,
    io::Read,
    path::PathBuf,
};

pub fn get_crates(
    findings: &Vec<Finding>,
    packages: &[crate::output::report::Package],
    metadata: &Metadata,
) -> Result<HashMap<String, bool>> {
    let mut ret = HashMap::<String, bool>::new();
    for package in packages.iter() {
        ret.insert(normalize_crate_name(&package.name), true);
    }
    for (name, ok) in get_crates_from_output(findings, packages, metadata)?.iter() {
        let normalized = normalize_crate_name(name);
        let val = ret.get_mut(&normalized);
        if let Some(val) = val {
            *val = *ok;
        }
    }

    Ok(ret)
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

fn get_crates_from_output(
    output: &Vec<Finding>,
    packages: &[crate::output::report::Package],
    metadata: &Metadata,
) -> Result<HashMap<String, bool>> {
    let mut ret = HashMap::<String, bool>::new();
    let package_ids = packages
        .iter()
        .map(|x| {
            let id = x.id.clone();
            let name = x.name.clone();
            (id, name)
        })
        .collect::<HashMap<_, _>>();
    let mut graph = None;

    for finding in output {
        let reason = finding.reason();
        if reason != "compiler-message" {
            continue;
        }
        let id = finding.package_id();

        let affected = if package_ids.contains_key(&id) {
            let name = finding.package();
            if name.is_empty() {
                continue;
            }
            vec![name]
        } else {
            if graph.is_none() {
                graph = Some(DependencyGraph::new(metadata)?);
            }
            graph
                .as_ref()
                .unwrap()
                .list_all_dependants(&id)?
                .iter()
                .map(|x| package_ids.get(x))
                .filter(|x| x.is_some())
                .map(|x| x.unwrap().clone())
                .collect::<Vec<_>>()
        };

        for name in affected {
            if let Some(previous) = ret.get(&name) {
                if !previous {
                    continue;
                }
            }
            ret.insert(name, !finding.is_compiler_error());
        }
    }

    Ok(ret)
}

pub fn temp_file_to_string(path: &PathBuf) -> Result<String> {
    let ret = {
        let mut file = std::fs::File::open(path)?;
        let mut ret = String::new();
        file.read_to_string(&mut ret)?;
        ret
    };
    let _ = std::fs::remove_file(&path);
    Ok(ret)
}

pub fn output_to_json(output: &str) -> Vec<Value> {
    output
        .lines()
        .map(|line| from_str::<Value>(line).unwrap())
        .collect::<Vec<Value>>()
}
