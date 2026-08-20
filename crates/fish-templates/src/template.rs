// Pipeline template structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineTemplate {
    pub name: String,
    pub version: String,
    pub description: String,
    pub template_content: String,
    pub required_variables: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateContext {
    pub variables: HashMap<String, String>,
    pub environment: String,
}
