use anyhow::{Result, anyhow};
use libloading::{Library, Symbol};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::{ffi::CString, path::PathBuf};

#[derive(Default, Debug, Clone)]
pub struct RawLintInfo {
    pub id: CString,
    pub name: CString,
    pub short_message: CString,
    pub long_message: CString,
    pub severity: CString,
    pub help: CString,
    pub vulnerability_class: CString,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct LintInfo {
    pub id: String,
    pub name: String,
    pub short_message: String,
    pub long_message: String,
    pub severity: String,
    pub help: String,
    pub vulnerability_class: String,
}

#[derive(Default, Serialize, Deserialize, Debug)]
pub struct LintStore {
    lints: HashMap<String, LintInfo>,
}

impl LintStore {
    pub fn new() -> Self {
        Self {
            lints: HashMap::new(),
        }
    }

    pub fn find_by_id(&self, id: &str) -> Option<&LintInfo> {
        self.lints.get(id)
    }

    pub fn insert(&mut self, lint: LintInfo) -> Option<LintInfo> {
        self.lints.insert(lint.id.clone(), lint)
    }

    pub fn iter(&self) -> impl Iterator<Item = &LintInfo> {
        self.lints.values()
    }
}

impl TryFrom<&RawLintInfo> for LintInfo {
    type Error = anyhow::Error;

    fn try_from(info: &RawLintInfo) -> Result<Self, Self::Error> {
        Ok(LintInfo {
            id: info.id.to_str()?.to_string(),
            name: info.name.to_str()?.to_string(),
            short_message: info.short_message.to_str()?.to_string(),
            long_message: info.long_message.to_str()?.to_string(),
            severity: info.severity.to_str()?.to_string(),
            help: info.help.to_str()?.to_string(),
            vulnerability_class: info.vulnerability_class.to_str()?.to_string(),
        })
    }
}

type LintInfoFunc = unsafe fn() -> *mut RawLintInfo;
type FreeLintInfoFunc = unsafe fn(*mut RawLintInfo);

#[tracing::instrument(level = "debug", skip_all)]
pub fn get_detectors_info(detectors_paths: &[PathBuf]) -> Result<LintStore> {
    let mut lint_store = LintStore::new();

    for detector_path in detectors_paths {
        let lib = unsafe {
            Library::new(detector_path)
                .map_err(|e| anyhow!("Failed to load library {}: {}", detector_path.display(), e))?
        };
        let lib = Arc::new(lib);

        let free_lint_info: Symbol<FreeLintInfoFunc> = unsafe {
            lib.get(b"free_lint_info")
                .map_err(|e| anyhow!("Failed to get free_lint_info function: {}", e))?
        };

        let lint_info_func: Symbol<LintInfoFunc> = unsafe {
            lib.get(b"lint_info").map_err(|e| {
                anyhow!(
                    "Failed to get lint_info function from {}: {}",
                    detector_path.display(),
                    e
                )
            })?
        };

        // Call the lint_info function to get the CLintInfo pointer
        let raw_info_ptr = unsafe { lint_info_func() };
        if raw_info_ptr.is_null() {
            return Err(anyhow!(
                "lint_info function from {} returned null pointer",
                detector_path.display()
            ));
        }

        // Convert the raw pointer to a reference and create SerializableLintInfo
        let raw_info = unsafe { &*raw_info_ptr };
        let lint_info = LintInfo::try_from(raw_info).map_err(|e| {
            unsafe { free_lint_info(raw_info_ptr) };
            anyhow!(
                "Failed to convert CLintInfo from {}: {}",
                detector_path.display(),
                e
            )
        })?;

        // Free the raw_info_ptr after successful conversion
        unsafe { free_lint_info(raw_info_ptr) };

        lint_store.insert(lint_info);
    }

    Ok(lint_store)
}
