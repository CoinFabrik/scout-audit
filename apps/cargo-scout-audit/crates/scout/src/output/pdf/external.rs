use anyhow::{Context, Result};
use libloading::Symbol;
use std::{os::raw::c_uchar, path::PathBuf, sync::Arc};
use util::build_and_run::PackageToBuild;

const URL: &str = "https://github.com/CoinFabrik/html-to-pdf";
const BRANCH: &str = "master";

pub fn build_library() -> Result<PathBuf> {
    let mut pkg = PackageToBuild::new_remote(URL, BRANCH, "library");
    pkg.build_message = "Building PDF generator. Please wait.".to_string();
    pkg.build_error_message = "Failed to build PDF generator".to_string();
    pkg.build_library(None)
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
