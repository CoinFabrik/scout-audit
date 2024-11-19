use std::{env, path::Path};

use crate::scout::blockchain::BlockChain;
use anyhow::{anyhow, Context, Result};
use cargo::core::{Dependency, GitReference, SourceId};
use git2::{RemoteCallbacks, Repository};
use tempfile::TempDir;

#[derive(Debug, Clone)]
pub struct DetectorConfig {
    pub dependency: Dependency,
    pub path: Option<String>,
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

const URL: &str = "https://github.com/CoinFabrik/scout-audit";

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

fn create_git_dependency(branch: &str) -> Result<Dependency> {
    Dependency::parse(
        "library",
        None,
        SourceId::for_git(
            &reqwest::Url::parse(URL)?,
            GitReference::Branch(branch.to_string()),
        )?,
    )
    .with_context(|| "Failed to create git dependency")
}

/// Returns list of detectors.
#[tracing::instrument(name = "GET REMOTE DETECTORS CONFIGURATION", skip_all, level = "debug")]
pub fn get_remote_detectors_configuration(
    blockchain: BlockChain,
) -> Result<DetectorsConfiguration> {
    let scout_version = env!("CARGO_PKG_VERSION");
    let default_branch = format!("release/{}", scout_version);

    let branch = if check_branch_exists(URL, &default_branch)? {
        default_branch
    } else {
        return Err(anyhow!("Could not find any suitable branch for detectors"));
    };

    let dependency = create_git_dependency(&branch)?;

    let blockchain_config = DetectorConfig {
        dependency: dependency.clone(),
        path: Some(blockchain.get_detectors_path().to_string()),
    };

    let base_config = DetectorConfig {
        dependency,
        path: Some("detectors/rust".to_string()),
    };

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
        .with_context(|| format!("Failed to create SourceId for path: {:?}", path))?;

    let dependency = Dependency::parse("library", None, source_id)
        .with_context(|| "Failed to parse local detector dependency")?;

    let base_config = DetectorConfig {
        dependency: dependency.clone(),
        path: Some("rust".to_string()),
    };

    let blockchain_config = DetectorConfig {
        dependency,
        path: Some(blockchain.to_string()),
    };

    Ok(DetectorsConfiguration::new(
        base_config,
        Some(blockchain_config),
    ))
}
