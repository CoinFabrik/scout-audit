use crate::cli_args::Scout;
use crate::consts::{SCOUT_BRANCH, SCOUT_REPO};
use crate::interop::scout::{ScoutInput, ScoutOutput};
use crate::util::build_and_run::PackageToBuild;
use anyhow::{Result, anyhow};
use std::path::PathBuf;

//#[tracing::instrument(name = "RUN DYLINT", skip_all)]
pub fn run_dylint(
    toolchain: &str,
    detectors_paths: &[PathBuf],
    opts: &Scout,
    inside_vscode: bool,
) -> Result<(bool, PathBuf)> {
    let input = ScoutInput {
        detectors_paths: crate::util::paths_to_strings(detectors_paths),
        opts: opts.clone(),
        inside_vscode,
    };

    let mut pkg = if let Some(root) = &opts.scout_source {
        PackageToBuild::new_local(root.clone())
    } else {
        PackageToBuild::new_remote(SCOUT_REPO, SCOUT_BRANCH, "scout-driver")
    };
    pkg.build_message = "Building scout-driver".to_string();
    pkg.build_error_message = "Failed to build scout-driver".to_string();
    pkg.internal_path = Some("apps/cargo-scout-audit/crates/scout-driver".into());
    let path = pkg.build_executable(Some("scout-driver"))?;

    let output =
        crate::interop::subprocess::run_subprocess::<_, ScoutOutput>(toolchain, &path, &input)?;

    match output.result {
        Ok(x) => Ok((x.success, x.output_file_path.into())),
        Err(e) => Err(anyhow!(e)),
    }
}
