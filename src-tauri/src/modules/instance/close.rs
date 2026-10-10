//! Process closing with verified termination.
use super::*;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use sysinfo::System;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// Close an instance's IDE processes and VERIFY they actually died.
///
/// Runs `close_instance`, then polls the OS process table until no PIDs
/// remain for the instance's data dir (or the timeout expires). If stragglers
/// survive the first pass, one more force-close is attempted before giving up.
///
/// Returns `Ok(true)` when the process tree is confirmed dead, `Ok(false)`
/// when PIDs survived both passes. A `false` is NOT an error: on some systems
/// (notably Windows) orphaned Electron child processes can outlive the
/// platform kill, and bricking the user's switch/restart over them is worse
/// than proceeding — the relaunch path removes stale lockfiles and re-injects
/// credentials anyway. Survivors are logged as warnings so they stay visible.
/// Close an instance's IDE processes and VERIFY they actually died.
///
/// Runs `close_instance`, then polls the OS process table until no PIDs
/// remain for the instance's data dir (or the timeout expires). If stragglers
/// survive the first pass, one more force-close is attempted before giving up.
///
/// Returns `Ok(true)` when the process tree is confirmed dead, `Ok(false)`
/// when PIDs survived both passes. A `false` is NOT an error: on some systems
/// (notably Windows) orphaned Electron child processes can outlive the
/// platform kill, and bricking the user's switch/restart over them is worse
/// than proceeding — the relaunch path removes stale lockfiles and re-injects
/// credentials anyway. Survivors are logged as warnings so they stay visible.
///
/// Every stage is recorded to the `[CLOSE_JOURNAL]` log channel so a stuck
/// close is diagnosable from the app log without guessing.
/// Remove stale Chromium/Electron lock artifacts (`lockfile`, `code.lock`,
/// `Singleton*`, `*.lock`) from an instance data directory so a relaunch never
/// collides with a dead session's singleton lock.
pub(crate) fn clear_stale_instance_lockfiles(target_data_path: &Path) {
    if !target_data_path.exists() {
        return;
    }
    let lockfile = target_data_path.join("lockfile");
    if lockfile.exists() {
        // Justification: cleanup of an optional file; absence is the normal case
        crate::error::record_ignored(fs::remove_file(&lockfile), "remove file");
    }
    let code_lock = target_data_path.join("code.lock");
    if code_lock.exists() {
        // Justification: cleanup of an optional file; absence is the normal case
        crate::error::record_ignored(fs::remove_file(&code_lock), "remove file");
    }
    if let Ok(entries) = fs::read_dir(target_data_path) {
        for entry in entries.flatten() {
            let fname = entry.file_name().to_string_lossy().to_lowercase();
            let is_stale_lock = fname == "lockfile"
                || fname.starts_with("singleton")
                || fname.ends_with(".lock")
                || fname == "code.lock";
            if is_stale_lock {
                // Justification: cleanup of an optional file; absence is the normal case
                crate::error::record_ignored(fs::remove_file(entry.path()), "remove file");
            }
        }
    }
}

pub fn close_instance_verified(
    instance_id: &str,
    timeout: std::time::Duration,
) -> Result<bool, String> {
    let registry = load_registry()?;
    let config = registry
        .instances
        .iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance {} not found", instance_id))?;
    let data_dir = config.data_dir.clone();
    let is_default = config.is_default || instance_id == "default";

    let live_pids = || {
        find_pids_for_data_dir(&data_dir, is_default)
            .into_iter()
            .filter(|&pid| is_pid_alive_targeted(pid))
            .collect::<Vec<u32>>()
    };

    let journal_start = std::time::Instant::now();
    for attempt in 0..2 {
        let targeted = live_pids();
        crate::modules::logger::log_info(&format!(
            "[CLOSE_JOURNAL] instance='{}' stage=kill_pass attempt={} pids={:?}",
            instance_id, attempt, targeted
        ));
        // close_instance only errors on registry/instance problems (both
        // validated above); the kill itself is best-effort per pass and the
        // verification poll below is what decides success.
        close_instance(instance_id)?;
        let deadline = std::time::Instant::now() + timeout;
        let mut last_poll_log = std::time::Instant::now();
        let mut first_poll = true;
        loop {
            let remaining = live_pids();
            let elapsed_ms = journal_start.elapsed().as_millis();
            if remaining.is_empty() {
                crate::modules::logger::log_info(&format!(
                    "[CLOSE_JOURNAL] instance='{}' stage=done confirmed_dead=true elapsed_ms={}",
                    instance_id, elapsed_ms
                ));
                invalidate_instance_process_cache(instance_id);
                return Ok(true);
            }
            // Log the first poll immediately, then at most once per second to bound output.
            if first_poll || last_poll_log.elapsed() >= std::time::Duration::from_secs(1) {
                crate::modules::logger::log_info(&format!(
                    "[CLOSE_JOURNAL] instance='{}' stage=verify_poll elapsed_ms={} remaining={:?}",
                    instance_id, elapsed_ms, remaining
                ));
                last_poll_log = std::time::Instant::now();
                first_poll = false;
            }
            if std::time::Instant::now() >= deadline {
                if attempt == 0 {
                    crate::modules::logger::log_warn(&format!(
                        "[Instance] PIDs {:?} for instance '{}' survived close; retrying force-close once",
                        remaining, instance_id
                    ));
                }
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
    }

    let remaining = live_pids();
    crate::modules::logger::log_info(&format!(
        "[CLOSE_JOURNAL] instance='{}' stage=done confirmed_dead=false elapsed_ms={} survivors={:?}",
        instance_id,
        journal_start.elapsed().as_millis(),
        remaining
    ));
    crate::modules::logger::log_warn(&format!(
        "[Instance] PIDs {:?} for instance '{}' survived verified close; proceeding anyway — relaunch will replace the session",
        remaining, instance_id
    ));
    invalidate_instance_process_cache(instance_id);
    Ok(false)
}

/// Close only the process associated with this instance
pub fn close_instance(instance_id: &str) -> Result<(), String> {
    let registry = load_registry()?;
    let config = registry
        .instances
        .iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance {} not found", instance_id))?;

    let mut system = System::new();
    let refresh_kind = sysinfo::ProcessRefreshKind::new()
        .with_cmd(sysinfo::UpdateKind::Always)
        .with_exe(sysinfo::UpdateKind::Always);
    system.refresh_processes_specifics(sysinfo::ProcessesToUpdate::All, refresh_kind);

    // 1. Gather all candidate PIDs for THIS specific instance only
    let is_default_inst = config.is_default || instance_id == "default";
    let mut pids = find_pids_for_data_dir(&config.data_dir, is_default_inst);
    let norm_data = config.data_dir.to_lowercase().replace('\\', "/");
    let clean_data = norm_data.trim_end_matches('/');
    let canonical_data = std::fs::canonicalize(&config.data_dir).ok().map(|p| {
        let s = p.to_string_lossy().to_lowercase().replace('\\', "/");
        s.trim_start_matches("//?/")
            .trim_start_matches(r"\\?\")
            .trim_end_matches('/')
            .to_string()
    });

    if let Some(saved_pid) = config.pid.or_else(|| get_instance_saved_pid(instance_id)) {
        if let Some(proc) = system.process(sysinfo::Pid::from_u32(saved_pid)) {
            let proc_name = proc.name().to_string_lossy().to_lowercase();
            let proc_exe = proc
                .exe()
                .map(|p| p.to_string_lossy().to_lowercase())
                .unwrap_or_default();
            let is_antigravity =
                proc_name.contains("antigravity") || proc_exe.contains("antigravity");
            if is_antigravity && !pids.contains(&saved_pid) {
                let args_str = proc
                    .cmd()
                    .iter()
                    .map(|a| a.to_string_lossy().to_lowercase().replace('\\', "/"))
                    .collect::<Vec<String>>()
                    .join(" ");
                let inst_id_lower = instance_id.to_lowercase();
                let is_data_match = args_str.contains(clean_data)
                    || canonical_data
                        .as_ref()
                        .map_or(false, |c| !c.is_empty() && args_str.contains(c));
                if is_default_inst
                    || is_data_match
                    || proc_exe.contains(&inst_id_lower)
                    || proc_name.contains(&inst_id_lower)
                {
                    pids.push(saved_pid);
                } else {
                    crate::modules::logger::log_warn(&format!(
                        "[Instance] Saved PID {} does not match data_dir '{}' (args: {}), skipping to protect running sessions",
                        saved_pid, config.data_dir, args_str
                    ));
                }
            }
        }
    }

    if !is_default_inst {
        let inst_id_lower = instance_id.to_lowercase();
        let inst_marker_slash = format!("instances/{}", inst_id_lower);
        let inst_marker_bslash = format!("instances\\{}", inst_id_lower);

        let matches_instance = |args: &str, exe_path: &str, p_name: &str| -> bool {
            args.contains(clean_data)
                || canonical_data
                    .as_ref()
                    .map_or(false, |c| !c.is_empty() && args.contains(c))
                || args.contains(&inst_marker_slash)
                || args.contains(&inst_marker_bslash)
                || exe_path.contains(&inst_id_lower)
                || p_name.contains(&inst_id_lower)
        };

        pids.retain(|&pid| {
            if let Some(proc) = system.process(sysinfo::Pid::from_u32(pid)) {
                let p_name = proc.name().to_string_lossy().to_lowercase();
                let p_exe = proc
                    .exe()
                    .map(|p| p.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                let args_str = proc
                    .cmd()
                    .iter()
                    .map(|a| a.to_string_lossy().to_lowercase().replace('\\', "/"))
                    .collect::<Vec<String>>()
                    .join(" ");

                if matches_instance(&args_str, &p_exe, &p_name) {
                    return true;
                }

                // Check ancestors in process tree
                let mut curr = pid;
                for _ in 0..10 {
                    if let Some(p) = system
                        .process(sysinfo::Pid::from_u32(curr))
                        .and_then(|pr| pr.parent())
                        .map(|pp| pp.as_u32())
                    {
                        if let Some(parent_proc) = system.process(sysinfo::Pid::from_u32(p)) {
                            let parent_args = parent_proc
                                .cmd()
                                .iter()
                                .map(|a| a.to_string_lossy().to_lowercase().replace('\\', "/"))
                                .collect::<Vec<String>>()
                                .join(" ");
                            let parent_name = parent_proc.name().to_string_lossy().to_lowercase();
                            let parent_exe = parent_proc
                                .exe()
                                .map(|p| p.to_string_lossy().to_lowercase())
                                .unwrap_or_default();
                            if matches_instance(&parent_args, &parent_exe, &parent_name) {
                                return true;
                            }
                        }
                        curr = p;
                    } else {
                        break;
                    }
                }

                crate::modules::logger::log_warn(&format!(
                    "[Instance] Safety filter: PID {} rejected from close list for instance '{}' because process arguments do not match instance data_dir",
                    pid, instance_id
                ));
                false
            } else {
                false
            }
        });
    }

    // Justification: intentionally discards the stopped-worker count; the function is infallible and workers are re-stopped on the next stop pass
    let _ = crate::modules::repo_db::stop_prompt_goal_workers_for_instance(
        instance_id,
        &config.data_dir,
    );

    let (protected_pids, markers) = other_instance_protection(instance_id);
    pids.retain(|&pid| {
        let Some(proc) = system.process(sysinfo::Pid::from_u32(pid)) else {
            return false;
        };
        let name = proc.name().to_string_lossy().to_lowercase();
        let exe = proc
            .exe()
            .map(|p| p.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let args_str = proc
            .cmd()
            .iter()
            .map(|arg| arg.to_string_lossy().to_lowercase().replace('\\', "/"))
            .collect::<Vec<_>>()
            .join(" ");
        if should_spare_pid(pid, &args_str, &exe, &name, &protected_pids, &markers) {
            crate::modules::logger::log_warn(&format!(
                "[Instance] Leaving PID {} running; it belongs to another instance",
                pid
            ));
            return false;
        }
        let is_editor_or_manager = name.contains("cursor")
            || exe.contains("cursor")
            || name.contains("agm")
            || exe.contains("agm-alim");
        if is_editor_or_manager {
            crate::modules::logger::log_warn(&format!(
                "[Instance] Refusing to close protected process PID {} ({})",
                pid, name
            ));
            return false;
        }
        true
    });

    if pids.is_empty() {
        // Justification: registry flag update; live state is re-derived from the OS on next refresh
        crate::error::record_ignored(mark_instance_stopped(instance_id), "mark instance stopped");
        return Ok(());
    }

    crate::modules::logger::log_info(&format!(
        "[Instance] Terminating {} process(es) for instance '{}' (PIDs: {:?})",
        pids.len(),
        instance_id,
        pids
    ));

    for pid in &pids {
        #[cfg(target_os = "windows")]
        {
            // Justification: process termination is best-effort; the liveness check re-verifies
            crate::error::record_ignored(
                Command::new("taskkill")
                    .args(["/F", "/T", "/PID", &pid.to_string()])
                    .creation_flags(0x08000000)
                    .output(),
                "taskkill process tree",
            );
        }

        #[cfg(not(target_os = "windows"))]
        {
            // Justification: process termination is best-effort; the liveness check re-verifies
            crate::error::record_ignored(
                Command::new("kill")
                    .args(["-15", &pid.to_string()])
                    .output(),
                "kill process",
            );
        }
    }

    // Synchronously wait for processes to exit to prevent SQLite locks and state overwrite
    let start_wait = std::time::Instant::now();
    let max_graceful = std::time::Duration::from_millis(3000);

    let target_pids: Vec<sysinfo::Pid> = pids.iter().map(|&p| sysinfo::Pid::from_u32(p)).collect();

    loop {
        system.refresh_processes_specifics(
            sysinfo::ProcessesToUpdate::Some(&target_pids),
            sysinfo::ProcessRefreshKind::new(),
        );
        let has_alive = target_pids.iter().any(|&pid| system.process(pid).is_some());

        if !has_alive {
            crate::modules::logger::log_info(&format!(
                "[Instance] Successfully closed all processes for instance '{}'",
                instance_id
            ));
            break;
        }

        if start_wait.elapsed() > max_graceful {
            #[cfg(not(target_os = "windows"))]
            {
                crate::modules::logger::log_warn(&format!(
                    "[Instance] Graceful exit timed out for instance '{}', sending SIGKILL to remaining processes...",
                    instance_id
                ));
                for pid in &pids {
                    if system.process(sysinfo::Pid::from_u32(*pid)).is_some() {
                        // Justification: process termination is best-effort; the liveness check re-verifies
                        crate::error::record_ignored(
                            Command::new("kill").args(["-9", &pid.to_string()]).output(),
                            "kill process",
                        );
                    }
                }
                std::thread::sleep(std::time::Duration::from_millis(300));
            }
            break;
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // Clean any orphaned lock files in data_dir
    let target_data_path = PathBuf::from(&config.data_dir);
    let dt_port = target_data_path.join("DevToolsActivePort");
    if dt_port.exists() {
        // Justification: cleanup of an optional file; absence is the normal case
        crate::error::record_ignored(fs::remove_file(&dt_port), "remove file");
    }
    clear_stale_instance_lockfiles(&target_data_path);

    // Justification: registry flag update; live state is re-derived from the OS on next refresh
    crate::error::record_ignored(mark_instance_stopped(instance_id), "mark instance stopped");
    invalidate_instance_process_cache(instance_id);
    crate::modules::repo_db::invalidate_prompt_tree_cache(Some(instance_id));

    // Small settle delay to ensure OS flushes file handles and SQLite locks
    std::thread::sleep(std::time::Duration::from_millis(150));

    Ok(())
}
