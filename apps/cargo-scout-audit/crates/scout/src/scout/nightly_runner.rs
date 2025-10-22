#[cfg(not(windows))]
use anyhow::Context;
use anyhow::Result;
#[cfg(not(windows))]
use current_platform::CURRENT_PLATFORM;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::{env, path::PathBuf, process::Child};
#[cfg(not(windows))]
use std::{path::Path, process::Command};
#[cfg(not(windows))]
use util::print::{print_info, print_warning};

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

    let current_lib_path = env::var(LIBRARY_PATH_VAR.to_string()).unwrap_or_default();
    if !current_lib_path.contains(toolchain) {
        let rustup_home = env::var("RUSTUP_HOME");

        let rustup_home = match rustup_home {
            Ok(x) => PathBuf::from(x),
            Err(_) => {
                let mut home =
                    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "~".to_string()));
                home.push(".rustup");
                print_warning(&format!(
                    "Failed to get RUSTUP_HOME, defaulting to {:?}",
                    &home
                ));
                home
            }
        };

        let nightly_lib_path = Path::new(&rustup_home)
            .join("toolchains")
            .join(format!("{}-{}", toolchain, CURRENT_PLATFORM))
            .join("lib");

        let nightly_lib_path = nightly_lib_path
            .to_str()
            .map(|x| x.to_owned())
            .unwrap_or_default();

        ret.insert(LIBRARY_PATH_VAR.to_string(), nightly_lib_path);
    }
    Ok(ret)
}

#[cfg(windows)]
#[tracing::instrument(name = "RUN SCOUT IN NIGHTLY", skip_all)]
pub fn run_scout_in_nightly(toolchain: &str) -> Result<Option<Child>> {
    let _ = set_up_environment(toolchain)?;
    print_info("Re-running scout with nightly toolchain...");
    return Ok(None);
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

    util::command::set_env(&mut command, &environment);

    let child = command
        .spawn()
        .with_context(|| "Failed to spawn scout with nightly toolchain")?;
    print_info("Re-running scout with nightly toolchain...");
    Ok(Some(child))
}
