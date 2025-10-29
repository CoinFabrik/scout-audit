use cli_args::Scout;
use scout::finding::Finding;
use util::print::print_info;

use anyhow::{Context, Ok, Result};
use dylint::opts::{Check, Dylint, LibrarySelection, Operation};
use std::path::PathBuf;
use tempfile::NamedTempFile;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ScoutError {
    #[error("Failed to validate CLI options:\n     → {0}")]
    ValidateFailed(#[source] anyhow::Error),

    #[error("Failed to get project metadata:\n     → {0}")]
    MetadataFailed(#[source] anyhow::Error),

    #[error("Failed to get blockchain dependency:\n     → {0}")]
    BlockchainFailed(#[source] anyhow::Error),

    #[error("Failed to create default cargo configuration")]
    CargoConfigFailed,

    #[error("Failed to get detectors configuration:\n     → {0}")]
    DetectorsConfigFailed(#[source] anyhow::Error),

    #[error("Failed to get detector names:\n     → {0}")]
    GetDetectorNamesFailed(#[source] anyhow::Error),

    #[error("Failed to build detectors:\n     → {0}")]
    BuildDetectorsFailed(#[source] anyhow::Error),

    #[error("Failed to get project info:\n     → {0}")]
    GetProjectInfoFailed(#[source] anyhow::Error),

    #[error("Failed to run dylint:\n     → {0}")]
    RunDylintFailed(#[source] anyhow::Error),
}

#[derive(Default)]
pub struct ScoutResult {
    pub findings: Vec<Finding>,
    pub stdout_helper: String,
}

impl ScoutResult {
    pub fn new(findings: Vec<Finding>, stdout_helper: String) -> Self {
        Self {
            findings,
            stdout_helper,
        }
    }
    pub fn from_stdout(stdout_helper: String) -> Self {
        Self {
            findings: Vec::new(),
            stdout_helper,
        }
    }
    pub fn from_string<T: std::fmt::Display>(s: T) -> Self {
        Self::from_stdout(format!("{}\n", s))
    }
}

pub fn run_dylint(
    detectors_paths: Vec<PathBuf>,
    opts: &Scout,
    inside_vscode: bool,
) -> Result<(bool, NamedTempFile)> {
    print_info("Running scout...");

    // Convert detectors paths to string
    let detectors_paths: Vec<String> = detectors_paths
        .iter()
        .map(|path| path.to_string_lossy().into_owned())
        .collect();

    // Initialize temporary file for stdout
    let stdout_temp_file =
        NamedTempFile::new().with_context(|| "Failed to create stdout temporary file")?;
    let pipe_stdout = Some(stdout_temp_file.path().to_string_lossy().into_owned());

    // Get the manifest path
    let manifest_path = opts
        .manifest_path
        .as_ref()
        .map(|p| p.to_string_lossy().into_owned());

    let mut args = opts.args.to_owned();
    if !inside_vscode {
        args.push("--message-format=json-diagnostic-rendered-ansi".to_string());
    }

    let check_opts = Check {
        lib_sel: LibrarySelection {
            manifest_path,
            lib_paths: detectors_paths,
            ..Default::default()
        },
        no_deps: true,
        args,
        ..Default::default()
    };

    let options = Dylint {
        pipe_stdout,
        quiet: opts.verbose,
        operation: Operation::Check(check_opts.clone()),
        ..Default::default()
    };

    let success = dylint::run(&options).is_err();

    Ok((success, stdout_temp_file))
}
