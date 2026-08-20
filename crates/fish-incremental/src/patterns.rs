use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PatternSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternType {
    FrequentRebuilds,
    CascadingChanges,
    SharedDependency,
    Hotspot,
    PolyglotBoundary,
    MissingLockfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildPattern {
    pub pattern_type: PatternType,
    pub severity: PatternSeverity,
    pub description: String,
    pub affected_packages: Vec<String>,
    pub frequency: f64,
}
