use anyhow::{anyhow, Result};
use libloading::{Library, Symbol};
use serde_json::Value;
use std::ffi::CString;
use std::path::Path;

type ShouldIncludeFindingFunc =
    unsafe fn(*const std::os::raw::c_char, *const std::os::raw::c_char) -> bool;

struct FindingProcessor {
    lib: Library,
}

impl FindingProcessor {
    pub fn new<P: AsRef<Path>>(library_path: P) -> Result<Self> {
        let lib = unsafe {
            Library::new(library_path.as_ref()).map_err(|e| {
                anyhow!(
                    "Failed to load library {}: {}",
                    library_path.as_ref().display(),
                    e
                )
            })?
        };

        Ok(FindingProcessor { lib })
    }

    pub fn should_include_finding(&self, finding: &Value, all_findings: &[Value]) -> Result<bool> {
        let finding_json = serde_json::to_string(finding)?;
        let all_findings_json = serde_json::to_string(all_findings)?;

        let finding_cstring = CString::new(finding_json)?;
        let all_findings_cstring = CString::new(all_findings_json)?;

        let func: Symbol<ShouldIncludeFindingFunc> = unsafe {
            self.lib
                .get(b"should_include_finding")
                .map_err(|e| anyhow!("Failed to get should_include_finding function: {}", e))?
        };

        let result = unsafe { func(finding_cstring.as_ptr(), all_findings_cstring.as_ptr()) };

        Ok(result)
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
        successful_findings: Vec<Value>,
        output: Vec<Value>,
        inside_vscode: bool,
    ) -> Result<(Vec<Value>, String)> {
        // Console output
        let console_findings: Vec<_> = successful_findings
            .iter()
            .filter_map(|finding| {
                match self
                    .processor
                    .should_include_finding(finding, &successful_findings)
                {
                    Ok(true) => Some(finding.clone()),
                    Ok(false) => None,
                    Err(e) => {
                        eprintln!("Error processing finding: {}", e);
                        None
                    }
                }
            })
            .collect();

        // Vscode output
        let output_vscode: Vec<_> = if inside_vscode {
            let all_findings: Vec<_> = output
                .iter()
                .filter_map(|val| val.get("message").cloned())
                .collect();

            output
                .into_iter()
                .filter_map(|val| match val.get("message") {
                    Some(message) => {
                        match self
                            .processor
                            .should_include_finding(message, &all_findings)
                        {
                            Ok(true) => Some(val),
                            Ok(false) => None,
                            Err(e) => {
                                eprintln!("Error processing finding: {}", e);
                                None
                            }
                        }
                    }
                    None => Some(val),
                })
                .collect()
        } else {
            Vec::new()
        };

        // Convert output_vscode to a string, but not only one json, each value of the array is a json
        let output_string_vscode = output_vscode
            .into_iter()
            .map(|finding| serde_json::to_string(&finding).unwrap_or_default())
            .collect::<Vec<_>>()
            .join("\n");

        Ok((console_findings, output_string_vscode))
    }
}
