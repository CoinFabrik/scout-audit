#![feature(rustc_private)]
extern crate rustc_driver;

use cargo_scout_audit::interop::{
    helper::{HelperInput, HelperOutput},
    subprocess::subprocess_wrapper,
};
use cargo_scout_audit::util::detectors_info::get_detectors_info;
use std::path::PathBuf;

fn main() {
    subprocess_wrapper::<HelperInput, HelperOutput, _>(|i| {
        let detectors_paths = i
            .detectors_paths
            .iter()
            .map(PathBuf::from)
            .collect::<Vec<_>>();
        let result = get_detectors_info(&detectors_paths);

        match result {
            Ok(x) => HelperOutput { result: Ok(x) },
            Err(e) => HelperOutput {
                result: Err(e.to_string()),
            },
        }
    });
}
