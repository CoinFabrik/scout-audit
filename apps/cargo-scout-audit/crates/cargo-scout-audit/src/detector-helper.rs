use crate::{
    consts::{SCOUT_BRANCH, SCOUT_REPO},
    interop::helper::{HelperInput, HelperOutput},
    util::{build_and_run::PackageToBuild, detectors_info::LintStore},
};
use anyhow::{Result, anyhow};
use std::path::PathBuf;

pub fn get_detectors_info(
    toolchain: &str,
    detectors_paths: &[PathBuf],
    scout_sources: Option<&PathBuf>,
) -> Result<LintStore> {
    let input = HelperInput {
        detectors_paths: crate::util::paths_to_strings(detectors_paths),
    };

    let mut pkg = match scout_sources {
        Some(root) => PackageToBuild::new_local(root.clone()),
        None => PackageToBuild::new_remote(SCOUT_REPO, SCOUT_BRANCH, "scout-driver"),
    };
    pkg.build_message = "Building detector-helper".to_string();
    pkg.build_error_message = "Failed to build detector-helper".to_string();
    pkg.toolchain = Some(format!("+{}", toolchain.trim_start_matches('+')));
    pkg.internal_path = Some("apps/cargo-scout-audit/crates/scout-driver".into());
    let path = pkg.build_executable(Some("scout-driver"), "detector-helper")?;

    let output =
        crate::interop::subprocess::run_subprocess::<_, HelperOutput>(toolchain, &path, &input)?;

    match output.result {
        Ok(x) => Ok(x),
        Err(x) => Err(anyhow!(x)),
    }
}
