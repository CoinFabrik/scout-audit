use std::path::Path;

use anyhow::Result;
use cargo::{
    core::{Dependency, GitReference, SourceId},
    util::IntoUrl,
};
use scout_audit_internal::BlockChain;

#[derive(Debug, Clone)]
pub struct DetectorConfiguration {
    pub dependency: Dependency,
    pub path: Option<String>,
}

pub type DetectorsConfigurationList = Vec<DetectorConfiguration>;

/// Returns list of detectors.
pub fn get_detectors_configuration(dep: BlockChain) -> Result<DetectorsConfigurationList> {
    let path = Some(match dep {
        BlockChain::Ink => String::from("https://github.com/CoinFabrik/scout"),
        BlockChain::Soroban => String::from("https://github.com/CoinFabrik/scout-soroban"),
    });


    let detectors = vec![DetectorConfiguration {
        dependency: Dependency::parse(
            "library",
            None,
            SourceId::for_git(
                &path.unwrap().into_url()?,
                GitReference::DefaultBranch,
            )?,
        )?,
        path: Some("detectors".to_string()) ,
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
