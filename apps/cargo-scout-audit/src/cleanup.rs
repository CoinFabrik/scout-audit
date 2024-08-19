use cargo_metadata::{Metadata, PackageId};
use std::{
    collections::HashSet,
    fs,
    path::PathBuf,
};

pub(crate) fn clean_up_before_run(metadata: &Metadata) {
    let mut dylint_target = metadata.target_directory.clone();
    dylint_target.push("dylint");
    dylint_target.push("target");
    let result = fs::read_dir(dylint_target);
    if result.is_err() {
        return;
    }
    let result = result.unwrap();
    for path in result {
        if path.is_err() {
            continue;
        }
        let entry = path.unwrap();
        let entry_metadata = entry.metadata();
        if entry_metadata.is_err() || !entry_metadata.unwrap().is_dir() {
            continue;
        }
        let mut deps = entry.path();
        deps.push("wasm32-unknown-unknown");
        deps.push("debug");
        deps.push("deps");
        clean_up_deps(deps, metadata);
    }
}

fn special_escape(string: &str) -> String {
    let mut ret = String::new();
    for c in string.chars() {
        let c2 = if c.is_alphanumeric() { c } else { '.' };
        ret.push(c2);
    }
    ret
}

fn get_targets_for_workspace(metadata: &Metadata) -> Vec<regex::Regex> {
    let mut ret = Vec::<regex::Regex>::new();
    let members = HashSet::<&PackageId>::from_iter(metadata.workspace_members.iter());
    for package in metadata.packages.iter() {
        if !members.contains(&package.id) {
            continue;
        }
        for target in package.targets.iter() {
            let target_name = special_escape(&target.name);
            let pattern = format!("^lib{target_name}-[0-9A-Fa-f]{{16}}\\.rmeta$");
            let regex = regex::Regex::new(&pattern);
            if regex.is_err() {
                continue;
            }
            ret.push(regex.unwrap());
        }
    }
    ret
}

fn needs_cleanup(name: String, targets: &[regex::Regex]) -> bool {
    for target in targets.iter() {
        if target.is_match(&name) {
            return true;
        }
    }
    false
}

fn clean_up_deps(deps: PathBuf, metadata: &Metadata) {
    let targets = get_targets_for_workspace(metadata);
    let result = fs::read_dir(deps);
    if result.is_err() {
        return;
    }
    let result = result.unwrap();
    for path in result {
        if path.is_err() {
            continue;
        }
        let entry = path.unwrap();
        let name = entry.file_name();
        let name = name.to_str();
        if name.is_none() {
            continue;
        }
        if needs_cleanup(name.unwrap().into(), &targets) {
            let _ = fs::remove_file(entry.path());
        }
    }
}
