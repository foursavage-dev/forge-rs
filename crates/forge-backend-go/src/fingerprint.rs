use std::path::Path;

use forge_core::{FingerprintUtils, DEFAULT_EXCLUDED_DIRS};

pub fn compute_go_fingerprint(
    project_dir: &Path,
    toolchain_version: &str,
    tags: &[String],
) -> Result<String, std::io::Error> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(toolchain_version.as_bytes());
    hasher.update(b"\ntags:\n");
    for tag in tags {
        hasher.update(tag.as_bytes());
        hasher.update(b"\n");
    }

    let go_mod = project_dir.join("go.mod");
    if go_mod.exists() {
        let _ = FingerprintUtils::hash_file_into(&go_mod, &mut hasher);
    }

    let go_sum = project_dir.join("go.sum");
    if go_sum.exists() {
        let _ = FingerprintUtils::hash_file_into(&go_sum, &mut hasher);
    }

    let go_work = project_dir.join("go.work");
    if go_work.exists() {
        let _ = FingerprintUtils::hash_file_into(&go_work, &mut hasher);
    }

    FingerprintUtils::hash_directory_with_extensions(
        project_dir,
        &["go"],
        DEFAULT_EXCLUDED_DIRS,
        &mut hasher,
    )?;

    Ok(hasher.finalize().to_hex().to_string())
}
