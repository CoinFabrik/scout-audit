use crate::{
    cli::Scout,
    detectors::{
        builder::DetectorBuilder,
        configuration::{get_local_detectors_configuration, get_remote_detectors_configuration},
    },
    digest,
    finding::Finding,
    output::report::Report,
    scout::{
        blockchain::BlockChain,
        findings::{get_crates, output_to_json, split_findings, temp_file_to_string},
        nightly_runner::run_scout_in_nightly,
        project_info::ProjectInfo,
        version_checker::VersionChecker,
    },
    utils::{
        config::{open_config_and_sync_detectors, profile_enabled_detectors},
        detectors::{get_excluded_detectors, get_filtered_detectors, list_detectors},
        detectors_info::get_detectors_info,
        print::{print_error, print_info, print_warning},
    },
};
use anyhow::{anyhow, bail, Context, Ok, Result};
use cargo::{core::Verbosity, GlobalContext};
use cargo_metadata::{Metadata, MetadataCommand};
use dylint::opts::{Check, Dylint, LibrarySelection, Operation};
use serde_json::to_string_pretty;
use std::{collections::HashSet, fs, io::Write, path::PathBuf};
use tempfile::NamedTempFile;
use terminal_color_builder::OutputFormatter;

fn get_project_metadata(manifest_path: &Option<PathBuf>) -> Result<Metadata> {
    let mut metadata_command = MetadataCommand::new();

    if let Some(manifest_path) = manifest_path {
        if !manifest_path.ends_with("Cargo.toml") {
            bail!(
                "Invalid manifest path, ensure scout is being run in a Rust project, and the path is set to the Cargo.toml file.\n     → Manifest path: {:?}",
                manifest_path
            );
        }

        fs::metadata(manifest_path).context(format!(
            "Cargo.toml file not found, ensure the path is a valid file path.\n     → Manifest path: {:?}",
            manifest_path
        ))?;

        metadata_command.manifest_path(manifest_path);
    }

    metadata_command
        .exec()
        .map_err(|e| {
            anyhow!("Failed to execute metadata command on this path, ensure this is a valid rust project or workspace directory.\n\n     → Caused by: {}", e.to_string())})
}

#[tracing::instrument(name = "RUN SCOUT", skip_all)]
pub fn run_scout(mut opts: Scout) -> Result<Vec<Finding>> {
    opts.validate()?;
    opts.prepare_args();

    if opts.src_hash {
        println!("{}", digest::SOURCE_DIGEST);
        return Ok(vec![]);
    }

    let metadata = get_project_metadata(&opts.manifest_path)?;
    let blockchain = BlockChain::get_blockchain_dependency(&metadata)?;
    let toolchain = blockchain.get_toolchain();

    if opts.toolchain {
        println!("{}", toolchain);
        return Ok(vec![]);
    }

    if let Some(mut child) = run_scout_in_nightly(toolchain)? {
        child
            .wait()
            .with_context(|| "Failed to wait for nightly child process")?;
        return Ok(vec![]);
    }

    if let Err(e) = VersionChecker::new().check_for_updates() {
        print_error(&format!(
            "Failed to check for scout updates.\n\n     → Caused by: {}",
            e
        ));
    }

    let cargo_config =
        GlobalContext::default().with_context(|| "Failed to create default cargo configuration")?;
    cargo_config.shell().set_verbosity(if opts.verbose {
        Verbosity::Verbose
    } else {
        Verbosity::Quiet
    });

    let detectors_config = match &opts.local_detectors {
        Some(path) => get_local_detectors_configuration(path, blockchain).map_err(|e| {
            anyhow!(
                "Failed to get local detectors configuration.\n\n     → Caused by: {}",
                e
            )
        })?,
        None => get_remote_detectors_configuration(blockchain).map_err(|e| {
            anyhow!(
                "Failed to get remote detectors configuration.\n\n     → Caused by: {}",
                e
            )
        })?,
    };

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
        .map_err(|e| anyhow!("Failed to get detector names.\n\n     → Caused by: {}", e))?;

    let profile_detectors = match &opts.profile {
        Some(profile) => {
            let (config, config_path) =
                open_config_and_sync_detectors(blockchain, &detectors_names).map_err(|err| {
                    anyhow!(
                    "Failed to open and synchronize configuration file.\n\n     → Caused by: {}",
                    err
                )
                })?;

            print_warning(&format!(
                "Using profile '{}' to filter detectors. To edit this profile, open the configuration file at: {}",
                profile,
                config_path.display()
            ));

            profile_enabled_detectors(&config, profile, &config_path, &detectors_names)?
        }
        None => detectors_names.clone(),
    };

    if opts.list_detectors {
        list_detectors(&profile_detectors);
        return Ok(vec![]);
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
        .map_err(|e| {
            anyhow!(
                "Failed to build detectors.\n\n     → Caused by: {}",
                e.to_string()
            )
        })?;

    let detectors_info = get_detectors_info(&detectors_paths)?;

    if opts.detectors_metadata {
        let json = to_string_pretty(&detectors_info);
        println!("{}", json.unwrap());
        return Ok(vec![]);
    }

    let project_info = ProjectInfo::get_project_info(&metadata)
        .map_err(|err| anyhow!("Failed to get project info.\n\n     → Caused by: {}", err))?;

    let inside_vscode = opts.args.contains(&"--message-format=json".to_string());

    // Run dylint
    let (_successful_build, stdout) = run_dylint(detectors_paths.clone(), &opts, inside_vscode)
        .map_err(|err| anyhow!("Failed to run dylint.\n\n     → Caused by: {}", err))?;

    let raw_findings_string = temp_file_to_string(stdout)?;
    let raw_findings = output_to_json(&raw_findings_string)
        .into_iter()
        .map(Finding::new)
        .collect::<Vec<_>>();
    let crates = get_crates(&raw_findings, &project_info.packages);
    let detector_names = HashSet::from_iter(filtered_detectors.iter().cloned());
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
        return Ok(vec![]);
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
            .with_context(|| ("Failed to write stdout content"))?;
    } else {
        crate::output::console::render_report(&console_findings, &crates, &detectors_info)?;
        Report::generate(
            &console_findings,
            raw_findings,
            &crates,
            project_info,
            &detectors_info,
            opts.output_path,
            &opts.output_format,
        )?;
    }

    Ok(console_findings)
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
        NamedTempFile::new().with_context(|| ("Failed to create stdout temporary file"))?;
    let pipe_stdout = Some(stdout_temp_file.path().to_string_lossy().into_owned());

    // Get the manifest path
    let manifest_path = opts
        .manifest_path
        .as_ref()
        .map(|p| p.to_string_lossy().into_owned());

    let mut args = opts.args.to_owned();
    if !inside_vscode {
        args.push("--message-format=json".to_string());
    }

    let check_opts = Check {
        lib_sel: LibrarySelection {
            manifest_path,
            lib_paths: detectors_paths,
            ..Default::default()
        },
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
