use crate::models::Account;
use crate::modules::{db, device, process, version};
#[cfg(not(windows))]
use std::process::Command;

use super::*;

/// Helper: Write account token to host OS Keychain/Credentials Manager
pub fn write_to_system_keyring(account: &crate::models::Account) -> Result<(), String> {
    // 1. Build token JSON payload and format expiry to RFC3339 with microsecond precision
    let expiry_datetime = chrono::DateTime::from_timestamp(account.token.expiry_timestamp, 0)
        .unwrap_or_else(|| chrono::Utc::now());
    let expiry_str = expiry_datetime.to_rfc3339_opts(chrono::SecondsFormat::Micros, true);

    #[derive(serde::Serialize)]
    struct KeyringTokenDetails {
        access_token: String,
        token_type: String,
        refresh_token: String,
        expiry: String,
    }

    #[derive(serde::Serialize)]
    struct KeyringPayload {
        token: KeyringTokenDetails,
        auth_method: String,
    }

    let payload_json = serde_json::to_string(&KeyringPayload {
        token: KeyringTokenDetails {
            access_token: account.token.access_token.clone(),
            token_type: "Bearer".to_string(),
            refresh_token: account.token.refresh_token.clone(),
            expiry: expiry_str,
        },
        auth_method: "consumer".to_string(),
    })
    .map_err(|e| format!("Failed to serialize keyring JSON: {}", e))?;

    crate::modules::logger::log_info(&format!(
        "[Desktop] Writing token to system credential store for: {}",
        account.email
    ));

    // 2. 跨平台凭据注入
    #[cfg(target_os = "macos")]
    {
        use base64::{engine::general_purpose::STANDARD, Engine as _};
        let encoded_payload = STANDARD.encode(&payload_json);
        let full_keyring_value = format!("go-keyring-base64:{}", encoded_payload);

        // 2.1 macOS Keychain Access
        // 删除旧的
        // Justification: deletes a possibly-nonexistent old keychain entry; failure is expected when no entry exists
        crate::error::record_ignored(
            Command::new("security")
                .args([
                    "delete-generic-password",
                    "-s",
                    "gemini",
                    "-a",
                    "antigravity",
                ])
                .output(),
            "delete old macOS keychain entry",
        );

        // 写入新的 (-A 参数允许所有本地应用免密码、无感直接读取凭据)
        let output = Command::new("security")
            .args([
                "add-generic-password",
                "-s",
                "gemini",
                "-a",
                "antigravity",
                "-w",
                &full_keyring_value,
                "-A",
            ])
            .output()
            .map_err(|e| format!("Failed to execute security command: {}", e))?;

        if !output.status.success() {
            let err_msg = String::from_utf8_lossy(&output.stderr);
            return Err(format!("macOS security command failed: {}", err_msg.trim()));
        }
    }

    #[cfg(target_os = "windows")]
    {
        // 2.2 Windows Credential Manager direct Win32 API calls to write raw UTF-8 bytes
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
            credential_blob: *const u8,
            persist: u32,
            attribute_count: u32,
            attributes: *const std::ffi::c_void,
            target_alias: *const u16,
            user_name: *const u16,
        }

        #[link(name = "advapi32")]
        extern "system" {
            fn CredWriteW(credential: *const CREDENTIALW, flags: u32) -> i32;
            fn CredDeleteW(target_name: *const u16, type_: u32, flags: u32) -> i32;
        }

        let target = "gemini:antigravity";
        let user = "antigravity";
        let secret = payload_json.as_bytes();

        let target_wide: Vec<u16> = std::ffi::OsStr::new(target)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let user_wide: Vec<u16> = std::ffi::OsStr::new(user)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect();

        let cred = CREDENTIALW {
            flags: 0,
            cred_type: 1, // CRED_TYPE_GENERIC
            target_name: target_wide.as_ptr(),
            comment: ptr::null(),
            last_written: FILETIME {
                dw_low_date_time: 0,
                dw_high_date_time: 0,
            },
            credential_blob_size: secret.len() as u32,
            credential_blob: secret.as_ptr(),
            persist: 2, // CRED_PERSIST_LOCAL_MACHINE
            attribute_count: 0,
            attributes: ptr::null(),
            target_alias: ptr::null(),
            user_name: user_wide.as_ptr(),
        };

        unsafe {
            // Delete first to ensure we write clean
            // Justification: CredDeleteW returns BOOL, not a Result; a missing entry fails harmlessly and CredWriteW below overwrites anyway (its result is checked)
            let _ = CredDeleteW(target_wide.as_ptr(), 1, 0);

            let res = CredWriteW(&cred, 0);
            if res == 0 {
                let err = std::io::Error::last_os_error();
                return Err(format!("Windows CredWriteW failed: {}", err));
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        // 2.3 Linux Secret Service API
        // [FIX #3418] 在 Linux GNOME 环境下，Secret Service 往往同时存在 'login' 集合与 'default' 集合。
        // agy CLI 读取凭据时严格从 'login' 集合检索。若未指定 --collection，secret-tool 会写入 default 集合，
        // 导致两个集合内容分叉，agy 持续读取到 login 集合中的旧账号。
        // 此处封装辅助函数：优先写入 login 集合，同时确保与 default 集合同步。
        use std::io::Write;
        use std::sync::mpsc;

        let store_to_collection = |collection_opt: Option<&str>,
                                   payload: &[u8]|
         -> Result<(), String> {
            let mut cmd = Command::new("secret-tool");
            cmd.arg("store");
            if let Some(col) = collection_opt {
                cmd.arg(format!("--collection={}", col));
                cmd.arg("--label=Password for 'antigravity' on 'gemini'");
            } else {
                cmd.arg("--label=gemini");
            }
            cmd.args(["service", "gemini", "username", "antigravity"]);
            cmd.stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped());

            let mut child = match cmd.spawn() {
                Ok(child) => child,
                Err(e) => {
                    if e.kind() == std::io::ErrorKind::NotFound {
                        return Err(
                            "Linux Secret Service utility 'secret-tool' not found (未检测到 secret-tool 工具)。\n\
                             Please install libsecret-tools to enable Keyring credential storage:\n\
                             • Ubuntu / Debian: sudo apt install -y libsecret-tools\n\
                             • Fedora / RHEL: sudo dnf install -y libsecret\n\
                             • Arch Linux: sudo pacman -S libsecret"
                                .to_string(),
                        );
                    }
                    return Err(format!("Failed to spawn secret-tool: {}", e));
                }
            };

            if let Some(mut stdin) = child.stdin.take() {
                stdin
                    .write_all(payload)
                    .map_err(|e| format!("Failed to write to secret-tool stdin: {}", e))?;
            }

            let child_pid = child.id();
            let (tx, rx) = mpsc::channel::<Result<std::process::Output, std::io::Error>>();
            std::thread::spawn(move || {
                // Justification: channel send fails only if the receiver already timed out and killed the child; the timeout path is handled
                crate::error::record_ignored(
                    tx.send(child.wait_with_output()),
                    "forward secret-tool output to channel",
                );
            });

            let output = match rx.recv_timeout(std::time::Duration::from_secs(10)) {
                Ok(result) => {
                    result.map_err(|e| format!("Failed to wait for secret-tool: {}", e))?
                }
                Err(_) => {
                    // Justification: killing the hung secret-tool child is best-effort cleanup; the timeout error is returned regardless
                    crate::error::record_ignored(
                        Command::new("kill")
                            .args(["-9", &child_pid.to_string()])
                            .output(),
                        "kill hung secret-tool child",
                    );
                    crate::modules::logger::log_error(
                        "[Desktop] secret-tool store blocked for >10s — D-Bus session bus unreachable.",
                    );
                    return Err(
                        "Keyring write timed out (10s). The D-Bus session bus is not reachable from this process."
                            .to_string(),
                    );
                }
            };

            if !output.status.success() {
                let err_msg = String::from_utf8_lossy(&output.stderr);
                return Err(format!("Linux secret-tool failed: {}", err_msg.trim()));
            }

            Ok(())
        };

        // 1. 优先尝试写入 'login' 集合（agy CLI 所需）
        let login_res = store_to_collection(Some("login"), payload_json.as_bytes());

        // 2. 同时写入默认集合（保证其他依赖 default collection 的系统工具也能读取）
        let default_res = store_to_collection(None, payload_json.as_bytes());

        // 尝试优先同步写入本地文件凭据 (~/.gemini/oauth_creds.json)
        // Justification: file-credential mirror is opportunistic; the keyring writes are the authoritative path
        crate::error::record_ignored(
            write_to_file_credentials(account),
            "mirror credentials to file",
        );

        // 若两者均失败，则返回错误；若至少一个成功，则记录并继续
        if login_res.is_err() && default_res.is_err() {
            return Err(login_res.unwrap_err());
        } else if let Err(e) = login_res {
            crate::modules::logger::log_warn(&format!(
                "[Desktop] Failed to write token to 'login' collection, falling back to default collection: {}",
                e
            ));
        } else {
            crate::modules::logger::log_info(
                "[Desktop] Successfully synced credential to Secret Service 'login' collection.",
            );
        }
    }

    crate::modules::logger::log_info(
        "[Desktop] Successfully wrote token to system credential store.",
    );

    // 同步写入 ~/.gemini/ 目录下的文件凭据，兼容 SSH 会话、容器环境和无 Keyring/D-Bus 场景
    if let Err(e) = write_to_file_credentials(account) {
        crate::modules::logger::log_warn(&format!("[Desktop] File credential sync warning: {}", e));
    }

    Ok(())
}
