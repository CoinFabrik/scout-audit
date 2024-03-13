use core::panic;
use std::{fs, path::PathBuf};

use anyhow::{bail, Context, Result};
use cargo::Config;
use cargo_metadata::MetadataCommand;
use clap::{Parser, Subcommand, ValueEnum};
use dylint::Dylint;

use crate::{
    detectors::{get_detectors_configuration, get_local_detectors_configuration, Detectors},
    utils::{
        detectors::{get_excluded_detectors, get_filtered_detectors, list_detectors},
        output::{format_into_json, format_into_sarif},
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
    Markdown,
    Sarif,
    Text,
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

pub fn run_scout(opts: Scout) -> Result<()> {
    // Validations
    if opts.filter.is_some() && opts.exclude.is_some() {
        panic!("You can't use `--exclude` and `--filter` at the same time.");
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
    if opts.exclude.is_some() || opts.filter.is_some() {
        remove_target_dylint(&opts.manifest_path)?;
    }

    // Instantiate detectors
    let detectors = Detectors::new(cargo_config, detectors_config, metadata, opts.verbose);
    let detectors_names = detectors
        .get_detector_names()
        .context("Failed to build detectors")?;

    if opts.list_detectors {
        list_detectors(detectors_names);
        return Ok(());
    }

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

    // Run dylint
    run_dylint(detectors_paths, opts, bc_dependency).context("Failed to run dylint")?;

    Ok(())
}

fn run_dylint(detectors_paths: Vec<PathBuf>, opts: Scout, bc_dependency: BlockChain) -> Result<()> {
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
            // Generate HTML
            // let html_path = report.generate_html()?;

            // Open the HTML report in the default web browser
            // webbrowser::open(&html_path).context("Failed to open HTML report")?;
        }
        OutputFormat::Json => {
            let mut json_file = match &opts.output_path {
                Some(path) => fs::File::create(path)?,
                None => fs::File::create("report.json")?,
            };
            std::io::Write::write_all(
                &mut json_file,
                format_into_json(stderr_file, stdout_file, bc_dependency)?.as_bytes(),
            )?;
        }
        OutputFormat::Markdown => {
            // Generate Markdown
            // let markdown_path = report.generate_markdown()?;

            // Open the Markdown report in the default text editor
            // open::that(markdown_path).context("Failed to open Markdown report")?;
        }
        OutputFormat::Sarif => {
            let mut sarif_file = match &opts.output_path {
                Some(path) => fs::File::create(path)?,
                None => fs::File::create("report.sarif")?,
            };
            std::io::Write::write_all(
                &mut sarif_file,
                format_into_sarif(stderr_file, stdout_file, bc_dependency)?.as_bytes(),
            )?;
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
    }

    stderr_temp_file.close()?;
    stdout_temp_file.close()?;

    Ok(())
}

fn remove_target_dylint(manifest_path: &Option<PathBuf>) -> Result<()> {
    let target_dylint_path = match manifest_path {
        Some(manifest_path) => {
            let manifest_path_parent = manifest_path
                .parent()
                .context("Error getting manifest path parent")?;
            manifest_path_parent.join("target").join("dylint")
        }
        None => std::env::current_dir()?.join("target").join("dylint"),
    };
    if target_dylint_path.exists() {
        fs::remove_dir_all(target_dylint_path)?;
    }
    Ok(())
}
