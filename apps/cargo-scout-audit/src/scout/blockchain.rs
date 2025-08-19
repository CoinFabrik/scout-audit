use anyhow::{bail, Result};
use cargo_metadata::Metadata;
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, process::Command};
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

    #[error("Failed to determine project toolchain: {0}")]
    ToolchainError(String),
}

impl BlockChain {
    pub fn variants() -> Vec<String> {
        Self::iter().map(|e| e.to_string()).collect()
    }

    pub fn get_detectors_path(&self) -> &str {
        match self {
            BlockChain::Ink => "ink",
            BlockChain::Soroban => "soroban",
            BlockChain::SubstratePallets => "substrate-pallets",
        }
    }

    fn get_project_toolchain(metadata: &Metadata) -> Result<Option<String>> {
        let output = Command::new("rustup")
            .current_dir(&metadata.workspace_root)
            .args(["show", "active-toolchain"])
            .output()
            .map_err(|e| BlockchainError::ToolchainError(e.to_string()))?;

        if !output.status.success() {
            return Ok(None);
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        // The output format is like "nightly-2024-07-11-aarch64-apple-darwin (default)"
        // We only want the nightly-YYYY-MM-DD part
        let toolchain = output_str
            .split_whitespace()
            .next()
            .and_then(|s| s.split('-').take(4).collect::<Vec<_>>().join("-").into());

        Ok(toolchain)
    }

    pub fn get_toolchain(&self, metadata: &Metadata) -> Result<String> {
        // First try to get the project's active toolchain
        if let Some(toolchain) = Self::get_project_toolchain(metadata)? {
            if toolchain.starts_with("nightly-") {
                return Ok(toolchain);
            }
        }

        // If no nightly toolchain found, use defaults based on blockchain
        let default_toolchain = match self {
            BlockChain::SubstratePallets => "nightly-2023-12-16",
            BlockChain::Soroban => "nightly-2025-08-07",
            _ => "nightly-2024-07-11",
        };

        Ok(default_toolchain.to_string())
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
