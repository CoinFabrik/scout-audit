use crate::util::{git::download_git_repo, library::Library};
use anyhow::{Context, Result};
use cargo::{
    GlobalContext,
    core::{Dependency, GitReference, SourceId, Verbosity},
};
use cargo_metadata::{Metadata, MetadataCommand};
use std::{fs::canonicalize, path::PathBuf};

enum PackageSource {
    Remote {
        url: String,
        branch: String,
        name: String,
    },
    Local {
        root: PathBuf,
    },
}

pub struct PackageToBuild {
    source: PackageSource,
    pub internal_path: Option<PathBuf>,
    pub build_message: String,
    pub build_error_message: String,
}

impl PackageToBuild {
    pub fn new_remote(url: &str, branch: &str, name: &str) -> Self {
        Self {
            source: PackageSource::Remote {
                url: url.to_string(),
                branch: branch.to_string(),
                name: name.to_string(),
            },
            internal_path: None,
            build_message: String::new(),
            build_error_message: String::new(),
        }
    }

    pub fn new_local(root: impl Into<PathBuf>) -> Self {
        Self {
            source: PackageSource::Local { root: root.into() },
            internal_path: None,
            build_message: String::new(),
            build_error_message: String::new(),
        }
    }

    fn first_phase(&self) -> Result<(PathBuf, Metadata)> {
        match &self.source {
            PackageSource::Local { root } => {
                let mut repo_path = canonicalize(root).with_context(|| {
                    format!(
                        "Failed to canonicalize local scout sources at '{}'",
                        root.display()
                    )
                })?;

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

            PackageSource::Remote { url, branch, name } => {
                let dependency = Dependency::parse(
                    name.clone(),
                    None,
                    SourceId::for_git(
                        &reqwest::Url::parse(url)?,
                        GitReference::Branch(branch.clone()),
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
        }
    }
    fn second_phase(&self, repo_path: PathBuf, ret: PathBuf) -> Result<PathBuf> {
        if !std::fs::exists(&ret)? {
            if !self.build_message.is_empty() {
                crate::util::print::print_info(&self.build_message);
            }
            let result = crate::util::cargo::call_cargo(&["build", "--release"], true, None)
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
