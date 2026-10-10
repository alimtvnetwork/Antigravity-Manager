#[cfg(not(windows))]
use std::process::Command;

use super::*;

/// 辅助方法：从宿主操作系统的 Keychain/Credentials Manager 读取 Token
pub fn read_from_system_keyring() -> Result<crate::modules::migration::ImportedOAuthState, String> {
    #[cfg(target_os = "macos")]
    {
        use base64::{engine::general_purpose::STANDARD, Engine as _};
        let output = Command::new("security")
            .args([
                "find-generic-password",
                "-s",
                "gemini",
                "-a",
                "antigravity",
                "-w",
            ])
            .output()
            .map_err(|e| format!("Failed to execute security command: {}", e))?;

        if !output.status.success() {
            if let Ok(file_state) = read_from_file_credentials() {
                return Ok(file_state);
            }
            return Err("No credential found in macOS Keychain".to_string());
        }

        let secret_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let payload_str = if secret_str.starts_with("go-keyring-base64:") {
            let b64_part = &secret_str["go-keyring-base64:".len()..];
            let decoded = STANDARD
                .decode(b64_part)
                .map_err(|e| format!("Base64 decode failed: {}", e))?;
            String::from_utf8(decoded).map_err(|e| format!("UTF-8 decode failed: {}", e))?
        } else {
            secret_str
        };

        return parse_keyring_payload(&payload_str);
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::ffi::OsStrExt;
        use std::ptr;

        #[repr(C)]
        struct FILETIME {
            dw_low_date_time: u32,
            dw_high_date_time: u32,
        }

        #[repr(C)]
        struct CREDENTIALW {
            flags: u32,
            cred_type: u32,
            target_name: *const u16,
            comment: *const u16,
            last_written: FILETIME,
            credential_blob_size: u32,
            credential_blob: *mut u8,
            persist: u32,
            attribute_count: u32,
            attributes: *const std::ffi::c_void,
            target_alias: *const u16,
            user_name: *const u16,
        }

        #[link(name = "advapi32")]
        extern "system" {
            fn CredReadW(
                target_name: *const u16,
                type_: u32,
                flags: u32,
                credential: *mut *mut CREDENTIALW,
            ) -> i32;
            fn CredFree(buffer: *mut std::ffi::c_void);
        }

        let target = "gemini:antigravity";
        let target_wide: Vec<u16> = std::ffi::OsStr::new(target)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let mut cred_ptr: *mut CREDENTIALW = ptr::null_mut();
        unsafe {
            let res = CredReadW(target_wide.as_ptr(), 1, 0, &mut cred_ptr);
            if res == 0 || cred_ptr.is_null() {
                if let Ok(file_state) = read_from_file_credentials() {
                    return Ok(file_state);
                }
                return Err("No credential found in Windows Credential Manager".to_string());
            }

            let cred = &*cred_ptr;
            let blob = std::slice::from_raw_parts(
                cred.credential_blob,
                cred.credential_blob_size as usize,
            );
            let payload_str = String::from_utf8_lossy(blob).to_string();
            CredFree(cred_ptr as *mut std::ffi::c_void);

            return parse_keyring_payload(&payload_str);
        }
    }

    #[cfg(target_os = "linux")]
    {
        let output = match Command::new("secret-tool")
            .args(["lookup", "service", "gemini", "username", "antigravity"])
            .output()
        {
            Ok(out) => out,
            Err(e) => {
                if let Ok(file_state) = read_from_file_credentials() {
                    return Ok(file_state);
                }
                if e.kind() == std::io::ErrorKind::NotFound {
                    return Err(
                        "Linux Secret Service utility 'secret-tool' not found (未检测到 secret-tool 工具)。\n\
                         Please install libsecret-tools to enable Keyring storage:\n\
                         • Ubuntu / Debian: sudo apt install -y libsecret-tools\n\
                         • Fedora / RHEL: sudo dnf install -y libsecret\n\
                         • Arch Linux: sudo pacman -S libsecret"
                            .to_string(),
                    );
                }
                return Err(format!("Failed to execute secret-tool: {}", e));
            }
        };

        if !output.status.success() {
            if let Ok(file_state) = read_from_file_credentials() {
                return Ok(file_state);
            }
            return Err("No credential found in Linux secret-tool".to_string());
        }

        let payload_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        return parse_keyring_payload(&payload_str);
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    {
        Err("Keyring not supported on this operating system".to_string())
    }
}

pub(crate) fn parse_keyring_payload(
    payload_str: &str,
) -> Result<crate::modules::migration::ImportedOAuthState, String> {
    let json: serde_json::Value = serde_json::from_str(payload_str)
        .map_err(|e| format!("Failed to parse keyring payload JSON: {}", e))?;

    let refresh_token = json
        .get("token")
        .and_then(|t| t.get("refresh_token"))
        .and_then(|v| v.as_str())
        .or_else(|| json.get("refresh_token").and_then(|v| v.as_str()))
        .ok_or_else(|| "Refresh Token not found in keyring payload".to_string())?
        .to_string();

    Ok(crate::modules::migration::ImportedOAuthState {
        refresh_token,
        is_gcp_tos: true,
        project_id: None,
    })
}
