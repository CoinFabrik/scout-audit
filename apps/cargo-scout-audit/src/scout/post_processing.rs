use anyhow::{anyhow, Context, Result};
use libloading::{Library, Symbol};
use std::ffi::{c_char, CStr, CString};
use std::path::Path;
use crate::finding::Finding;

type ProcessFindingsFunc = unsafe fn(*const c_char, *const c_char, bool) -> *mut c_char;
type FreeStringFunc = unsafe fn(*mut c_char);

struct FindingProcessor {
    lib: Library,
}

impl FindingProcessor {
    pub fn new<P: AsRef<Path>>(library_path: P) -> Result<Self> {
        let lib = unsafe {
            Library::new(library_path.as_ref()).with_context(|| {
                format!("Failed to load library {}", library_path.as_ref().display())
            })?
        };

        Ok(FindingProcessor { lib })
    }

    pub fn process_findings(
        &self,
        successful_findings: &[Finding],
        output: &[Finding],
        inside_vscode: bool,
    ) -> Result<(Vec<Finding>, String)> {
        let successful_findings_json = serde_json::to_string(
                &successful_findings
                    .iter()
                    .cloned()
                    .map(|x| x.decompose())
                    .collect::<Vec<_>>()
            )
            .with_context(|| "Failed to serialize successful_findings")?;
        let output_json = serde_json::to_string(
                &output
                    .iter()
                    .cloned()
                    .map(|x| x.decompose())
                    .collect::<Vec<_>>()
            ).with_context(|| "Failed to serialize output")?;

        let successful_findings_cstring = CString::new(successful_findings_json)
            .with_context(|| "Failed to create CString for successful_findings")?;
        let output_cstring =
            CString::new(output_json).with_context(|| "Failed to create CString for output")?;

        let process_func: Symbol<ProcessFindingsFunc> = unsafe {
            self.lib
                .get(b"process_findings")
                .with_context(|| "Failed to get process_findings function")?
        };

        let free_func: Symbol<FreeStringFunc> = unsafe {
            self.lib
                .get(b"free_string")
                .with_context(|| "Failed to get free_string function")?
        };

        let result_ptr = unsafe {
            process_func(
                successful_findings_cstring.as_ptr(),
                output_cstring.as_ptr(),
                inside_vscode,
            )
        };

        if result_ptr.is_null() {
            return Err(anyhow!("process_findings returned null"));
        }

        let result_cstr = unsafe { CStr::from_ptr(result_ptr) };
        let result_str = result_cstr
            .to_str()
            .with_context(|| "Failed to convert result to str")?;
        let result: serde_json::Value =
            serde_json::from_str(result_str).with_context(|| "Failed to parse result JSON")?;

        // Ensure we free the memory allocated by the C function
        unsafe { free_func(result_ptr) };

        let console_findings = result["console_findings"]
            .as_array()
            .ok_or_else(|| anyhow!("Failed to parse console_findings"))?
            .clone();
        let output_string_vscode = result["output_string_vscode"]
            .as_str()
            .ok_or_else(|| anyhow!("Failed to parse output_string_vscode"))?
            .to_string();

        let console_findings = console_findings
            .into_iter()
            .map(|x| Finding::new(x))
            .collect::<Vec<_>>();

        Ok((console_findings, output_string_vscode))
    }
}

pub struct PostProcessing {
    processor: FindingProcessor,
}

impl PostProcessing {
    pub fn new<P: AsRef<Path>>(library_path: P) -> Result<Self> {
        let processor = FindingProcessor::new(library_path)?;
        Ok(PostProcessing { processor })
    }

    pub fn process(
        &self,
        successful_findings: Vec<Finding>,
        output: Vec<Finding>,
        inside_vscode: bool,
    ) -> Result<(Vec<Finding>, String)> {
        self.processor
            .process_findings(&successful_findings, &output, inside_vscode)
    }
}
