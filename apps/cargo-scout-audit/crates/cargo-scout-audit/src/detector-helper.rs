use util::{
    detectors_info::LintStore,
    build_and_run::PackageToBuild,
};
use std::{
    path::PathBuf,
};
use interop::helper::{HelperInput, HelperOutput};
use anyhow::{
    Result,
    anyhow,
};
use crate::consts::{
    SCOUT_REPO,
    SCOUT_BRANCH,
};

#[cfg(not(windows))]
pub fn get_detectors_info(toolchain: &String, detectors_paths: &[PathBuf]) -> Result<LintStore> {
    let input = HelperInput{
        detectors_paths: util::paths_to_strings(detectors_paths)
    };

    let mut pkg = PackageToBuild::new(SCOUT_REPO, SCOUT_BRANCH, "detector-helper");
    pkg.build_message = "Building detector-helper".to_string();
    pkg.build_error_message = "Failed to build detector-helper".to_string();
    pkg.internal_path = Some("apps/cargo-scout-audit/crates/detector-helper".into());
    let path = pkg.build_executable(Some("detector-helper"))?;

    let output = interop::subprocess::run_subprocess::<_, HelperOutput>(
        toolchain,
        &path,
        &input
    )?;

    match output.result{
        Ok(x) => Ok(x),
        Err(x) => Err(anyhow!(x)),
    }
}
