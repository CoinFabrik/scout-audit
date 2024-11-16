use crate::detectors::{library::Library, source::download_git_repo};
use anyhow::{Context, Result};
use cargo::{
    core::{Dependency, GitReference, SourceId, Verbosity},
    GlobalContext,
};
use cargo_metadata::MetadataCommand;
use libloading::Symbol;
use std::os::raw::c_uchar;
use std::path::PathBuf;
use std::sync::Arc;

const URL: &str = "https://github.com/CoinFabrik/html-to-pdf";
const BRANCH: &str = "master";

pub fn build_library() -> Result<PathBuf> {
    let dependency = Dependency::parse(
        "library",
        None,
        SourceId::for_git(
            &reqwest::Url::parse(URL)?,
            GitReference::Branch(BRANCH.to_string()),
        )?,
    )
    .with_context(|| "Failed to create git dependency")?;

    let cargo_config =
        GlobalContext::default().with_context(|| "Failed to create default cargo configuration")?;
    cargo_config.shell().set_verbosity(Verbosity::Quiet);

    let path = download_git_repo(&dependency, &cargo_config)
        .with_context(|| "Failed to download PDF generator repository")?;

    let metadata = MetadataCommand::new()
        .current_dir(&path)
        .no_deps()
        .exec()
        .with_context(|| {
            format!(
                "Could not get metadata for the workspace at {}",
                path.to_string_lossy()
            )
        })?;

    let ret = Library::get_compiled_library_paths(&metadata, None)
        .first()
        .cloned()
        .with_context(|| "Failed to determine the PDF generator's expected location")?;

    if !std::fs::exists(&ret)? {
        println!("Building PDF generator. Please wait.");
        crate::utils::cargo::call_cargo(&["build", "--release"], true, None)
            .current_dir(&path)
            .success()
            .with_context(|| "Failed to build PDF generator")?;
    }

    Ok(ret)
}

fn to_vec(s: &str) -> Vec<u8> {
    let mut ret = Vec::from_iter(s.as_bytes().iter().cloned());
    ret.push(0);
    ret
}

type GeneratePdfFunc = unsafe fn(url: *const c_uchar, output_path: *const c_uchar) -> bool;

pub fn call(path: &PathBuf, url: &str, output_path: &str) -> Result<bool> {
    let url = to_vec(url);
    let output_path = to_vec(output_path);

    let lib = unsafe { libloading::Library::new(path) }
        .with_context(|| "Failed to load PDF generator")?;
    let lib = Arc::new(lib);

    let generate_pdf: Symbol<GeneratePdfFunc> =
        unsafe { lib.get(b"generate_pdf") }.with_context(|| "Failed to load PDF generator")?;

    let urlp = url.as_ptr();
    let output_pathp = output_path.as_ptr();

    Ok(unsafe { generate_pdf(urlp, output_pathp) })
}
