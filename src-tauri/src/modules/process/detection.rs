use super::*;

/// Get Antigravity executable path (cross-platform)
///
/// Search strategy (highest to lowest priority):
/// 1. Get path from running process (most reliable, supports any location)
/// Comprehensive detection engine that returns either the found PathBuf or a detailed AppError with audit trail and stack trace
pub fn detect_antigravity_with_diagnostics(
    target_ide: Option<&str>,
) -> Result<std::path::PathBuf, crate::error::AppError> {
    let mut audit_log = Vec::new();

    // Strategy 1: Check running processes (supports any custom location)
    if let Some(path) = get_path_from_running_process(target_ide) {
        crate::modules::logger::log_info(&format!(
            "[IDE Discovery] Located via running process: {:?}",
            path
        ));
        return Ok(path);
    }
    audit_log.push("No active Antigravity process found in process table".to_string());

    // Strategy 2: Check config paths (supports user-configured locations)
    if let Ok(config) = crate::modules::config::load_app_config() {
        let manual = if target_ide == Some("ide") {
            config.antigravity_ide_executable.as_ref()
        } else {
            config.antigravity_executable.as_ref()
        };

        if let Some(p) = manual {
            let path = std::path::PathBuf::from(p);
            if path.exists() {
                crate::modules::logger::log_info(&format!(
                    "[IDE Discovery] Located via user configuration: {:?}",
                    path
                ));
                return Ok(path);
            }
            audit_log.push(format!(
                "Configured path '{:?}' does not exist on disk",
                path
            ));
        } else {
            audit_log.push("No custom executable path configured in Settings".to_string());
        }
    }

    // Strategy 3: Standard installation locations, PATH, and Desktop entries
    let (found, checked) = audit_standard_locations(target_ide);
    if let Some(path) = found {
        crate::modules::logger::log_info(&format!(
            "[IDE Discovery] Located via filesystem search: {:?}",
            path
        ));
        return Ok(path);
    }

    let stack = std::backtrace::Backtrace::capture().to_string();
    let diag_summary = format!(
        "Audit Log:\n{}\n\nTested {} candidate locations across PATH, standard directories, Snap, Flatpak, and .desktop entries.\nNone matched an existing executable binary.",
        audit_log.join("\n"),
        checked.len()
    );

    Err(crate::error::AppError::IdeNotFound {
        message: format!(
            "Could not locate Antigravity executable on this system (tested {} locations).",
            checked.len()
        ),
        target_ide: target_ide.map(|s| s.to_string()),
        searched_locations: checked,
        diagnostics: diag_summary,
        stack_trace: stack,
    })
}

/// Fallback wrapper returning Option for backward compatibility
pub fn get_antigravity_executable_path(target_ide: Option<&str>) -> Option<std::path::PathBuf> {
    detect_antigravity_with_diagnostics(target_ide).ok()
}

/// Resolve the platform-specific path to the IDE discovery log:
/// `{data_local_dir}/antigravity/ide-discovery.log`
pub fn get_ide_discovery_log_path() -> Option<std::path::PathBuf> {
    dirs::data_local_dir().map(|d| d.join("antigravity").join("ide-discovery.log"))
}

/// Append a structured JSON diagnostic entry to `{data_local_dir}/antigravity/ide-discovery.log`.
/// Never propagates I/O errors, only logging failures via `tracing::warn!`.
pub fn append_ide_discovery_log(
    found: bool,
    status: Option<&str>,
    path: &str,
    method: &str,
    checked_paths: Option<&[String]>,
    backtrace: &str,
) {
    let log_path = match get_ide_discovery_log_path() {
        Some(p) => p,
        None => return,
    };

    if let Some(parent) = log_path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            tracing::warn!(
                "[IDE Discovery] Failed to create log directory {:?}: {}",
                parent,
                e
            );
            return;
        }
    }

    let mut entry = serde_json::json!({
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "found": found,
        "status": status.unwrap_or(if found { "SUCCESS" } else { "FAILED" }),
        "path": path,
        "method": method,
        "backtrace": backtrace,
    });
    if let Some(paths) = checked_paths {
        entry["checked_paths"] = serde_json::json!(paths);
    }

    match std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        Ok(mut file) => {
            use std::io::Write;
            let mut line = match serde_json::to_string(&entry) {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!("[IDE Discovery] Failed to serialize discovery log: {}", e);
                    return;
                }
            };
            line.push('\n');
            if let Err(e) = file.write_all(line.as_bytes()) {
                tracing::warn!(
                    "[IDE Discovery] Failed to write to log file {:?}: {}",
                    log_path,
                    e
                );
            }
        }
        Err(e) => {
            tracing::warn!(
                "[IDE Discovery] Failed to open log file {:?}: {}",
                log_path,
                e
            );
        }
    }
}

/// Discover Antigravity IDE on first-time startup or when unconfigured,
/// persisting the located executable into gui_config.json.
pub fn discover_and_persist_initial_ide_info() -> Option<std::path::PathBuf> {
    let mut config = crate::modules::config::load_app_config().unwrap_or_default();

    if let Some(ref exe_str) = config.antigravity_executable {
        if !exe_str.trim().is_empty() {
            let path = std::path::PathBuf::from(exe_str);
            if path.exists() {
                crate::modules::logger::log_info(&format!(
                    "[IDE Discovery] Configured Antigravity IDE executable already exists: {:?}",
                    path
                ));
                append_ide_discovery_log(
                    true,
                    Some("SUCCESS"),
                    exe_str,
                    "existing_config",
                    None,
                    "",
                );
                return Some(path);
            }
        }
    }

    match detect_antigravity_with_diagnostics(None) {
        Ok(path) => {
            let path_str = path.to_string_lossy().to_string();
            config.antigravity_executable = Some(path_str.clone());
            if let Err(e) = crate::modules::config::save_app_config(&config) {
                crate::modules::logger::log_warn(&format!(
                    "[IDE Discovery] Failed to persist discovered IDE to config: {}",
                    e
                ));
            }
            crate::modules::logger::log_info(&format!(
                "[IDE Discovery] First-time startup located Antigravity IDE: {:?}",
                path
            ));
            append_ide_discovery_log(
                true,
                Some("SUCCESS"),
                &path_str,
                "initial_discovery",
                None,
                "",
            );
            Some(path)
        }
        Err(err) => {
            let trace = std::backtrace::Backtrace::capture();
            let (diagnostics, original_stack, searched) = match err {
                crate::error::AppError::IdeNotFound {
                    ref diagnostics,
                    ref stack_trace,
                    ref searched_locations,
                    ..
                } => (
                    diagnostics.as_str(),
                    stack_trace.as_str(),
                    Some(searched_locations.as_slice()),
                ),
                _ => ("No detailed diagnostics available", "", None),
            };
            crate::modules::logger::log_warn(&format!(
                "[IDE Discovery] No Antigravity IDE detected during initial discovery.\nDiagnostics:\n{}\nStack trace:\n{:?}\nOriginal Stack:\n{}",
                diagnostics, trace, original_stack
            ));
            let trace_str = format!("{:?}\nOriginal Stack:\n{}", trace, original_stack);
            append_ide_discovery_log(
                false,
                Some("FAILED"),
                "",
                "initial_discovery",
                searched,
                &trace_str,
            );
            None
        }
    }
}
