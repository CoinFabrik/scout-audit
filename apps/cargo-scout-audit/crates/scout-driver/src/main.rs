#![feature(rustc_private)]
extern crate rustc_driver;

use scout_driver::{
    startup::run_dylint,
};
use tracing::level_filters::LevelFilter;
use interop::{
    scout::{
        ScoutInput,
        ScoutOutput,
        DylintOutput,
    },
    subprocess::subprocess_wrapper,
};
use std::path::PathBuf;

fn main() {
    subprocess_wrapper::<ScoutInput, ScoutOutput, _>(|i|{
        let detectors_paths = i.detectors_paths
            .iter()
            .map(|x| PathBuf::from(x))
            .collect::<Vec<_>>();
        let result = run_dylint(detectors_paths, &i.opts, i.inside_vscode);

        match result{
            Ok((success, mut temp)) => {
                temp.disable_cleanup(true);
                let output_file_path = util::path_to_string(temp.path());
                ScoutOutput{
                    result: Ok(DylintOutput{
                        success,
                        output_file_path,
                    })
                }
            },
            Err(e) => {
                ScoutOutput{
                    result: Err(e.to_string()),
                }
            }
        }
    });
}
