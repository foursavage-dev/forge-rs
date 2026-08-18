// Forge Templates - Build Pipeline Templates
// Shareable pipeline templates for common workflows

#![forbid(unsafe_code)]
#![allow(missing_docs)]
#![warn(clippy::all)]

pub mod registry;
pub mod renderer;
pub mod template;

pub use registry::TemplateRegistry;
pub use renderer::TemplateRenderer;
pub use template::{PipelineTemplate, TemplateContext};

/// Main templates service
pub struct TemplatesService {
    registry: TemplateRegistry,
    renderer: TemplateRenderer,
}

impl TemplatesService {
    pub fn new() -> Self {
        Self {
            registry: TemplateRegistry::new(),
            renderer: TemplateRenderer::new(),
        }
    }

    pub async fn render_template(
        &self,
        template_name: &str,
        context: TemplateContext,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let template = self.registry.get_template(template_name)?;
        self.renderer.render(&template, context).await
    }
}

impl Default for TemplatesService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_render_builtin_template() {
        let service = TemplatesService::new();
        let mut vars = HashMap::new();
        vars.insert("project_name".to_string(), "my-forge-monorepo".to_string());
        let ctx = TemplateContext {
            variables: vars,
            environment: "production".to_string(),
        };
        let output = service.render_template("monorepo", ctx).await.unwrap();
        assert!(output.contains("my-forge-monorepo"));
    }

    #[tokio::test]
    async fn test_custom_template_registration_and_render() {
        let mut registry = TemplateRegistry::new();
        let custom = PipelineTemplate {
            name: "custom_ci".to_string(),
            version: "0.1.0".to_string(),
            description: "Custom CI Pipeline".to_string(),
            template_content: "pipeline: {{variables.job_name}} in {{environment}}".to_string(),
            required_variables: vec!["job_name".to_string()],
        };
        registry.register_template(custom);

        let retrieved = registry.get_template("custom_ci").unwrap();
        assert_eq!(retrieved.version, "0.1.0");

        let renderer = TemplateRenderer::new();
        let mut vars = HashMap::new();
        vars.insert("job_name".to_string(), "build_and_deploy".to_string());
        let ctx = TemplateContext {
            variables: vars,
            environment: "staging".to_string(),
        };
        let rendered = renderer.render(&retrieved, ctx).await.unwrap();
        assert_eq!(rendered, "pipeline: build_and_deploy in staging");
    }

    #[test]
    fn test_missing_template_error() {
        let registry = TemplateRegistry::new();
        let result = registry.get_template("non_existent");
        assert!(result.is_err());
    }
}
