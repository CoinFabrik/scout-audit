use super::{
    configuration::{DetectorConfig, DetectorsConfiguration},
    library::DetectorLibrary,
};
use crate::util::{git::download_git_repo, logger::TracedError, print::print_info};
use anyhow::{Result, ensure};
use cargo::{GlobalContext, core::SourceId};
use cargo_metadata::Metadata;
use current_platform::CURRENT_PLATFORM;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BuilderError {
    #[error("Failed to build detector library (Path: {0})")]
    BuildError(PathBuf),

    #[error("Unsupported source id: {0}")]
    UnsupportedSourceId(SourceId),

    #[error("Path source should have a local path: {0}")]
    InvalidPathSource(SourceId),

    #[error("Path could refer to '{path}', which is outside of '{root}'")]
    PathOutsideRoot { path: PathBuf, root: PathBuf },

    #[error("Could not canonicalize (Path: {0})")]
    CanonicalizeError(PathBuf),
}

#[derive(Debug)]
pub struct DetectorBuilder<'a> {
    cargo_config: &'a GlobalContext,
    detectors_config: &'a DetectorsConfiguration,
    root_metadata: &'a Metadata,
    verbose: bool,
    toolchain: &'a str,
}

impl<'a> DetectorBuilder<'a> {
    #[allow(dead_code)]
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
    pub fn build(&self, used_detectors: &[String]) -> Result<Vec<PathBuf>> {
        print_info("Compiling detectors...");
        let all_library_paths = self.build_all_libraries()?;
        self.filter_detectors(&all_library_paths, used_detectors)
    }

    #[tracing::instrument(skip_all, level = "debug")]
    pub fn get_detector_names(&self) -> Result<Vec<String>> {
        print_info("Getting detector names...");
        let mut all_names = Vec::new();
        let libraries = self.get_all_libraries()?;

        for library in libraries {
            all_names.extend(library.lib.metadata.packages.into_iter().map(|p| p.name));
        }

        Ok(all_names)
    }

    #[tracing::instrument(skip_all, level = "debug")]
    fn build_all_libraries(&self) -> Result<Vec<PathBuf>> {
        let mut all_library_paths = Vec::new();

        let libraries = self.get_all_libraries()?;

        for library in libraries {
            let library_paths = library
                .build(self.verbose)
                .map_err(BuilderError::BuildError(library.lib.root).traced())?;
            all_library_paths.extend(library_paths);
        }

        Ok(all_library_paths)
    }

    #[tracing::instrument(skip_all, level = "debug")]
    fn get_all_libraries(&self) -> Result<Vec<DetectorLibrary>> {
        let mut all_libraries = Vec::new();

        for config in self.detectors_config.iter_configs() {
            let library = self.get_library(config)?;
            all_libraries.push(library);
        }

        let deduplicated_libraries = DetectorLibrary::deduplicate_libraries(all_libraries);

        Ok(deduplicated_libraries)
    }

    #[tracing::instrument(skip_all, level = "debug")]
    fn get_library(&self, config: &DetectorConfig) -> Result<DetectorLibrary> {
        let detector_root = self.get_detector(config)?;
        let workspace_path = self.parse_library_path(config, &detector_root)?;
        let toolchain = format!("{}-{}", self.toolchain, CURRENT_PLATFORM);

        DetectorLibrary::create(
            workspace_path,
            toolchain,
            self.root_metadata
                .target_directory
                .clone()
                .into_std_path_buf(),
        )
    }

    #[tracing::instrument(skip_all, level = "debug")]
    fn get_detector(&self, config: &DetectorConfig) -> Result<PathBuf> {
        let source_id = config.dependency.source_id();

        match (source_id.is_git(), source_id.is_path()) {
            (true, _) => download_git_repo(&config.dependency, self.cargo_config),
            (_, true) => source_id
                .local_path()
                .ok_or_else(|| BuilderError::InvalidPathSource(source_id).into()),
            _ => Err(BuilderError::UnsupportedSourceId(source_id).into()),
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
            .map_err(BuilderError::CanonicalizeError(path.clone()).traced())?;

        let canonical_root = dunce::canonicalize(dependency_root)
            .map_err(BuilderError::CanonicalizeError(dependency_root.clone()).traced())?;

        ensure!(
            canonical_path.starts_with(&canonical_root),
            BuilderError::PathOutsideRoot {
                path: canonical_path,
                root: canonical_root,
            }
        );

        Ok(canonical_path)
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
