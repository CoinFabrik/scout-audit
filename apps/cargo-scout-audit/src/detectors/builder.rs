use anyhow::{bail, ensure, Context, Result};
use cargo::GlobalContext;
use cargo_metadata::{Metadata, MetadataCommand};
use current_platform::CURRENT_PLATFORM;
use std::path::{Path, PathBuf};

use super::{
    configuration::{DetectorConfig, DetectorsConfiguration},
    library::Library,
    source::download_git_repo,
};
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
    const LIB_PREFIX: &'static str = "lib";

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

    #[tracing::instrument(skip_all, level = "debug")]
    pub fn build(&self, bc: &BlockChain, used_detectors: &[String]) -> Result<Vec<PathBuf>> {
        let all_library_paths = self.build_all_libraries(bc)?;
        self.filter_detectors(&all_library_paths, used_detectors)
    }

    #[tracing::instrument(skip_all, level = "debug")]
    pub fn get_detector_names(&self) -> Result<Vec<String>> {
        let mut all_names = Vec::new();

        for config in self.detectors_config.iter_configs() {
            let library = self
                .get_library(config)
                .with_context(|| "Failed to get library for detector names")?;

            all_names.extend(library.metadata.packages.into_iter().map(|p| p.name));
        }

        Ok(all_names)
    }

    #[tracing::instrument(skip_all, level = "debug")]
    fn build_all_libraries(&self, bc: &BlockChain) -> Result<Vec<PathBuf>> {
        let mut all_library_paths = Vec::new();

        for config in self.detectors_config.iter_configs() {
            let library = self
                .get_library(config)
                .with_context(|| "Failed to get library while building")?;

            let library_paths = library
                .build(bc, self.verbose)
                .with_context(|| "Failed to build library")?;

            all_library_paths.extend(library_paths);
        }

        Ok(all_library_paths)
    }

    #[tracing::instrument(skip_all, level = "debug")]
    fn get_library(&self, config: &DetectorConfig) -> Result<Library> {
        let detector_root = self.get_detector(config)?;
        let workspace_path = self.parse_library_path(config, &detector_root)?;
        self.create_library(workspace_path)
    }

    #[tracing::instrument(skip_all, level = "debug")]
    fn get_detector(&self, config: &DetectorConfig) -> Result<PathBuf> {
        let source_id = config.dependency.source_id();

        match (source_id.is_git(), source_id.is_path()) {
            (true, _) => download_git_repo(&config.dependency, self.cargo_config),
            (_, true) => source_id.local_path().map(PathBuf::from).ok_or_else(|| {
                anyhow::anyhow!("Path source should have a local path: {}", source_id)
            }),
            _ => bail!("Unsupported source id: {}", source_id),
        }
    }

    #[tracing::instrument(skip_all, level = "debug")]
    fn parse_library_path(
        &self,
        config: &DetectorConfig,
        dependency_root: &PathBuf,
    ) -> Result<PathBuf> {
        let path = match &config.path {
            Some(p) => dependency_root.join(p),
            None => dependency_root.clone(),
        };

        let canonical_path = dunce::canonicalize(&path)
            .with_context(|| format!("Could not canonicalize {path:?}"))?;

        let canonical_root = dunce::canonicalize(dependency_root)
            .with_context(|| format!("Could not canonicalize {dependency_root:?}"))?;

        ensure!(
            canonical_path.starts_with(&canonical_root),
            "Path could refer to `{}`, which is outside of `{}`",
            canonical_path.to_string_lossy(),
            canonical_root.to_string_lossy()
        );

        Ok(canonical_path)
    }

    #[tracing::instrument(skip_all, level = "debug")]
    fn create_library(&self, workspace_path: PathBuf) -> Result<Library> {
        ensure!(
            workspace_path.is_dir(),
            "Not a directory: {}",
            workspace_path.to_string_lossy()
        );

        let package_metadata = self.get_package_metadata(&workspace_path)?;
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

    fn get_package_metadata(&self, workspace_path: &PathBuf) -> Result<Metadata> {
        MetadataCommand::new()
            .current_dir(workspace_path)
            .no_deps()
            .exec()
            .with_context(|| {
                format!(
                    "Could not get metadata for the workspace at {}",
                    workspace_path.to_string_lossy()
                )
            })
    }

    #[tracing::instrument(skip_all, level = "debug")]
    fn filter_detectors(
        &self,
        detector_paths: &[PathBuf],
        used_detectors: &[String],
    ) -> Result<Vec<PathBuf>> {
        Ok(detector_paths
            .iter()
            .filter(|path| self.matches_detector_name(path, used_detectors))
            .cloned()
            .collect())
    }

    fn matches_detector_name(&self, path: &Path, used_detectors: &[String]) -> bool {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(|name| {
                let name = self.normalize_detector_name(name);
                used_detectors.contains(&name)
            })
            .unwrap_or(false)
    }

    fn normalize_detector_name(&self, name: &str) -> String {
        #[cfg(not(windows))]
        let name = name.strip_prefix(Self::LIB_PREFIX).unwrap_or(name);

        name.split('@').next().unwrap_or(name).replace('_', "-")
    }
}
