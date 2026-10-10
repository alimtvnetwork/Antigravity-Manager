use super::*;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

const VERSION_PROBE_TIMEOUT: Duration = Duration::from_secs(5);

const OPENCLAW_DIR: &str = ".openclaw";

const OPENCLAW_CONFIG_FILE: &str = "openclaw.json";

const BACKUP_SUFFIX: &str = ".antigravity-manager.bak";

pub(crate) const PROVIDER_ID: &str = "antigravity-manager";

static OPENCLAW_CONFIG_MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());

pub(crate) fn acquire_openclaw_config_lock() -> std::sync::MutexGuard<'static, ()> {
    OPENCLAW_CONFIG_MUTEX.lock().unwrap_or_else(|poisoned| {
        tracing::warn!("OPENCLAW_CONFIG_MUTEX was poisoned, recovering lock");
        poisoned.into_inner()
    })
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct OpenClawStatus {
    pub installed: bool,
    pub version: Option<String>,
    pub detected_version_target: Option<String>, // "v1" (<2026.8.1) or "v2" (>=2026.8.1)
    pub is_synced: bool,
    pub has_backup: bool,
    pub current_base_url: Option<String>,
    pub files: Vec<String>,
    pub configured_models: Vec<String>,
    pub is_active: bool,
    pub default_model: Option<String>,
    pub synced_version: Option<String>, // "v1" or "v2"
}

pub(crate) fn get_openclaw_dir() -> Option<PathBuf> {
    env::var_os("OPENCLAW_DIR")
        .filter(|p| !p.is_empty())
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|h| h.join(OPENCLAW_DIR)))
}

pub(crate) fn get_config_path() -> Option<PathBuf> {
    env::var_os("OPENCLAW_CONFIG_PATH")
        .filter(|p| !p.is_empty())
        .map(PathBuf::from)
        .or_else(|| get_openclaw_dir().map(|dir| dir.join(OPENCLAW_CONFIG_FILE)))
}

pub(crate) fn get_backup_path() -> Option<PathBuf> {
    get_config_path().map(|path| {
        path.with_file_name(format!(
            "{}{}",
            path.file_name().unwrap_or_default().to_string_lossy(),
            BACKUP_SUFFIX
        ))
    })
}

pub(crate) fn normalize_base_url(input: &str) -> String {
    let trimmed = input.trim().trim_end_matches('/');
    if trimmed.ends_with("/v1") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/v1")
    }
}

pub(crate) fn find_in_path(executable: &str) -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        if let Ok(path_var) = env::var("PATH") {
            for dir in path_var.split(';') {
                for ext in ["cmd", "exe", "bat"] {
                    let path = PathBuf::from(dir).join(format!("{executable}.{ext}"));
                    if path.exists() {
                        return Some(path);
                    }
                }
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(path_var) = env::var("PATH") {
            for dir in path_var.split(':') {
                let path = PathBuf::from(dir).join(executable);
                if path.exists() {
                    return Some(path);
                }
            }
        }
    }
    None
}

pub(crate) fn resolve_openclaw_path() -> Option<PathBuf> {
    if let Some(path) = find_in_path("openclaw") {
        return Some(path);
    }
    #[cfg(not(target_os = "windows"))]
    {
        if let Some(home) = dirs::home_dir() {
            for path in [
                home.join(".openclaw/bin/openclaw"),
                home.join(".local/bin/openclaw"),
                PathBuf::from("/opt/homebrew/bin/openclaw"),
                PathBuf::from("/usr/local/bin/openclaw"),
                PathBuf::from("/usr/bin/openclaw"),
            ] {
                if path.exists() {
                    return Some(path);
                }
            }
        }
    }
    None
}

pub(crate) fn extract_version(raw: &str) -> String {
    let trimmed = raw.trim();
    for part in trimmed.split_whitespace() {
        let candidate = part.rsplit('/').next().unwrap_or(part);
        let clean = candidate
            .strip_prefix('v')
            .or_else(|| candidate.strip_prefix('V'))
            .unwrap_or(candidate);
        if is_valid_version(clean) {
            return clean.to_string();
        }
    }
    let fallback: String = trimmed
        .chars()
        .skip_while(|c| !c.is_ascii_digit())
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    if fallback.contains('.') {
        fallback
    } else {
        "unknown".to_string()
    }
}

pub(crate) fn is_valid_version(value: &str) -> bool {
    value.chars().next().is_some_and(|c| c.is_ascii_digit())
        && value.contains('.')
        && value.chars().all(|c| c.is_ascii_digit() || c == '.')
}

/// 判断版本是否 >= 2026.8.1 (v2.0)
/// 例如: 2026.8.1 -> v2, 2026.9.2 -> v2, 2.0.0 -> v2
/// 2026.7.749 -> v1, 1.0.0 -> v1
pub fn classify_openclaw_version(version_str: &str) -> &'static str {
    let nums: Vec<u64> = version_str
        .split('.')
        .filter_map(|part| part.parse::<u64>().ok())
        .collect();

    if nums.is_empty() {
        return "v2"; // 缺省使用最新 2.0
    }

    // 经典 SemVer: 2.x.x
    if nums[0] >= 2 && nums[0] < 1000 {
        return "v2";
    }

    // 日期版本号: 2026.8.1 边界判断
    if nums[0] > 2026 {
        return "v2";
    }
    if nums[0] == 2026 {
        let minor = nums.get(1).copied().unwrap_or(0);
        let patch = nums.get(2).copied().unwrap_or(0);
        if minor > 8 || (minor == 8 && patch >= 1) {
            return "v2";
        }
        return "v1";
    }

    "v1"
}

pub(crate) async fn run_version_command(mut command: Command, timeout: Duration) -> Option<String> {
    command.kill_on_drop(true);
    #[cfg(target_os = "windows")]
    command.creation_flags(CREATE_NO_WINDOW);
    let output = tokio::time::timeout(timeout, command.output()).await;

    match output {
        Ok(Ok(output)) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            Some(extract_version(if stdout.trim().is_empty() {
                &stderr
            } else {
                &stdout
            }))
        }
        _ => None,
    }
}

pub(crate) async fn run_openclaw_version(path: &PathBuf) -> Option<String> {
    let mut command = Command::new(path);
    command.arg("--version");
    run_version_command(command, VERSION_PROBE_TIMEOUT).await
}

pub async fn check_openclaw_installed() -> (bool, Option<String>, Option<String>) {
    match resolve_openclaw_path() {
        Some(path) => {
            let version = run_openclaw_version(&path).await;
            let target = version
                .as_deref()
                .map(classify_openclaw_version)
                .map(str::to_string);
            (true, version, target)
        }
        None => (false, None, None),
    }
}

pub(crate) fn read_openclaw_config(path: &PathBuf) -> Result<Value, String> {
    match fs::read_to_string(path) {
        Ok(content) if content.trim().is_empty() => Ok(json!({})),
        Ok(content) => serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse openclaw.json: {e}")),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(json!({})),
        Err(e) => Err(format!("Failed to read openclaw.json: {e}")),
    }
}

pub(crate) fn atomically_write_config(path: &PathBuf, config: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("Failed to create directory: {e}"))?;
    }
    let json_str = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {e}"))?;

    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(OPENCLAW_CONFIG_FILE);
    let tmp_path = path.with_file_name(format!("{file_name}.tmp.{}", uuid::Uuid::new_v4()));

    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }

    let mut file = options
        .open(&tmp_path)
        .map_err(|e| format!("Failed to create temp file: {e}"))?;

    let write_res = (|| -> std::io::Result<()> {
        use std::io::Write;
        #[cfg(unix)]
        if let Ok(metadata) = fs::metadata(path) {
            file.set_permissions(metadata.permissions())?;
        }
        file.write_all(json_str.as_bytes())?;
        file.sync_all()
    })();

    drop(file);

    if let Err(e) = write_res {
        // Justification: best-effort cleanup; a leftover file is harmless
        crate::error::record_ignored(fs::remove_file(&tmp_path), "remove_file");
        return Err(format!("Failed to write config: {e}"));
    }

    fs::rename(&tmp_path, path).map_err(|e| {
        // Justification: best-effort cleanup; a leftover file is harmless
        crate::error::record_ignored(fs::remove_file(&tmp_path), "remove_file");
        format!("Failed to rename config file: {e}")
    })
}

pub(crate) fn create_backup(path: &PathBuf) -> Result<(), String> {
    let backup = path.with_file_name(format!(
        "{}{}",
        path.file_name().unwrap_or_default().to_string_lossy(),
        BACKUP_SUFFIX
    ));
    if !backup.exists() {
        let current = read_openclaw_config(path)?;
        atomically_write_config(&backup, &current)?;
    }
    Ok(())
}

pub(crate) fn is_managed_provider(provider_or_model: &str) -> bool {
    provider_or_model == PROVIDER_ID || provider_or_model.starts_with(&format!("{PROVIDER_ID}/"))
}

pub(crate) fn redact_json_value(val: &mut Value) {
    match val {
        Value::Object(map) => {
            for (k, v) in map.iter_mut() {
                let lower = k.to_ascii_lowercase();
                if lower == "api_key"
                    || lower == "apikey"
                    || lower.ends_with("_api_key")
                    || lower == "token"
                    || lower.ends_with("_token")
                    || lower == "secret"
                    || lower.ends_with("_secret")
                    || lower == "password"
                    || lower.ends_with("_password")
                {
                    if let Value::String(_) = v {
                        *v = Value::String("[REDACTED]".to_string());
                    }
                } else {
                    redact_json_value(v);
                }
            }
        }
        Value::Array(arr) => {
            for item in arr.iter_mut() {
                redact_json_value(item);
            }
        }
        _ => {}
    }
}

/// 根据模型名称动态推导精确的上下文窗口 (contextWindow)、最大输出 (maxTokens)、思考开关及输入模态
/// 用户特别强调：
/// - Gemini 3 及以上：上下文 1024000 (1024K)，输出 65536
/// - Claude 系列：上下文基本都是 256K (256000)，输出 65536
pub fn resolve_model_specs(model_id: &str) -> (u64, u64, bool, Vec<&'static str>) {
    let lower = model_id.to_lowercase();

    let is_gemini = lower.contains("gemini");
    let is_claude = lower.contains("claude");
    let is_reasoning = lower.contains("thinking")
        || lower.contains("reasoning")
        || lower.contains("claude-3-7")
        || lower.contains("gemini-2.5")
        || lower.contains("gemini-3")
        || lower.contains("o1")
        || lower.contains("o3")
        || lower.contains("r1");

    let inputs = vec!["text", "image"];

    if is_gemini {
        // 用户指定: Gemini 3 及以上上下文 1024000，输出 65536
        let context_window = 1024000;
        let max_tokens = 65536;
        (context_window, max_tokens, is_reasoning, inputs)
    } else if is_claude {
        // 用户指定: Claude 系列上下文基本都是 256K (256000)，输出 65536
        let context_window = 256000;
        let max_tokens = 65536;
        (context_window, max_tokens, is_reasoning, inputs)
    } else if lower.contains("o1") || lower.contains("o3") || lower.contains("o4") {
        let context_window = 200000;
        let max_tokens = 100000;
        (context_window, max_tokens, true, inputs)
    } else if lower.contains("gpt-4") || lower.contains("codex") {
        let context_window = 128000;
        let max_tokens = 16384;
        (context_window, max_tokens, is_reasoning, inputs)
    } else if lower.contains("deepseek") {
        let context_window = 128000;
        let max_tokens = if is_reasoning { 65536 } else { 8192 };
        (context_window, max_tokens, is_reasoning, vec!["text"])
    } else {
        // 缺省通用配置
        let context_window = 128000;
        let max_tokens = 16384;
        (context_window, max_tokens, is_reasoning, inputs)
    }
}
