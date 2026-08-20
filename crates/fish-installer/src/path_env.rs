use std::path::Path;

#[cfg(windows)]
use std::ffi::OsStr;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

pub fn add_dir_to_user_path(dir: &Path) -> Result<bool, String> {
    let dir_str = dir.to_str().ok_or("Invalid directory path encoding")?;

    #[cfg(windows)]
    {
        add_to_windows_user_path(dir_str)
    }

    #[cfg(not(windows))]
    {
        add_to_unix_user_path(dir_str)
    }
}

pub fn remove_dir_from_user_path(dir: &Path) -> Result<bool, String> {
    let dir_str = dir.to_str().ok_or("Invalid directory path encoding")?;

    #[cfg(windows)]
    {
        remove_from_windows_user_path(dir_str)
    }

    #[cfg(not(windows))]
    {
        remove_from_unix_user_path(dir_str)
    }
}

#[cfg(windows)]
fn to_wide_chars(s: &str) -> Vec<u16> {
    OsStr::new(s)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect()
}

#[cfg(windows)]
fn add_to_windows_user_path(new_entry: &str) -> Result<bool, String> {
    use windows_sys::Win32::System::Registry::{
        HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_EXPAND_SZ, REG_SZ,
        RegCloseKey, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SMTO_ABORTIFHUNG, SendMessageTimeoutW, WM_SETTINGCHANGE,
    };

    let subkey = to_wide_chars("Environment");
    let val_name = to_wide_chars("Path");
    let mut hkey: HKEY = std::ptr::null_mut();

    let status = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            0,
            KEY_QUERY_VALUE | KEY_SET_VALUE,
            &mut hkey,
        )
    };

    if status != 0 {
        return Err(format!(
            "Failed to open Windows User Environment registry key: code {status}"
        ));
    }

    let mut val_type: u32 = 0;
    let mut data_size: u32 = 0;

    unsafe {
        RegQueryValueExW(
            hkey,
            val_name.as_ptr(),
            std::ptr::null_mut(),
            &mut val_type,
            std::ptr::null_mut(),
            &mut data_size,
        );
    }

    let mut current_path = String::new();
    if data_size > 0 {
        let mut buffer: Vec<u16> = vec![0; (data_size as usize / 2) + 1];
        let query_res = unsafe {
            RegQueryValueExW(
                hkey,
                val_name.as_ptr(),
                std::ptr::null_mut(),
                &mut val_type,
                buffer.as_mut_ptr() as *mut u8,
                &mut data_size,
            )
        };
        if query_res == 0 {
            if let Some(pos) = buffer.iter().position(|&c| c == 0) {
                buffer.truncate(pos);
            }
            current_path = String::from_utf16_lossy(&buffer);
        }
    }

    let entries: Vec<&str> = current_path
        .split(';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    if entries.iter().any(|&e| e.eq_ignore_ascii_case(new_entry)) {
        unsafe { RegCloseKey(hkey) };
        return Ok(false);
    }

    let updated_path = if current_path.is_empty() {
        new_entry.to_string()
    } else {
        format!("{};{}", current_path.trim_end_matches(';'), new_entry)
    };

    let wide_updated = to_wide_chars(&updated_path);
    let bytes_len = (wide_updated.len() * 2) as u32;

    let write_type = if val_type == REG_EXPAND_SZ {
        REG_EXPAND_SZ
    } else {
        REG_SZ
    };
    let set_res = unsafe {
        RegSetValueExW(
            hkey,
            val_name.as_ptr(),
            0,
            write_type,
            wide_updated.as_ptr() as *const u8,
            bytes_len,
        )
    };

    unsafe { RegCloseKey(hkey) };

    if set_res != 0 {
        return Err(format!(
            "Failed to write updated PATH to registry: code {set_res}"
        ));
    }

    let env_wide = to_wide_chars("Environment");
    let mut result: usize = 0;
    let hwnd_broadcast = 0xffff_usize as *mut std::ffi::c_void;
    unsafe {
        SendMessageTimeoutW(
            hwnd_broadcast,
            WM_SETTINGCHANGE,
            0,
            env_wide.as_ptr() as isize,
            SMTO_ABORTIFHUNG,
            5000,
            &mut result as *mut usize,
        );
    }

    Ok(true)
}

#[cfg(windows)]
fn remove_from_windows_user_path(entry_to_remove: &str) -> Result<bool, String> {
    use windows_sys::Win32::System::Registry::{
        HKEY, HKEY_CURRENT_USER, KEY_QUERY_VALUE, KEY_SET_VALUE, REG_EXPAND_SZ, REG_SZ,
        RegCloseKey, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SMTO_ABORTIFHUNG, SendMessageTimeoutW, WM_SETTINGCHANGE,
    };

    let subkey = to_wide_chars("Environment");
    let val_name = to_wide_chars("Path");
    let mut hkey: HKEY = std::ptr::null_mut();

    let status = unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            subkey.as_ptr(),
            0,
            KEY_QUERY_VALUE | KEY_SET_VALUE,
            &mut hkey,
        )
    };

    if status != 0 {
        return Err(format!(
            "Failed to open Windows User Environment registry key: code {status}"
        ));
    }

    let mut val_type: u32 = 0;
    let mut data_size: u32 = 0;

    unsafe {
        RegQueryValueExW(
            hkey,
            val_name.as_ptr(),
            std::ptr::null_mut(),
            &mut val_type,
            std::ptr::null_mut(),
            &mut data_size,
        );
    }

    if data_size == 0 {
        unsafe { RegCloseKey(hkey) };
        return Ok(false);
    }

    let mut buffer: Vec<u16> = vec![0; (data_size as usize / 2) + 1];
    let query_res = unsafe {
        RegQueryValueExW(
            hkey,
            val_name.as_ptr(),
            std::ptr::null_mut(),
            &mut val_type,
            buffer.as_mut_ptr() as *mut u8,
            &mut data_size,
        )
    };

    if query_res != 0 {
        unsafe { RegCloseKey(hkey) };
        return Ok(false);
    }

    if let Some(pos) = buffer.iter().position(|&c| c == 0) {
        buffer.truncate(pos);
    }
    let current_path = String::from_utf16_lossy(&buffer);

    let entries: Vec<&str> = current_path
        .split(';')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    let initial_count = entries.len();
    let filtered_entries: Vec<&str> = entries
        .into_iter()
        .filter(|&e| !e.eq_ignore_ascii_case(entry_to_remove))
        .collect();

    if filtered_entries.len() == initial_count {
        unsafe { RegCloseKey(hkey) };
        return Ok(false);
    }

    let updated_path = filtered_entries.join(";");
    let wide_updated = to_wide_chars(&updated_path);
    let bytes_len = (wide_updated.len() * 2) as u32;

    let write_type = if val_type == REG_EXPAND_SZ {
        REG_EXPAND_SZ
    } else {
        REG_SZ
    };
    let set_res = unsafe {
        RegSetValueExW(
            hkey,
            val_name.as_ptr(),
            0,
            write_type,
            wide_updated.as_ptr() as *const u8,
            bytes_len,
        )
    };

    unsafe { RegCloseKey(hkey) };

    if set_res != 0 {
        return Err(format!("Failed to update PATH in registry: code {set_res}"));
    }

    let env_wide = to_wide_chars("Environment");
    let mut result: usize = 0;
    let hwnd_broadcast = 0xffff_usize as *mut std::ffi::c_void;
    unsafe {
        SendMessageTimeoutW(
            hwnd_broadcast,
            WM_SETTINGCHANGE,
            0,
            env_wide.as_ptr() as isize,
            SMTO_ABORTIFHUNG,
            5000,
            &mut result as *mut usize,
        );
    }

    Ok(true)
}

#[cfg(not(windows))]
fn add_to_unix_user_path(new_entry: &str) -> Result<bool, String> {
    let home = std::env::var("HOME").map_err(|e| e.to_string())?;
    let rc_files = [
        format!("{home}/.bashrc"),
        format!("{home}/.zshrc"),
        format!("{home}/.profile"),
    ];
    let export_line = format!("\nexport PATH=\"{new_entry}:$PATH\"\n");

    let mut modified = false;
    for rc in &rc_files {
        let path = Path::new(rc);
        if path.exists() {
            let content = std::fs::read_to_string(path).unwrap_or_default();
            if !content.contains(new_entry) {
                let mut f = std::fs::OpenOptions::new()
                    .append(true)
                    .open(path)
                    .map_err(|e| e.to_string())?;
                use std::io::Write;
                f.write_all(export_line.as_bytes())
                    .map_err(|e| e.to_string())?;
                modified = true;
            }
        }
    }
    Ok(modified)
}

#[cfg(not(windows))]
fn remove_from_unix_user_path(entry_to_remove: &str) -> Result<bool, String> {
    let home = std::env::var("HOME").map_err(|e| e.to_string())?;
    let rc_files = [
        format!("{home}/.bashrc"),
        format!("{home}/.zshrc"),
        format!("{home}/.profile"),
    ];

    let mut modified = false;
    for rc in &rc_files {
        let path = Path::new(rc);
        if path.exists() {
            let content = std::fs::read_to_string(path).unwrap_or_default();
            if content.contains(entry_to_remove) {
                let lines: Vec<&str> = content
                    .lines()
                    .filter(|line| !line.contains(entry_to_remove))
                    .collect();
                std::fs::write(path, lines.join("\n")).map_err(|e| e.to_string())?;
                modified = true;
            }
        }
    }
    Ok(modified)
}
