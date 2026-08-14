#![forbid(unsafe_code)]

pub mod backend;
pub mod error;
pub mod project;

pub use backend::BuildBackend;
pub use error::{ForgeError, Result};
