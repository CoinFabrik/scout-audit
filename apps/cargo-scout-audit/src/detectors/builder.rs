use anyhow::{bail, ensure, Context, Result};
use cargo::GlobalContext;
use cargo_metadata::{Metadata, MetadataCommand};
use current_platform::CURRENT_PLATFORM;
use std::path::PathBuf;

use super::{configuration::DetectorsConfiguration, library::Library, source::download_git_repo};
use crate::scout::blockchain::BlockChain;

#[derive(Debug)]
pub struct DetectorBuilder<'a> {
    cargo_config: &'a GlobalContext,
    detectors_config: &'a DetectorsConfiguration,
    root_metadata: &'a Metadata,
    verbose: bool,
    toolchain: &'a str,
}

impl<'a> DetectorBuilder<'a> {
    pub fn new(
        cargo_config: &'a GlobalContext,
        detectors_config: &'a DetectorsConfiguration,
        root_metadata: &'a Metadata,
        verbose: bool,
        toolchain: &'a str,
    ) -> Self {
        Self {
            cargo_config,
            detectors_config,
            root_metadata,
            verbose,
            toolchain,
        }
    }

    pub fn build(&self, bc: &BlockChain, used_detectors: &[String]) -> Result<Vec<PathBuf>> {
        let library = self.get_library()?;
        let library_paths = library.build(bc, self.verbose)?;
        self.filter_detectors(&library_paths, used_detectors)
    }

    pub fn get_detector_names(&self) -> Result<Vec<String>> {
        let library = self.get_library()?;
        Ok(library
            .metadata
            .packages
            .into_iter()
            .map(|p| p.name)
            .collect())
    }

    fn get_library(&self) -> Result<Library> {
        let detector_root = self.get_detector()?;
        let workspace_path = self.parse_library_path(&detector_root)?;
        self.create_library(workspace_path)
    }

    fn get_detector(&self) -> Result<PathBuf> {
        let source_id = self.detectors_config.dependency.source_id();
        if source_id.is_git() {
            download_git_repo(&self.detectors_config.dependency, self.cargo_config)
        } else if source_id.is_path() {
            source_id.local_path().map(PathBuf::from).ok_or_else(|| {
                anyhow::anyhow!("Path source should have a local path: {}", source_id)
            })
        } else {
            bail!("Unsupported source id: {}", source_id)
        }
    }

    fn parse_library_path(&self, dependency_root: &PathBuf) -> Result<PathBuf> {
        let path = self
            .detectors_config
            .path
            .as_ref()
            .map(|p| dependency_root.join(p))
            .unwrap_or_else(|| dependency_root.clone());

        let path = dunce::canonicalize(&path)
            .with_context(|| format!("Could not canonicalize {path:?}"))?;
        let dependency_root = dunce::canonicalize(dependency_root)
            .with_context(|| format!("Could not canonicalize {dependency_root:?}"))?;

        ensure!(
            path.starts_with(&dependency_root),
            "Path could refer to `{}`, which is outside of `{}`",
            path.to_string_lossy(),
            dependency_root.to_string_lossy()
        );
        Ok(path)
    }

    fn create_library(&self, workspace_path: PathBuf) -> Result<Library> {
        println!("workspace_path: {}", workspace_path.display());
        ensure!(
            workspace_path.is_dir(),
            "Not a directory: {}",
            workspace_path.to_string_lossy()
        );

        let package_metadata = MetadataCommand::new()
            .current_dir(&workspace_path)
            .no_deps()
            .exec()
            .with_context(|| {
                format!(
                    "Could not get metadata for the workspace at {}",
                    workspace_path.to_string_lossy()
                )
            })?;

        let toolchain = format!("{}-{}", self.toolchain, CURRENT_PLATFORM);

        Ok(Library::new(
            workspace_path,
            toolchain,
            self.root_metadata
                .target_directory
                .clone()
                .into_std_path_buf(),
            package_metadata,
        ))
    }

    fn filter_detectors(
        &self,
        detector_paths: &[PathBuf],
        used_detectors: &[String],
    ) -> Result<Vec<PathBuf>> {
        Ok(detector_paths
            .iter()
            .filter(|path| {
                let detector_name = path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .map(|name| {
                        #[cfg(not(windows))]
                        let name = name.strip_prefix("lib").unwrap_or(name);
                        name.split('@').next().unwrap_or(name).replace('_', "-")
                    })
                    .unwrap_or_default();
                used_detectors.contains(&detector_name)
            })
            .cloned()
            .collect())
    }
}
