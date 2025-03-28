use crate::build_config::TOOLCHAIN;
use anyhow::{bail, Result};
use cargo_metadata::Metadata;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};
use thiserror::Error;

#[derive(Debug, Deserialize, Serialize, Copy, Clone, EnumIter, Display, EnumString)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum BlockChain {
    Ink,
    Soroban,
    SubstratePallets,
}

#[derive(Error, Debug)]
pub enum BlockchainError {
    #[error("No supported dependency found in Cargo.toml.\n     → Supported dependencies:\n{0}")]
    UnsupportedDependency(String),
}

impl BlockChain {
    pub fn variants() -> Vec<String> {
        Self::iter().map(|e| e.to_string()).collect()
    }

    pub fn get_detectors_path(&self) -> &str {
        match self {
            BlockChain::Ink => "detectors/ink",
            BlockChain::Soroban => "detectors/soroban",
            BlockChain::SubstratePallets => "detectors/substrate-pallets",
        }
    }

    pub fn get_toolchain(&self) -> &str {
        match self {
            BlockChain::Ink => TOOLCHAIN,
            BlockChain::Soroban => TOOLCHAIN,
            BlockChain::SubstratePallets => TOOLCHAIN,
        }
    }

    fn get_immediate_dependencies(metadata: &Metadata) -> HashSet<String> {
        let mut ret = HashSet::<String>::new();
        let root_packages = metadata
            .workspace_members
            .iter()
            .filter_map(|x| metadata.packages.iter().find(|p| p.id == *x));
        for package in root_packages {
            for dep in package.dependencies.iter() {
                ret.insert(dep.name.clone());
            }
        }
        ret
    }

    #[tracing::instrument(name = "GET BLOCKCHAIN DEPENDENCY", level = "debug", skip_all)]
    pub fn get_blockchain_dependency(metadata: &Metadata) -> Result<Self> {
        let immediate_dependencies = Self::get_immediate_dependencies(metadata);
        if immediate_dependencies.contains("soroban-sdk") {
            Ok(BlockChain::Soroban)
        } else if immediate_dependencies.contains("ink") {
            Ok(BlockChain::Ink)
        } else if immediate_dependencies.contains("frame-system") {
            Ok(BlockChain::SubstratePallets)
        } else {
            let supported_dependencies = BlockChain::variants()
                .into_iter()
                .map(|chain| format!("        - {}", chain))
                .collect::<Vec<_>>()
                .join("\n");
            bail!(BlockchainError::UnsupportedDependency(
                supported_dependencies,
            ));
        }
    }
}
