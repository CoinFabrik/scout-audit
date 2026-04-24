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
            Severity::Critical => "Critical",
            Severity::Enhancement => "Enhancement",
            Severity::Medium => "Medium",
            Severity::Minor => "Minor",
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
            VulnerabilityClass::Arithmetic => "Arithmetic",
            VulnerabilityClass::Authorization => "Authorization",
            VulnerabilityClass::BestPractices => "Best Practices",
            VulnerabilityClass::BlockAttributes => "Block Attributes",
            VulnerabilityClass::DoS => "DoS",
            VulnerabilityClass::ErrorHandling => "Error Handling",
            VulnerabilityClass::GasUsage => "Gas Usage",
            VulnerabilityClass::KnownBugs => "Known Bugs",
            VulnerabilityClass::MEV => "MEV",
            VulnerabilityClass::Panic => "Panic",
            VulnerabilityClass::Reentrancy => "Reentrancy",
            VulnerabilityClass::ResourceManagement => "Resource Management",
            VulnerabilityClass::Upgradability => "Upgradability",
        }
    }
}
