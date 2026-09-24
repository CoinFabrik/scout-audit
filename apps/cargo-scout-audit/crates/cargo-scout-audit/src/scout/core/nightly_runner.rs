use crate::util::print::print_info;
#[cfg(not(windows))]
use anyhow::Context;
use anyhow::Result;
use lazy_static::lazy_static;
use std::{collections::HashMap, env, process::Child};
#[cfg(not(windows))]
use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
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

    // Keep Cargo behind the rustup proxy when Scout hands control to
    // scout-driver and Dylint. Dylint deliberately removes RUSTUP_TOOLCHAIN
    // before compiling its temporary driver and relies on its rust-toolchain
    // file to select the compiler. Putting the real toolchain bin directory
    // first would bypass rustup and leave the driver without a selected
    // toolchain.
    let current_path = env::var_os("PATH").unwrap_or_default();
    let rustup_proxy_path = find_rustup_proxy_path(&current_path)?;
    let rustup_is_first = env::split_paths(&current_path)
        .next()
        .is_some_and(|path| path == rustup_proxy_path);

    if !rustup_is_first {
        let updated_path = prepend_path(&current_path, &rustup_proxy_path)?;

        ret.insert(
            "PATH".to_string(),
            updated_path.to_string_lossy().into_owned(),
        );
    }
    Ok(ret)
}

#[cfg(not(windows))]
fn find_rustup_proxy_path(path: &OsStr) -> Result<PathBuf> {
    for directory in env::split_paths(path) {
        let rustup = directory.join("rustup");
        if !is_executable(&rustup) {
            continue;
        }

        // Homebrew exposes rustup through a symlink into its Cellar. Resolve
        // that symlink so the sibling Cargo proxy is found in rustup's actual
        // installation directory.
        let Ok(rustup) = rustup.canonicalize() else {
            continue;
        };
        let Some(directory) = rustup.parent() else {
            continue;
        };

        if is_executable(&directory.join("cargo")) {
            return Ok(directory.to_path_buf());
        }
    }

    anyhow::bail!("Failed to locate a rustup proxy directory containing cargo in PATH")
}

#[cfg(not(windows))]
fn prepend_path(path: &OsStr, directory: &Path) -> Result<OsString> {
    env::join_paths(
        std::iter::once(directory.to_path_buf())
            .chain(env::split_paths(path).filter(|entry| entry != directory)),
    )
    .with_context(|| "Failed to construct PATH with the rustup proxy")
}

#[cfg(unix)]
fn is_executable(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt;

    path.metadata()
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(all(not(unix), not(windows)))]
fn is_executable(path: &Path) -> bool {
    path.is_file()
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

#[cfg(all(test, unix))]
mod tests {
    use super::{find_rustup_proxy_path, prepend_path};
    use std::{
        env,
        fs::{self, File},
        os::unix::fs::{PermissionsExt, symlink},
        path::Path,
    };
    use tempfile::tempdir;

    fn executable(path: &Path) {
        File::create(path).unwrap();
        let mut permissions = fs::metadata(path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).unwrap();
    }

    #[test]
    fn finds_canonical_rustup_proxy_directory() {
        let temp = tempdir().unwrap();
        let exposed_bin = temp.path().join("exposed-bin");
        let installation_bin = temp.path().join("installation-bin");
        fs::create_dir_all(&exposed_bin).unwrap();
        fs::create_dir_all(&installation_bin).unwrap();
        executable(&installation_bin.join("rustup"));
        executable(&installation_bin.join("cargo"));
        symlink(installation_bin.join("rustup"), exposed_bin.join("rustup")).unwrap();

        let path = env::join_paths([&exposed_bin]).unwrap();

        assert_eq!(
            find_rustup_proxy_path(&path).unwrap(),
            installation_bin.canonicalize().unwrap()
        );
    }

    #[test]
    fn ignores_rustup_installations_without_a_cargo_proxy() {
        let temp = tempdir().unwrap();
        let invalid_bin = temp.path().join("invalid-bin");
        let valid_bin = temp.path().join("valid-bin");
        fs::create_dir_all(&invalid_bin).unwrap();
        fs::create_dir_all(&valid_bin).unwrap();
        executable(&invalid_bin.join("rustup"));
        executable(&valid_bin.join("rustup"));
        executable(&valid_bin.join("cargo"));

        let path = env::join_paths([&invalid_bin, &valid_bin]).unwrap();

        assert_eq!(
            find_rustup_proxy_path(&path).unwrap(),
            valid_bin.canonicalize().unwrap()
        );
    }

    #[test]
    fn prepends_proxy_without_dropping_other_path_entries() {
        let temp = tempdir().unwrap();
        let package_manager_bin = temp.path().join("package-manager-bin");
        let toolchain_bin = temp.path().join("toolchain-bin");
        let rustup_bin = temp.path().join("rustup-bin");
        let path = env::join_paths([
            &package_manager_bin,
            &toolchain_bin,
            &rustup_bin,
            &rustup_bin,
        ])
        .unwrap();

        let updated = prepend_path(&path, &rustup_bin).unwrap();
        let entries = env::split_paths(&updated).collect::<Vec<_>>();

        assert_eq!(
            entries,
            vec![rustup_bin, package_manager_bin, toolchain_bin]
        );
    }
}
