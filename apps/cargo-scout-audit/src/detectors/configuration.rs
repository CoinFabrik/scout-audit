use crate::scout::blockchain::BlockChain;
use anyhow::{anyhow, Context, Result};
use cargo::core::{Dependency, GitReference, SourceId};
use git2::{RemoteCallbacks, Repository};
use std::{env, path::Path};
use tempfile::TempDir;

// Constants
const SCOUT_REPO_URL: &str = "https://github.com/CoinFabrik/scout-audit";
const BASE_DETECTOR_PATH: &str = "detectors/rust";
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
}

#[tracing::instrument(name = "CHECK BRANCH EXISTS", skip_all, level = "debug")]
pub fn check_branch_exists(url: &str, branch: &str) -> Result<bool> {
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

/// Returns list of detectors from remote repository.
#[tracing::instrument(name = "GET REMOTE DETECTORS CONFIGURATION", skip_all, level = "debug")]
pub fn get_remote_detectors_configuration(
    blockchain: BlockChain,
) -> Result<DetectorsConfiguration> {
    let scout_version = env!("CARGO_PKG_VERSION");
    let default_branch = format!("release/{scout_version}");

    if !check_branch_exists(SCOUT_REPO_URL, &default_branch)? {
        return Err(anyhow!("Could not find suitable branch for detectors"));
    }

    let dependency = create_git_dependency(&default_branch)?;
    let source_id = dependency.source_id();

    let base_config = DetectorConfig::with_dependency_and_path(source_id, BASE_DETECTOR_PATH)?;

    let blockchain_config =
        DetectorConfig::with_dependency_and_path(source_id, blockchain.get_detectors_path())?;

    Ok(DetectorsConfiguration::new(
        base_config,
        Some(blockchain_config),
    ))
}

/// Returns local detectors configuration from custom path.
#[tracing::instrument(name = "GET LOCAL DETECTORS CONFIGURATION", skip_all, level = "debug")]
pub fn get_local_detectors_configuration(
    path: &Path,
    blockchain: BlockChain,
) -> Result<DetectorsConfiguration> {
    let source_id = SourceId::for_path(path)
        .with_context(|| format!("Failed to create SourceId for path: {path:?}"))?;

    let base_config =
        DetectorConfig::with_dependency_and_path(source_id, LOCAL_BASE_DETECTOR_PATH)?;

    let blockchain_config =
        DetectorConfig::with_dependency_and_path(source_id, blockchain.to_string())?;

    Ok(DetectorsConfiguration::new(
        base_config,
        Some(blockchain_config),
    ))
}
