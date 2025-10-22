use crate::{git::download_git_repo, library::Library};
use anyhow::{Context, Result};
use cargo::{
    GlobalContext,
    core::{Dependency, GitReference, SourceId, Verbosity},
};
use cargo_metadata::{Metadata, MetadataCommand};
use std::path::PathBuf;

pub struct PackageToBuild {
    pub url: String,
    pub branch: String,
    pub name: String,
    pub internal_path: Option<PathBuf>,
    pub build_message: String,
    pub build_error_message: String,
}

impl PackageToBuild {
    pub fn new(url: &str, branch: &str, name: &str) -> Self {
        let url = url.to_string();
        let branch = branch.to_string();
        let name = name.to_string();
        Self {
            url,
            branch,
            name,
            internal_path: None,
            build_message: String::new(),
            build_error_message: String::new(),
        }
    }
    fn first_phase(&self) -> Result<(PathBuf, Metadata)> {
        let dependency = Dependency::parse(
            self.name.clone(),
            None,
            SourceId::for_git(
                &reqwest::Url::parse(&self.url)?,
                GitReference::Branch(self.branch.clone()),
            )?,
        )
        .with_context(|| "Failed to create git dependency")?;

        let cargo_config = GlobalContext::default()
            .with_context(|| "Failed to create default cargo configuration")?;
        cargo_config.shell().set_verbosity(Verbosity::Quiet);

        let mut repo_path = download_git_repo(&dependency, &cargo_config)
            .with_context(|| "Failed to download git repository")?;

        if let Some(internal_path) = &self.internal_path {
            repo_path.push(internal_path);
        }

        let metadata = MetadataCommand::new()
            .current_dir(&repo_path)
            .no_deps()
            .exec()
            .with_context(|| {
                format!(
                    "Could not get metadata for the workspace at {}",
                    repo_path.to_string_lossy()
                )
            })?;

        Ok((repo_path, metadata))
    }
    fn second_phase(&self, repo_path: PathBuf, ret: PathBuf) -> Result<PathBuf> {
        if !std::fs::exists(&ret)? {
            if !self.build_message.is_empty() {
                crate::print::print_info(&self.build_message);
            }
            let result = crate::cargo::call_cargo(&["build", "--release"], true, None)
                .current_dir(&repo_path)
                .success();
            if !self.build_error_message.is_empty() {
                result.with_context(|| self.build_error_message.clone())
            } else {
                result
            }?;
        }

        Ok(ret)
    }
    pub fn build_library(&self, package_name: Option<&str>) -> Result<PathBuf> {
        let (repo_path, metadata) = self.first_phase()?;

        let ret = Library::new_from_metadata(metadata)
            .get_compiled_paths(package_name, None)
            .first()
            .cloned()
            .with_context(|| "Failed to determine the binary's expected location")?;

        self.second_phase(repo_path, ret)
    }
    pub fn build_executable(&self, package_name: Option<&str>) -> Result<PathBuf> {
        let (repo_path, metadata) = self.first_phase()?;

        let ret = Library::new_from_metadata(metadata)
            .get_compiled_paths(package_name, Some("bin"))
            .first()
            .cloned()
            .with_context(|| "Failed to determine the binary's expected location")?;

        self.second_phase(repo_path, ret)
    }
}
