use anyhow::{bail, Result};
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;
use thiserror::Error;

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

#[derive(Debug, Default, Clone, ValueEnum, PartialEq)]
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

#[derive(Clone, Debug, Default, Parser)]
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
        conflicts_with = "exclude",
        conflicts_with = "profile"
    )]
    pub filter: Option<String>,

    // Select profiles in configuration
    #[clap(
        short,
        long,
        value_name = "PROFILE",
        help = "Use a predefined detector profile",
        conflicts_with = "filter"
    )]
    pub profile: Option<String>,

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
        help = "Report the analysis result via the status code"
    )]
    pub cicd: bool,
}

impl Scout {
    pub fn prepare_args(&mut self) {
        if !self.args.iter().any(|x| x.contains("--target=")) {
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
        if let Some(path) = &self.output_path {
            if path.is_dir() {
                bail!(CliError::OutputPathIsDirectory(path.clone()));
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
}
