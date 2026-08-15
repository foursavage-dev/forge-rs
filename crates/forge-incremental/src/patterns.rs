// Build pattern detection

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildPattern {
    pub pattern_type: PatternType,
    pub frequency: f64,
    pub affected_packages: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternType {
    FrequentRebuilds,
    CascadingChanges,
    SharedDependency,
    Hotspot,
}
