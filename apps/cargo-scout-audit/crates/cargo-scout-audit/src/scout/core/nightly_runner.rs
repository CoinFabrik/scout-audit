use crate::util::print::print_info;
#[cfg(not(windows))]
use anyhow::Context;
use anyhow::Result;
use lazy_static::lazy_static;
use std::{collections::HashMap, env, process::Child};
#[cfg(not(windows))]
use std::{path::PathBuf, process::Command};

lazy_static! {
    static ref LIBRARY_PATH_VAR: &'static str = match env::consts::OS {
        "linux" => "LD_LIBRARY_PATH",
        "macos" => "DYLD_FALLBACK_LIBRARY_PATH",
        _ => panic!("Unsupported operating system: {}", env::consts::OS),
    };
}

#[cfg(windows)]
#[tracing::instrument(name = "RUN SCOUT IN NIGHTLY", skip_all)]
pub fn set_up_environment(toolchain: &str) -> Result<HashMap<String, String>> {
    use std::os::windows::ffi::OsStrExt;
    use windows::{Win32::System::LibraryLoader::SetDllDirectoryW, core::PCWSTR};

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

    Ok(HashMap::new())
}

#[cfg(not(windows))]
#[tracing::instrument(name = "RUN SCOUT IN NIGHTLY", skip_all)]
pub fn set_up_environment(toolchain: &str) -> Result<HashMap<String, String>> {
    let mut ret = HashMap::new();

    let rustc_output = Command::new("rustup")
        .args(["which", "--toolchain", toolchain, "rustc"])
        .output()
        .with_context(|| format!("Failed to locate rustc for toolchain {toolchain}"))?;
    if !rustc_output.status.success() {
        anyhow::bail!(
            "Failed to locate rustc for toolchain {toolchain}: {}",
            String::from_utf8_lossy(&rustc_output.stderr).trim()
        );
    }

    let rustc_path = PathBuf::from(String::from_utf8_lossy(&rustc_output.stdout).trim());
    let toolchain_bin_path = rustc_path
        .parent()
        .with_context(|| format!("rustup returned an invalid rustc path: {rustc_path:?}"))?
        .to_path_buf();
    let toolchain_path = toolchain_bin_path
        .parent()
        .with_context(|| format!("rustup returned an invalid toolchain path: {rustc_path:?}"))?;

    let current_lib_path = env::var(LIBRARY_PATH_VAR.to_string()).unwrap_or_default();
    if !current_lib_path.contains(toolchain) {
        let nightly_lib_path = toolchain_path.join("lib");

        let nightly_lib_path = nightly_lib_path
            .to_str()
            .map(|x| x.to_owned())
            .unwrap_or_default();

        ret.insert(LIBRARY_PATH_VAR.to_string(), nightly_lib_path);
    }

    // Keep Cargo, rustc, rustdoc, and the target libraries on the same pinned
    // toolchain when Scout hands control to scout-driver and Dylint. This is
    // needed when a package-manager Rust installation appears before rustup in
    // PATH.
    let current_path = env::var_os("PATH").unwrap_or_default();
    let toolchain_is_first = env::split_paths(&current_path)
        .next()
        .is_some_and(|path| path == toolchain_bin_path);

    if !toolchain_is_first {
        let updated_path = env::join_paths(
            std::iter::once(toolchain_bin_path.clone())
                .chain(env::split_paths(&current_path).filter(|path| path != &toolchain_bin_path)),
        )
        .with_context(|| "Failed to construct PATH for the selected Rust toolchain")?;

        ret.insert(
            "PATH".to_string(),
            updated_path.to_string_lossy().into_owned(),
        );
    }
    Ok(ret)
}

#[cfg(windows)]
#[tracing::instrument(name = "RUN SCOUT IN NIGHTLY", skip_all)]
pub fn run_scout_in_nightly(toolchain: &str) -> Result<Option<Child>> {
    let _ = set_up_environment(toolchain)?;
    print_info("Re-running scout with nightly toolchain...");
    Ok(None)
}

#[cfg(not(windows))]
#[tracing::instrument(name = "RUN SCOUT IN NIGHTLY", skip_all)]
pub fn run_scout_in_nightly(toolchain: &str) -> Result<Option<Child>> {
    let environment = set_up_environment(toolchain)?;

    if environment.is_empty() {
        return Ok(None);
    }

    let program_name =
        env::current_exe().with_context(|| "Failed to get current executable path")?;

    let mut command = Command::new(program_name);
    command.args(env::args().skip(1));

    crate::util::command::set_env(&mut command, &environment);

    let child = command
        .spawn()
        .with_context(|| "Failed to spawn scout with nightly toolchain")?;
    print_info("Re-running scout with nightly toolchain...");
    Ok(Some(child))
}
