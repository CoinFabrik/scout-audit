use core::panic;
use current_platform::CURRENT_PLATFORM;
use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Child, Command},
};
use tempfile::NamedTempFile;

use anyhow::{anyhow, bail, Context, Ok, Result};
use cargo::Config;
use cargo_metadata::{Metadata, MetadataCommand};
use clap::{Parser, Subcommand, ValueEnum};
use dylint::Dylint;

use crate::{
    detectors::{get_local_detectors_configuration, get_remote_detectors_configuration, Detectors},
    output::raw_report::RawReport,
    scout::{blockchain::BlockChain, project_info::ProjectInfo},
    utils::{
        config::{open_config_or_default, profile_enabled_detectors},
        detectors::{get_excluded_detectors, get_filtered_detectors, list_detectors},
        detectors_info::{get_detectors_info, LintInfo},
        print::{print_error, print_warning},
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

impl Scout {
    fn prepare_args(&mut self) {
        if !self.args.iter().any(|x| x.contains("--target=")) {
            self.args.extend([
                "--target=wasm32-unknown-unknown".to_string(),
                "--no-default-features".to_string(),
                "-Zbuild-std=std,core,alloc".to_string(),
            ]);
        }

        if self.output_format.is_some() {
            self.args.push("--message-format=json".to_string());
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
                "Invalid manifest path, ensure scout is being run in a Rust project, and the path is set to the Cargo.toml file.\nManifest path: {:?}",
                manifest_path
            );
        }

        fs::metadata(manifest_path).with_context(|| {
            format!(
                "Cargo.toml file not found, ensure the path is a valid file path.\nManifest path: {:?}",
                manifest_path
            )
        })?;

        metadata_command.manifest_path(manifest_path);
    }

    metadata_command
        .exec()
        .map_err(|e| {
            anyhow!("Failed to execute metadata command on this path, ensure this is a valid rust project or workspace directory.\nCaused by: {}", e.to_string())})
}

#[tracing::instrument(name = "RUN SCOUT", skip_all)]
pub fn run_scout(mut opts: Scout) -> Result<()> {
    opts.validate()?;
    opts.prepare_args();

    if let Some(mut child) = run_scout_in_nightly()? {
        child
            .wait()
            .with_context(|| "Failed to wait for nightly child process")?;
        return Ok(());
    }

    let metadata = get_project_metadata(&opts.manifest_path)?;

    let bc_dependency = BlockChain::get_blockchain_dependency(&metadata)?;

    let cargo_config =
        Config::default().with_context(|| "Failed to create default cargo configuration")?;
    cargo_config.shell().set_verbosity(if opts.verbose {
        cargo::core::Verbosity::Verbose
    } else {
        cargo::core::Verbosity::Quiet
    });

    let detectors_config = match &opts.local_detectors {
        Some(path) => get_local_detectors_configuration(&PathBuf::from(path))
            .with_context(|| "Failed to get local detectors configuration")?,
        None => get_remote_detectors_configuration(bc_dependency)
            .with_context(|| "Failed to get remote detectors configuration")?,
    };

    // Instantiate detectors
    let detectors = Detectors::new(cargo_config, detectors_config, &metadata, opts.verbose);

    let mut detectors_names = detectors
        .get_detector_names()
        .with_context(|| "Failed to get detector names")?;

    if opts.list_detectors {
        list_detectors(&detectors_names);
        return Ok(());
    }

    detectors_names = if let Some(profile) = &opts.profile {
        let config = open_config_or_default(bc_dependency, detectors_names.clone())
            .with_context(|| "Failed to load or generate config")?;

        profile_enabled_detectors(config, profile.clone())?
    } else {
        detectors_names
    };

    let used_detectors = if let Some(filter) = &opts.filter {
        get_filtered_detectors(filter, &detectors_names)?
    } else if let Some(excluded) = &opts.exclude {
        get_excluded_detectors(excluded, &detectors_names)?
    } else {
        detectors_names
    };

    let detectors_paths = detectors
        .build(bc_dependency, &used_detectors)
        .with_context(|| "Failed to build detectors")?;

    let detectors_info = get_detectors_info(&detectors_paths)?;

    if opts.detectors_metadata {
        let json = serde_json::to_string_pretty(&detectors_info);
        println!("{}", json.unwrap());
        return Ok(());
    }

    let project_info =
        ProjectInfo::get_project_info(&metadata).with_context(|| "Failed to get project info")?;

    // Run dylint
    let stdout_temp_file = run_dylint(detectors_paths, &opts, bc_dependency)
        .with_context(|| "Failed to run dylint")?;

    // Generate report
    if let Some(output_format) = opts.output_format {
        generate_report(
            &stdout_temp_file,
            project_info,
            detectors_info,
            opts.output_path,
            output_format,
        )?;
    }

    stdout_temp_file.close()?;

    Ok(())
}

const NIGHTLY_VERSION: &str = "nightly-2023-12-16";

lazy_static! {
    static ref LIBRARY_PATH_VAR: &'static str = match env::consts::OS {
        "linux" => "LD_LIBRARY_PATH",
        "macos" => "DYLD_FALLBACK_LIBRARY_PATH",
        _ => panic!("Unsupported operating system: {}", env::consts::OS),
    };
}

#[tracing::instrument(name = "RUN SCOUT IN NIGHTLY", skip_all)]
fn run_scout_in_nightly() -> Result<Option<Child>> {
    let current_lib_path = env::var(LIBRARY_PATH_VAR.to_string()).unwrap_or_default();
    if current_lib_path.contains(NIGHTLY_VERSION) {
        return Ok(None);
    }

    let rustup_home = env::var("RUSTUP_HOME").unwrap_or_else(|_| {
        print_error("Failed to get RUSTUP_HOME, defaulting to '~/.rustup'");
        "~/.rustup".to_string()
    });

    let nightly_lib_path = Path::new(&rustup_home)
        .join("toolchains")
        .join(format!("{}-{}", NIGHTLY_VERSION, CURRENT_PLATFORM))
        .join("lib");

    let program_name =
        env::current_exe().with_context(|| "Failed to get current executable path")?;

    let mut command = Command::new(program_name);
    command
        .args(env::args().skip(1))
        .env(LIBRARY_PATH_VAR.to_string(), nightly_lib_path);

    let child = command
        .spawn()
        .with_context(|| "Failed to spawn scout with nightly toolchain")?;
    Ok(Some(child))
}

#[tracing::instrument(name = "RUN DYLINT", skip(detectors_paths, opts, _bc_dependency))]
fn run_dylint(
    detectors_paths: Vec<PathBuf>,
    opts: &Scout,
    _bc_dependency: BlockChain,
) -> Result<NamedTempFile> {
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

    let options = Dylint {
        paths: detectors_paths,
        args: opts.args.clone(),
        pipe_stdout,
        manifest_path,
        quiet: !opts.verbose,
        ..Default::default()
    };

    if dylint::run(&options).is_err() {
        print_error("Failed to run dylint, most likely due to an issue in the code.");
        if opts.output_format.is_some() {
            print_warning("This report is incomplete as some files could not be fully analyzed due to compilation errors. We strongly recommend to address all issues and executing Scout again.");
        }
    }

    if options.args.contains(&"--message-format=json".to_string()) && opts.output_format.is_none() {
        let stdout_content = fs::read(stdout_temp_file.path())
            .with_context(|| ("Failed to read stdout temporary file"))?;
        std::io::stdout()
            .lock()
            .write_all(&stdout_content)
            .with_context(|| ("Failed to write stdout content"))?;
    }

    Ok(stdout_temp_file)
}

#[tracing::instrument(name = "GENERATE REPORT", skip_all)]
fn generate_report(
    stdout_temp_file: &NamedTempFile,
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
                    .with_context(|| "Path conversion to string failed")?,
            )
            .with_context(|| "Failed to open HTML report")?;
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

            std::io::Write::write(&mut json_file, content.as_bytes())?;
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

    Ok(())
}
