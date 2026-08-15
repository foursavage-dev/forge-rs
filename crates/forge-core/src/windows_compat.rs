#![cfg(windows)]
#![forbid(unsafe_code)]

//! Windows-specific compatibility fixes for Forge
//! 
//! This module handles Windows-specific issues:
//! - Symlink creation without Admin rights
//! - File locking and access violations
//! - Path handling differences

use std::fs;
use std::io;
use std::path::Path;

/// Try to create a symlink, falling back to hard copy if it fails
/// This handles Windows symlink restrictions without requiring Admin
pub fn try_symlink_or_copy(src: &Path, dst: &Path) -> io::Result<()> {
    // Try symlink first
    #[cfg(windows)]
    {
        if std::os::windows::fs::symlink_file(src, dst).is_ok() {
            return Ok(());
        }
    }
    
    #[cfg(unix)]
    {
        if let Ok(_) = std::os::unix::fs::symlink(src, dst) {
            return Ok(());
        }
    }
    
    // Fallback to hard copy
    fs::copy(src, dst)?;
    Ok(())
}

/// Check if a file is locked (Windows-specific)
pub fn is_file_locked(path: &Path) -> bool {
    #[cfg(windows)]
    {
        use std::fs::OpenOptions;
        
        if let Ok(file) = OpenOptions::new()
            .read(true)
            .write(false)
            .open(path)
        {
            // If we can open it for reading, it's not locked
            drop(file);
            false
        } else {
            true
        }
    }
    
    #[cfg(not(windows))]
    {
        // On Unix, file locking is advisory, so we assume not locked
        false
    }
}

/// Windows-safe file replacement
/// Handles locked files by writing to temp file then replacing
pub fn safe_replace_file(src: &Path, dst: &Path) -> io::Result<()> {
    let temp_path = dst.with_extension(".tmp");
    
    // Copy to temp file first
    fs::copy(src, &temp_path)?;
    
    // Try to replace the target
    #[cfg(windows)]
    {
        // On Windows, we may need to retry if the file is locked
        let mut retries = 5;
        loop {
            match fs::rename(&temp_path, dst) {
                Ok(_) => return Ok(()),
                Err(_e) if retries > 0 => {
                    retries -= 1;
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
                Err(e) => {
                    // Clean up temp file
                    let _ = fs::remove_file(&temp_path);
                    return Err(e);
                }
            }
        }
    }
    
    #[cfg(not(windows))]
    {
        fs::rename(&temp_path, dst)?;
        Ok(())
    }
}

/// Get Windows OS version for environment fingerprinting
#[cfg(windows)]
pub fn get_windows_version() -> String {
    use std::process::Command;
    
    if let Ok(output) = Command::new("cmd")
        .args(["/c", "ver"])
        .output()
    {
        let version = String::from_utf8_lossy(&output.stdout);
        version.trim().to_string()
    } else {
        "Windows Unknown".to_string()
    }
}

/// Check if Developer Mode is enabled (for symlink creation)
#[cfg(windows)]
pub fn is_developer_mode_enabled() -> bool {
    use std::process::Command;
    
    if let Ok(output) = Command::new("powershell")
        .args(["-Command", "Get-WindowsDeveloperLicense | Select-Object -ExpandProperty IsLicensed"])
        .output()
    {
        let result = String::from_utf8_lossy(&output.stdout);
        result.trim() == "True"
    } else {
        false
    }
}