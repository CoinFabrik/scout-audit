use cli_args::Scout;
use anyhow::{anyhow, Result};
use std::path::PathBuf;
use interop::scout::{
    ScoutInput,
    ScoutOutput,
};
use crate::consts::{
    SCOUT_REPO,
    SCOUT_BRANCH,
};
use util::build_and_run::PackageToBuild;

//#[tracing::instrument(name = "RUN DYLINT", skip_all)]
pub fn run_dylint(toolchain: &String, detectors_paths: &Vec<PathBuf>, opts: &Scout, inside_vscode: bool) -> Result<(bool, PathBuf)> {
    let input = ScoutInput{
        detectors_paths: util::paths_to_strings(detectors_paths),
        opts: opts.clone(),
        inside_vscode,
    };

    let mut pkg = PackageToBuild::new(SCOUT_REPO, SCOUT_BRANCH, "scout-driver");
    pkg.build_message = "Building scout-driver".to_string();
    pkg.build_error_message = "Failed to build scout-driver".to_string();
    pkg.internal_path = Some("apps/cargo-scout-audit/crates/scout-driver".into());
    let path = pkg.build_executable(Some("scout-driver"))?;

    let output = interop::subprocess::run_subprocess::<_, ScoutOutput>(toolchain, &path, &input)?;

    match output.result{
        Ok(x) => Ok((x.success, x.output_file_path.into())),
        Err(e) => Err(anyhow!(e))
    }
}
