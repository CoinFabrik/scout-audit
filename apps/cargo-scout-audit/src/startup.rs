use crate::{
    detectors::{
        builder::DetectorBuilder,
        configuration::{get_local_detectors_configuration, get_remote_detectors_configuration},
    },
    digest,
    output::raw_report::RawReport,
    scout::{
        blockchain::BlockChain, nightly_runner::run_scout_in_nightly,
        project_info::ProjectInfo,
        version_checker::VersionChecker,
    },
    utils::{
        config::{open_config_and_sync_detectors, profile_enabled_detectors},
        detectors::{get_excluded_detectors, get_filtered_detectors, list_detectors},
        detectors_info::{get_detectors_info, LintStore},
        print::{print_error, print_warning},
    },
    finding::Finding,
};
use anyhow::{anyhow, bail, Context, Ok, Result};
use cargo::{core::Verbosity, GlobalContext};
use cargo_metadata::{Metadata, MetadataCommand};
use clap::{Parser, Subcommand, ValueEnum};
use dylint::opts::{Check, Dylint, LibrarySelection, Operation};
use serde_json::{from_str, to_string_pretty, Value};
use std::{
    collections::{HashMap, HashSet}, fs, io::Write, path::PathBuf
};
use tempfile::NamedTempFile;
use terminal_color_builder::OutputFormatter;

#[derive(Debug, Parser)]
#[clap(display_name = "cargo")]
pub struct Cli {
    #[clap(subcommand)]
    pub subcmd: CargoSubCommand,
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
    #[clap(short, long, value_name = "path", help = "Path to Cargo.toml.")]
    pub manifest_path: Option<PathBuf>,

    // Exlude detectors
    #[clap(
        short,
        long,
        value_name = "detector/s",
        help = "Exclude the given detectors, separated by commas."
    )]
    pub exclude: Option<String>,

    // Filter by detectors
    #[clap(
        short,
        long,
        value_name = "detector/s",
        help = "Filter by the given detectors, separated by commas."
    )]
    pub filter: Option<String>,

    // Select profiles in configuration
    #[clap(
        short,
        long,
        value_name = "profile",
        help = "Filter detectors using profiles."
    )]
    pub profile: Option<String>,

    // List all the available detectors
    #[clap(short, long, help = "List all the available detectors")]
    pub list_detectors: bool,

    #[clap(last = true, help = "Arguments for `cargo check`.")]
    pub args: Vec<String>,

    #[clap(
        short,
        long,
        value_name = "type",
        help = "Set the output type",
        value_delimiter = ','
    )]
    pub output_format: Vec<OutputFormat>,

    #[clap(long, value_name = "path", help = "Path to the output file.")]
    pub output_path: Option<PathBuf>,

    #[clap(long, value_name = "path", help = "Path to detectors workspace.")]
    pub local_detectors: Option<PathBuf>,

    #[clap(
        long,
        help = "Force fallback to secondary detectors branch.",
        default_value_t = false
    )]
    pub force_fallback: bool,

    #[clap(
        short,
        long,
        help = "Print detectors metadata",
        default_value_t = false
    )]
    pub verbose: bool,

    #[clap(
        name = "toolchain",
        long,
        help = "Print the detectors current toolchain",
        default_value_t = false
    )]
    pub toolchain: bool,

    #[clap(
        name = "metadata",
        long,
        help = "Print metadata information",
        default_value_t = false
    )]
    pub detectors_metadata: bool,

    #[clap(
        name = "debug",
        long,
        help = "Analyze the project in debug build",
        default_value_t = false
    )]
    pub debug: bool,

    #[clap(
        name = "src-hash",
        long,
        help = "Prints a hash of the sources at the time of build",
        default_value_t = false
    )]
    pub src_hash: bool,
}

impl Scout {
    fn prepare_args(&mut self) {
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

    fn validate(&self) -> Result<()> {
        if self.filter.is_some() && self.exclude.is_some() {
            bail!("The flags `--filter` and `--exclude` can't be used together");
        }
        if self.filter.is_some() && self.profile.is_some() {
            bail!("The flags `--filter` and `--profile` can't be used together");
        }
        if let Some(path) = &self.output_path {
            if path.is_dir() {
                bail!("The output path can't be a directory");
            }
        }
        Ok(())
    }
}

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

pub fn temp_file_to_string(mut file: NamedTempFile) -> Result<String> {
    let mut ret = String::new();
    std::io::Read::read_to_string(&mut file, &mut ret)?;
    let _ = file.close();
    Ok(ret)
}

pub fn output_to_json(output: &str) -> Vec<Value> {
    output
        .lines()
        .map(|line| from_str::<Value>(line).unwrap())
        .collect::<Vec<Value>>()
}

//In some cases, rustc (or dylint, or clipply, or whoever) has returned the
//package name where it should be returning the crate name. If you run into
//problems in the future, try removing the call to this function.
pub fn normalize_crate_name(s: &str) -> String {
    s.replace("-", "_")
}

fn get_crates_from_output(output: &Vec<Finding>) -> HashMap<String, bool> {
    let mut ret = HashMap::<String, bool>::new();

    for finding in output {
        let reason = finding.reason();
        if reason != "compiler-message"{
            continue;
        }
        let name = finding.package();
        if name.is_empty(){
            continue;
        }
        if let Some(previous) = ret.get(&name) {
            if !previous {
                continue;
            }
        }
        ret.insert(name, !finding.is_compiler_error());
    }

    ret
}

fn get_crates(
    findings: &Vec<Finding>,
    packages: &[crate::output::report::Package],
) -> HashMap<String, bool> {
    let mut ret = HashMap::<String, bool>::new();
    for package in packages.iter() {
        ret.insert(normalize_crate_name(&package.name), true);
    }
    for (name, ok) in get_crates_from_output(findings).iter() {
        if ret.contains_key(name) {
            ret.insert(name.clone(), *ok);
        }
    }

    ret
}

fn split_findings(
    findings: &Vec<Finding>,
    crates: &HashMap<String, bool>,
) -> (Vec<Finding>, Vec<Finding>) {
    let mut successful_findings = Vec::<Finding>::new();
    let mut failed_findings = Vec::<Finding>::new();

    for finding in findings.iter() {
        let krate = finding.krate();
        if krate.is_empty(){
            continue;
        }
        if *crates.get(&krate).unwrap_or(&true) {
            &mut successful_findings
        } else {
            &mut failed_findings
        }
        .push(finding.clone());
    }

    (successful_findings, failed_findings)
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
            "Failed to check for updates.\n\n     → Caused by: {}",
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
        Some(path) => get_local_detectors_configuration(&PathBuf::from(path)).map_err(|e| {
            anyhow!(
                "Failed to get local detectors configuration.\n\n     → Caused by: {}",
                e
            )
        })?,
        None => {
            get_remote_detectors_configuration(blockchain, opts.force_fallback).map_err(|e| {
                anyhow!(
                    "Failed to get remote detectors configuration.\n\n     → Caused by: {}",
                    e
                )
            })?
        }
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
        .map(|x| Finding::new(x))
        .collect::<Vec<_>>();
    let crates = get_crates(&raw_findings, &project_info.packages);
    let detector_names = HashSet::from_iter(filtered_detectors.iter().cloned());
    let findings = raw_findings
        .iter()
        .cloned()
        .filter(|x| x.is_scout_finding(&detector_names))
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
    do_report(
        &console_findings,
        raw_findings,
        crates,
        project_info,
        detectors_info,
        output_string_vscode,
        opts,
        inside_vscode,
    )?;

    Ok(console_findings)
}

fn do_report(
    findings: &Vec<Finding>,
    raw_findings: Vec<Finding>,
    crates: HashMap<String, bool>,
    project_info: ProjectInfo,
    detectors_info: LintStore,
    output_string: String,
    opts: Scout,
    inside_vscode: bool,
) -> Result<()> {
    if inside_vscode {
        std::io::stdout()
            .lock()
            .write_all(output_string.as_bytes())
            .with_context(|| ("Failed to write stdout content"))?;
    } else {
        crate::output::console::render_report(findings, &crates, &detectors_info)?;
        generate_report(
            findings,
            raw_findings,
            &crates,
            project_info,
            &detectors_info,
            opts.output_path,
            &opts.output_format,
        )?;
    }

    Ok(())
}

#[tracing::instrument(name = "RUN DYLINT", skip_all)]
fn run_dylint(
    detectors_paths: Vec<PathBuf>,
    opts: &Scout,
    inside_vscode: bool,
) -> Result<(bool, NamedTempFile)> {
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

#[tracing::instrument(name = "GENERATE REPORT", skip_all)]
fn generate_report(
    findings: &Vec<Finding>,
    raw_findings: Vec<Finding>,
    crates: &HashMap<String, bool>,
    project_info: ProjectInfo,
    detectors_info: &LintStore,
    output_path: Option<PathBuf>,
    output_format: &[OutputFormat],
) -> Result<()> {
    let report = RawReport::generate_report(findings, crates, &project_info, detectors_info)?;

    tracing::trace!(?output_format, "Output format");
    tracing::trace!(?report, "Report");

    for format in output_format.iter() {
        let path = report.write_out(findings, &raw_findings, output_path.clone(), format)?;

        if let Some(path) = path {
            let path = path
                .to_str()
                .with_context(|| "Path conversion to string failed")?;
            let string = OutputFormatter::new()
                .fg()
                .green()
                .text_str(format!("{path} successfully generated.").as_str())
                .print();
            println!("{string}");
        }
    }

    Ok(())
}
