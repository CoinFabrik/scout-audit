use std::path::Path;

use anyhow::{Context, Result};
use cargo::{
    core::{Dependency, GitReference, SourceId},
    util::IntoUrl,
};

use crate::scout::blockchain::BlockChain;

#[derive(Debug, Clone)]
pub struct DetectorConfiguration {
    pub dependency: Dependency,
    pub path: Option<String>,
}

pub type DetectorsConfigurationList = Vec<DetectorConfiguration>;

/// Returns list of detectors.
pub fn get_detectors_configuration(blockchain: BlockChain) -> Result<DetectorsConfigurationList> {
    let dependency = Dependency::parse(
        "library",
        None,
        SourceId::for_git(
            &blockchain
                .get_detectors_url()
                .into_url()
                .with_context(|| format!("Failed to get URL for {} blockchain", blockchain))?,
            GitReference::DefaultBranch,
        )?,
    )?;

    let detectors = vec![DetectorConfiguration {
        dependency,
        path: Some("detectors".to_string()),
    }];

    Ok(detectors)
}

/// Returns local detectors configuration from custom path.
pub fn get_local_detectors_configuration(path: &Path) -> Result<DetectorsConfigurationList> {
    let detectors = vec![DetectorConfiguration {
        dependency: Dependency::parse(
            "library",
            None,
            SourceId::for_path(path)
                .with_context(|| format!("Failed to create SourceId for path: {:?}", path))?,
        )
        .with_context(|| "Failed to parse local detector dependency")?,
        path: None,
    }];
    Ok(detectors)
}
