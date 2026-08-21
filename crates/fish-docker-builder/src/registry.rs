// Docker registry integration

#[derive(Debug, Clone)]
pub struct RegistryConfig {
    pub url: String,
    pub username: Option<String>,
    pub password: Option<String>,
}

pub struct Registry {
    config: RegistryConfig,
}

impl Registry {
    pub fn new(config: RegistryConfig) -> Self {
        Self { config }
    }

    pub async fn push(&self, image: &crate::image::DockerImage) -> Result<(), anyhow::Error> {
        // The registry push is not implemented yet. Failing loudly prevents a
        // caller from believing an image was published when it was not.
        Err(anyhow::anyhow!(
            "Docker registry push is not implemented yet (image `{}` -> `{}`)",
            image.id,
            self.config.url
        ))
    }
}
