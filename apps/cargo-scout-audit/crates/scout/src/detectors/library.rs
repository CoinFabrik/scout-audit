use crate::scout::blockchain::BlockChain;
use anyhow::{Result, bail};
use cargo_metadata::{Metadata, Package};
use itertools::Itertools;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use util::library::{Library, LibraryError};
#[cfg(not(feature = "docker_container"))]
use util::{cargo, env, logger::TracedError};

/// Represents a Rust library.
#[derive(Debug, Clone)]
pub struct DetectorLibrary {
    pub lib: Library,
}

impl DetectorLibrary {
    /// Creates a new instance of `Library`.
    pub fn new(root: PathBuf, toolchain: String, target_dir: PathBuf, metadata: Metadata) -> Self {
        Self {
            lib: Library::new(root, toolchain, target_dir, metadata),
        }
    }

    /// Creates a library from a workspace path
    pub fn create(workspace_path: PathBuf, toolchain: String, target_dir: PathBuf) -> Result<Self> {
        Ok(Self {
            lib: Library::create(workspace_path, toolchain, target_dir)?,
        })
    }

    #[cfg(not(feature = "docker_container"))]
    fn build_workspace(&self, verbose: bool) -> Result<()> {
        cargo::build("detectors", &self.lib.toolchain, !verbose)
            .sanitize_environment()
            .env_remove(env::RUSTFLAGS)
            .current_dir(&self.lib.root)
            .args(["--release"])
            .success()
            .map_err(LibraryError::CargoBuildError(self.lib.root.clone()).traced())?;
        Ok(())
    }

    #[cfg(feature = "docker_container")]
    fn build_workspace(&self, _: bool) -> Result<()> {
        Ok(())
    }

    #[cfg(not(feature = "docker_container"))]
    fn create_target_directory(&self) -> Result<()> {
        let target_dir = self.target_directory();
        if !target_dir.exists() {
            std::fs::create_dir_all(&target_dir)
                .map_err(LibraryError::FileSystemError(target_dir).traced())?;
        }
        Ok(())
    }

    #[cfg(feature = "docker_container")]
    fn create_target_directory(&self) -> Result<()> {
        Ok(())
    }

    /// Builds the library and returns its path.
    pub fn build(&self, verbose: bool) -> Result<Vec<PathBuf>> {
        // Build entire workspace
        self.build_workspace(verbose)?;

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

        self.create_target_directory()?;

        Ok(compiled_library_paths)
    }

    pub fn target_directory(&self) -> PathBuf {
        let ret = self.lib.target_dir.join("scout/libraries");
        if !self.lib.toolchain.is_empty() {
            ret.join(&self.lib.toolchain)
        } else {
            ret
        }
    }

    pub fn get_compiled_library_paths(&self) -> Vec<PathBuf> {
        self.lib.get_compiled_paths(None, None)
    }

    pub fn deduplicate_libraries(libraries: Vec<DetectorLibrary>) -> Vec<DetectorLibrary> {
        let mut unique_packages: HashMap<String, (Package, DetectorLibrary)> = HashMap::new();

        for library in libraries {
            // Determine the detector type for this library (rust or blockchain)
            let detector_type = match library
                .lib
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
            for package in library.lib.metadata.packages.clone() {
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
        let mut result_libraries: HashMap<PathBuf, DetectorLibrary> = HashMap::new();

        for (_, (package, library)) in unique_packages {
            let entry = result_libraries
                .entry(library.lib.root.clone())
                .or_insert_with(|| {
                    let mut lib = library.clone();
                    lib.lib.metadata.packages = Vec::new();
                    lib
                });

            entry.lib.metadata.packages.push(package);
        }

        result_libraries.into_values().collect_vec()
    }
}

fn get_path_str(path: &Path) -> Option<&str> {
    path.file_name().and_then(|n| n.to_str())
}
