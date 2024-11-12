use crate::{
    detectors::{
        builder::DetectorBuilder,
        configuration::{get_local_detectors_configuration, get_remote_detectors_configuration},
    },
    output::raw_report::{json_to_string, json_to_string_opt, RawReport},
    scout::{
        blockchain::BlockChain, nightly_runner::run_scout_in_nightly,
        post_processing::PostProcessing, project_info::ProjectInfo,
        version_checker::VersionChecker,
    },
    server::capture_output,
    utils::{
        config::{open_config_and_sync_detectors, profile_enabled_detectors},
        detectors::{get_excluded_detectors, get_filtered_detectors, list_detectors},
        detectors_info::{get_detectors_info, LintInfo},
        print::{print_error, print_warning},
    },
};
use anyhow::{anyhow, bail, Context, Ok, Result};
use cargo::{core::Verbosity, GlobalContext};
use cargo_metadata::{Metadata, MetadataCommand};
use clap::{Parser, Subcommand, ValueEnum};
use dylint::opts::{Check, Dylint, LibrarySelection, Operation};
use serde_json::{from_str, to_string_pretty, Value};
use std::{
    collections::{HashMap, HashSet},
    fs,
    io::Write,
    path::PathBuf,
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

fn temp_file_to_string(mut file: NamedTempFile) -> Result<String> {
    let mut ret = String::new();
    std::io::Read::read_to_string(&mut file, &mut ret)?;
    let _ = file.close();
    Ok(ret)
}

fn output_to_json(output: &str) -> Vec<Value> {
    output
        .lines()
        .map(|line| from_str::<Value>(line).unwrap())
        .collect::<Vec<Value>>()
}

fn get_crate_from_finding(finding: &Value) -> Option<String> {
    json_to_string_opt(finding.get("target").and_then(|x| x.get("name")))
}

//In some cases, rustc (or dylint, or clipply, or whoever) has returned the
//package name where it should be returning the crate name. If you run into
//problems in the future, try removing the call to this function.
fn normalize_crate_name(s: &str) -> String {
    let mut ret = String::new();
    ret.reserve(s.len());
    for c in s.chars() {
        ret.push(if c == '-' { '_' } else { c });
    }
    ret
}

fn get_crates_from_output(output: &Vec<Value>) -> HashMap<String, bool> {
    let mut ret = HashMap::<String, bool>::new();

    for val in output {
        let reason = val.get("reason");
        let message = val.get("message");
        if reason.is_none() || message.is_none() || reason.unwrap() != "compiler-message" {
            continue;
        }
        let message = message.unwrap();

        let name = get_crate_from_finding(val);
        if name.is_none() {
            continue;
        }
        let name = normalize_crate_name(&name.unwrap());
        if let Some(previous) = ret.get(&name) {
            if !previous {
                continue;
            }
        }
        let level = message.get("level");
        let ok = level.is_none() || level.unwrap() != "error";
        ret.insert(name, ok);
    }

    ret
}

fn get_crates_from_findings(findings: &Vec<String>) -> HashSet<String> {
    let mut ret = HashSet::<String>::new();

    for s in findings {
        let value = from_str::<Value>(s).unwrap();
        let krate = json_to_string(value.get("crate").unwrap());
        ret.insert(krate);
    }

    ret
}

fn get_crates(
    output: &Vec<Value>,
    findings: &Vec<String>,
    packages: &[crate::output::report::Package],
) -> HashMap<String, bool> {
    let mut ret = HashMap::<String, bool>::new();
    for package in packages.iter() {
        ret.insert(normalize_crate_name(&package.name), true);
    }
    for (name, ok) in get_crates_from_output(output).iter() {
        if ret.contains_key(name) {
            ret.insert(name.clone(), *ok);
        }
    }
    for krate in get_crates_from_findings(findings) {
        ret.entry(krate).or_insert(true);
    }

    ret
}

fn split_findings(
    raw_findings: Vec<String>,
    crates: &HashMap<String, bool>,
) -> (Vec<Value>, Vec<Value>) {
    let mut findings = Vec::new();

    for s in raw_findings.iter() {
        if s.trim().is_empty() {
            continue;
        }
        let value = from_str::<Value>(s).unwrap();
        findings.push(value);
    }
    let mut successful_findings = Vec::<Value>::new();
    let mut failed_findings = Vec::<Value>::new();

    for finding in findings.iter() {
        let krate = finding.get("crate");
        let message = finding.get("message");
        if krate.is_none() || message.is_none() {
            continue;
        }
        let krate = json_to_string(krate.unwrap());
        let message = message.unwrap();
        let mut message = message.clone();
        message["crate"] = Value::String(krate.clone());
        if *crates.get(&krate).unwrap_or(&true) {
            &mut successful_findings
        } else {
            &mut failed_findings
        }
        .push(message);
    }

    (successful_findings, failed_findings)
}

fn capture_noop<T, E, F: FnOnce() -> Result<T, E>>(cb: F) -> Result<(Vec<String>, T), E> {
    use std::result::Result::Ok;
    match cb() {
        Ok(r) => Ok((Vec::<String>::new(), r)),
        Err(e) => Err(e),
    }
}

#[tracing::instrument(name = "RUN SCOUT", skip_all)]
pub fn run_scout(mut opts: Scout) -> Result<Vec<Value>> {
    opts.validate()?;
    opts.prepare_args();

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

    let wrapper_function = if inside_vscode {
        capture_noop
    } else {
        capture_output
    };

    let (findings, (_successful_build, stdout)) = wrapper_function(|| {
        // Run dylint
        run_dylint(detectors_paths.clone(), &opts, &metadata, inside_vscode)
            .map_err(|err| anyhow!("Failed to run dylint.\n\n     → Caused by: {}", err))
    })?;

    let output_string = temp_file_to_string(stdout)?;
    //println!("{}", output_string);
    let output = output_to_json(&output_string);
    let crates = get_crates(&output, &findings, &project_info.packages);

    if crates.is_empty() && !inside_vscode {
        let string = OutputFormatter::new()
            .fg()
            .red()
            .text_str("Nothing was analyzed. Check your build system for errors.")
            .print();
        println!("{}", string);
        return Ok(vec![]);
    }

    let (successful_findings, _failed_findings) = split_findings(findings, &crates);

    // Get the path of the 'unnecessary_lint_allow' detector
    let unnecessary_lint_allow_path = detectors_paths.iter().find_map(|path| {
        path.to_str()
            .filter(|s| s.contains("unnecessary_lint_allow"))
            .map(|_| path)
    });

    // Create and run post processor if the path is found, otherwise use default values
    let (console_findings, output_string_vscode) = if let Some(path) = unnecessary_lint_allow_path {
        match PostProcessing::new(path) {
            std::result::Result::Ok(post_processor) => {
                match post_processor.process(
                    successful_findings.clone(),
                    output.clone(),
                    inside_vscode,
                ) {
                    std::result::Result::Ok(result) => result,
                    Err(e) => {
                        print_error(&format!("Error running post process: {}", e));
                        (successful_findings, output_string)
                    }
                }
            }
            Err(e) => {
                print_error(&format!("Error creating PostProcessing: {}", e));
                (successful_findings, output_string)
            }
        }
    } else {
        (successful_findings, output_string)
    };
    // Generate report
    do_report(
        &console_findings,
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
    findings: &Vec<Value>,
    crates: HashMap<String, bool>,
    project_info: ProjectInfo,
    detectors_info: HashSet<LintInfo>,
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
    metadata: &Metadata,
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

    crate::cleanup::clean_up_before_run(metadata);

    let success = dylint::run(&options).is_err();

    Ok((success, stdout_temp_file))
}

#[tracing::instrument(name = "GENERATE REPORT", skip_all)]
fn generate_report(
    findings: &Vec<Value>,
    crates: &HashMap<String, bool>,
    project_info: ProjectInfo,
    detectors_info: &HashSet<LintInfo>,
    output_path: Option<PathBuf>,
    output_format: &[OutputFormat],
) -> Result<()> {
    let report = RawReport::generate_report(findings, crates, &project_info, detectors_info)?;

    tracing::trace!(?output_format, "Output format");
    tracing::trace!(?report, "Report");

    for format in output_format.iter() {
        let path = report.write_out(findings, output_path.clone(), format)?;

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
