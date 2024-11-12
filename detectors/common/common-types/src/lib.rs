mod lint_info;
pub use lint_info::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    Enhancement,
    Medium,
    Minor,
}

impl AsRef<str> for Severity {
    fn as_ref(&self) -> &str {
        match self {
            Severity::Critical => "critical",
            Severity::Enhancement => "enhancement",
            Severity::Medium => "medium",
            Severity::Minor => "minor",
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum VulnerabilityClass {
    Arithmetic,
    Authorization,
    BestPractices,
    BlockAttributes,
    DoS,
    ErrorHandling,
    GasUsage,
    KnownBugs,
    MEV,
    Panic,
    Reentrancy,
    ResourceManagement,
    Upgradability,
}

impl AsRef<str> for VulnerabilityClass {
    fn as_ref(&self) -> &str {
        match self {
            VulnerabilityClass::Arithmetic => "arithmetic",
            VulnerabilityClass::Authorization => "authorization",
            VulnerabilityClass::BestPractices => "best-practices",
            VulnerabilityClass::BlockAttributes => "block-attributes",
            VulnerabilityClass::DoS => "dos",
            VulnerabilityClass::ErrorHandling => "error-handling",
            VulnerabilityClass::GasUsage => "gas-usage",
            VulnerabilityClass::KnownBugs => "known-bugs",
            VulnerabilityClass::MEV => "mev",
            VulnerabilityClass::Panic => "panic",
            VulnerabilityClass::Reentrancy => "reentrancy",
            VulnerabilityClass::ResourceManagement => "resource-management",
            VulnerabilityClass::Upgradability => "upgradability",
        }
    }
}
