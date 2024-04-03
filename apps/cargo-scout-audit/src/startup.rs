use core::panic;
use current_platform::CURRENT_PLATFORM;
use std::{
    collections::HashMap,
    env, fs,
    hash::{Hash, Hasher},
    iter::Map,
    os::unix::process::CommandExt,
    path::PathBuf,
    process::{Child, Command},
};

use anyhow::{bail, Context, Error, Result};
use cargo::{ops::new, Config};
use cargo_metadata::MetadataCommand;
use clap::{Parser, Subcommand, ValueEnum};
use dylint::Dylint;

use crate::{
    detectors::{get_detectors_configuration, get_local_detectors_configuration, Detectors},
    output::report::generate_report,
    utils::{
        config::{open_config_or_default, profile_enabled_detectors},
        detectors::{get_excluded_detectors, get_filtered_detectors, list_detectors},
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
    #[clap(name = "md")]
    Markdown,
    Sarif,
    Text,
    Pdf,
}

type LintInfoFunc = unsafe fn(info: &mut LintInfo);

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
        help = "Sets the output type",
        default_value = "text"
    )]
    pub output_format: OutputFormat,

    #[clap(long, value_name = "path", help = "Path to the output file.")]
    pub output_path: Option<PathBuf>,

    #[clap(long, value_name = "path", help = "Path to detectors workspace.")]
    pub local_detectors: Option<PathBuf>,

    #[clap(
        short,
        long,
        help = "Prints verbose information.",
        default_value_t = false
    )]
    pub verbose: bool,
}

#[derive(Debug, Copy, Clone)]
pub enum BlockChain {
    Ink,
    Soroban,
}

pub struct ProjectInfo {
    pub name: String,
    pub description: String,
    pub hash: String,
    pub worspace_root: PathBuf,
}

#[derive(Default, Debug, Clone)]
pub struct LintInfo {
    pub id: String,
    pub name: String,
    pub short_message: String,
    pub long_message: String,
    pub severity: String,
    pub help: String,
    pub vulnerability_class: String,
}

pub fn run_scout(opts: Scout) -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let opt_child = run_scout_in_nightly()?;
    if let Some(mut child) = opt_child {
        child.wait()?;
        return Ok(());
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

    // Misc configurations
    // If there is a need to exclude or filter by detector, the dylint tool needs to be recompiled.
    // TODO: improve detector system so that doing this isn't necessary.
    /*if opts.exclude.is_some() || opts.filter.is_some() {
        remove_target_dylint(&opts.manifest_path)?;
    }*/

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

    let root = metadata.root_package().unwrap();

    let mut hasher = std::hash::DefaultHasher::new();

    root.id.hash(&mut hasher);

    let info = ProjectInfo {
        name: root.name.clone(),
        description: root.description.clone().unwrap_or_default(),
        hash: hasher.finish().to_string(),
        worspace_root: metadata.workspace_root.clone().into(),
    };

    // Run dylint
    run_dylint(detectors_paths, opts, bc_dependency, info, detectors_info)
        .context("Failed to run dylint")?;

    Ok(())
}

fn run_scout_in_nightly() -> Result<Option<Child>> {
    let toolchain = std::env::var("LD_LIBRARY_PATH")?;
    if !toolchain.contains("nightly-2023-12-16") {
        let current_platform = CURRENT_PLATFORM;
        let rustup_home = env::var("RUSTUP_HOME")?;

        let lib_path =
            rustup_home.clone() + "/toolchains/nightly-2023-12-16-" + current_platform + "/lib";

        let args: Vec<String> = env::args().collect();
        let program = args[0].clone();

        let mut command = Command::new(&program);
        for arg in args.iter().skip(1) {
            command.arg(arg);
        }

        command.env("LD_LIBRARY_PATH", lib_path);
        let child = command.spawn()?;
        Ok(Some(child))
    } else {
        Ok(None)
    }
}

fn get_detectors_info(detectors_paths: &Vec<PathBuf>) -> Result<HashMap<String, LintInfo>> {
    let mut lint_store = HashMap::<String, LintInfo>::default();

    for detector_path in detectors_paths {
        unsafe {
            let lib_res = libloading::os::unix::Library::open(
                Some(detector_path),
                libloading::os::unix::RTLD_LAZY | libloading::os::unix::RTLD_LOCAL,
            );
            //dbg!(&lib_res);
            let lib = lib_res.unwrap();
            let lint_info_func_res = lib.get::<LintInfoFunc>(b"lint_info");
            if lint_info_func_res.is_ok() {
                let lint_info_func = lint_info_func_res.unwrap();
                let mut info = LintInfo::default();
                lint_info_func(&mut info);
                lint_store.insert(info.id.clone(), info);
            }
            //dbg!(&name, &desc);
        }
    }
    dbg!(&lint_store);
    return Ok(lint_store);
}

fn run_dylint(
    detectors_paths: Vec<PathBuf>,
    mut opts: Scout,
    bc_dependency: BlockChain,
    info: ProjectInfo,
    detectors_info: HashMap<String, LintInfo>,
) -> Result<()> {
    // Convert detectors paths to string
    let detectors_paths: Vec<String> = detectors_paths
        .iter()
        .map(|path| path.to_string_lossy().to_string())
        .collect();

    // Initialize options
    let stderr_temp_file = tempfile::NamedTempFile::new()?;
    let stdout_temp_file = tempfile::NamedTempFile::new()?;

    let is_output_stdout = opts.output_format == OutputFormat::Text && opts.output_path.is_none();
    let is_output_stdout_json = opts.args.contains(&"--message-format=json".to_string());

    let pipe_stdout = Some(stdout_temp_file.path().to_string_lossy().to_string());
    let pipe_stderr = if is_output_stdout && !is_output_stdout_json {
        None
    } else {
        Some(stderr_temp_file.path().to_string_lossy().to_string())
    };

    if opts.output_format != OutputFormat::Text {
        opts.args.push("--message-format=json".to_string());
    }

    let options = Dylint {
        paths: detectors_paths,
        args: opts.args,
        manifest_path: opts.manifest_path.map(|p| p.to_string_lossy().to_string()),
        pipe_stdout,
        pipe_stderr,
        quiet: !opts.verbose,
        ..Default::default()
    };

    dylint::run(&options)?;

    // Format output and write to file (if necessary)
    if is_output_stdout && !is_output_stdout_json {
        return Ok(());
    }

    let mut stderr_file = fs::File::open(stderr_temp_file.path())?;
    let mut stdout_file = fs::File::open(stdout_temp_file.path())?;

    // Generate report with Report::new(...)
    // let report = Report::new(name, description, date, source_url, summary, categories, findings);
    match opts.output_format {
        OutputFormat::Html => {
            //read json_file to a string
            let mut content = String::new();
            std::io::Read::read_to_string(&mut stdout_file, &mut content)?;

            let report = generate_report(content, info, detectors_info);

            // Generate HTML
            let html = report.generate_html()?;

            let html_path = match opts.output_path {
                Some(path) => path,
                None => PathBuf::from("report.html"),
            };
            let mut html_file = fs::File::create(&html_path)?;
            std::io::Write::write_all(&mut html_file, html.as_bytes())?;

            // Open the HTML report in the default web browser
            webbrowser::open(html_path.to_str().unwrap()).context("Failed to open HTML report")?;
        }
        OutputFormat::Json => {
            let mut json_file = match &opts.output_path {
                Some(path) => fs::File::create(path)?,
                None => fs::File::create("report.json")?,
            };

            let mut cts = String::new();

            std::io::Read::read_to_string(&mut stdout_file, &mut cts)?;

            std::io::Write::write(&mut json_file, &cts.as_bytes())?;
        }
        OutputFormat::Markdown => {
            let mut content = String::new();
            std::io::Read::read_to_string(&mut stdout_file, &mut content)?;

            let report = generate_report(content, info, detectors_info);

            // Generate Markdown
            let markdown_path = report.generate_markdown()?;

            let mut md_file = match &opts.output_path {
                Some(path) => fs::File::create(path)?,
                None => fs::File::create("report.html")?,
            };

            std::io::Write::write_all(&mut md_file, markdown_path.as_bytes())?;
        }
        OutputFormat::Sarif => {
            let path = if let Some(path) = opts.output_path {
                path
            } else {
                PathBuf::from("report.sarif")
            };

            let mut sarif_file = fs::File::create(&path)?;

            let mut content = String::new();
            std::io::Read::read_to_string(&mut stdout_file, &mut content)?;

            let child = std::process::Command::new("clippy-sarif")
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .spawn()?;

            std::io::Write::write_all(&mut child.stdin.as_ref().unwrap(), content.as_bytes())?;

            std::io::Write::write_all(&mut sarif_file, &child.wait_with_output()?.stdout)?;
        }
        OutputFormat::Text => {
            // If the output path is not set, dylint prints the report to stdout
            if let Some(output_file) = opts.output_path {
                let mut txt_file = fs::File::create(output_file)?;
                std::io::copy(&mut stderr_file, &mut txt_file)?;
            } else {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                std::io::copy(&mut stdout_file, &mut handle)
                    .expect("Error writing dylint result to stdout");
            }
        }
        OutputFormat::Pdf => {
            let mut content = String::new();
            std::io::Read::read_to_string(&mut stdout_file, &mut content)?;

            let report = generate_report(content, info, detectors_info);

            let path = if let Some(path) = opts.output_path {
                path
            } else {
                PathBuf::from("report.pdf")
            };
            report.generate_pdf(&path)?;
        }
    }

    stderr_temp_file.close()?;
    stdout_temp_file.close()?;

    Ok(())
}
