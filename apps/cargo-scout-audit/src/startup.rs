use core::panic;
use current_platform::CURRENT_PLATFORM;
use regex::Regex;
use std::{
    collections::HashMap,
    env, fs,
    path::PathBuf,
    process::{Child, Command},
};

use anyhow::{bail, Context, Ok, Result};
use cargo::Config;
use cargo_metadata::{Metadata, MetadataCommand};
use clap::{Parser, Subcommand, ValueEnum};
use dylint::Dylint;

use crate::{
    detectors::{get_detectors_configuration, get_local_detectors_configuration, Detectors},
    output::{raw_report::RawReport, report::Package},
    utils::{
        config::{open_config_or_default, profile_enabled_detectors},
        detectors::{get_excluded_detectors, get_filtered_detectors, list_detectors},
        detectors_info::{get_detectors_info, LintInfo},
    },
};

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

    #[clap(short, long, value_name = "type", help = "Sets the output type")]
    pub output_format: Option<OutputFormat>,

    #[clap(long, value_name = "path", help = "Path to the output file.")]
    pub output_path: Option<PathBuf>,

    #[clap(long, value_name = "path", help = "Path to detectors workspace.")]
    pub local_detectors: Option<PathBuf>,

    #[clap(
        short,
        long,
        help = "Prints detectors metadata.",
        default_value_t = false
    )]
    pub verbose: bool,

    #[clap(
        name = "metadata",
        long,
        help = "Prints metadata information.",
        default_value_t = false
    )]
    pub detectors_metadata: bool,
}

#[derive(Debug, Copy, Clone)]
pub enum BlockChain {
    Ink,
    Soroban,
}

#[derive(Debug)]
pub struct ProjectInfo {
    pub name: String,
    pub date: String,
    pub workspace_root: PathBuf,
    pub packages: Vec<Package>,
}

pub fn run_scout(mut opts: Scout) -> Result<()> {
    let opt_child = run_scout_in_nightly()?;
    if let Some(mut child) = opt_child {
        child.wait()?;
        return Ok(());
    }

    // If the target is not set to wasm32-unknown-unknown, set it
    let target_args_flag = "--target=wasm32-unknown-unknown".to_string();
    let no_default = "--no-default-features".to_string();
    let z_build_std = "-Zbuild-std=std,core,alloc".to_string();

    if !opts.args.iter().any(|x| x.contains("--target=")) {
        opts.args
            .extend([target_args_flag, no_default, z_build_std])
    }

    // Validations
    if opts.filter.is_some() && opts.exclude.is_some() {
        panic!("You can't use `--exclude` and `--filter` at the same time.");
    }

    if opts.filter.is_some() && opts.profile.is_some() {
        panic!("You can't use `--exclude` and `--profile` at the same time.");
    }

    if let Some(path) = &opts.output_path {
        if path.is_dir() {
            bail!("The output path can't be a directory.");
        }
    }

    // Prepare configurations
    let mut metadata = MetadataCommand::new();
    if let Some(manifest_path) = &opts.manifest_path {
        metadata.manifest_path(manifest_path);
    }
    let metadata = metadata.exec().context("Failed to get metadata")?;

    let bc_dependency = metadata
        .packages
        .iter()
        .find_map(|p| match p.name.as_str() {
            "soroban-sdk" => Some(BlockChain::Soroban),
            "ink" => Some(BlockChain::Ink),
            _ => None,
        })
        .expect("Blockchain dependency not found");

    let cargo_config = Config::default().context("Failed to get config")?;
    cargo_config.shell().set_verbosity(if opts.verbose {
        cargo::core::Verbosity::Verbose
    } else {
        cargo::core::Verbosity::Quiet
    });
    let detectors_config = match &opts.local_detectors {
        Some(path) => get_local_detectors_configuration(&PathBuf::from(path))
            .context("Failed to get local detectors configuration")?,
        None => get_detectors_configuration(bc_dependency)
            .context("Failed to get detectors configuration")?,
    };

    // Instantiate detectors
    let detectors = Detectors::new(
        cargo_config,
        detectors_config,
        metadata.clone(),
        opts.verbose,
    );
    let mut detectors_names = detectors
        .get_detector_names()
        .context("Failed to build detectors")?;

    if opts.list_detectors {
        list_detectors(detectors_names);
        return Ok(());
    }

    detectors_names = if let Some(profile) = &opts.profile {
        let config = open_config_or_default(bc_dependency, detectors_names.clone())
            .context("Failed to load or generate config")?;

        profile_enabled_detectors(config, profile.clone())?
    } else {
        detectors_names
    };

    let used_detectors = if let Some(filter) = &opts.filter {
        get_filtered_detectors(filter.to_string(), detectors_names)?
    } else if let Some(excluded) = &opts.exclude {
        get_excluded_detectors(excluded.to_string(), detectors_names)?
    } else {
        detectors_names
    };

    let detectors_paths = detectors
        .build(bc_dependency, used_detectors)
        .context("Failed to build detectors bis")?;

    let detectors_info = get_detectors_info(&detectors_paths)?;

    if opts.detectors_metadata {
        let json = serde_json::to_string_pretty(&detectors_info);
        println!("{}", json.unwrap());
        return Ok(());
    }

    let project_info = get_project_info(metadata.clone()).context("Failed to get project info")?;

    // Run dylint
    // let (stdout_temp_file, stderr_temp_file) =
    let stdout_temp_file =
        run_dylint(detectors_paths, &opts, bc_dependency).context("Failed to run dylint")?;

    // Generate report
    if let Some(output_format) = opts.output_format {
        generate_report(
            stdout_temp_file,
            project_info,
            detectors_info,
            opts.output_path,
            output_format,
        )?;
    }

    Ok(())
}

#[tracing::instrument(name = "GET PROJECT INFO", skip_all)]
fn get_project_info(metadata: Metadata) -> Result<ProjectInfo> {
    let mut packages = Vec::new();
    if let Some(root_package) = metadata.root_package() {
        let root = root_package.manifest_path.parent().context(format!(
            "Root package manifest at '{}' has no parent directory",
            root_package.manifest_path
        ))?;
        packages.push(Package {
            name: root_package.name.clone(),
            root: root.into(),
        });
    } else if !metadata.workspace_default_members.is_empty() {
        for package_id in metadata.workspace_default_members.iter() {
            let package = metadata
                .packages
                .iter()
                .find(|p| p.id == *package_id)
                .context(format!(
                    "Package ID '{}' not found in the workspace",
                    package_id
                ))?;
            let root = package.manifest_path.parent().context(format!(
                "Package manifest at '{}' has no parent directory",
                package.manifest_path
            ))?;
            packages.push(Package {
                name: package.name.clone(),
                root: root.into(),
            });
        }
    } else {
        bail!("No packages found in the workspace. Ensure that workspace is configured properly and contains at least one package.");
    }

    let mut project_name = String::new();
    if let Some(name) = metadata.workspace_root.file_name() {
        project_name = name.replace('-', " ");
        let re = Regex::new(r"(^|\s)\w").unwrap();
        project_name = re
            .replace_all(&project_name, |caps: &regex::Captures| {
                caps.get(0).unwrap().as_str().to_uppercase()
            })
            .to_string();
    }

    let project_info = ProjectInfo {
        name: project_name,
        date: chrono::Local::now().format("%Y-%m-%d").to_string(),
        workspace_root: metadata.workspace_root.into_std_path_buf(),
        packages,
    };
    tracing::trace!(?project_info, "Project info");
    Ok(project_info)
}

#[tracing::instrument(name = "RUN SCOUT IN NIGHTLY", skip())]
fn run_scout_in_nightly() -> Result<Option<Child>> {
    #[cfg(target_os = "linux")]
    let var_name = "LD_LIBRARY_PATH";
    #[cfg(target_os = "macos")]
    let var_name = "DYLD_FALLBACK_LIBRARY_PATH";
    let toolchain = std::env::var(var_name)?;
    if !toolchain.contains("nightly-2023-12-16") {
        let current_platform = CURRENT_PLATFORM;
        let rustup_home = env::var("RUSTUP_HOME")?;

        let lib_path =
            rustup_home.clone() + "/toolchains/nightly-2023-12-16-" + current_platform + "/lib";

        let args: Vec<String> = env::args().collect();
        let program = args[0].clone();

        let mut command = Command::new(program);
        for arg in args.iter().skip(1) {
            command.arg(arg);
        }

        command.env(var_name, lib_path);
        let child = command.spawn()?;
        Ok(Some(child))
    } else {
        Ok(None)
    }
}

#[tracing::instrument(name = "RUN DYLINT", skip(detectors_paths, opts, _bc_dependency))]
fn run_dylint(
    detectors_paths: Vec<PathBuf>,
    opts: &Scout,
    _bc_dependency: BlockChain,
) -> Result<tempfile::NamedTempFile> {
    // Convert detectors paths to string
    let detectors_paths: Vec<String> = detectors_paths
        .iter()
        .map(|path| path.to_string_lossy().to_string())
        .collect();

    // Initialize temporary file for stdout
    let stdout_temp_file = tempfile::NamedTempFile::new()?;
    let pipe_stdout = Some(stdout_temp_file.path().to_string_lossy().to_string());

    // Get the manifest path
    let manifest_path = opts
        .manifest_path
        .as_ref()
        .map(|p| p.to_string_lossy().to_string());

    // Prepare arguments
    let mut args = opts.args.clone();
    if opts.output_format.is_some() {
        args.push("--message-format=json".to_string());
    }

    let options = Dylint {
        paths: detectors_paths,
        args,
        pipe_stdout,
        manifest_path,
        quiet: !opts.verbose,
        ..Default::default()
    };

    dylint::run(&options)?;

    Ok(stdout_temp_file)
}

#[tracing::instrument(name = "GENERATE REPORT", skip_all)]
fn generate_report(
    stdout_temp_file: tempfile::NamedTempFile,
    project_info: ProjectInfo,
    detectors_info: HashMap<String, LintInfo>,
    output_path: Option<PathBuf>,
    output_format: OutputFormat,
) -> Result<()> {
    // Read the stdout temporary file
    let mut stdout_file = fs::File::open(stdout_temp_file.path())?;

    // Generate report
    let mut content = String::new();
    std::io::Read::read_to_string(&mut stdout_file, &mut content)?;
    let report = RawReport::generate_report(&content, &project_info, &detectors_info)?;

    tracing::trace!(?output_format, "Output format");
    tracing::trace!(?report, "Report");

    // Save the report
    match output_format {
        OutputFormat::Html => {
            // Generate HTML report
            let html = report.generate_html()?;

            // Save to file
            let html_path = output_path.unwrap_or_else(|| PathBuf::from("report.html"));
            report.save_to_file(&html_path, html)?;

            // Open the HTML report in the default web browser
            webbrowser::open(
                html_path
                    .to_str()
                    .expect("Path conversion to string failed"),
            )
            .context("Failed to open HTML report")?;
        }

        OutputFormat::Json => {
            // Generate JSON report
            let json = report.generate_json()?;

            // Save to file
            let json_path = output_path.unwrap_or_else(|| PathBuf::from("report.json"));
            report.save_to_file(&json_path, json)?;
        }
        OutputFormat::RawJson => {
            let mut json_file = match &output_path {
                Some(path) => fs::File::create(path)?,
                None => fs::File::create("raw-report.json")?,
            };

            let mut cts = String::new();

            std::io::Read::read_to_string(&mut stdout_file, &mut cts)?;

            std::io::Write::write(&mut json_file, cts.as_bytes())?;
        }
        OutputFormat::Markdown => {
            // Generate Markdown
            let markdown = report.generate_markdown(true)?;

            // Save to file
            let md_path = output_path.unwrap_or_else(|| PathBuf::from("report.md"));
            report.save_to_file(&md_path, markdown)?;
        }
        OutputFormat::MarkdownGithub => {
            // Generate Markdown
            let markdown = report.generate_markdown(false)?;

            // Save to file
            let md_path = output_path.unwrap_or_else(|| PathBuf::from("report.md"));
            report.save_to_file(&md_path, markdown)?;
        }
        OutputFormat::Sarif => {
            let path = if let Some(path) = output_path {
                path
            } else {
                PathBuf::from("report.sarif")
            };

            let mut sarif_file = fs::File::create(path)?;

            let mut content = String::new();
            std::io::Read::read_to_string(&mut stdout_file, &mut content)?;

            let child = std::process::Command::new("clippy-sarif")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn()?;

            std::io::Write::write_all(&mut child.stdin.as_ref().unwrap(), content.as_bytes())?;

            std::io::Write::write_all(&mut sarif_file, &child.wait_with_output()?.stdout)?;
        }
        OutputFormat::Pdf => {
            let path = if let Some(path) = output_path {
                path
            } else {
                PathBuf::from("report.pdf")
            };
            report.generate_pdf(&path)?;
        }
    }

    stdout_temp_file.close()?;

    Ok(())
}
