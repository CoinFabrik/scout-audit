use anyhow::{Context, Result};
use cargo_scout_audit::{cli_args::Scout, scout::finding::Finding, util::print::print_info};
use dylint::opts::{Check, Dylint, LibrarySelection, Operation};
use std::path::{Path, PathBuf};
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

    // Soroban SDK 28 requires build systems targeting Wasm to acknowledge
    // spec-shaking v2 support. Scout only runs `cargo check` for static
    // analysis and does not emit a deployable Wasm artifact, so no subsequent
    // spec-shaking step is needed; setting the marker allows the SDK's build
    // script to proceed during analysis.
    if opts.args.iter().any(|arg| arg == "--target=wasm32v1-none") {
        std::env::set_var("SOROBAN_SDK_BUILD_SYSTEM_SUPPORTS_SPEC_SHAKING_V2", "1");
    }

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

    let success = dylint_succeeded(dylint::run(&options), stdout_temp_file.path())?;

    Ok((success, stdout_temp_file))
}

fn dylint_succeeded(result: Result<()>, stdout_path: &Path) -> Result<bool> {
    match result {
        Ok(()) => Ok(true),
        // Cargo emits analyzed-target compilation errors as JSON on the piped
        // stdout. Keep that file alive so the parent process can report them.
        Err(_) if contains_compiler_error(stdout_path) => Ok(false),
        Err(error) => Err(ScoutError::RunDylintFailed(error).into()),
    }
}

fn contains_compiler_error(stdout_path: &Path) -> bool {
    let Ok(output) = std::fs::read_to_string(stdout_path) else {
        return false;
    };

    output.lines().any(|line| {
        serde_json::from_str(line)
            .map(Finding::new)
            .is_ok_and(|finding| finding.is_compiler_error())
    })
}

#[cfg(test)]
mod tests {
    use super::dylint_succeeded;
    use anyhow::anyhow;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn preserves_target_compilation_failures_for_reporting() {
        let mut stdout = NamedTempFile::new().unwrap();
        writeln!(
            stdout,
            r#"{{"reason":"compiler-message","message":{{"level":"error"}}}}"#
        )
        .unwrap();

        let result = dylint_succeeded(Err(anyhow!("cargo check failed")), stdout.path());

        assert!(!result.unwrap());
    }

    #[test]
    fn propagates_dylint_failures_without_compiler_errors() {
        let stdout = NamedTempFile::new().unwrap();
        let result = dylint_succeeded(Err(anyhow!("synthetic dylint failure")), stdout.path());

        let error = match result {
            Ok(_) => panic!("Dylint failure was reported as success"),
            Err(error) => error,
        };
        let message = format!("{error:#}");

        assert!(message.contains("Failed to run dylint"));
        assert!(message.contains("synthetic dylint failure"));
    }
}
