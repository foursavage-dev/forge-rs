// Forge Templates - Build Pipeline Templates
// Shareable pipeline templates for common workflows

#![forbid(unsafe_code)]
#![allow(missing_docs)]
#![warn(clippy::all)]

pub mod template;
pub mod registry;
pub mod renderer;

pub use template::{PipelineTemplate, TemplateContext};
pub use registry::TemplateRegistry;
pub use renderer::TemplateRenderer;

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

    pub async fn render_template(&self, template_name: &str, context: TemplateContext) -> Result<String, Box<dyn std::error::Error>> {
        let template = self.registry.get_template(template_name)?;
        self.renderer.render(&template, context).await
    }
}

impl Default for TemplatesService {
    fn default() -> Self {
        Self::new()
    }
}
