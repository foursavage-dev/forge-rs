// Forge Multiplatform - Multi-Platform CI Matrix Generator
// Auto-generates test matrix for Linux, macOS, Windows across architectures

#![forbid(unsafe_code)]
#![allow(missing_docs)]
#![warn(clippy::all)]

pub mod platform;
pub mod matrix;
pub mod generator;

pub use platform::{Platform, Architecture, Target};
pub use matrix::{TestMatrix, MatrixConfig};
pub use generator::MatrixGenerator;

/// Main multiplatform service
#[derive(Clone)]
pub struct MultiplatformService {
    generator: MatrixGenerator,
}

impl MultiplatformService {
    pub fn new() -> Self {
        Self {
            generator: MatrixGenerator::new(),
        }
    }

    pub fn generate_matrix(&self, config: MatrixConfig) -> TestMatrix {
        self.generator.generate(config)
    }
}

impl Default for MultiplatformService {
    fn default() -> Self {
        Self::new()
    }
}
