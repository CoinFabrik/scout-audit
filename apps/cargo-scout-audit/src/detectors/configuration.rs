use std::{env, path::Path};

use crate::{scout::blockchain::BlockChain, utils::print::print_warning};
use anyhow::{anyhow, Context, Result};
use cargo::{
    core::{Dependency, GitReference, SourceId},
    util::IntoUrl,
};
use git2::{RemoteCallbacks, Repository};
use tempfile::TempDir;

#[derive(Debug, Clone)]
pub struct DetectorsConfiguration {
    pub dependency: Dependency,
    pub path: Option<String>,
}

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

fn create_git_dependency(blockchain: &BlockChain, branch: &str) -> Result<Dependency> {
    let url = blockchain
        .get_detectors_url()
        .into_url()
        .with_context(|| format!("Failed to get URL for {} blockchain", blockchain))?;

    Dependency::parse(
        "library",
        None,
        SourceId::for_git(&url, GitReference::Branch(branch.to_string()))?,
    )
    .with_context(|| "Failed to create git dependency")
}

/// Returns list of detectors.
#[tracing::instrument(name = "GET REMOTE DETECTORS CONFIGURATION", skip_all, level = "debug")]
pub fn get_remote_detectors_configuration(
    blockchain: BlockChain,
    force_fallback: bool,
) -> Result<DetectorsConfiguration> {
    let toolchain = blockchain.get_toolchain();
    let scout_version = env!("CARGO_PKG_VERSION");
    let default_branch = format!("release/{}", scout_version);
    let fallback_branch = format!("release/{}-{}", scout_version, toolchain);

    let url = blockchain
        .get_detectors_url()
        .into_url()
        .with_context(|| format!("Failed to get URL for {} blockchain", blockchain))?;

    let branch = if !force_fallback && check_branch_exists(url.as_str(), &default_branch)? {
        default_branch
    } else if check_branch_exists(url.as_str(), &fallback_branch)? {
        if !force_fallback {
            print_warning(&format!(
                "Could not find branch {} for detectors, falling back to {}",
                default_branch, fallback_branch
            ));
        }
        fallback_branch
    } else {
        return Err(anyhow!("Could not find any suitable branch for detectors"));
    };

    let dependency = create_git_dependency(&blockchain, &branch)?;

    let detectors = DetectorsConfiguration {
        dependency,
        path: Some("detectors".to_string()),
    };

    Ok(detectors)
}

/// Returns local detectors configuration from custom path.
#[tracing::instrument(name = "GET LOCAL DETECTORS CONFIGURATION", skip_all, level = "debug")]
pub fn get_local_detectors_configuration(path: &Path) -> Result<DetectorsConfiguration> {
    let detectors = DetectorsConfiguration {
        dependency: Dependency::parse(
            "library",
            None,
            SourceId::for_path(path)
                .with_context(|| format!("Failed to create SourceId for path: {:?}", path))?,
        )
        .with_context(|| "Failed to parse local detector dependency")?,
        path: None,
    };
    Ok(detectors)
}
