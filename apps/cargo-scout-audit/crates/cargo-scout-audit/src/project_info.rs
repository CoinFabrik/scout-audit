use anyhow::{anyhow, bail, Context, Result};
use cargo_metadata::{camino::Utf8PathBuf, Metadata, MetadataCommand};
use lazy_static::lazy_static;
use regex::Regex;
use std::{fs, path::PathBuf};
use thiserror::Error;
use scout::output::report::Package;
use util::logger::TracedError;

#[derive(Debug)]
pub struct Project {
    pub name: String,
    pub date: String,
    pub workspace_root: PathBuf,
    pub packages: Vec<Package>,
}

lazy_static! {
    static ref NAME_REGEX: Regex = Regex::new(r"(^|\s)\w").expect("Invalid regex");
}

#[derive(Error, Debug)]
pub enum MetadataError {
    #[error("Invalid manifest path. Ensure scout is being run in a Rust project. (Path: {0})")]
    InvalidManifestPath(PathBuf),

    #[error("Failed to access Cargo.toml file. (Path: {0})")]
    CargoTomlAccessError(PathBuf),

    #[error("Failed to execute metadata command, ensure this is a valid rust project or workspace directory.")]
    MetadataCommandFailed,
}

impl Project {
    pub fn get_metadata(manifest_path: &Option<PathBuf>) -> Result<Metadata> {
        let mut metadata_command = MetadataCommand::new();

        if let Some(manifest_path) = manifest_path {
            if !manifest_path.ends_with("Cargo.toml") {
                bail!(MetadataError::InvalidManifestPath(manifest_path.clone()));
            }

            fs::metadata(manifest_path)
                .map_err(MetadataError::CargoTomlAccessError(manifest_path.clone()).traced())?;

            metadata_command.manifest_path(manifest_path);
        }

        metadata_command
            .exec()
            .map_err(MetadataError::MetadataCommandFailed.traced())
    }

    #[tracing::instrument(name = "GET PROJECT INFO", skip_all)]
    pub fn get_info(metadata: &Metadata) -> Result<Self> {
        let packages = Self::collect_packages(metadata)?;
        let project_name = Self::format_project_name(&metadata.workspace_root)?;
        let date = chrono::Local::now().format("%Y-%m-%d").to_string();

        let project_info = Project {
            name: project_name,
            date,
            workspace_root: metadata.workspace_root.clone().into_std_path_buf(),
            packages,
        };
        tracing::trace!(?project_info, "Project info");
        Ok(project_info)
    }

    fn collect_packages(metadata: &Metadata) -> Result<Vec<Package>> {
        let mut packages = Vec::new();
        let workspace_root = &metadata.workspace_root;

        let package_ids = if let Some(root_package) = metadata.root_package() {
            // Single package case
            vec![&root_package.id]
        } else if !metadata.workspace_default_members.is_empty() {
            // Multi-package case
            metadata.workspace_default_members.iter().collect()
        } else {
            bail!("No packages found in the workspace. Ensure that workspace is configured properly and contains at least one package.");
        };

        let is_single_package = metadata.root_package().is_some();

        for package_id in package_ids {
            let package = metadata
                .packages
                .iter()
                .find(|p| &p.id == package_id)
                .with_context(|| format!("Package ID '{package_id}' not found in the workspace"))?;
            let manifest_path = &package.manifest_path;
            let absolute_path: PathBuf = manifest_path.clone().into();

            // Calculate relative path
            let relative_path = if is_single_package {
                // Single package case
                PathBuf::from("./Cargo.toml")
            } else {
                // Multi-package case
                manifest_path
                    .strip_prefix(workspace_root)
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| {
                        // Fallback: use the manifest filename if stripping prefix fails
                        PathBuf::from(manifest_path.file_name().unwrap_or("Cargo.toml"))
                    })
            };

            packages.push(Package {
                name: package.name.clone(),
                id: package_id.to_string(),
                absolute_path,
                relative_path,
            });
        }
        Ok(packages)
    }

    fn format_project_name(workspace_root: &Utf8PathBuf) -> Result<String> {
        workspace_root
            .file_name()
            .ok_or_else(|| anyhow!("Invalid workspace root"))
            .map(|name| {
                let project_name = name.replace('-', " ");
                NAME_REGEX
                    .replace_all(&project_name, |caps: &regex::Captures| {
                        caps.get(0).unwrap().as_str().to_uppercase()
                    })
                    .to_string()
            })
    }
}
