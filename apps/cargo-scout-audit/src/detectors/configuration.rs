use std::path::Path;

use anyhow::Result;
use cargo::{
    core::{Dependency, GitReference, SourceId},
    util::IntoUrl,
};
use crate::startup::BlockChain;

#[derive(Debug, Clone)]
pub struct DetectorConfiguration {
    pub dependency: Dependency,
    pub path: Option<String>,
}

pub type DetectorsConfigurationList = Vec<DetectorConfiguration>;

/// Returns list of detectors.
pub fn get_detectors_configuration(dep : BlockChain) -> Result<DetectorsConfigurationList> {

    let url_old = match dep {
        BlockChain::Ink => "https://github.com/CoinFabrik/scout",
        BlockChain::Soroban => "https://github.com/CoinFabrik/scout-soroban",
    };

    let url = "https://github.com/CoinFabrik/scout-audit";
    let path = Some(match dep {
        BlockChain::Ink => String::from("ink_detectors"),
        BlockChain::Soroban => String::from("soroban_detectors"),
    });

    let detectors = vec![DetectorConfiguration {
        dependency: Dependency::parse(
            "library",
            None,
            SourceId::for_git(
                &url.into_url()?,
                GitReference::DefaultBranch,
            )?,
        )?,
        path//: Some("detectors".into()),
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
