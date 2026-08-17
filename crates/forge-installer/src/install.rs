use std::fs;
use std::path::{Path, PathBuf};

use crate::path_env;

pub fn get_default_install_dir() -> PathBuf {
    #[cfg(windows)]
    {
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            PathBuf::from(local_app_data)
                .join("Programs")
                .join("Forge")
                .join("bin")
        } else if let Ok(user_profile) = std::env::var("USERPROFILE") {
            PathBuf::from(user_profile).join(".forge").join("bin")
        } else {
            PathBuf::from("C:\\Program Files\\Forge\\bin")
        }
    }

    #[cfg(not(windows))]
    {
        if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".local").join("bin")
        } else {
            PathBuf::from("/usr/local/bin")
        }
    }
}

pub fn find_source_forge_binary() -> Option<PathBuf> {
    let binary_name = if cfg!(windows) { "forge.exe" } else { "forge" };

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            let candidate = parent.join(binary_name);
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }

    if let Ok(cwd) = std::env::current_dir() {
        let candidate = cwd.join(binary_name);
        if candidate.is_file() {
            return Some(candidate);
        }

        let debug_target = cwd.join("target").join("debug").join(binary_name);
        if debug_target.is_file() {
            return Some(debug_target);
        }

        let release_target = cwd.join("target").join("release").join(binary_name);
        if release_target.is_file() {
            return Some(release_target);
        }
    }

    None
}

pub fn perform_installation(target_dir: &Path, source_binary: Option<&Path>) -> Result<(), String> {
    fs::create_dir_all(target_dir)
        .map_err(|e| format!("Failed to create installation directory: {e}"))?;

    let target_binary_name = if cfg!(windows) { "forge.exe" } else { "forge" };
    let destination_file = target_dir.join(target_binary_name);

    if let Some(src) = source_binary {
        if src != destination_file {
            fs::copy(src, &destination_file).map_err(|e| {
                format!(
                    "Failed to copy binary from {} to {}: {e}",
                    src.display(),
                    destination_file.display()
                )
            })?;
        }
    } else if let Ok(current_exe) = std::env::current_exe() {
        if let Some(name) = current_exe.file_name() {
            if (name.to_string_lossy().eq_ignore_ascii_case("forge.exe")
                || name.to_string_lossy() == "forge")
                && current_exe != destination_file
            {
                fs::copy(&current_exe, &destination_file).map_err(|e| {
                    format!(
                        "Failed to copy current executable to {}: {e}",
                        destination_file.display()
                    )
                })?;
            }
        }
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if destination_file.exists() {
            let _ = fs::set_permissions(&destination_file, fs::Permissions::from_mode(0o755));
        }
    }

    let uninstaller_name = if cfg!(windows) {
        "forge-uninstall.exe"
    } else {
        "forge-uninstall"
    };
    if let Ok(current_exe) = std::env::current_exe() {
        let uninstaller_dest = target_dir.join(uninstaller_name);
        if current_exe != uninstaller_dest && current_exe.exists() {
            let _ = fs::copy(&current_exe, &uninstaller_dest);
        }
    }

    let metadata = serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "installed_at": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        "install_dir": target_dir.to_string_lossy(),
    });

    let meta_path = target_dir.join("forge-install.json");
    let _ = fs::write(
        meta_path,
        serde_json::to_string_pretty(&metadata).unwrap_or_default(),
    );

    path_env::add_dir_to_user_path(target_dir)?;

    Ok(())
}

pub fn perform_uninstallation(target_dir: &Path) -> Result<(), String> {
    let _ = path_env::remove_dir_from_user_path(target_dir);

    let target_binary_name = if cfg!(windows) { "forge.exe" } else { "forge" };
    let binary_path = target_dir.join(target_binary_name);
    if binary_path.exists() {
        let _ = fs::remove_file(binary_path);
    }

    let meta_path = target_dir.join("forge-install.json");
    if meta_path.exists() {
        let _ = fs::remove_file(meta_path);
    }

    if let Ok(read_dir) = fs::read_dir(target_dir) {
        if read_dir.count() == 0 {
            let _ = fs::remove_dir(target_dir);
        }
    }

    Ok(())
}
