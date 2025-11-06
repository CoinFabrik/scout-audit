use crate::util::detectors_info::LintStore;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct HelperInput {
    pub detectors_paths: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HelperOutput {
    pub result: Result<LintStore, String>,
}
