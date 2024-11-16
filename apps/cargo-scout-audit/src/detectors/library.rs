use anyhow::Result;
use cargo_metadata::Metadata;
use itertools::Itertools;
use std::{
    env::consts,
    path::{Path, PathBuf},
};

use crate::{
    scout::blockchain::BlockChain,
    utils::{cargo, env},
};
/// Represents a Rust library.
#[derive(Debug, Clone)]
pub struct Library {
    pub root: PathBuf,
    pub toolchain: String,
    pub target_dir: PathBuf,
    pub metadata: Metadata,
}

pub fn get_library_location(target_dir: &Path, toolchain: Option<&str>) -> PathBuf {
    let ret = target_dir.join("scout/libraries");
    match toolchain {
        Some(toolchain) => ret.join(toolchain),
        None => ret,
    }
}

impl Library {
    /// Creates a new instance of `Library`.
    pub fn new(root: PathBuf, toolchain: String, target_dir: PathBuf, metadata: Metadata) -> Self {
        Self {
            root,
            toolchain,
            target_dir,
            metadata,
        }
    }

    /// Builds the library and returns its path.
    pub fn build(&self, bc: &BlockChain, verbose: bool) -> Result<Vec<PathBuf>> {
        // Build entire workspace
        cargo::build("detectors", bc, !verbose)
            .sanitize_environment()
            .env_remove(env::RUSTFLAGS)
            .current_dir(&self.root)
            .args(["--release"])
            .success()?;

        // Verify all libraries were built
        let compiled_library_paths =
            Self::get_compiled_library_paths(&self.metadata, Some(&self.toolchain));

        let unexistant_libraries = compiled_library_paths
            .clone()
            .into_iter()
            .filter(|p| !p.exists())
            .collect_vec();
        if !unexistant_libraries.is_empty() {
            anyhow::bail!("Could not determine if {:?} exist", unexistant_libraries);
        }

        // Copy libraries to target directory
        let target_dir = self.target_directory();
        if !target_dir.exists() {
            std::fs::create_dir_all(&target_dir)?;
        }

        Ok(compiled_library_paths)
    }

    pub fn target_directory(&self) -> PathBuf {
        get_library_location(&self.target_dir, Some(&self.toolchain))
    }

    pub fn get_compiled_library_paths(
        metadata: &Metadata,
        toolchain: Option<&str>,
    ) -> Vec<PathBuf> {
        metadata
            .packages
            .clone()
            .into_iter()
            .map(|p| Self::path(metadata, p.name, toolchain))
            .collect_vec()
    }

    fn path(metadata: &Metadata, library_name: String, toolchain: Option<&str>) -> PathBuf {
        let filename = if let Some(toolchain) = toolchain {
            format!(
                "{}{}@{}{}",
                consts::DLL_PREFIX,
                library_name.replace('-', "_"),
                toolchain,
                consts::DLL_SUFFIX
            )
        } else {
            format!(
                "{}{}{}",
                consts::DLL_PREFIX,
                library_name.replace('-', "_"),
                consts::DLL_SUFFIX
            )
        };
        metadata
            .target_directory
            .clone()
            .into_std_path_buf()
            .join("release")
            .join(filename)
    }
}
