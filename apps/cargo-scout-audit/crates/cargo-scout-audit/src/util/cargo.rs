use crate::util::command::Command;
use ansi_term::Style;
use std::path::Path;
use std::{
    io::{IsTerminal, Write},
    process::Stdio,
};

#[must_use]
pub fn build(description: &str, toolchain: &str, quiet: bool) -> Command {
    cargo("build", "Building", description, quiet, toolchain)
}

fn cargo(subcommand: &str, verb: &str, description: &str, quiet: bool, toolchain: &str) -> Command {
    if !quiet {
        // smoelius: Writing directly to `stderr` avoids capture by `libtest`.
        let message = format!("{verb} {description}");
        std::io::stderr()
            .write_fmt(format_args!(
                "{}\n",
                if std::io::stdout().is_terminal() {
                    Style::new().bold()
                } else {
                    Style::new()
                }
                .paint(message)
            ))
            .expect("Could not write to stderr");
    }
    call_cargo(&[subcommand], quiet, Some(toolchain))
}

pub fn call_cargo(subcommand: &[&str], quiet: bool, toolchain: Option<&str>) -> Command {
    // `cargo +toolchain` only works when `cargo` is the rustup proxy. On
    // systems where Cargo comes from a package manager (for example Homebrew),
    // use rustup's explicit runner so the selected compiler is deterministic.
    let mut command = if let Some(toolchain) = toolchain {
        let toolchain = toolchain.trim_start_matches('+');
        let mut command = Command::new("rustup");
        command.args(["run", toolchain, "cargo"]);

        // `rustup run` selects Cargo, but a package-manager Rust installation
        // may still appear first in PATH when Cargo launches rustc. Prepending
        // the selected toolchain's bin directory keeps all compiler tools on
        // the same toolchain.
        if let Ok(output) = std::process::Command::new("rustup")
            .args(["which", "--toolchain", toolchain, "rustc"])
            .output()
            && output.status.success()
        {
            let rustc_path = String::from_utf8_lossy(&output.stdout);
            if let Some(toolchain_bin) = Path::new(rustc_path.trim()).parent() {
                let old_path = std::env::var_os("PATH").unwrap_or_default();
                if let Ok(new_path) = std::env::join_paths(
                    std::iter::once(toolchain_bin.to_path_buf())
                        .chain(std::env::split_paths(&old_path)),
                ) {
                    command.envs([("PATH", new_path)]);
                }
            }
        }
        command
    } else {
        Command::new("cargo")
    };
    command.args(subcommand);
    if quiet {
        command.stderr(Stdio::null());
    }
    command
}
