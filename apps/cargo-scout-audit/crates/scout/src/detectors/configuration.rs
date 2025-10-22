use crate::scout::blockchain::BlockChain;
use anyhow::{Context, Ok, Result, anyhow, bail};
use cargo::core::{Dependency, GitReference, SourceId};
use cargo_metadata::Metadata;
use git2::{RemoteCallbacks, Repository};
use std::{
    env,
    path::{Path, PathBuf},
};
use tempfile::TempDir;
use thiserror::Error;
use util::logger::TracedError;

// Constants
const SCOUT_REPO_URL: &str = "https://github.com/CoinFabrik/scout-audit";
const DETECTORS_BASE_PATH: &str = "nightly";
const LOCAL_BASE_DETECTOR_PATH: &str = "rust";
const LIBRARY_NAME: &str = "library";

#[derive(Debug, Clone)]
pub struct DetectorConfig {
    pub dependency: Dependency,
    pub path: Option<String>,
}

impl DetectorConfig {
    fn new(dependency: Dependency, path: Option<String>) -> Self {
        Self { dependency, path }
    }

    fn with_dependency_and_path(source_id: SourceId, path: impl Into<String>) -> Result<Self> {
        Ok(Self::new(
            Dependency::parse(LIBRARY_NAME, None, source_id)
                .with_context(|| "Failed to parse detector dependency")?,
            Some(path.into()),
        ))
    }
}

#[derive(Debug, Clone)]
pub struct DetectorsConfiguration {
    base_config: DetectorConfig,
    blockchain_config: Option<DetectorConfig>,
}

#[derive(Error, Debug)]
pub enum DetectorsConfigError {
    #[error("Remote configuration error:\n     → {0}")]
    Remote(#[source] anyhow::Error),

    #[error("Local configuration error:\n     → {0}")]
    Local(#[source] anyhow::Error),

    #[error("Could not find suitable branch for detectors")]
    BranchNotFound,

    #[error("Failed to create git dependency:\n     → {0}")]
    GitDependency(#[source] anyhow::Error),

    #[error("Failed to create SourceId for path: {0}")]
    SourceId(PathBuf),
}

impl DetectorsConfiguration {
    pub fn new(base_config: DetectorConfig, blockchain_config: Option<DetectorConfig>) -> Self {
        Self {
            base_config,
            blockchain_config,
        }
    }

    pub fn iter_configs(&self) -> impl Iterator<Item = &DetectorConfig> {
        std::iter::once(&self.base_config).chain(self.blockchain_config.as_ref())
    }

    pub fn get(
        blockchain: BlockChain,
        toolchain: &str,
        local_path: &Option<PathBuf>,
        metadata: &Metadata,
    ) -> Result<Self> {
        let detectors_config = match &local_path {
            Some(path) => Self::get_local_detectors_configuration(path, blockchain, metadata)
                .map_err(DetectorsConfigError::Local)?,
            None => Self::get_remote_detectors_configuration(blockchain, toolchain)
                .map_err(DetectorsConfigError::Remote)?,
        };

        Ok(detectors_config)
    }

    fn get_root_detector_path(base: &str, toolchain: &str) -> String {
        // Extract just the date part from the toolchain (e.g., "2025-08-07" from "nightly-2025-08-07")
        let date = toolchain.strip_prefix("nightly-").unwrap_or(toolchain);
        format!("{base}/{date}/detectors")
    }

    fn get_variable_detector_path(base: &str, toolchain: &str, subpath: &str) -> String {
        format!(
            "{}/{subpath}",
            Self::get_root_detector_path(base, toolchain)
        )
    }

    fn get_detector_path(toolchain: &str, subpath: &str) -> String {
        Self::get_variable_detector_path(DETECTORS_BASE_PATH, toolchain, subpath)
    }

    /// Returns list of detectors from remote repository.
    #[tracing::instrument(name = "GET REMOTE DETECTORS CONFIGURATION", skip_all, level = "debug")]
    fn get_remote_detectors_configuration(blockchain: BlockChain, toolchain: &str) -> Result<Self> {
        let scout_version = env!("CARGO_PKG_VERSION");
        let default_branch = format!("release/{scout_version}");

        if !check_branch_exists(SCOUT_REPO_URL, &default_branch)? {
            bail!(DetectorsConfigError::BranchNotFound);
        }

        let dependency =
            create_git_dependency(&default_branch).map_err(DetectorsConfigError::GitDependency)?;
        let source_id = dependency.source_id();

        let base_config = DetectorConfig::with_dependency_and_path(
            source_id,
            Self::get_detector_path(toolchain, "rust"),
        )?;

        let blockchain_config = DetectorConfig::with_dependency_and_path(
            source_id,
            Self::get_detector_path(toolchain, blockchain.get_detectors_path()),
        )?;

        Ok(Self::new(base_config, Some(blockchain_config)))
    }

    /// Returns local detectors configuration from custom path.
    //#[tracing::instrument(name = "GET LOCAL DETECTORS CONFIGURATION", skip_all, level = "debug")]
    fn get_local_detectors_configuration(
        base_path: &Path,
        blockchain: BlockChain,
        metadata: &Metadata,
    ) -> Result<Self> {
        let path = Self::get_root_detector_path(
            base_path
                .to_str()
                .ok_or_else(|| anyhow!("Could not get base detector path"))?,
            blockchain.get_toolchain(metadata)?.as_str(),
        );
        let path = Path::new(&path);

        let source_id = SourceId::for_path(path)
            .map_err(DetectorsConfigError::SourceId(path.to_path_buf()).traced())?;

        let base_config =
            DetectorConfig::with_dependency_and_path(source_id, LOCAL_BASE_DETECTOR_PATH)?;

        let blockchain_config =
            DetectorConfig::with_dependency_and_path(source_id, blockchain.to_string())?;

        Ok(Self::new(base_config, Some(blockchain_config)))
    }
}

#[tracing::instrument(name = "CHECK BRANCH EXISTS", skip_all, level = "debug")]
fn check_branch_exists(url: &str, branch: &str) -> Result<bool> {
    // Set up temporary repository and remote
    let temp_dir = TempDir::new()?;
    let repo = Repository::init_bare(temp_dir.path())?;
    let mut remote = repo.remote_anonymous(url)?;

    // Connect to the remote repository
    let callbacks = RemoteCallbacks::new();
    remote.connect_auth(git2::Direction::Fetch, Some(callbacks), None)?;

    // Check if the specified branch exists
    let references = remote.list()?;
    let branch_ref = format!("refs/heads/{}", branch);
    let branch_exists = references.iter().any(|r| r.name() == branch_ref);

    remote.disconnect()?;
    Ok(branch_exists)
}

#[tracing::instrument(name = "CREATE GIT DEPENDENCY", skip_all, level = "debug")]
fn create_git_dependency(branch: &str) -> Result<Dependency> {
    let source_id = SourceId::for_git(
        &reqwest::Url::parse(SCOUT_REPO_URL)?,
        GitReference::Branch(branch.to_string()),
    )?;

    Dependency::parse(LIBRARY_NAME, None, source_id)
        .with_context(|| "Failed to create git dependency")
}
