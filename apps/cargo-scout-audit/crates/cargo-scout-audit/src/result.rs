use crate::scout::finding::Finding;
use thiserror::Error;

#[derive(Default)]
pub struct ScoutResult {
    pub findings: Vec<Finding>,
    pub stdout_helper: String,
}

impl ScoutResult {
    pub fn new(findings: Vec<Finding>, stdout_helper: String) -> Self {
        Self {
            findings,
            stdout_helper,
        }
    }
    pub fn from_stdout(stdout_helper: String) -> Self {
        Self {
            findings: Vec::new(),
            stdout_helper,
        }
    }
    pub fn from_string<T: std::fmt::Display>(s: T) -> Self {
        Self::from_stdout(format!("{}\n", s))
    }
}

#[derive(Error, Debug)]
pub enum ScoutError {
    #[error("Failed to validate CLI options:\n     → {0}")]
    ValidateFailed(#[source] anyhow::Error),

    #[error("Failed to get project metadata:\n     → {0}")]
    MetadataFailed(#[source] anyhow::Error),

    #[error("Failed to get blockchain dependency:\n     → {0}")]
    BlockchainFailed(#[source] anyhow::Error),

    #[error("Failed to create default cargo configuration")]
    CargoConfigFailed,

    #[error("Failed to get detectors configuration:\n     → {0}")]
    DetectorsConfigFailed(#[source] anyhow::Error),

    #[error("Failed to get detector names:\n     → {0}")]
    GetDetectorNamesFailed(#[source] anyhow::Error),

    #[error("Failed to build detectors:\n     → {0}")]
    BuildDetectorsFailed(#[source] anyhow::Error),

    #[error("Failed to get project info:\n     → {0}")]
    GetProjectInfoFailed(#[source] anyhow::Error),

    #[error("Failed to run dylint:\n     → {0}")]
    RunDylintFailed(#[source] anyhow::Error),
}
