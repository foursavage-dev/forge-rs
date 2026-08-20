use std::io;
use std::path::Path;

use fish_core::{DEFAULT_EXCLUDED_DIRS, FingerprintUtils};

pub fn compute_ts_fingerprint(root: &Path, source_dirs: &[String]) -> Result<String, io::Error> {
    let mut hasher = blake3::Hasher::new();

    for config_name in &[
        "package.json",
        "tsconfig.json",
        "pnpm-lock.yaml",
        "package-lock.json",
        "yarn.lock",
        "bun.lockb",
        "bun.lock",
        "fish.ts.json",
        "vite.config.ts",
        "vite.config.js",
        "next.config.js",
        "next.config.mjs",
        "next.config.ts",
    ] {
        let file_path = root.join(config_name);
        if file_path.is_file() {
            hasher.update(config_name.as_bytes());
            let _ = FingerprintUtils::hash_file_into(&file_path, &mut hasher);
        }
    }

    let mut dirs_to_scan = Vec::new();
    if source_dirs.is_empty() {
        dirs_to_scan.push(root.to_path_buf());
    } else {
        for dir in source_dirs {
            let full_dir = root.join(dir);
            if full_dir.exists() {
                dirs_to_scan.push(full_dir);
            }
        }
        if dirs_to_scan.is_empty() {
            dirs_to_scan.push(root.to_path_buf());
        }
    }

    let extensions = &[
        "ts", "tsx", "js", "jsx", "mjs", "cjs", "json", "css", "scss", "html", "vue", "svelte",
    ];

    for dir in dirs_to_scan {
        FingerprintUtils::hash_directory_with_extensions(
            &dir,
            extensions,
            DEFAULT_EXCLUDED_DIRS,
            &mut hasher,
        )?;
    }

    Ok(hasher.finalize().to_hex().to_string())
}
