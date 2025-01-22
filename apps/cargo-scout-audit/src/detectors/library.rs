use crate::{
    scout::blockchain::BlockChain,
    utils::{cargo, env, telemetry::TracedError},
};
use anyhow::{bail, Result};
use cargo_metadata::{Metadata, MetadataCommand, Package};
use itertools::Itertools;
use std::{
    collections::HashMap,
    env::consts,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LibraryError {
    #[error("\n     → Expected a valid workspace directory, but '{0}' is not a directory")]
    NotADirectory(PathBuf),

    #[error("\n     → Failed to find the following compiled libraries:\n{}", .0.iter().map(|p| format!("  - {}", p.display())).collect::<Vec<_>>().join("\n"))]
    MissingLibraries(Vec<PathBuf>),

    #[error("\n     → Failed to get cargo metadata for workspace at '{0}'.")]
    MetadataError(PathBuf),

    #[error("\n     → Failed to execute cargo build at {0}.")]
    CargoBuildError(PathBuf),

    #[error("\n     → Failed to create or access directory at {0}.")]
    FileSystemError(PathBuf),
}

/// Represents a Rust library.
#[derive(Debug, Clone)]
pub struct Library {
    pub root: PathBuf,
    pub toolchain: String,
    pub target_dir: PathBuf,
    pub metadata: Metadata,
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

    /// Creates a library from a workspace path
    pub fn create(workspace_path: PathBuf, toolchain: String, target_dir: PathBuf) -> Result<Self> {
        if !workspace_path.is_dir() {
            bail!(LibraryError::NotADirectory(workspace_path));
        }

        let metadata = Self::get_package_metadata(&workspace_path)?;

        Ok(Library::new(
            workspace_path,
            toolchain,
            target_dir,
            metadata,
        ))
    }

    /// Builds the library and returns its path.
    pub fn build(&self, bc: &BlockChain, verbose: bool) -> Result<Vec<PathBuf>> {
        // Build entire workspace
        cargo::build("detectors", bc, !verbose)
            .sanitize_environment()
            .env_remove(env::RUSTFLAGS)
            .current_dir(&self.root)
            .args(["--release"])
            .success()
            .map_err(LibraryError::CargoBuildError(self.root.clone()).traced())?;

        // Verify all libraries were built
        let compiled_library_paths = self.get_compiled_library_paths();

        let unexistant_libraries = compiled_library_paths
            .clone()
            .into_iter()
            .filter(|p| !p.exists())
            .collect_vec();

        if !unexistant_libraries.is_empty() {
            bail!(LibraryError::MissingLibraries(unexistant_libraries));
        }

        // Ensure target directory exists
        let target_dir = self.target_directory();
        if !target_dir.exists() {
            std::fs::create_dir_all(&target_dir)
                .map_err(LibraryError::FileSystemError(target_dir).traced())?;
        }

        Ok(compiled_library_paths)
    }

    pub fn target_directory(&self) -> PathBuf {
        let ret = self.target_dir.join("scout/libraries");
        if !self.toolchain.is_empty() {
            ret.join(&self.toolchain)
        } else {
            ret
        }
    }

    pub fn get_compiled_library_paths(&self) -> Vec<PathBuf> {
        self.metadata
            .packages
            .clone()
            .into_iter()
            .map(|p| self.path(p.name))
            .collect_vec()
    }

    fn path(&self, library_name: String) -> PathBuf {
        let filename = if self.toolchain.is_empty() {
            format!(
                "{}{}{}",
                consts::DLL_PREFIX,
                library_name.replace('-', "_"),
                consts::DLL_SUFFIX
            )
        } else {
            format!(
                "{}{}@{}{}",
                consts::DLL_PREFIX,
                library_name.replace('-', "_"),
                self.toolchain,
                consts::DLL_SUFFIX
            )
        };

        self.metadata
            .target_directory
            .clone()
            .into_std_path_buf()
            .join("release")
            .join(filename)
    }

    fn get_package_metadata(workspace_path: &PathBuf) -> Result<Metadata> {
        MetadataCommand::new()
            .current_dir(workspace_path)
            .no_deps()
            .exec()
            .map_err(LibraryError::MetadataError(workspace_path.clone()).traced())
    }

    pub fn deduplicate_libraries(libraries: Vec<Library>) -> Vec<Library> {
        let mut unique_packages: HashMap<String, (Package, Library)> = HashMap::new();

        for library in libraries {
            // Determine the detector type for this library (rust or blockchain)
            let detector_type = match library
                .root
                .ancestors()
                .find(|p| {
                    get_path_str(p).is_some_and(|name| {
                        name == "rust" || BlockChain::variants().contains(&name.to_string())
                    })
                })
                .and_then(get_path_str)
            {
                Some(dtype) => dtype,
                None => continue,
            };

            // Process each package in the library
            for package in library.metadata.packages.clone() {
                let should_insert = match unique_packages.get(&package.name) {
                    None => true,
                    Some(_) => detector_type != "rust",
                };

                if should_insert {
                    unique_packages.insert(package.name.clone(), (package, library.clone()));
                }
            }
        }

        // Reconstruct libraries with deduplicated packages
        let mut result_libraries: HashMap<PathBuf, Library> = HashMap::new();

        for (_, (package, library)) in unique_packages {
            let entry = result_libraries
                .entry(library.root.clone())
                .or_insert_with(|| {
                    let mut lib = library.clone();
                    lib.metadata.packages = Vec::new();
                    lib
                });

            entry.metadata.packages.push(package);
        }

        result_libraries.into_values().collect_vec()
    }
}

fn get_path_str(path: &Path) -> Option<&str> {
    path.file_name().and_then(|n| n.to_str())
}
