// Template renderer using Handlebars

use crate::template::{PipelineTemplate, TemplateContext};
use handlebars::Handlebars;

pub struct TemplateRenderer {
    handlebars: Handlebars<'static>,
}

impl TemplateRenderer {
    pub fn new() -> Self {
        let mut handlebars = Handlebars::new();
        handlebars.register_escape_fn(handlebars::no_escape);
        Self { handlebars }
    }

    pub async fn render(&self, template: &PipelineTemplate, context: TemplateContext) -> Result<String, Box<dyn std::error::Error>> {
        let result = self.handlebars.render_template(&template.template_content, &context)?;
        Ok(result)
    }
}

impl Default for TemplateRenderer {
    fn default() -> Self {
        Self::new()
    }
}
