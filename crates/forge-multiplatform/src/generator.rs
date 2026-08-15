// Matrix generator

use crate::matrix::{MatrixConfig, TestMatrix};

#[derive(Clone)]
pub struct MatrixGenerator;

impl MatrixGenerator {
    pub fn new() -> Self {
        Self
    }

    pub fn generate(&self, config: MatrixConfig) -> TestMatrix {
        TestMatrix {
            targets: config.platforms,
            rust_versions: config.include_rust_versions,
            node_versions: config.include_node_versions,
        }
    }
}

impl Default for MatrixGenerator {
    fn default() -> Self {
        Self::new()
    }
}
