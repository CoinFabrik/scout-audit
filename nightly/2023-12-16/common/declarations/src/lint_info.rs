use std::ffi::CString;

use thiserror::Error;

use crate::{Severity, VulnerabilityClass};

#[derive(Debug, Clone, Copy)]
pub struct LintInfo<'a> {
    pub name: &'a str,
    pub short_message: &'a str,
    pub long_message: &'a str,
    pub severity: Severity,
    pub help: &'a str,
    pub vulnerability_class: VulnerabilityClass,
}

#[repr(C)]
pub struct CLintInfo {
    pub id: CString,
    pub name: CString,
    pub short_message: CString,
    pub long_message: CString,
    pub severity: CString,
    pub help: CString,
    pub vulnerability_class: CString,
}

#[derive(Error, Debug)]
pub enum LintInfoError {
    #[error("Failed to convert string to CString: {0}")]
    StringConversion(#[from] std::ffi::NulError),
    #[error("Null pointer encountered")]
    NullPointer,
}

impl<'a> LintInfo<'a> {
    pub fn into_c(&self) -> Result<CLintInfo, LintInfoError> {
        Ok(CLintInfo {
            id: CString::new(self.name.to_lowercase().replace('-', "_"))?,
            name: CString::new(snake_to_title_case(self.name))?,
            short_message: CString::new(self.short_message)?,
            long_message: CString::new(self.long_message)?,
            severity: CString::new(self.severity.as_ref())?,
            help: CString::new(self.help)?,
            vulnerability_class: CString::new(self.vulnerability_class.as_ref())?,
        })
    }

    pub fn create_lint_info(info: &'static Self) -> *mut CLintInfo {
        match info.into_c() {
            Ok(c_info) => Box::into_raw(Box::new(c_info)),
            Err(_) => std::ptr::null_mut(),
        }
    }
}

/// # Safety
///
/// This function is unsafe because it deallocates the memory of the `CLintInfo` struct.
#[no_mangle]
pub unsafe extern "C" fn free_lint_info(ptr: *mut CLintInfo) {
    if !ptr.is_null() {
        let _ = unsafe { Box::from_raw(ptr) };
    }
}

fn snake_to_title_case(input: &str) -> String {
    // Split the string by underscores
    let words: Vec<&str> = input.split('-').collect();

    // Capitalize first letter of each word and join with spaces
    words
        .iter()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<String>>()
        .join(" ")
}
