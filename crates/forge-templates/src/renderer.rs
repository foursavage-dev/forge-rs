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

    pub async fn render(
        &self,
        template: &PipelineTemplate,
        context: TemplateContext,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut map = serde_json::Map::new();
        map.insert(
            "environment".to_string(),
            serde_json::Value::String(context.environment.clone()),
        );
        let mut vars_map = serde_json::Map::new();
        for (k, v) in &context.variables {
            map.insert(k.clone(), serde_json::Value::String(v.clone()));
            vars_map.insert(k.clone(), serde_json::Value::String(v.clone()));
        }
        map.insert("variables".to_string(), serde_json::Value::Object(vars_map));
        let root = serde_json::Value::Object(map);
        let result = self
            .handlebars
            .render_template(&template.template_content, &root)?;
        Ok(result)
    }
}

impl Default for TemplateRenderer {
    fn default() -> Self {
        Self::new()
    }
}
