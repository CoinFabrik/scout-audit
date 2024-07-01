use anyhow::{bail, Context, Result};
use cargo_metadata::Metadata;
use lazy_static::lazy_static;
use regex::Regex;
use std::path::PathBuf;

use crate::output::report::Package;

#[derive(Debug)]
pub struct ProjectInfo {
    pub name: String,
    pub date: String,
    pub workspace_root: PathBuf,
    pub packages: Vec<Package>,
}

lazy_static! {
    static ref NAME_REGEX: Regex = Regex::new(r"(^|\s)\w").expect("Invalid regex");
}

impl ProjectInfo {
    pub fn get_project_info(metadata: &Metadata) -> Result<ProjectInfo> {
        let mut packages = Vec::new();
        let workspace_root = &metadata.workspace_root;
        if let Some(root_package) = metadata.root_package() {
            // Scout is executed on a single project
            let root = &root_package.manifest_path;
            println!("root: {:?}", root);
            packages.push(Package {
                name: root_package.name.clone(),
                absolute_path: root.into(),
                relative_path: PathBuf::from("Cargo.toml"),
            });
        } else if !metadata.workspace_default_members.is_empty() {
            // Scout is executed on a workspace
            for package_id in metadata.workspace_default_members.iter() {
                let package = metadata
                    .packages
                    .iter()
                    .find(|p| p.id == *package_id)
                    .with_context(|| {
                        format!("Package ID '{}' not found in the workspace", package_id)
                    })?;
                let root = &package.manifest_path;
                packages.push(Package {
                    name: package.name.clone(),
                    absolute_path: root.into(),
                    relative_path: root
                        .strip_prefix(workspace_root)
                        .with_context(|| "Failed to strip prefix")?
                        .into(),
                });
            }
        } else {
            bail!("No packages found in the workspace. Ensure that workspace is configured properly and contains at least one package.");
        }

        let mut project_name = String::new();
        if let Some(name) = metadata.workspace_root.file_name() {
            project_name = name.replace('-', " ");
            let re = Regex::new(r"(^|\s)\w").unwrap();
            project_name = re
                .replace_all(&project_name, |caps: &regex::Captures| {
                    caps.get(0).unwrap().as_str().to_uppercase()
                })
                .to_string();
        }

        let project_info = ProjectInfo {
            name: project_name,
            date: chrono::Local::now().format("%Y-%m-%d").to_string(),
            workspace_root: metadata.workspace_root.clone().into_std_path_buf(),
            packages,
        };
        tracing::trace!(?project_info, "Project info");
        Ok(project_info)
    }

    // #[tracing::instrument(name = "GET PROJECT INFO", skip_all)]
    // pub fn get_project_info(metadata: &Metadata) -> Result<Self> {
    //     let packages = Self::collect_packages(metadata)?;
    //     let project_name = Self::format_project_name(&metadata.workspace_root)?;
    //     let date = chrono::Local::now().format("%Y-%m-%d").to_string();

    //     let project_info = ProjectInfo {
    //         name: project_name,
    //         date,
    //         workspace_root: metadata.workspace_root.clone().into_std_path_buf(),
    //         packages,
    //     };
    //     tracing::trace!(?project_info, "Project info");
    //     Ok(project_info)
    // }

    // fn collect_packages(metadata: &Metadata) -> Result<Vec<Package>> {
    //     let mut packages = Vec::new();
    //     let workspace_root = &metadata.workspace_root;
    //     let package_ids = if let Some(root_package) = metadata.root_package() {
    //         vec![&root_package.id]
    //     } else if !metadata.workspace_default_members.is_empty() {
    //         metadata.workspace_default_members.iter().collect()
    //     } else {
    //         bail!("No packages found in the workspace. Ensure that workspace is configured properly and contains at least one package.");
    //     };

    //     for package_id in package_ids {
    //         let package = metadata
    //             .packages
    //             .iter()
    //             .find(|p| &p.id == package_id)
    //             .with_context(|| {
    //                 format!("Package ID '{}' not found in the workspace", package_id)
    //             })?;
    //         let root = &package.manifest_path;
    //         println!("root: {:?}", root);
    //         packages.push(Package {
    //             name: package.name.clone(),
    //             absolute_path: root.into(),
    //             relative_path: root
    //                 .strip_prefix(workspace_root)
    //                 .with_context(|| "Failed to strip prefix")?
    //                 .into(),
    //         });
    //     }
    //     Ok(packages)
    // }

    // fn format_project_name(workspace_root: &Utf8PathBuf) -> Result<String> {
    //     workspace_root
    //         .file_name()
    //         .ok_or_else(|| anyhow::anyhow!("Invalid workspace root"))
    //         .map(|name| {
    //             let project_name = name.replace('-', " ");
    //             NAME_REGEX
    //                 .replace_all(&project_name, |caps: &regex::Captures| {
    //                     caps.get(0).unwrap().as_str().to_uppercase()
    //                 })
    //                 .to_string()
    //         })
    // }
}
