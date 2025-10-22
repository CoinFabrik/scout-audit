use crate::logger::TracedError;
use anyhow::{Result, bail};
use cargo_metadata::{Metadata, MetadataCommand, Package};
use std::{env::consts, path::PathBuf};
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

    /// Creates a new instance of `Library`.
    pub fn new_from_metadata(metadata: Metadata) -> Self {
        Self {
            root: PathBuf::new(),
            toolchain: String::new(),
            target_dir: PathBuf::new(),
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

    pub fn get_compiled_paths(
        &self,
        package_name: Option<&str>,
        kind: Option<&str>,
    ) -> Vec<PathBuf> {
        let mut mapped = self
            .metadata
            .packages
            .clone()
            .into_iter()
            .collect::<Vec<_>>();
        if let Some(package_name) = package_name {
            mapped = mapped
                .into_iter()
                .filter(|x| x.name == package_name)
                .collect::<Vec<_>>()
        };
        if let Some(kind) = kind {
            mapped.iter().flat_map(|p| self.paths(p, kind)).collect()
        } else {
            mapped.iter().filter_map(|p| self.path(p)).collect()
        }
    }

    fn path(&self, pkg: &Package) -> Option<PathBuf> {
        let target = pkg.targets.first()?;
        if target.kind.len() != 1 {
            return None;
        }
        self.path_with_kind(pkg, target.kind.first()?)
    }

    fn path_with_kind(&self, pkg: &Package, kind: &str) -> Option<PathBuf> {
        Some(if kind == "lib" || kind == "cdylib" {
            self.library_path(pkg.name.clone())
        } else {
            self.binary_path(pkg.name.clone())
        })
    }

    fn paths(&self, pkg: &Package, expected_kind: &str) -> Vec<PathBuf> {
        let mut ret = Vec::new();

        for target in pkg.targets.iter() {
            if !target.kind.iter().any(|k| k == expected_kind) {
                continue;
            }
            let Some(path) = self.path_with_kind(pkg, expected_kind) else {
                continue;
            };
            ret.push(path);
        }

        ret
    }

    fn library_path(&self, library_name: String) -> PathBuf {
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

    fn binary_path(&self, binary_name: String) -> PathBuf {
        let filename = if self.toolchain.is_empty() {
            format!("{}{}", binary_name.replace('_', "-"), consts::EXE_SUFFIX)
        } else {
            format!(
                "{}@{}{}",
                binary_name.replace('_', "-"),
                self.toolchain,
                consts::EXE_SUFFIX
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
}
