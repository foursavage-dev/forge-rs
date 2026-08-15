#![forbid(unsafe_code)]

pub mod backend;
pub mod environment;
pub mod error;
pub mod project;

#[cfg(windows)]
pub mod windows_compat;

pub use backend::BuildBackend;
pub use environment::EnvironmentFingerprint;
pub use error::{ForgeError, Result};

#[cfg(windows)]
pub use windows_compat::{try_symlink_or_copy, is_file_locked, safe_replace_file, get_windows_version, is_developer_mode_enabled};
