use std::path::Path;

use anyhow::Result;
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
pub fn get_detectors_configuration(dep: BlockChain) -> Result<DetectorsConfigurationList> {
    let dependency = Dependency::parse(
        "library",
        None,
        match dep {
            BlockChain::Ink => SourceId::for_git(
                &"https://github.com/CoinFabrik/scout".into_url()?,
                GitReference::DefaultBranch,
            )?,
            BlockChain::Soroban => SourceId::for_git(
                &"https://github.com/CoinFabrik/scout-soroban".into_url()?,
                GitReference::DefaultBranch,
            )?,
        },
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
        dependency: Dependency::parse("library", None, SourceId::for_path(path)?)?,
        path: None,
    }];
    Ok(detectors)
}
