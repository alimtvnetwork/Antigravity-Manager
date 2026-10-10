use super::*;
use crate::proxy::opencode_sync::lock::BACKUP_SUFFIX;

pub fn get_hermes_dir() -> Option<PathBuf> {
    env::var_os("HERMES_HOME")
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|home| home.join(HERMES_DIR)))
}

pub fn get_config_path() -> Option<PathBuf> {
    get_hermes_dir().map(|dir| dir.join(HERMES_CONFIG_FILE))
}

pub fn get_backup_path() -> Option<PathBuf> {
    get_config_path().map(|path| {
        path.with_file_name(format!(
            "{}{}",
            path.file_name().unwrap_or_default().to_string_lossy(),
            BACKUP_SUFFIX
        ))
    })
}

pub fn normalize_base_url(input: &str) -> String {
    let trimmed = input.trim().trim_end_matches('/');
    if trimmed.ends_with("/v1") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/v1")
    }
}

pub fn find_in_path(executable: &str) -> Option<PathBuf> {
    #[cfg(target_os = "windows")]
    {
        for dir in env::var("PATH").ok()?.split(';') {
            for ext in ["exe", "cmd", "bat"] {
                let path = PathBuf::from(dir).join(format!("{executable}.{ext}"));
                if path.exists() {
                    return Some(path);
                }
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        for dir in env::var("PATH").ok()?.split(':') {
            let path = PathBuf::from(dir).join(executable);
            if path.exists() {
                return Some(path);
            }
        }
    }
    None
}

pub fn resolve_hermes_path() -> Option<PathBuf> {
    if let Some(path) = find_in_path("hermes") {
        return Some(path);
    }
    #[cfg(not(target_os = "windows"))]
    {
        let home = dirs::home_dir()?;
        for path in [
            home.join(".hermes/bin/hermes"),
            home.join(".local/bin/hermes"),
            PathBuf::from("/opt/homebrew/bin/hermes"),
            PathBuf::from("/usr/local/bin/hermes"),
            PathBuf::from("/usr/bin/hermes"),
        ] {
            if path.exists() {
                return Some(path);
            }
        }
    }
    None
}

pub fn extract_version(raw: &str) -> String {
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
        .skip_while(|character| !character.is_ascii_digit())
        .take_while(|character| character.is_ascii_digit() || *character == '.')
        .collect();
    if fallback.contains('.') {
        fallback
    } else {
        "unknown".to_string()
    }
}

pub fn is_valid_version(value: &str) -> bool {
    value
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_digit())
        && value.contains('.')
        && value
            .chars()
            .all(|character| character.is_ascii_digit() || character == '.')
}

pub async fn run_version_command(mut command: Command, timeout: Duration) -> Option<String> {
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

pub async fn run_hermes_version(path: &PathBuf) -> Option<String> {
    let mut command = Command::new(path);
    command.arg("--version");
    run_version_command(command, VERSION_PROBE_TIMEOUT).await
}

pub async fn check_hermes_installed() -> (bool, Option<String>) {
    match resolve_hermes_path() {
        Some(path) => (true, run_hermes_version(&path).await),
        None => (false, None),
    }
}
