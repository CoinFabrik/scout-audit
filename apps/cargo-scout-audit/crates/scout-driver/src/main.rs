#![feature(rustc_private)]
extern crate rustc_driver;

use cargo_scout_audit::interop::{
    scout::{DylintOutput, ScoutInput, ScoutOutput},
    subprocess::subprocess_wrapper,
};
use scout_driver::startup::run_dylint;
use std::path::PathBuf;

fn main() {
    subprocess_wrapper::<ScoutInput, ScoutOutput, _>(|i| {
        let detectors_paths = i
            .detectors_paths
            .iter()
            .map(PathBuf::from)
            .collect::<Vec<_>>();
        let result = run_dylint(detectors_paths, &i.opts, i.inside_vscode);

        match result {
            Ok((success, mut temp)) => {
                temp.disable_cleanup(true);
                let output_file_path = cargo_scout_audit::util::path_to_string(temp.path());
                ScoutOutput {
                    result: Ok(DylintOutput {
                        success,
                        output_file_path,
                    }),
                }
            }
            Err(e) => ScoutOutput {
                result: Err(e.to_string()),
            },
        }
    });
}
