use std::path::Path;

use crate::{scout::blockchain::BlockChain, utils::print::print_warning};
use anyhow::{anyhow, Context, Result};
use cargo::{
    core::{Dependency, GitReference, SourceId},
    util::IntoUrl,
};
use reqwest::blocking::Client;

#[derive(Debug, Clone)]
pub struct DetectorsConfiguration {
    pub dependency: Dependency,
    pub path: Option<String>,
}

fn check_branch_exists(url: &str, branch: &str) -> Result<bool> {
    // Extract owner and repo from the URL
    let parts: Vec<&str> = url.trim_end_matches(".git").split('/').collect();
    let owner = parts[parts.len() - 2];
    let repo = parts[parts.len() - 1];

    let client = Client::new();
    let response = client
        .get(format!(
            "https://api.github.com/repos/{}/{}/branches/{}",
            owner, repo, branch
        ))
        .header("User-Agent", "scout")
        .send()?;

    Ok(response.status().is_success())
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
) -> Result<DetectorsConfiguration> {
    let toolchain = blockchain.get_toolchain();
    let scout_version = env!("CARGO_PKG_VERSION");
    let default_branch = format!("release/{}", scout_version);
    let fallback_branch = format!("release/{}-{}", scout_version, toolchain);

    let url = blockchain
        .get_detectors_url()
        .into_url()
        .with_context(|| format!("Failed to get URL for {} blockchain", blockchain))?;

    let branch = if check_branch_exists(url.as_str(), &default_branch)? {
        default_branch
    } else if check_branch_exists(url.as_str(), &fallback_branch)? {
        print_warning(&format!(
            "Could not find branch {} for detectors, falling back to {}",
            default_branch, fallback_branch
        ));
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
