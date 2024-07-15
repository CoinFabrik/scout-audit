use anyhow::{Context, Result};
use current_platform::CURRENT_PLATFORM;
use lazy_static::lazy_static;
use std::{
    env,
    path::Path,
    process::{Child, Command},
};

use crate::{build_config::TOOLCHAIN, utils::print::print_error};

lazy_static! {
    static ref LIBRARY_PATH_VAR: &'static str = match env::consts::OS {
        "linux" => "LD_LIBRARY_PATH",
        "macos" => "DYLD_FALLBACK_LIBRARY_PATH",
        _ => panic!("Unsupported operating system: {}", env::consts::OS),
    };
}

#[tracing::instrument(name = "RUN SCOUT IN NIGHTLY", skip_all)]
pub fn run_scout_in_nightly() -> Result<Option<Child>> {
    let current_lib_path = env::var(LIBRARY_PATH_VAR.to_string()).unwrap_or_default();
    if current_lib_path.contains(TOOLCHAIN) {
        return Ok(None);
    }

    let rustup_home = env::var("RUSTUP_HOME").unwrap_or_else(|_| {
        print_error("Failed to get RUSTUP_HOME, defaulting to '~/.rustup'");
        "~/.rustup".to_string()
    });

    let nightly_lib_path = Path::new(&rustup_home)
        .join("toolchains")
        .join(format!("{}-{}", TOOLCHAIN, CURRENT_PLATFORM))
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
