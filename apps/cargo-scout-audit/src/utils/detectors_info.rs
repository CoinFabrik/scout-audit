use anyhow::{Ok, Result};
use serde::Serialize;
use std::{collections::HashMap, ffi, path::PathBuf};

#[derive(Default, Debug, Clone, Serialize)]
pub struct RawLintInfo {
    pub id: ffi::CString,
    pub name: ffi::CString,
    pub short_message: ffi::CString,
    pub long_message: ffi::CString,
    pub severity: ffi::CString,
    pub help: ffi::CString,
    pub vulnerability_class: ffi::CString,
}

#[derive(Default, Debug, Clone, Serialize)]
pub struct LintInfo {
    pub id: String,
    pub name: String,
    pub short_message: String,
    pub long_message: String,
    pub severity: String,
    pub help: String,
    pub vulnerability_class: String,
}

impl From<&RawLintInfo> for LintInfo {
    fn from(info: &RawLintInfo) -> Self {
        LintInfo {
            id: info.id.to_str().unwrap().to_string(),
            name: info.name.to_str().unwrap().to_string(),
            short_message: info.short_message.to_str().unwrap().to_string(),
            long_message: info.long_message.to_str().unwrap().to_string(),
            severity: info.severity.to_str().unwrap().to_string(),
            help: info.help.to_str().unwrap().to_string(),
            vulnerability_class: info.vulnerability_class.to_str().unwrap().to_string(),
        }
    }
}

type LintInfoFunc = unsafe fn(info: &mut RawLintInfo);

pub fn get_detectors_info(detectors_paths: &Vec<PathBuf>) -> Result<HashMap<String, LintInfo>> {
    let mut lint_store = HashMap::<String, LintInfo>::default();

    for detector_path in detectors_paths {
        unsafe {
            let lib_res = libloading::os::unix::Library::open(
                Some(detector_path),
                libloading::os::unix::RTLD_LAZY | libloading::os::unix::RTLD_LOCAL,
            );

            let lib = lib_res.unwrap();
            let lint_info_func_res = lib.get::<LintInfoFunc>(b"lint_info");
            if lint_info_func_res.is_ok() {
                let lint_info_func = lint_info_func_res.unwrap();
                let mut info = RawLintInfo::default();
                lint_info_func(&mut info);
                let lint_info = LintInfo::from(&info);
                lint_store.insert(lint_info.id.clone(), lint_info);
            }
        }
    }
    Ok(lint_store)
}
