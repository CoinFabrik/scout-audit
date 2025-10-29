use cli_args::Scout;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct ScoutInput {
    pub detectors_paths: Vec<String>,
    pub opts: Scout,
    pub inside_vscode: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DylintOutput {
    pub success: bool,
    pub output_file_path: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ScoutOutput {
    pub result: Result<DylintOutput, String>,
}
