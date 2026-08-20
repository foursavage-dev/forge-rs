// Fish Docker Builder - Docker Image Building as First-Class Artifacts
// Build Docker images like build artifacts with layer caching

#![forbid(unsafe_code)]
#![allow(missing_docs)]
#![warn(clippy::all)]

pub mod builder;
pub mod image;
pub mod registry;

pub use builder::{BuildOptions, DockerBuilder};
pub use image::{DockerImage, ImageMetadata};
pub use registry::{Registry, RegistryConfig};

/// Main Docker builder service
pub struct DockerBuilderService {
    builder: DockerBuilder,
}

impl DockerBuilderService {
    pub fn new() -> Self {
        Self {
            builder: DockerBuilder::new(),
        }
    }

    pub async fn build(
        &self,
        dockerfile_path: &str,
        options: BuildOptions,
    ) -> Result<DockerImage, anyhow::Error> {
        self.builder.build(dockerfile_path, options).await
    }
}

impl Default for DockerBuilderService {
    fn default() -> Self {
        Self::new()
    }
}
