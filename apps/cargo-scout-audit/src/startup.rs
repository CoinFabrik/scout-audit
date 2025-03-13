use crate::{
    cli::Scout,
    detectors::{builder::DetectorBuilder, configuration::DetectorsConfiguration},
    digest,
    finding::Finding,
    output::report::Report,
    scout::{
        blockchain::BlockChain,
        findings::{get_crates, output_to_json, split_findings, temp_file_to_string},
        nightly_runner::run_scout_in_nightly,
        project_info::Project,
        telemetry::TelemetryClient,
        version_checker::VersionChecker,
    },
    utils::{
        config::ProfileConfig,
        detectors::{get_excluded_detectors, get_filtered_detectors, list_detectors},
        detectors_info::{get_detectors_info, LintStore},
        logger::TracedError,
        print::{print_error, print_info},
    },
};
use anyhow::{anyhow, Context, Ok, Result};
use cargo::{core::Verbosity, GlobalContext};
use dylint::opts::{Check, Dylint, LibrarySelection, Operation};
use serde_json::to_string_pretty;
use std::{collections::HashSet, io::Write, path::PathBuf};
use tempfile::NamedTempFile;
use terminal_color_builder::OutputFormatter;
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

fn set_severity(
    raw_findings_string: &mut String,
    raw_findings: &[Finding],
    detector_names: &HashSet<String>,
    detectors_info: &LintStore,
) -> Vec<Finding> {
    for finding in raw_findings.iter() {
        if finding.is_scout_finding(detector_names) {
            if let Some(detector) = detectors_info.find_by_id(&finding.code()) {
                let val = finding.message();
                let severity_tag = format!("[{}]", detector.severity.to_uppercase());
                if raw_findings_string.contains(&val)
                    && !raw_findings_string.contains(&format!("{} {}", severity_tag, val))
                {
                    *raw_findings_string =
                        raw_findings_string.replace(&val, &format!("{} {}", severity_tag, val));
                }
            }
        }
    }
    output_to_json(raw_findings_string)
        .into_iter()
        .map(Finding::new)
        .collect::<Vec<_>>()
}

#[tracing::instrument(name = "RUN SCOUT", skip_all)]
pub fn run_scout(mut opts: Scout) -> Result<ScoutResult> {
    opts.validate().map_err(ScoutError::ValidateFailed)?;

    if let Some(path) = opts.get_fail_path() {
        let _ = std::fs::File::create(path);
    }

    if opts.src_hash {
        println!("{}", digest::SOURCE_DIGEST);
        return Ok(ScoutResult::from_string(digest::SOURCE_DIGEST));
    }

    let metadata =
        Project::get_metadata(&opts.manifest_path).map_err(ScoutError::MetadataFailed)?;
    let blockchain =
        BlockChain::get_blockchain_dependency(&metadata).map_err(ScoutError::BlockchainFailed)?;

    // Prepare the args after we know the Blockchain type
    opts.prepare_args(blockchain);

    let toolchain = blockchain.get_toolchain();

    if opts.toolchain {
        println!("{}", toolchain);
        return Ok(ScoutResult::from_string(toolchain));
    }

    if let Some(mut child) = run_scout_in_nightly(toolchain)? {
        child
            .wait()
            .with_context(|| "Failed to wait for nightly child process")?;
        return Ok(ScoutResult::default());
    }

    // Send telemetry data
    let client_type = TelemetryClient::detect_client_type(&opts);
    let telemetry_client = TelemetryClient::new(blockchain, client_type);
    let _ = telemetry_client.send_report();

    if let Err(e) = VersionChecker::new().check_for_updates() {
        // This is not a critical error, so we don't need to bail and we don't need a ScoutError
        print_error(&format!(
            "Failed to check for scout updates.\n     → Caused by: {}",
            e
        ));
    }

    let cargo_config = GlobalContext::default().map_err(ScoutError::CargoConfigFailed.traced())?;
    cargo_config.shell().set_verbosity(if opts.verbose {
        Verbosity::Verbose
    } else {
        Verbosity::Quiet
    });

    let detectors_config = DetectorsConfiguration::get(blockchain, &opts.local_detectors)
        .map_err(ScoutError::DetectorsConfigFailed)?;

    // Instantiate detectors
    let detector_builder = DetectorBuilder::new(
        &cargo_config,
        &detectors_config,
        &metadata,
        opts.verbose,
        toolchain,
    );

    let detectors_names = detector_builder
        .get_detector_names()
        .map_err(ScoutError::GetDetectorNamesFailed)?;

    let profile_config =
        ProfileConfig::new(blockchain, detectors_names, opts.output_format.clone())
            .get_config(&metadata)?;

    let profile_detectors = profile_config.detector_names;
    let output_format = profile_config.output_format;

    if opts.list_detectors {
        list_detectors(&profile_detectors);
        return Ok(ScoutResult::default());
    }

    let filtered_detectors = if let Some(filter) = &opts.filter {
        get_filtered_detectors(filter, &profile_detectors)?
    } else if let Some(excluded) = &opts.exclude {
        get_excluded_detectors(excluded, &profile_detectors)
    } else {
        profile_detectors
    };

    let detectors_paths = detector_builder
        .build(&blockchain, &filtered_detectors)
        .map_err(ScoutError::BuildDetectorsFailed)?;

    let detectors_info = get_detectors_info(&detectors_paths)?;

    if opts.detectors_metadata {
        let metadata = to_string_pretty(&detectors_info).unwrap();
        println!("{}", metadata);
        return Ok(ScoutResult::from_string(metadata));
    }

    let project_info = Project::get_info(&metadata).map_err(ScoutError::GetProjectInfoFailed)?;

    let inside_vscode = opts.args.contains(&"--message-format=json".to_string());

    // Run dylint
    let (_, stdout) = run_dylint(detectors_paths.clone(), &opts, inside_vscode)
        .map_err(ScoutError::RunDylintFailed)?;

    let mut raw_findings_string = temp_file_to_string(stdout)?;

    let raw_findings = output_to_json(&raw_findings_string)
        .into_iter()
        .map(Finding::new)
        .collect::<Vec<_>>();
    let crates = get_crates(&raw_findings, &project_info.packages, &metadata)?;
    let detector_names = HashSet::from_iter(filtered_detectors.iter().cloned());

    let raw_findings = set_severity(
        &mut raw_findings_string,
        &raw_findings,
        &detector_names,
        &detectors_info,
    );

    let findings = raw_findings
        .iter()
        .filter(|&x| x.is_scout_finding(&detector_names))
        .cloned()
        .collect::<Vec<_>>();

    if crates.is_empty() && !inside_vscode {
        let string = OutputFormatter::new()
            .fg()
            .red()
            .text_str("Nothing was analyzed. Check your build system for errors.")
            .print();
        println!("{}", string);
        return Err(anyhow!(
            "Nothing was analyzed. Check your build system for errors."
        ));
    }

    let (successful_findings, _failed_findings) = split_findings(&findings, &crates);

    // Get the path of the 'unnecessary_lint_allow' detector
    let unnecessary_lint_allow_path = detectors_paths.iter().find_map(|path| {
        path.to_str()
            .filter(|s| s.contains("unnecessary_lint_allow"))
            .map(|_| path)
    });

    // Create and run post processor if the path is found, otherwise use default values
    let (console_findings, output_string_vscode) = if unnecessary_lint_allow_path.is_some() {
        crate::scout::post_processing::process(
            successful_findings.clone(),
            raw_findings.clone(),
            inside_vscode,
        )
    } else {
        (successful_findings, raw_findings_string)
    };

    // Generate report
    if inside_vscode {
        std::io::stdout()
            .lock()
            .write_all(output_string_vscode.as_bytes())
            .with_context(|| "Failed to write stdout content")?;
    } else {
        crate::output::console::render_report(&console_findings, &crates, &detectors_info)?;
        Report::generate(
            &console_findings,
            raw_findings,
            &crates,
            project_info,
            &detectors_info,
            opts.output_path.clone(),
            &output_format,
        )?;
    }

    if let Some(path) = opts.get_fail_path() {
        if console_findings.is_empty() {
            let _ = std::fs::remove_file(path);
        }
    }

    Ok(ScoutResult::new(console_findings, output_string_vscode))
}

#[tracing::instrument(name = "RUN DYLINT", skip_all)]
fn run_dylint(
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
