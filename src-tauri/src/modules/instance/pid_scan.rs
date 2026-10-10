//! OS process-table scanning for instance PIDs.
use super::*;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use sysinfo::System;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[derive(Clone)]
pub(crate) struct CachedProcessInfo {
    pid: u32,
    parent: Option<u32>,
    name: String,
    exe: String,
    args_str: String,
}

pub(crate) static PROCESS_SCAN_CACHE: Lazy<
    std::sync::Mutex<(std::time::Instant, Vec<CachedProcessInfo>)>,
> = Lazy::new(|| {
    std::sync::Mutex::new((
        std::time::Instant::now() - std::time::Duration::from_secs(10),
        Vec::new(),
    ))
});

pub(crate) fn get_cached_antigravity_processes() -> Vec<CachedProcessInfo> {
    // Recover (rather than panic) if a previous holder panicked while holding
    // the lock; the cache is refresh-on-stale so poisoned data is harmless.
    let mut cache = PROCESS_SCAN_CACHE.lock().unwrap_or_else(|e| e.into_inner());
    if cache.0.elapsed() < std::time::Duration::from_millis(4000) && !cache.1.is_empty() {
        return cache.1.clone();
    }

    let refresh_kind = sysinfo::ProcessRefreshKind::new()
        .with_cmd(sysinfo::UpdateKind::OnlyIfNotSet)
        .with_exe(sysinfo::UpdateKind::OnlyIfNotSet);

    let mut system =
        System::new_with_specifics(sysinfo::RefreshKind::new().with_processes(refresh_kind));
    system.refresh_processes_specifics(sysinfo::ProcessesToUpdate::All, refresh_kind);

    let mut procs = Vec::new();
    for (pid, process) in system.processes() {
        let name = process.name().to_string_lossy().to_lowercase();
        let exe = process
            .exe()
            .and_then(|p| p.to_str())
            .unwrap_or("")
            .to_lowercase();
        let args = process.cmd();
        let args_str = args
            .iter()
            .map(|a| a.to_string_lossy().to_lowercase().replace('\\', "/"))
            .collect::<Vec<String>>()
            .join(" ");

        let is_non_ide = crate::modules::process::is_non_ide_binary(&name, &exe, &args_str);
        let is_self_process = if let Ok(current) = std::env::current_exe() {
            let cur_str = current.to_string_lossy().to_lowercase().replace('\\', "/");
            !cur_str.is_empty()
                && (exe == cur_str
                    || name
                        == current
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_lowercase())
        } else {
            false
        };
        let is_antigravity = !is_non_ide
            && !is_self_process
            && (name.contains("antigravity")
                || exe.contains("antigravity")
                || exe.contains("/tmp/.mount_")
                || name == "apprun")
            && !name.contains("agm")
            && !exe.contains("agm")
            && !name.contains("antigravity-manager")
            && !exe.contains("antigravity-manager")
            && !name.contains("antigravity_manager")
            && !exe.contains("antigravity_manager")
            && !name.contains("antigravity manager")
            && !exe.contains("antigravity manager")
            && !name.ends_with("manager.exe")
            && !exe.ends_with("manager.exe")
            && !name.ends_with("manager")
            && !exe.ends_with("manager")
            && !name.contains("webview")
            && !exe.contains("webview")
            && !args_str.contains("embedded-browser-webview");

        if is_antigravity {
            procs.push(CachedProcessInfo {
                pid: pid.as_u32(),
                parent: process.parent().map(|p| p.as_u32()),
                name,
                exe,
                args_str,
            });
        }
    }

    cache.0 = std::time::Instant::now();
    cache.1 = procs.clone();
    procs
}

#[cfg(target_os = "windows")]
fn get_windows_8_3_short_path(path: &str) -> Option<String> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    extern "system" {
        fn GetShortPathNameW(
            lpszLongPath: *const u16,
            lpszShortPath: *mut u16,
            cchBuffer: u32,
        ) -> u32;
    }
    let wide: Vec<u16> = OsStr::new(path)
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let mut buf = vec![0u16; 512];
    let len = unsafe { GetShortPathNameW(wide.as_ptr(), buf.as_mut_ptr(), buf.len() as u32) };
    if len > 0 && (len as usize) < buf.len() {
        let short = String::from_utf16_lossy(&buf[..len as usize]);
        Some(
            short
                .to_lowercase()
                .replace('\\', "/")
                .trim_end_matches('/')
                .to_string(),
        )
    } else {
        None
    }
}

#[cfg(not(target_os = "windows"))]
fn get_windows_8_3_short_path(_path: &str) -> Option<String> {
    None
}

/// Inspect running Antigravity processes matching an instance data_dir
pub fn find_pids_for_data_dir(data_dir: &str, is_default: bool) -> Vec<u32> {
    let processes = get_cached_antigravity_processes();

    let norm_slash = data_dir.to_lowercase().replace('\\', "/");
    let clean_slash = norm_slash.trim_end_matches('/').to_string();
    let norm_bslash = data_dir.to_lowercase().replace('/', "\\");
    let clean_bslash = norm_bslash.trim_end_matches('\\').to_string();

    let canonical_expanded = std::fs::canonicalize(data_dir).ok().map(|p| {
        let s = p.to_string_lossy().to_lowercase().replace('\\', "/");
        s.trim_start_matches("//?/")
            .trim_start_matches(r"\\?\")
            .trim_end_matches('/')
            .to_string()
    });

    #[cfg(target_os = "windows")]
    let short_path_opt = get_windows_8_3_short_path(data_dir);
    #[cfg(not(target_os = "windows"))]
    let short_path_opt: Option<String> = None;

    let mut matched_pids = Vec::new();

    // Map child PID -> parent PID to trace process lineage
    let mut parent_map: std::collections::HashMap<u32, u32> = std::collections::HashMap::new();
    let mut instance_root_pids: std::collections::HashSet<u32> = std::collections::HashSet::new();
    let mut default_candidate_pids: Vec<u32> = Vec::new();

    for proc in &processes {
        let pid_u32 = proc.pid;
        if let Some(parent) = proc.parent {
            parent_map.insert(pid_u32, parent);
        }

        let name = &proc.name;
        let exe = &proc.exe;
        let args_str = &proc.args_str;

        let is_helper = args_str.contains("--type=")
            || name.contains("helper")
            || name.contains("crashpad")
            || exe.contains("crashpad")
            || name.contains("utility")
            || args_str.contains("utility");

        let inst_id_opt = if clean_slash.contains("/instances/") {
            clean_slash
                .split("/instances/")
                .nth(1)
                .and_then(|s| s.split('/').next())
        } else {
            None
        };

        let matches_cloned_exe = if let Some(inst_id) = inst_id_opt {
            let marker = format!("antigravity-{}", inst_id.to_lowercase());
            exe.contains(&marker) || name.contains(&marker)
        } else {
            false
        };

        let has_user_data_arg = args_str.contains("--user-data-dir");
        let has_instance_marker = args_str.contains(".antigravity_tools")
            || args_str.contains("/instances/")
            || args_str.contains("\\instances\\")
            || exe.contains(".antigravity_tools")
            || matches_cloned_exe;

        let has_target_match = (!clean_slash.is_empty() && args_str.contains(&clean_slash))
            || (!clean_bslash.is_empty() && args_str.contains(&clean_bslash))
            || canonical_expanded
                .as_ref()
                .map_or(false, |c| !c.is_empty() && args_str.contains(c))
            || short_path_opt
                .as_ref()
                .map_or(false, |s| !s.is_empty() && args_str.contains(s));

        let is_default_candidate = is_default
            && !has_instance_marker
            && (!has_user_data_arg || has_target_match)
            && !is_helper
            && !args_str.contains(".antigravity_tools")
            && !args_str.contains("/instances/")
            && !args_str.contains("\\instances\\")
            && !name.contains("manager")
            && !exe.contains("manager");

        if (has_target_match && !is_helper)
            || (has_user_data_arg && has_instance_marker && !is_helper)
            || (matches_cloned_exe && !is_helper)
        {
            instance_root_pids.insert(pid_u32);
            if has_target_match || matches_cloned_exe {
                matched_pids.push(pid_u32);
            }
        } else if is_default_candidate {
            let exe_path = std::path::Path::new(exe);
            let has_ide_markers = if let Some(parent) = exe_path.parent() {
                parent.join("resources").join("app.asar").exists()
                    || parent
                        .join("resources")
                        .join("bin")
                        .join("language_server.exe")
                        .exists()
                    || parent
                        .join("resources")
                        .join("bin")
                        .join("language_server")
                        .exists()
                    || exe.ends_with("antigravity.exe")
                    || exe.ends_with("/antigravity")
                    || name == "antigravity.exe"
                    || name == "antigravity"
            } else {
                name == "antigravity.exe" || name == "antigravity"
            };
            if has_ide_markers {
                default_candidate_pids.push(pid_u32);
            }
        }
    }

    if is_default {
        // Only accept processes whose ancestors do NOT belong to any instance process
        for cand_pid in default_candidate_pids {
            let mut curr = cand_pid;
            let mut is_instance_descendant = false;
            for _ in 0..10 {
                if instance_root_pids.contains(&curr) {
                    is_instance_descendant = true;
                    break;
                }
                if let Some(&p) = parent_map.get(&curr) {
                    curr = p;
                } else {
                    break;
                }
            }
            if !is_instance_descendant {
                matched_pids.push(cand_pid);
            }
        }
    } else {
        // For instances, also include any child processes that descend from matched root PIDs
        let matched_set: std::collections::HashSet<u32> = matched_pids.iter().cloned().collect();
        for proc_info in &processes {
            let pid_u32 = proc_info.pid;
            if matched_set.contains(&pid_u32) {
                continue;
            }
            let mut curr = pid_u32;
            for _ in 0..10 {
                if let Some(&p) = parent_map.get(&curr) {
                    if matched_set.contains(&p) || instance_root_pids.contains(&p) {
                        matched_pids.push(pid_u32);
                        break;
                    }
                    curr = p;
                } else {
                    break;
                }
            }
        }
    }

    matched_pids
}
