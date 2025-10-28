use anyhow::{Result, bail};
use cargo_metadata::Metadata;
use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use std::{collections::HashSet, path::PathBuf, process::Command};
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};
use thiserror::Error;
use util::print::print_info;

#[derive(Error, Debug)]
pub enum BlockchainError {
    #[error("No supported dependency found in Cargo.toml.\n     → Supported dependencies:\n{0}")]
    UnsupportedDependency(String),

    #[error("Failed to determine project toolchain: {0}")]
    ToolchainError(String),
}

#[derive(Debug, Deserialize, Serialize, Copy, Clone, EnumIter, Display, EnumString)]
#[strum(serialize_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum BlockChain {
    Ink,
    Soroban,
    SubstratePallets,
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
        // The output format is like "nightly-2025-08-07-aarch64-apple-darwin (default)"
        // We only want the nightly-YYYY-MM-DD part
        let toolchain = output_str
            .split_whitespace()
            .next()
            .and_then(|s| s.split('-').take(4).collect::<Vec<_>>().join("-").into());

        Ok(toolchain)
    }

    pub fn get_toolchain(&self, metadata: &Metadata) -> Result<String> {
        // First try to get the project's active toolchain
        if let Some(toolchain) = Self::get_project_toolchain(metadata)?
            && toolchain.starts_with("nightly-")
        {
            return Ok(toolchain);
        }

        // If no nightly toolchain found, use defaults based on blockchain
        let default_toolchain = "nightly-2025-08-07";

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

#[derive(Debug, Parser)]
#[clap(display_name = "cargo")]
pub struct Cli {
    #[clap(subcommand)]
    pub subcmd: CargoSubCommand,
}

#[derive(Error, Debug)]
pub enum CliError {
    #[error("The output path cannot be a directory (Path: '{0}')")]
    OutputPathIsDirectory(PathBuf),

    #[error("The scout sources path does not exist (Path: '{0}')")]
    ScoutSourcesPathDoesNotExist(PathBuf),

    #[error("The scout sources path must be a directory (Path: '{0}')")]
    ScoutSourcesPathIsNotDirectory(PathBuf),

    #[error("Local detectors path does not exist (Path: '{0}')")]
    LocalDetectorsPathDoesNotExist(PathBuf),

    #[error("Local detectors path must be a directory (Path: '{0}')")]
    LocalDetectorsPathIsNotDirectory(PathBuf),

    #[error("Manifest path does not exist (Path: '{0}')")]
    ManifestPathDoesNotExist(PathBuf),

    #[error("Manifest path must be a valid Cargo.toml file (Path: '{0}')")]
    ManifestPathIsNotFile(PathBuf),
}

#[derive(Debug, Subcommand)]
pub enum CargoSubCommand {
    ScoutAudit(Scout),
}

#[derive(Debug, Default, Clone, ValueEnum, PartialEq, Serialize, Deserialize)]
pub enum OutputFormat {
    #[default]
    Html,
    Json,
    RawJson,
    RawSingleJson,
    UnfilteredJson,
    #[clap(name = "md")]
    Markdown,
    #[clap(name = "md-gh")]
    MarkdownGithub,
    Sarif,
    Pdf,
}

#[derive(Clone, Debug, Default, Parser, Serialize, Deserialize)]
#[command(author, version, about, long_about = None)]
pub struct Scout {
    #[clap(short, long, value_name = "PATH", help = "Path to Cargo.toml", value_hint = clap::ValueHint::FilePath)]
    pub manifest_path: Option<PathBuf>,

    // Exlude detectors
    #[clap(
        short,
        long,
        value_name = "DETECTORS",
        help = "Exclude specific detectors (comma-separated)",
        conflicts_with = "filter"
    )]
    pub exclude: Option<String>,

    // Filter by detectors
    #[clap(
        short,
        long,
        value_name = "DETECTORS",
        help = "Only run specified detectors (comma-separated)",
        conflicts_with = "exclude"
    )]
    pub filter: Option<String>,

    // List all the available detectors
    #[clap(short, long, help = "Display available detectors")]
    pub list_detectors: bool,

    #[clap(last = true, help = "Additional arguments passed to `cargo check`")]
    pub args: Vec<String>,

    #[clap(
        short,
        long,
        value_name = "FORMAT",
        help = "Output format(s) for the results",
        value_delimiter = ','
    )]
    pub output_format: Vec<OutputFormat>,

    #[clap(long, value_name = "PATH", help = "Path to save the output file", value_hint = clap::ValueHint::FilePath)]
    pub output_path: Option<PathBuf>,

    #[clap(long, value_name = "PATH", help = "Path to custom detectors workspace", value_hint = clap::ValueHint::DirPath)]
    pub local_detectors: Option<PathBuf>,

    #[clap(short, long, help = "Enable verbose output")]
    pub verbose: bool,

    #[clap(
        name = "toolchain",
        long,
        help = "Print the detectors current toolchain"
    )]
    pub toolchain: bool,

    #[clap(name = "metadata", long, help = "Show detector metadata information")]
    pub detectors_metadata: bool,

    #[clap(name = "debug", long, help = "Analyze the project in debug build")]
    pub debug: bool,

    #[clap(
        name = "src-hash",
        long,
        help = "Prints a hash of the sources at the time of build"
    )]
    pub src_hash: bool,

    #[clap(
        name = "cicd",
        long,
        help = "Report the analysis result via a file",
        value_hint = clap::ValueHint::FilePath,
    )]
    pub cicd: Option<PathBuf>,

    #[clap(
        long,
        value_name = "PATH",
        help = "Reuse an existing clone of the scout repository",
        value_hint = clap::ValueHint::DirPath
    )]
    pub scout_source: Option<PathBuf>,
}

impl Scout {
    pub fn prepare_args(&mut self, blockchain: BlockChain) {
        // Only add default target args if not a substrate-pallet project
        let is_substrate_pallet = matches!(blockchain, BlockChain::SubstratePallets);
        if !is_substrate_pallet && !self.args.iter().any(|x| x.contains("--target=")) {
            self.args.extend([
                "--target=wasm32-unknown-unknown".to_string(),
                "--no-default-features".to_string(),
                "-Zbuild-std=std,core,alloc".to_string(),
            ]);
        }

        if !self.debug {
            self.args.push("--release".to_string());
        }
    }

    pub fn validate(&self) -> Result<()> {
        print_info("Validating CLI arguments...");
        if let Some(path) = &self.output_path
            && path.is_dir()
        {
            bail!(CliError::OutputPathIsDirectory(path.clone()));
        }

        if let Some(path) = &self.scout_source {
            if !path.exists() {
                bail!(CliError::ScoutSourcesPathDoesNotExist(path.clone()));
            }
            if !path.is_dir() {
                bail!(CliError::ScoutSourcesPathIsNotDirectory(path.clone()));
            }
        }

        if let Some(path) = &self.local_detectors {
            if !path.exists() {
                bail!(CliError::LocalDetectorsPathDoesNotExist(path.clone()));
            }
            if !path.is_dir() {
                bail!(CliError::LocalDetectorsPathIsNotDirectory(path.clone()));
            }
        }

        if let Some(path) = &self.manifest_path {
            if !path.exists() {
                bail!(CliError::ManifestPathDoesNotExist(path.clone()));
            }
            if !path.is_file() {
                bail!(CliError::ManifestPathIsNotFile(path.clone()));
            }
        }

        Ok(())
    }

    pub fn get_fail_path(&self) -> Option<PathBuf> {
        self.cicd.as_ref().map(|path| path.join("FAIL"))
    }
}
