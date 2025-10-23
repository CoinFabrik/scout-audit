use crate::{
    detector_helper::get_detectors_info as get_detectors_info_helped,
    digest,
    result::{ScoutError, ScoutResult},
    scout_driver::run_dylint,
};
use anyhow::{Context, Ok, Result, anyhow};
use cargo::{GlobalContext, core::Verbosity};
use cargo_metadata::Metadata;
use cli_args::{BlockChain, OutputFormat, Scout};
use config::ProfileConfig;
use scout::{
    detectors::{builder::DetectorBuilder, configuration::DetectorsConfiguration},
    finding::Finding,
    output::report::Report,
    scout::{
        findings::{get_crates, output_to_json, split_findings, temp_file_to_string},
        project_info::Project,
        telemetry::TelemetryClient,
        version_checker::VersionChecker,
    },
};
use serde_json::to_string_pretty;
use std::{collections::HashSet, io::Write, path::PathBuf};
use terminal_color_builder::OutputFormatter;
use util::{
    detectors::{get_excluded_detectors, get_filtered_detectors, list_detectors},
    detectors_info::LintStore,
    logger::TracedError,
    print::print_error,
};

#[allow(clippy::large_enum_variant)]
enum EitherInfoOrScoutResult {
    Info(RunInfo),
    ScoutResult(ScoutResult),
}

fn prepare_scout_input(opts: &mut Scout) -> Result<EitherInfoOrScoutResult> {
    opts.validate().map_err(ScoutError::ValidateFailed)?;

    if let Some(path) = opts.get_fail_path() {
        let _ = std::fs::File::create(path);
    }

    if opts.src_hash {
        println!("{}", digest::SOURCE_DIGEST);
        return Ok(EitherInfoOrScoutResult::ScoutResult(
            ScoutResult::from_string(digest::SOURCE_DIGEST),
        ));
    }

    let metadata =
        Project::get_metadata(&opts.manifest_path).map_err(ScoutError::MetadataFailed)?;
    let blockchain =
        BlockChain::get_blockchain_dependency(&metadata).map_err(ScoutError::BlockchainFailed)?;

    // Prepare the args after we know the Blockchain type
    opts.prepare_args(blockchain);

    let toolchain = blockchain.get_toolchain(&metadata)?;

    if opts.toolchain {
        println!("{}", toolchain);
        return Ok(EitherInfoOrScoutResult::ScoutResult(
            ScoutResult::from_string(toolchain),
        ));
    }

    // Send telemetry data
    let client_type = TelemetryClient::detect_client_type(opts);
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

    let detectors_config =
        DetectorsConfiguration::get(blockchain, &toolchain, &opts.local_detectors, &metadata)
            .map_err(ScoutError::DetectorsConfigFailed)?;

    // Instantiate detectors
    let detector_builder = DetectorBuilder::new(
        &cargo_config,
        &detectors_config,
        &metadata,
        opts.verbose,
        &toolchain,
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
        return Ok(EitherInfoOrScoutResult::ScoutResult(ScoutResult::default()));
    }

    let filtered_detectors = if let Some(filter) = &opts.filter {
        get_filtered_detectors(filter, &profile_detectors)?
    } else if let Some(excluded) = &opts.exclude {
        get_excluded_detectors(excluded, &profile_detectors)
    } else {
        profile_detectors
    };

    let detectors_paths = detector_builder
        .build(&filtered_detectors)
        .map_err(ScoutError::BuildDetectorsFailed)?;

    let detectors_info = get_detectors_info_helped(&toolchain, &detectors_paths)?;

    if opts.detectors_metadata {
        let metadata = to_string_pretty(&detectors_info).unwrap();
        println!("{}", metadata);
        return Ok(EitherInfoOrScoutResult::ScoutResult(
            ScoutResult::from_string(metadata),
        ));
    }

    let project_info = Project::get_info(&metadata).map_err(ScoutError::GetProjectInfoFailed)?;

    let inside_vscode = opts.args.contains(&"--message-format=json".to_string());

    Ok(EitherInfoOrScoutResult::Info(RunInfo {
        inside_vscode,
        project_info,
        metadata,
        filtered_detectors,
        detectors_info,
        detectors_paths,
        output_format,
        toolchain,
    }))
}

struct RunInfo {
    pub inside_vscode: bool,
    pub project_info: Project,
    pub metadata: Metadata,
    pub filtered_detectors: Vec<String>,
    pub detectors_info: LintStore,
    pub detectors_paths: Vec<PathBuf>,
    pub output_format: Vec<OutputFormat>,
    pub toolchain: String,
}

pub fn run_scout(mut opts: Scout) -> Result<ScoutResult> {
    let either = prepare_scout_input(&mut opts)?;

    let info = match either {
        EitherInfoOrScoutResult::Info(run_info) => run_info,
        EitherInfoOrScoutResult::ScoutResult(scout_result) => {
            return Ok(scout_result);
        }
    };

    let RunInfo {
        inside_vscode,
        project_info,
        metadata,
        filtered_detectors,
        detectors_info,
        detectors_paths,
        output_format,
        toolchain,
    } = info;

    // Run dylint
    let (_, stdout) = run_dylint(&toolchain, &detectors_paths, &opts, inside_vscode)
        .map_err(ScoutError::RunDylintFailed)?;

    let mut raw_findings_string = temp_file_to_string(&stdout)?;

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
        scout::scout::post_processing::process(
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
        scout::output::console::render_report(&console_findings, &crates, &detectors_info)?;
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

    if let Some(path) = opts.get_fail_path()
        && console_findings.is_empty()
    {
        let _ = std::fs::remove_file(path);
    }

    Ok(ScoutResult::new(console_findings, output_string_vscode))
}

fn set_severity(
    raw_findings_string: &mut String,
    raw_findings: &[Finding],
    detector_names: &HashSet<String>,
    detectors_info: &LintStore,
) -> Vec<Finding> {
    for finding in raw_findings.iter() {
        if finding.is_scout_finding(detector_names)
            && let Some(detector) = detectors_info.find_by_id(&finding.code())
        {
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
    output_to_json(raw_findings_string)
        .into_iter()
        .map(Finding::new)
        .collect::<Vec<_>>()
}
