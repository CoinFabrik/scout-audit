use ansi_term::Style;
#[cfg(windows)]
use std::path::Path;
use std::{
    io::{IsTerminal, Write},
    process::Stdio,
};

use crate::scout::blockchain::BlockChain;

const INK_TOOLCHAIN: &str = "+nightly-2023-12-16";
const SOROBAN_TOOLCHAIN: &str = "+nightly-2024-07-11";

use super::command::Command;
#[must_use]
pub fn build(description: &str, bc: BlockChain, quiet: bool) -> Command {
    cargo("build", "Building", description, quiet, bc)
}
fn cargo(subcommand: &str, verb: &str, description: &str, quiet: bool, bc: BlockChain) -> Command {
    let toolchain = get_toolchain(bc);

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
    let mut command = Command::new("cargo");
    #[cfg(windows)]
    {
        // Dylint annotation
        // smoelius: Work around: https://github.com/rust-lang/rustup/pull/2978
        let cargo_home = home::cargo_home().unwrap();
        let old_path = std::env::var("PATH").unwrap();
        let new_path = std::env::join_paths(
            std::iter::once(Path::new(&cargo_home).join("bin"))
                .chain(std::env::split_paths(&old_path)),
        )
        .unwrap();
        command.envs(vec![("PATH", new_path)]);
    }
    command.args([toolchain, subcommand]);
    if quiet {
        command.stderr(Stdio::null());
    }
    command
}

fn get_toolchain(bc: BlockChain) -> &'static str {
    match bc {
        BlockChain::Ink => INK_TOOLCHAIN,
        BlockChain::Soroban => SOROBAN_TOOLCHAIN,
    }
}
