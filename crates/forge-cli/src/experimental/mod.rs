#![allow(dead_code)]

//! # Experimental Features
//! 
//! ⚠️ **WARNING**: These features are highly experimental and potentially dangerous.
//! They are disabled by default and should only be used in development environments.
//! 
//! ## Known Issues:
//! - **hotpatch**: Can cause SIGSEGV/crash if code changes struct layout or calling conventions
//! - **kernel_bypass**: Requires Linux with raw memory/DMA access, blocked by container security
//! - **turbolink**: mold linker compatibility issues on Windows/macOS
//! - **daemon_pool**: May have race conditions in multi-process scenarios
//! 
//! ## Safety:
//! These features require `#[allow(unsafe_code)]` and bypass normal safety checks.
//! Use at your own risk in isolated development environments only.

pub mod daemon_pool;
pub mod hotpatch;
pub mod kernel_bypass;
pub mod micro_jit;
pub mod speculative;
pub mod super_opt;
pub mod turbolink;
pub mod wasm_sandbox;

use std::sync::atomic::{AtomicBool, Ordering};

/// Global flag to enable experimental features
/// Set via FORGE_EXPERIMENTAL=1 environment variable
static EXPERIMENTAL_ENABLED: AtomicBool = AtomicBool::new(false);

/// Check if experimental features are enabled
pub fn is_enabled() -> bool {
    EXPERIMENTAL_ENABLED.load(Ordering::Relaxed)
}

/// Enable experimental features (called by CLI flag)
pub fn enable() {
    EXPERIMENTAL_ENABLED.store(true, Ordering::Relaxed);
    eprintln!("⚠️  Experimental features enabled. Use at your own risk!");
}

/// Safety check before using experimental features
pub fn require_enabled(feature_name: &str) -> Result<(), String> {
    if !is_enabled() {
        Err(format!(
            "Experimental feature '{}' is disabled. Use --experimental flag to enable.",
            feature_name
        ))
    } else {
        Ok(())
    }
}
