use serde::de::Error;
use serde_json::Value;
use std::{collections::HashMap, ffi::CStr, ffi::CString, os::raw::c_char};

#[derive(Debug, Clone)]
struct Finding {
    detector: String,
    file_name: String,
    span: (usize, usize),
    allowed_lint: Option<String>,
}

struct FileFindings {
    unnecessary_allows: Vec<Finding>,
    other_findings: Vec<Finding>,
}

struct FindingsCache {
    by_file: HashMap<String, FileFindings>,
}

impl FindingsCache {
    fn new(all_findings: &[Value]) -> Self {
        let mut by_file: HashMap<String, FileFindings> = HashMap::new();

        for finding in all_findings {
            if let Some(parsed) = parse_finding(finding) {
                by_file
                    .entry(parsed.file_name.clone())
                    .or_insert_with(|| FileFindings {
                        unnecessary_allows: Vec::new(),
                        other_findings: Vec::new(),
                    })
                    .add_finding(parsed);
            }
        }

        FindingsCache { by_file }
    }
}

impl FileFindings {
    fn add_finding(&mut self, finding: Finding) {
        if finding.detector == "unnecessary_lint_allow" {
            self.unnecessary_allows.push(finding);
        } else {
            self.other_findings.push(finding);
        }
    }
}

fn parse_finding(finding: &Value) -> Option<Finding> {
    let detector = finding.get("code")?.get("code")?.as_str()?;
    let span = finding.get("spans")?.as_array()?.first()?;
    let file_name = span.get("file_name")?.as_str()?;
    let line_start = span.get("line_start")?.as_u64()?;
    let line_end = span.get("line_end")?.as_u64()?;

    let allowed_lint = if detector == "unnecessary_lint_allow" {
        finding
            .get("children")?
            .get(0)?
            .get("message")?
            .as_str()
            .and_then(|msg| msg.split('`').nth(1).map(String::from))
    } else {
        None
    };

    let start = usize::try_from(line_start).ok()?;
    let end = usize::try_from(line_end).ok()?;

    Some(Finding {
        detector: detector.to_owned(),
        file_name: file_name.to_owned(),
        span: (start, end),
        allowed_lint,
    })
}

fn spans_overlap(span1: (usize, usize), span2: (usize, usize)) -> bool {
    span1.0 <= span2.1 && span2.0 <= span1.1
}

fn process_findings_impl(
    successful_findings: Vec<Value>,
    output: Vec<Value>,
    inside_vscode: bool,
) -> (Vec<Value>, String) {
    let findings_cache = FindingsCache::new(&successful_findings);

    let console_findings: Vec<_> = successful_findings
        .into_iter()
        .filter(|finding| should_include_finding_impl(finding, &findings_cache))
        .collect();

    let output_vscode: Vec<_> = if inside_vscode {
        output
            .into_iter()
            .filter(|val| {
                val.get("message")
                    .map(|message| should_include_finding_impl(message, &findings_cache))
                    .unwrap_or(true)
            })
            .collect()
    } else {
        Vec::new()
    };

    let output_string_vscode = output_vscode
        .into_iter()
        .filter_map(|finding| serde_json::to_string(&finding).ok())
        .collect::<Vec<_>>()
        .join("\n");

    (console_findings, output_string_vscode)
}

fn should_include_finding_impl(finding: &Value, cache: &FindingsCache) -> bool {
    let current_finding = match parse_finding(finding) {
        Some(f) => f,
        None => return false, // If we can't parse the finding, we don't include it
    };

    if let Some(file_findings) = cache.by_file.get(&current_finding.file_name) {
        if current_finding.detector == "unnecessary_lint_allow" {
            if let Some(allowed_lint) = &current_finding.allowed_lint {
                !file_findings.other_findings.iter().any(|f| {
                    &f.detector == allowed_lint && spans_overlap(f.span, current_finding.span)
                })
            } else {
                true // Include if we can't determine the allowed lint
            }
        } else {
            !file_findings.unnecessary_allows.iter().any(|allow| {
                allow
                    .allowed_lint
                    .as_ref()
                    .map_or(false, |lint| lint == &current_finding.detector)
                    && spans_overlap(allow.span, current_finding.span)
            })
        }
    } else {
        true // If we can't find the file, we include it by default
    }
}

/// Process the findings to filter out unnecessary findings.
///
/// # Safety
///
/// This function is marked as unsafe because it deals with raw pointers.
/// The caller is responsible for ensuring the safety of the pointers passed as arguments.
#[no_mangle]
pub unsafe extern "C" fn process_findings(
    successful_findings_json: *const c_char,
    output_json: *const c_char,
    inside_vscode: bool,
) -> *mut c_char {
    let successful_findings = match parse_json(successful_findings_json) {
        Ok(Value::Array(v)) => v,
        _ => return std::ptr::null_mut(),
    };

    let output = match parse_json(output_json) {
        Ok(Value::Array(v)) => v,
        _ => return std::ptr::null_mut(),
    };

    let (console_findings, output_string_vscode) =
        process_findings_impl(successful_findings, output, inside_vscode);

    let result = serde_json::json!({
        "console_findings": console_findings,
        "output_string_vscode": output_string_vscode
    });

    match serde_json::to_string(&result) {
        Ok(result_string) => match CString::new(result_string) {
            Ok(c_str) => c_str.into_raw(),
            Err(_) => std::ptr::null_mut(),
        },
        Err(_) => std::ptr::null_mut(),
    }
}

unsafe fn parse_json(input: *const c_char) -> Result<Value, serde_json::Error> {
    let c_str = CStr::from_ptr(input);
    let input_str = c_str
        .to_str()
        .map_err(|_| serde_json::Error::custom("Invalid UTF-8"))?;
    serde_json::from_str(input_str)
}

// Free the string allocated by the `process_findings` function. Should be called after processing everything
#[no_mangle]
pub unsafe extern "C" fn free_string(ptr: *mut c_char) {
    if !ptr.is_null() {
        let _ = CString::from_raw(ptr);
    }
}
