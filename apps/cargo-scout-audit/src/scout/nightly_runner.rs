use crate::utils::print::print_info;
use anyhow::Result;
use lazy_static::lazy_static;
use std::{
    env,
    process::Child,
};
#[cfg(not(windows))]
use crate::utils::print::print_error;
#[cfg(not(windows))]
use anyhow::Context;
#[cfg(not(windows))]
use current_platform::CURRENT_PLATFORM;
#[cfg(not(windows))]
use std::{
    path::Path,
    process::Command,
};

lazy_static! {
    static ref LIBRARY_PATH_VAR: &'static str = match env::consts::OS {
        "linux" => "LD_LIBRARY_PATH",
        "macos" => "DYLD_FALLBACK_LIBRARY_PATH",
        _ => panic!("Unsupported operating system: {}", env::consts::OS),
    };
}

#[cfg(windows)]
#[tracing::instrument(name = "RUN SCOUT IN NIGHTLY", skip_all)]
pub fn run_scout_in_nightly(toolchain: &str) -> Result<Option<Child>> {
    use std::os::windows::ffi::OsStrExt;
    use windows::{core::PCWSTR, Win32::System::LibraryLoader::SetDllDirectoryW};

    let user_profile = env::var("USERPROFILE")
        .map_err(|e| anyhow::anyhow!("Unable to get user profile directory: {e}"))?;
    let mut user_profile = std::path::PathBuf::from(user_profile);
    user_profile.push(".rustup");
    user_profile.push("toolchains");
    user_profile.push(format!("{toolchain}-x86_64-pc-windows-msvc"));
    user_profile.push("bin");

    let user_profile = user_profile.as_os_str();
    let directory = user_profile
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    unsafe {
        let _ = SetDllDirectoryW(PCWSTR(directory.as_ptr()));
    }
    print_info("Re-running scout with nightly toolchain...");
    return Ok(None);
}

#[cfg(not(windows))]
#[tracing::instrument(name = "RUN SCOUT IN NIGHTLY", skip_all)]
pub fn run_scout_in_nightly(toolchain: &str) -> Result<Option<Child>> {
    let current_lib_path = env::var(LIBRARY_PATH_VAR.to_string()).unwrap_or_default();
    if current_lib_path.contains(toolchain) {
        return Ok(None);
    }

    let rustup_home = env::var("RUSTUP_HOME").unwrap_or_else(|_| {
        print_error("Failed to get RUSTUP_HOME, defaulting to '~/.rustup'");
        "~/.rustup".to_string()
    });

    let nightly_lib_path = Path::new(&rustup_home)
        .join("toolchains")
        .join(format!("{}-{}", toolchain, CURRENT_PLATFORM))
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
    print_info("Re-running scout with nightly toolchain...");
    Ok(Some(child))
}
