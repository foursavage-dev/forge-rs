// Template registry

use crate::template::PipelineTemplate;
use std::collections::HashMap;

#[derive(Clone)]
pub struct TemplateRegistry {
    templates: HashMap<String, PipelineTemplate>,
}

impl TemplateRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            templates: HashMap::new(),
        };
        registry.register_builtin_templates();
        registry
    }

    fn register_builtin_templates(&mut self) {
        // Register built-in templates
        self.templates.insert(
            "monorepo".to_string(),
            PipelineTemplate {
                name: "monorepo".to_string(),
                version: "1.0.0".to_string(),
                description: "Standard monorepo pipeline template".to_string(),
                template_content: include_str!("../templates/monorepo.hbs").to_string(),
                required_variables: vec!["project_name".to_string()],
            },
        );
    }

    pub fn get_template(&self, name: &str) -> Result<PipelineTemplate, anyhow::Error> {
        self.templates
            .get(name)
            .cloned()
            .ok_or_else(|| format!("Template '{}' not found", name).into())
    }

    pub fn register_template(&mut self, template: PipelineTemplate) {
        self.templates.insert(template.name.clone(), template);
    }
}

impl Default for TemplateRegistry {
    fn default() -> Self {
        Self::new()
    }
}
