use std::path::Path;

use crate::ZigBackendError;
use crate::config::ZigTarget;
use forge_core::{DEFAULT_EXCLUDED_DIRS, FingerprintUtils};

pub fn compute_zig_fingerprint(
    project_dir: &Path,
    zig_version: &str,
    target: &ZigTarget,
) -> Result<String, ZigBackendError> {
    let mut hasher = blake3::Hasher::new();

    hasher.update(zig_version.as_bytes());
    hasher.update(target.as_str().as_bytes());

    let extensions = &["zig", "zon", "c", "h", "cpp", "hpp"];
    FingerprintUtils::hash_directory_with_extensions(
        project_dir,
        extensions,
        DEFAULT_EXCLUDED_DIRS,
        &mut hasher,
    )
    .map_err(ZigBackendError::Io)?;

    for name in &["build.zig", "build.zig.zon", "zig.zon"] {
        let path = project_dir.join(name);
        if path.exists() {
            let _ = FingerprintUtils::hash_file_into(&path, &mut hasher);
        }
    }

    Ok(hasher.finalize().to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_zig_fingerprint() {
        let temp = tempdir().unwrap();
        let src_dir = temp.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();

        let zig_file = src_dir.join("main.zig");
        fs::write(
            &zig_file,
            "const std = @import(\"std\"); pub fn main() !void { std.debug.print(\"Hello\\n\"); }",
        )
        .unwrap();

        let build_file = temp.path().join("build.zig");
        fs::write(&build_file, "const std = @import(\"std\");").unwrap();

        let fingerprint =
            compute_zig_fingerprint(temp.path(), "0.11.0", &ZigTarget::Native).unwrap();

        assert!(!fingerprint.is_empty());
        assert_eq!(fingerprint.len(), 64);
    }

    #[test]
    fn test_fingerprint_changes_with_source() {
        let temp = tempdir().unwrap();
        let src_dir = temp.path().join("src");
        fs::create_dir_all(&src_dir).unwrap();

        let zig_file = src_dir.join("main.zig");
        fs::write(
            &zig_file,
            "const std = @import(\"std\"); pub fn main() !void { std.debug.print(\"Hello\\n\"); }",
        )
        .unwrap();

        let fp1 = compute_zig_fingerprint(temp.path(), "0.11.0", &ZigTarget::Native).unwrap();

        fs::write(&zig_file, "const std = @import(\"std\"); pub fn main() !void { std.debug.print(\"Goodbye\\n\"); }").unwrap();

        let fp2 = compute_zig_fingerprint(temp.path(), "0.11.0", &ZigTarget::Native).unwrap();

        assert_ne!(fp1, fp2);
    }
}
