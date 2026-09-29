use crate::models::Account;
use crate::modules::{db, device, process, version};
use std::fs;
// Command 仅用于 macos/Linux 分支（security / secret-tool / kill），Windows 裁剪不导入以免 unused
#[cfg(not(windows))]
use std::process::Command;

#[allow(async_fn_in_trait)]
pub trait SystemIntegration: Send + Sync {
    /// 当切换账号时执行的系统层操作（如杀进程、写入文件、注入数据库）
    async fn on_account_switch(
        &self,
        account: &crate::models::Account,
        target_ide: Option<&str>,
    ) -> Result<(), String>;

    /// 更新系统托盘（如果适用）
    fn update_tray(&self);

    /// 发送系统通知
    fn show_notification(&self, title: &str, body: &str);
}

/// 根据目标参数、进程运行态及可执行文件存在性决策最终切换环境
pub fn resolve_effective_target(
    target_ide: Option<&str>,
    classic_running: bool,
    ide_running: bool,
    has_classic_exe: bool,
    ide_exe_path: Option<&str>,
) -> (bool, Option<&'static str>) {
    let is_explicit_ide = target_ide == Some("ide");
    let is_explicit_classic = target_ide == Some("classic");

    if is_explicit_ide {
        return (true, Some("ide"));
    }
    if is_explicit_classic {
        return (false, Some("classic"));
    }

    // target_ide 为 None 或未指定时进行智能环境探查（经典版桌面端优先，严禁仅凭静态 IDE 数据库文件劫持经典版目标）
    let mut is_ide = false;
    if classic_running {
        // 原生经典版正在运行，确定目标为经典版
        is_ide = false;
    } else if ide_running {
        // 经典版未运行，但 IDE 正在运行，推导为 IDE
        is_ide = true;
    } else if has_classic_exe {
        // 原生经典版可执行文件存在，优先保持经典版
        is_ide = false;
    } else if let Some(exe_str) = ide_exe_path {
        // 原生经典版不存在，检查是否存在 IDE 可执行文件
        let path_lower = exe_str.to_lowercase();
        if path_lower.contains("antigravity ide") || path_lower.contains("antigravity-ide") {
            is_ide = true;
        }
    }

    let effective = if is_ide {
        Some("ide")
    } else if is_explicit_classic {
        Some("classic")
    } else {
        None
    };

    (is_ide, effective)
}

/// 桌面版实现：包含完整的进程控制 and UI 同步
pub struct DesktopIntegration {
    pub app_handle: Option<tauri::AppHandle>,
}

/// 写入账号凭据：>= 2.0.0 的原生应用走系统 Keyring，旧架构与定制 IDE 走 SQLite 注入。
///
/// 抽成独立函数是为了让「热切号」能在终止 language_server 子进程**之前**完成凭据写入
/// （见 `on_account_switch` 的顺序说明），同时让完整重启路径保持原有的「先杀后写」顺序。
fn apply_account_credentials(
    account: &Account,
    effective_target: Option<&str>,
    is_ide: bool,
    active_exe_path: Option<&std::path::Path>,
) -> Result<(), String> {
    let mut use_keyring = false;

    if !is_ide {
        // 经典原生版：自动探测版本号（优先使用预快照路径）
        match version::get_antigravity_version_with_path(
            effective_target,
            active_exe_path.as_deref(),
        ) {
            Ok(ver) => {
                // 如果版本号 >= 2.0.0
                if version::compare_version(&ver.short_version, "2.0.0") != std::cmp::Ordering::Less
                {
                    use_keyring = true;
                    crate::modules::logger::log_info(&format!(
                        "[Desktop] Detected Antigravity version {} >= 2.0.0, using system Keyring.",
                        ver.short_version
                    ));
                } else {
                    crate::modules::logger::log_info(&format!(
                        "[Desktop] Detected Antigravity version {} < 2.0.0, falling back to legacy SQLite injection.",
                        ver.short_version
                    ));
                }
            }
            Err(e) => {
                // 如果探测失败，优先检查本地是否存在可用的 SQLite 数据库 (state.vscdb)
                // 若存在数据库，说明是经典的 VS Code/IDE 架构，优先使用 SQLite 注入，防止无 secret-tool 时报错
                let has_sqlite_db = db::get_db_path(effective_target)
                    .map(|p| p.exists())
                    .unwrap_or(false);

                if has_sqlite_db {
                    use_keyring = false;
                    crate::modules::logger::log_info(&format!(
                        "[Desktop] Failed to detect Antigravity version ({}), but detected existing SQLite database. Falling back to SQLite injection.",
                        e
                    ));
                } else {
                    use_keyring = true;
                    crate::modules::logger::log_warn(&format!(
                        "[Desktop] Failed to detect Antigravity version ({}) and no SQLite database found, defaulting to system Keyring.",
                        e
                    ));
                }
            }
        }
    }

    if use_keyring {
        // ================== 最新版 Antigravity 原生应用逻辑 (>= 2.0.0) ==================
        // 2.1 写入系统 Keychain/Keyring
        if let Err(keyring_err) = write_to_system_keyring(account) {
            let db_fallback = if let Ok(db_path) = db::get_db_path(effective_target) {
                if db_path.exists() {
                    crate::modules::logger::log_warn(&format!(
                        "[Desktop] Keyring write failed ({}), but found SQLite DB at {:?}. Falling back to SQLite token injection.",
                        keyring_err, db_path
                    ));
                    let backup_path = db_path.with_extension("vscdb.backup");
                    let _ = fs::copy(&db_path, &backup_path);
                    let _ = db::inject_token(
                        &db_path,
                        &account.token.access_token,
                        &account.token.refresh_token,
                        account.token.expiry_timestamp,
                        &account.email,
                        account.token.is_gcp_tos,
                        account.token.project_id.as_deref(),
                        account.token.id_token.as_deref(),
                        account.token.oauth_client_key.as_deref(),
                        effective_target,
                    );
                    if let Some(ref profile) = account.device_profile {
                        let _ = db::write_service_machine_id(&db_path, &profile.mac_machine_id);
                    }
                    true
                } else {
                    false
                }
            } else {
                false
            };

            if !db_fallback {
                return Err(keyring_err);
            }
        }

        // 2.2 同步写入 storage.json 与 state.vscdb（若存在），确保新版客户端与本地扩展状态完全一致
        if let Ok(storage_path) = device::get_storage_path(effective_target) {
            if let Some(ref profile) = account.device_profile {
                let _ = device::write_profile(&storage_path, profile);
            }
        }
        if let Ok(db_path) = db::get_db_path(effective_target) {
            if db_path.exists() {
                let backup_path = db_path.with_extension("vscdb.backup");
                let _ = fs::copy(&db_path, &backup_path);
                let _ = db::inject_token(
                    &db_path,
                    &account.token.access_token,
                    &account.token.refresh_token,
                    account.token.expiry_timestamp,
                    &account.email,
                    account.token.is_gcp_tos,
                    account.token.project_id.as_deref(),
                    account.token.id_token.as_deref(),
                    account.token.oauth_client_key.as_deref(),
                    effective_target,
                );
                if let Some(ref profile) = account.device_profile {
                    let _ = db::write_service_machine_id(&db_path, &profile.mac_machine_id);
                }
            }
        }
    } else {
        // ================== 原有 Antigravity 旧版或定制 IDE 逻辑 (< 2.0.0) ==================
        let _ = write_to_system_keyring(account);

        // 2.1 获取存储路径
        let storage_path = device::get_storage_path(effective_target)?;

        // 2.2 写入设备 Profile
        if let Some(ref profile) = account.device_profile {
            device::write_profile(&storage_path, profile)?;
        }

        // 2.3 数据库处理与 Token 注入
        let db_path = db::get_db_path(effective_target)?;
        if db_path.exists() {
            let backup_path = db_path.with_extension("vscdb.backup");
            let _ = fs::copy(&db_path, &backup_path);
        }

        db::inject_token(
            &db_path,
            &account.token.access_token,
            &account.token.refresh_token,
            account.token.expiry_timestamp,
            &account.email,
            account.token.is_gcp_tos,
            account.token.project_id.as_deref(),
            account.token.id_token.as_deref(),
            account.token.oauth_client_key.as_deref(),
            effective_target,
        )?;

        // 2.4 同步 Service Machine ID 到数据库
        if let Some(ref profile) = account.device_profile {
            let _ = db::write_service_machine_id(&db_path, &profile.mac_machine_id);
        }
    }

    Ok(())
}

impl SystemIntegration for DesktopIntegration {
    async fn on_account_switch(
        &self,
        account: &crate::models::Account,
        target_ide: Option<&str>,
    ) -> Result<(), String> {
        crate::modules::logger::log_info(&format!(
            "[Desktop] Executing unified 5-step account switch for: {} (target_ide: {:?})",
            account.email, target_ide
        ));

        if let Some(target) = target_ide {
            if let Some(inst_id) = target.strip_prefix("instance:") {
                let res = Box::pin(crate::modules::instance::switch_account_to_instance(
                    &account.id,
                    Some(inst_id),
                ))
                .await;
                if let Some(ref h) = self.app_handle {
                    let _ = crate::modules::tray::update_tray_menus(h);
                }
                return res;
            }
        }

        if target_ide == Some("agy") {
            write_to_system_keyring(account)?;

            if let Ok(storage_path) = device::get_storage_path(target_ide) {
                if let Some(ref profile) = account.device_profile {
                    let _ = device::write_profile(&storage_path, profile);
                }
            }

            let is_running = process::is_process_running_by_name("agy");
            let msg = if is_running {
                format!(
                    "Account {} activated. Agy is running, token will be picked up automatically.",
                    account.email
                )
            } else {
                format!(
                    "Account {} activated. Token is ready for your next CLI command.",
                    account.email
                )
            };
            self.show_notification("Antigravity CLI", &msg);
            self.update_tray();

            return Ok(());
        }

        // Resolve target environment without misclassifying classic Antigravity.exe due to child language_server
        let ide_running = process::is_antigravity_running(Some("ide"));
        let classic_running = process::is_antigravity_running(None);
        let classic_exe = process::get_antigravity_executable_path(None);
        let ide_exe = process::get_antigravity_executable_path(Some("ide"));
        let ide_exe_str = ide_exe.as_ref().map(|p| p.to_string_lossy().to_string());

        let (is_ide, effective_target) = resolve_effective_target(
            target_ide,
            classic_running,
            ide_running,
            classic_exe.is_some(),
            ide_exe_str.as_deref(),
        );

        let active_exe_path = process::get_antigravity_executable_path(effective_target)
            .or_else(|| process::get_antigravity_executable_path(None));
        let active_args = process::get_args_from_running_process(effective_target)
            .or_else(|| process::get_args_from_running_process(None));

        // =========================================================================
        // STEP 1: Backup Running Prompts via AGM (before closing Antigravity IDE)
        // =========================================================================
        crate::modules::logger::log_info(
            "[Desktop] [Step 1/5] Backing up running prompts via AGM before closing Antigravity IDE...",
        );
        let _ = crate::modules::repo_db::backup_running_prompts("default");
        let _ = crate::modules::backup_prompts_db::backup_active_running_prompts_for_instance(
            Some("default"),
            None,
        );

        // =========================================================================
        // STEP 2: Close the Antigravity IDE
        // =========================================================================
        crate::modules::logger::log_info(
            "[Desktop] [Step 2/5] Closing running Antigravity IDE processes...",
        );
        if process::is_antigravity_running(effective_target) {
            process::close_antigravity(20, effective_target)?;
        }
        if effective_target != target_ide
            && target_ide.is_some()
            && process::is_antigravity_running(target_ide)
        {
            process::close_antigravity(20, target_ide)?;
        }
        if process::is_antigravity_running(None) {
            let _ = process::close_antigravity(20, None);
        }
        let _ = crate::modules::instance::close_instance("default");

        // =========================================================================
        // STEP 3: Switch the Account Credentials (OS Keyring + state.vscdb + storage.json)
        // =========================================================================
        crate::modules::logger::log_info(&format!(
            "[Desktop] [Step 3/5] Injecting switched account credentials for '{}'...",
            account.email
        ));
        apply_account_credentials(
            account,
            effective_target,
            is_ide,
            active_exe_path.as_deref(),
        )?;
        let _ = crate::modules::instance::bind_account_to_instance(
            "default",
            &account.id,
            &account.email,
        );

        // =========================================================================
        // STEP 4: Re-Open / Re-Run the Antigravity IDE
        // =========================================================================
        crate::modules::logger::log_info(
            "[Desktop] [Step 4/5] Re-launching Antigravity IDE with workspace arguments...",
        );
        process::start_antigravity_with_fallback_path(
            effective_target,
            active_exe_path.as_deref(),
            active_args.as_deref(),
        )?;

        // =========================================================================
        // STEP 5: Re-Inject the Backed-Up Running Prompts
        // =========================================================================
        crate::modules::logger::log_info(
            "[Desktop] [Step 5/5] Re-injecting backed-up running prompts across workspaces...",
        );
        let _ = crate::modules::repo_db::resend_all_running_commands(20);
        let _ = crate::modules::backup_prompts_db::restore_running_prompts_for_instance(
            Some("default"),
            false,
            None,
        );
        let _ = crate::modules::repo_db::dispatch_running_prompts("default");
        let _ = crate::modules::process::focus_antigravity_window(effective_target);

        if let Some(ref h) = self.app_handle {
            let _ = crate::modules::tray::update_tray_menus(h);
        }

        Ok(())
    }

    fn update_tray(&self) {
        if let Some(ref h) = self.app_handle {
            let _ = crate::modules::tray::update_tray_menus(h);
        }
    }

    fn show_notification(&self, title: &str, body: &str) {
        // 使用 tauri-plugin-dialog 或原生通知（此处简化）
        crate::modules::logger::log_info(&format!("[Notification] {}: {}", title, body));
    }
}

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
        let _ = Command::new("security")
            .args([
                "delete-generic-password",
                "-s",
                "gemini",
                "-a",
                "antigravity",
            ])
            .output();

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
                let _ = tx.send(child.wait_with_output());
            });

            let output = match rx.recv_timeout(std::time::Duration::from_secs(10)) {
                Ok(result) => {
                    result.map_err(|e| format!("Failed to wait for secret-tool: {}", e))?
                }
                Err(_) => {
                    let _ = Command::new("kill")
                        .args(["-9", &child_pid.to_string()])
                        .output();
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
        let _ = write_to_file_credentials(account);

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

/// 辅助方法：同步写入指定目录下的本地文件凭据 (<base_home>/.gemini/oauth_creds.json, google_accounts.json, jetski-standalone-oauth-token)
pub fn write_to_file_credentials_at(
    base_home: &std::path::Path,
    account: &crate::models::Account,
) -> Result<(), String> {
    let gemini_dir = if base_home.ends_with(".gemini") {
        base_home.to_path_buf()
    } else {
        base_home.join(".gemini")
    };

    if !gemini_dir.exists() {
        if let Err(e) = std::fs::create_dir_all(&gemini_dir) {
            crate::modules::logger::log_warn(&format!(
                "[Desktop] Failed to create .gemini directory at {:?}: {}",
                gemini_dir, e
            ));
            return Err(format!("Failed to create .gemini directory: {}", e));
        }
    }

    // Also copy installation_id if it exists from global .gemini/antigravity/installation_id
    if let Some(global_home) = dirs::home_dir() {
        let global_inst_id = global_home
            .join(".gemini")
            .join("antigravity")
            .join("installation_id");
        if global_inst_id.exists() {
            let target_antigravity = gemini_dir.join("antigravity");
            let _ = std::fs::create_dir_all(&target_antigravity);
            let target_inst_id = target_antigravity.join("installation_id");
            if !target_inst_id.exists() {
                let _ = std::fs::copy(&global_inst_id, &target_inst_id);
            }
        }
    }

    let expiry_ms = if account.token.expiry_timestamp > 10_000_000_000 {
        account.token.expiry_timestamp
    } else {
        account.token.expiry_timestamp * 1000
    };

    let expiry_datetime = chrono::DateTime::from_timestamp(account.token.expiry_timestamp, 0)
        .unwrap_or_else(chrono::Utc::now);
    let expiry_rfc3339 = expiry_datetime.to_rfc3339_opts(chrono::SecondsFormat::Micros, true);

    #[derive(serde::Serialize)]
    struct OAuthCredsFile {
        access_token: String,
        refresh_token: String,
        token_type: String,
        expiry_date: i64,
        #[serde(skip_serializing_if = "Option::is_none")]
        id_token: Option<String>,
        scope: String,
    }

    let creds = OAuthCredsFile {
        access_token: account.token.access_token.clone(),
        refresh_token: account.token.refresh_token.clone(),
        token_type: "Bearer".to_string(),
        expiry_date: expiry_ms,
        id_token: account.token.id_token.clone(),
        scope: "https://www.googleapis.com/auth/userinfo.email openid https://www.googleapis.com/auth/cloud-platform https://www.googleapis.com/auth/userinfo.profile".to_string(),
    };

    let creds_path = gemini_dir.join("oauth_creds.json");
    let json_str = serde_json::to_string_pretty(&creds)
        .map_err(|e| format!("Failed to serialize oauth_creds JSON: {}", e))?;

    if let Err(e) = std::fs::write(&creds_path, json_str) {
        crate::modules::logger::log_warn(&format!(
            "[Desktop] Failed to write oauth_creds.json at {:?}: {}",
            creds_path, e
        ));
        return Err(format!("Failed to write oauth_creds.json: {}", e));
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&creds_path, std::fs::Permissions::from_mode(0o600));
    }

    #[derive(serde::Serialize)]
    struct GoogleAccountsFile {
        active: String,
        old: Vec<String>,
    }

    let accounts_info = GoogleAccountsFile {
        active: account.email.clone(),
        old: vec![],
    };

    let accounts_path = gemini_dir.join("google_accounts.json");
    if let Ok(accounts_json_str) = serde_json::to_string_pretty(&accounts_info) {
        let _ = std::fs::write(&accounts_path, accounts_json_str);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ =
                std::fs::set_permissions(&accounts_path, std::fs::Permissions::from_mode(0o600));
        }
    }

    // Sync jetski-standalone-oauth-token in ~/.gemini for Go language_server worker fallback
    let mut token_obj = serde_json::json!({
        "access_token": account.token.access_token,
        "token_type": "Bearer",
        "refresh_token": account.token.refresh_token,
        "expiry": expiry_rfc3339,
    });
    if let Some(ref id_tok) = account.token.id_token {
        token_obj["id_token"] = serde_json::Value::String(id_tok.clone());
    }
    let jetski_payload = serde_json::json!({
        "token": token_obj,
        "auth_method": "consumer"
    });
    let jetski_path = gemini_dir.join("jetski-standalone-oauth-token");
    if let Ok(jetski_json) = serde_json::to_string(&jetski_payload) {
        let _ = std::fs::write(&jetski_path, jetski_json);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&jetski_path, std::fs::Permissions::from_mode(0o600));
        }
    }

    // Also mirror to .gemini subdirectories for language_server, agy CLI, and background workers
    for sub in &["antigravity", "antigravity-ide", "antigravity-cli", "cache"] {
        let target_sub = gemini_dir.join(sub);
        if std::fs::create_dir_all(&target_sub).is_ok() {
            let _ = std::fs::copy(&creds_path, target_sub.join("oauth_creds.json"));
            let _ = std::fs::copy(&accounts_path, target_sub.join("google_accounts.json"));
            let _ = std::fs::copy(
                &jetski_path,
                target_sub.join("jetski-standalone-oauth-token"),
            );
        }
    }

    if !base_home.ends_with(".gemini") {
        let _ = std::fs::copy(&creds_path, base_home.join("oauth_creds.json"));
        let _ = std::fs::copy(&accounts_path, base_home.join("google_accounts.json"));
        let _ = std::fs::copy(
            &jetski_path,
            base_home.join("jetski-standalone-oauth-token"),
        );
    }

    for marker_name in &[
        "antigravity-keyring-unavailable",
        "antigravity-ide-keyring-unavailable",
        "antigravity-cli-keyring-unavailable",
    ] {
        let _ = std::fs::write(gemini_dir.join(marker_name), b"1\n");
        let _ = std::fs::write(gemini_dir.join("antigravity").join(marker_name), b"1\n");
        let _ = std::fs::write(gemini_dir.join("antigravity-ide").join(marker_name), b"1\n");
        let _ = std::fs::write(gemini_dir.join("antigravity-cli").join(marker_name), b"1\n");
        let _ = std::fs::write(gemini_dir.join("cache").join(marker_name), b"1\n");
        if !base_home.ends_with(".gemini") {
            let _ = std::fs::write(base_home.join(marker_name), b"1\n");
        }
    }

    crate::modules::logger::log_info(&format!(
        "[Desktop] Successfully synced file-based credentials to {:?} for: {}",
        gemini_dir, account.email
    ));

    Ok(())
}

/// 辅助方法：同步写入本地文件凭据 (~/.gemini/oauth_creds.json, ~/.gemini/google_accounts.json, 以及 ~/.gemini/jetski-standalone-oauth-token)
/// 用于在 SSH 会话、容器环境或无系统 Keyring / D-Bus 的场景下保障 CLI/工具/Worker 的凭据兼容性
pub fn write_to_file_credentials(account: &crate::models::Account) -> Result<(), String> {
    let home = match dirs::home_dir() {
        Some(h) => h,
        None => return Err("Failed to resolve user home directory".to_string()),
    };
    write_to_file_credentials_at(&home, account)
}

/// 辅助方法：从本地文件凭据 (~/.gemini/oauth_creds.json) 读取 Token 作为跨平台回退
fn read_from_file_credentials() -> Result<crate::modules::migration::ImportedOAuthState, String> {
    let home =
        dirs::home_dir().ok_or_else(|| "Failed to resolve user home directory".to_string())?;
    let creds_path = home.join(".gemini").join("oauth_creds.json");
    if !creds_path.exists() {
        return Err("No ~/.gemini/oauth_creds.json found".to_string());
    }
    let content = fs::read_to_string(&creds_path)
        .map_err(|e| format!("Failed to read oauth_creds.json: {}", e))?;
    let json: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| format!("Failed to parse oauth_creds.json: {}", e))?;
    let refresh_token = json
        .get("refresh_token")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Refresh token not found in oauth_creds.json".to_string())?
        .to_string();
    Ok(crate::modules::migration::ImportedOAuthState {
        refresh_token,
        is_gcp_tos: true,
        project_id: None,
    })
}

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

fn parse_keyring_payload(
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

/// Headless/Docker 实现：仅执行数据层操作，忽略 UI 和进程控制
pub struct HeadlessIntegration;

impl SystemIntegration for HeadlessIntegration {
    async fn on_account_switch(
        &self,
        account: &crate::models::Account,
        target_ide: Option<&str>,
    ) -> Result<(), String> {
        if target_ide == Some("agy") {
            return Err(
                "Switching to the agy CLI is not supported in headless mode (no host keyring access)."
                    .to_string(),
            );
        }

        crate::modules::logger::log_info(&format!(
            "[Headless] Delegating account switch for '{}' to DesktopIntegration without GUI handle",
            account.email
        ));
        let desktop = DesktopIntegration { app_handle: None };
        desktop.on_account_switch(account, target_ide).await
    }

    fn update_tray(&self) {
        // No-op
    }

    fn show_notification(&self, title: &str, body: &str) {
        crate::modules::logger::log_info(&format!("[Log Notification] {}: {}", title, body));
    }
}

/// 系统集成管理器：替代 Arc<dyn SystemIntegration> 以解决 async trait 的 dyn 兼容性问题
#[derive(Clone)]
pub enum SystemManager {
    Desktop(tauri::AppHandle),
    Headless,
}

impl SystemManager {
    pub async fn on_account_switch(
        &self,
        account: &Account,
        target_ide: Option<&str>,
    ) -> Result<(), String> {
        match self {
            SystemManager::Desktop(handle) => {
                let integration = DesktopIntegration {
                    app_handle: Some(handle.clone()),
                };
                integration.on_account_switch(account, target_ide).await
            }
            SystemManager::Headless => {
                let integration = DesktopIntegration { app_handle: None };
                integration.on_account_switch(account, target_ide).await
            }
        }
    }

    pub fn update_tray(&self) {
        if let SystemManager::Desktop(handle) = self {
            let integration = DesktopIntegration {
                app_handle: Some(handle.clone()),
            };
            integration.update_tray();
        }
    }

    pub fn show_notification(&self, title: &str, body: &str) {
        match self {
            SystemManager::Desktop(handle) => {
                let integration = DesktopIntegration {
                    app_handle: Some(handle.clone()),
                };
                integration.show_notification(title, body);
            }
            SystemManager::Headless => {
                let integration = HeadlessIntegration;
                integration.show_notification(title, body);
            }
        }
    }
}

impl SystemIntegration for SystemManager {
    async fn on_account_switch(
        &self,
        account: &crate::models::Account,
        target_ide: Option<&str>,
    ) -> Result<(), String> {
        match self {
            SystemManager::Desktop(handle) => {
                let integration = DesktopIntegration {
                    app_handle: Some(handle.clone()),
                };
                integration.on_account_switch(account, target_ide).await
            }
            SystemManager::Headless => {
                let integration = DesktopIntegration { app_handle: None };
                integration.on_account_switch(account, target_ide).await
            }
        }
    }

    fn update_tray(&self) {
        self.update_tray();
    }

    fn show_notification(&self, title: &str, body: &str) {
        self.show_notification(title, body);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_keyring_payload_nested_token() {
        let payload = r#"{
            "token": {
                "access_token": "ya29.test",
                "token_type": "Bearer",
                "refresh_token": "1//test_refresh_token_123",
                "expiry": "2026-09-19T10:00:00.000000Z"
            },
            "auth_method": "consumer"
        }"#;
        let state = parse_keyring_payload(payload).expect("Failed to parse nested keyring payload");
        assert_eq!(state.refresh_token, "1//test_refresh_token_123");
        assert!(state.is_gcp_tos);
    }

    #[test]
    fn test_parse_keyring_payload_flat_token() {
        let payload = r#"{
            "access_token": "ya29.test",
            "refresh_token": "1//test_refresh_token_flat"
        }"#;
        let state = parse_keyring_payload(payload).expect("Failed to parse flat keyring payload");
        assert_eq!(state.refresh_token, "1//test_refresh_token_flat");
    }

    #[test]
    fn test_parse_keyring_payload_missing_token() {
        let payload = r#"{ "auth_method": "consumer" }"#;
        let res = parse_keyring_payload(payload);
        assert!(res.is_err());
    }

    #[test]
    fn test_resolve_effective_target_explicit_classic() {
        // 显式指定 classic，即便 IDE 正在运行或只有 IDE exe，也必须严格判定为经典版
        let (is_ide, effective) = resolve_effective_target(
            Some("classic"),
            false,
            true,
            false,
            Some("/Applications/Antigravity IDE.app"),
        );
        assert!(!is_ide);
        assert_eq!(effective, Some("classic"));
    }

    #[test]
    fn test_resolve_effective_target_explicit_ide() {
        // 显式指定 ide，必须判定为 ide
        let (is_ide, effective) = resolve_effective_target(Some("ide"), true, false, true, None);
        assert!(is_ide);
        assert_eq!(effective, Some("ide"));
    }

    #[test]
    fn test_resolve_effective_target_autodetect_classic_running() {
        // target_ide 为 None，经典版正在运行，必须优先保持经典版
        let (is_ide, effective) = resolve_effective_target(
            None,
            true,
            true,
            true,
            Some("/Applications/Antigravity IDE.app"),
        );
        assert!(!is_ide);
        assert_eq!(effective, None);
    }

    #[test]
    fn test_resolve_effective_target_autodetect_ide_running_only() {
        // target_ide 为 None，仅 IDE 正在运行，推导为 IDE
        let (is_ide, effective) = resolve_effective_target(
            None,
            false,
            true,
            true,
            Some("/Applications/Antigravity IDE.app"),
        );
        assert!(is_ide);
        assert_eq!(effective, Some("ide"));
    }

    #[test]
    fn test_resolve_effective_target_autodetect_classic_exe_exists() {
        // target_ide 为 None，两者均未运行，但经典版 exe 存在，优先经典版
        let (is_ide, effective) = resolve_effective_target(
            None,
            false,
            false,
            true,
            Some("/Applications/Antigravity IDE.app"),
        );
        assert!(!is_ide);
        assert_eq!(effective, None);
    }

    #[test]
    fn test_resolve_effective_target_autodetect_fallback_ide_exe() {
        // target_ide 为 None，两者均未运行，无经典版但有 IDE exe，推导为 IDE
        let (is_ide, effective) = resolve_effective_target(
            None,
            false,
            false,
            false,
            Some("/Applications/Antigravity IDE.app"),
        );
        assert!(is_ide);
        assert_eq!(effective, Some("ide"));
    }

    #[test]
    fn test_resolve_effective_target_autodetect_default_fallback() {
        // target_ide 为 None，均未运行且均未检测到 exe，默认保底经典版
        let (is_ide, effective) = resolve_effective_target(None, false, false, false, None);
        assert!(!is_ide);
        assert_eq!(effective, None);
    }
}
