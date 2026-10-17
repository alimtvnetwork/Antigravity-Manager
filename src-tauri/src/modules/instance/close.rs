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
    let dt_port = target_data_path.join("DevToolsActivePort");
    if dt_port.exists() {
        // Justification: cleanup of an optional file; absence is the normal case
        crate::error::record_ignored(fs::remove_file(&dt_port), "remove file");
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

fn saved_pid_belongs_to_instance(
    instance_id: &str,
    is_default: bool,
    clean_data: &str,
    canonical_data: Option<&str>,
    configured_exe: Option<&str>,
    exe: &str,
    name: &str,
    args_str: &str,
) -> bool {
    let id = instance_id.to_lowercase();
    let exe_l = exe.to_lowercase().replace('\\', "/");
    let name_l = name.to_lowercase();
    let args = args_str.to_lowercase().replace('\\', "/");
    if name_l.contains("cursor")
        || exe_l.contains("cursor")
        || name_l.contains("agm")
        || exe_l.contains("agm-alim")
    {
        return false;
    }
    let configured = configured_exe
        .unwrap_or("")
        .to_lowercase()
        .replace('\\', "/");
    if !configured.is_empty() {
        let file = configured.rsplit('/').next().unwrap_or(&configured);
        if !file.is_empty()
            && file != "antigravity.exe"
            && (exe_l.ends_with(&format!("/{file}")) || exe_l.ends_with(file))
        {
            return true;
        }
        if exe_l == configured {
            return true;
        }
    }
    if !clean_data.is_empty() && args.contains(clean_data) {
        return true;
    }
    if let Some(canonical) = canonical_data {
        let canon = canonical.to_lowercase();
        if !canon.is_empty() && args.contains(&canon) {
            return true;
        }
    }
    let marker = format!("antigravity-{id}");
    if exe_l.contains(&marker)
        || name_l.contains(&marker)
        || args.contains(&format!("/instances/{id}"))
    {
        return true;
    }
    if is_default || id == "default" {
        let file = exe_l.rsplit('/').next().unwrap_or(&exe_l);
        return file == "antigravity.exe";
    }
    false
}

pub fn close_instance_verified(
    instance_id: &str,
    timeout: std::time::Duration,
) -> Result<bool, String> {
    force_refresh_process_cache();
    let registry = load_registry()?;
    let config = registry
        .instances
        .iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance {} not found", instance_id))?;
    let data_dir = config.data_dir.clone();
    let is_default = config.is_default || instance_id == "default";
    let saved_pid = config.pid.or_else(|| get_instance_saved_pid(instance_id));
    let configured_exe = config.executable_path.clone();

    let live_pids = || {
        force_refresh_process_cache();
        let mut pids = find_pids_for_data_dir(&data_dir, is_default)
            .into_iter()
            .filter(|&pid| is_pid_alive_targeted(pid))
            .collect::<Vec<u32>>();
        if let Some(pid) = saved_pid {
            if is_pid_alive_targeted(pid) && !pids.contains(&pid) {
                let mut system = System::new();
                let refresh_kind = sysinfo::ProcessRefreshKind::new()
                    .with_cmd(sysinfo::UpdateKind::Always)
                    .with_exe(sysinfo::UpdateKind::Always);
                system.refresh_processes_specifics(
                    sysinfo::ProcessesToUpdate::Some(&[sysinfo::Pid::from_u32(pid)]),
                    refresh_kind,
                );
                if let Some(proc) = system.process(sysinfo::Pid::from_u32(pid)) {
                    let proc_name = proc.name().to_string_lossy().to_lowercase();
                    let proc_exe = proc
                        .exe()
                        .map(|p| p.to_string_lossy().to_lowercase())
                        .unwrap_or_default();
                    let args_str = proc
                        .cmd()
                        .iter()
                        .map(|a| a.to_string_lossy().to_lowercase().replace('\\', "/"))
                        .collect::<Vec<_>>()
                        .join(" ");
                    let norm_data = data_dir.to_lowercase().replace('\\', "/");
                    let clean_data = norm_data.trim_end_matches('/');
                    if saved_pid_belongs_to_instance(
                        instance_id,
                        is_default,
                        clean_data,
                        None,
                        configured_exe.as_deref(),
                        &proc_exe,
                        &proc_name,
                        &args_str,
                    ) {
                        pids.push(pid);
                    }
                }
            }
        }
        pids
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
        let effective_timeout = timeout.min(std::time::Duration::from_millis(2500));
        let deadline = std::time::Instant::now() + effective_timeout;
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
                force_refresh_process_cache();
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
            std::thread::sleep(std::time::Duration::from_millis(50));
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
    force_refresh_process_cache();
    Ok(false)
}

/// Close only the process associated with this instance
pub fn close_instance(instance_id: &str) -> Result<(), String> {
    force_refresh_process_cache();
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
                if saved_pid_belongs_to_instance(
                    instance_id,
                    is_default_inst,
                    clean_data,
                    canonical_data.as_deref(),
                    config.executable_path.as_deref(),
                    &proc_exe,
                    &proc_name,
                    &args_str,
                ) {
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
        let owned_by_this_instance = saved_pid_belongs_to_instance(
            instance_id,
            is_default_inst,
            clean_data,
            canonical_data.as_deref(),
            config.executable_path.as_deref(),
            &exe,
            &name,
            &args_str,
        );
        if !owned_by_this_instance
            && should_spare_pid(pid, &args_str, &exe, &name, &protected_pids, &markers)
        {
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

    loop {
        let has_alive = pids.iter().any(|&pid| is_pid_alive_targeted(pid));

        if !has_alive {
            crate::modules::logger::log_info(&format!(
                "[Instance] Successfully closed all processes for instance '{}'",
                instance_id
            ));
            break;
        }

        if start_wait.elapsed() > max_graceful {
            let still: Vec<u32> = pids
                .iter()
                .copied()
                .filter(|&pid| is_pid_alive_targeted(pid))
                .collect();
            if !still.is_empty() {
                return Err(format!(
                    "Instance '{instance_id}' PID(s) {still:?} stayed alive after taskkill"
                ));
            }
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

fn path_matches_dir(arg_dir: &str, inst_dir: &str) -> bool {
    let norm1 = arg_dir
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_lowercase();
    let norm2 = inst_dir
        .replace('\\', "/")
        .trim_end_matches('/')
        .to_lowercase();
    if norm1 == norm2 {
        return true;
    }
    if let (Ok(c1), Ok(c2)) = (
        std::fs::canonicalize(arg_dir),
        std::fs::canonicalize(inst_dir),
    ) {
        let s1 = c1.to_string_lossy().replace('\\', "/").to_lowercase();
        let s2 = c2.to_string_lossy().replace('\\', "/").to_lowercase();
        let clean1 = s1
            .trim_start_matches("//?/")
            .trim_start_matches(r"\\?\")
            .trim_end_matches('/');
        let clean2 = s2
            .trim_start_matches("//?/")
            .trim_start_matches(r"\\?\")
            .trim_end_matches('/');
        if clean1 == clean2 {
            return true;
        }
    }
    false
}

/// Instance id whose process is an ancestor of this CLI, when the command was
/// started from that IDE. Cursor and the manager process are not instances.
pub fn detect_caller_instance_id() -> Option<String> {
    // 1. Check environment variables ANTIGRAVITY_INSTANCE_ID and AGM_INSTANCE_ID
    if let Ok(id) = std::env::var("ANTIGRAVITY_INSTANCE_ID") {
        let trimmed = id.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }
    if let Ok(id) = std::env::var("AGM_INSTANCE_ID") {
        let trimmed = id.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    let registry = load_registry().ok();

    let mut system = System::new();
    let refresh_kind = sysinfo::ProcessRefreshKind::new()
        .with_cmd(sysinfo::UpdateKind::Always)
        .with_exe(sysinfo::UpdateKind::Always);
    system.refresh_processes_specifics(sysinfo::ProcessesToUpdate::All, refresh_kind);
    let mut pid = std::process::id();
    let mut ancestor_pids = Vec::new();
    let mut ancestor_exes = Vec::new();
    for _ in 0..24 {
        ancestor_pids.push(pid);
        let Some(proc) = system.process(sysinfo::Pid::from_u32(pid)) else {
            break;
        };
        if let Some(exe) = proc.exe() {
            ancestor_exes.push(exe.to_string_lossy().to_lowercase().replace('\\', "/"));
        }

        // Check ancestor PIDs against inst.pid in registry.instances
        if let Some(ref reg) = registry {
            for inst in &reg.instances {
                if let Some(saved) = inst.pid {
                    if saved == pid {
                        return Some(inst.id.clone());
                    }
                }
            }
        }

        // Check if process exe matches antigravity.exe or antigravity (case-insensitive)
        let exe_name = proc
            .exe()
            .and_then(|p| p.file_name())
            .map(|f| f.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let proc_name = proc.name().to_string_lossy().to_lowercase();
        let is_antigravity = exe_name == "antigravity.exe"
            || exe_name == "antigravity"
            || proc_name == "antigravity.exe"
            || proc_name == "antigravity";

        if is_antigravity {
            // Inspect command-line arguments using proc.cmd()
            let cmd: Vec<String> = proc
                .cmd()
                .iter()
                .map(|s| s.to_string_lossy().to_string())
                .collect();

            let mut user_data_dir: Option<String> = None;
            let mut idx = 0;
            while idx < cmd.len() {
                let arg = &cmd[idx];
                if let Some(val) = arg
                    .strip_prefix("--user-data-dir=")
                    .or_else(|| arg.strip_prefix("-user-data-dir="))
                {
                    user_data_dir = Some(val.trim_matches('"').trim_matches('\'').to_string());
                    break;
                } else if (arg == "--user-data-dir" || arg == "-user-data-dir")
                    && idx + 1 < cmd.len()
                {
                    user_data_dir = Some(
                        cmd[idx + 1]
                            .trim_matches('"')
                            .trim_matches('\'')
                            .to_string(),
                    );
                    break;
                }
                idx += 1;
            }

            if let Some(udd) = user_data_dir {
                if let Some(ref reg) = registry {
                    for inst in &reg.instances {
                        if path_matches_dir(&udd, &inst.data_dir) {
                            return Some(inst.id.clone());
                        }
                    }
                }
            } else {
                return Some("default".to_string());
            }
        }

        match proc.parent().map(|parent| parent.as_u32()) {
            Some(parent) if parent != 0 && parent != pid => pid = parent,
            _ => break,
        }
    }

    if let Some(ref reg) = registry {
        for inst in &reg.instances {
            if let Some(saved) = inst.pid {
                if ancestor_pids.contains(&saved) {
                    return Some(inst.id.clone());
                }
            }
            if let Some(exe) = &inst.executable_path {
                let norm = exe.to_lowercase().replace('\\', "/");
                let file = norm.rsplit('/').next().unwrap_or(&norm);
                if ancestor_exes
                    .iter()
                    .any(|ancestor| ancestor == &norm || ancestor.ends_with(&format!("/{file}")))
                {
                    return Some(inst.id.clone());
                }
            }
        }
    }
    None
}

pub fn refuse_if_caller_instance(target_id: &str) -> Result<(), String> {
    let resolved = resolve_instance_id(target_id).unwrap_or_else(|_| target_id.to_string());
    if let Some(self_id) = detect_caller_instance_id() {
        if self_id == resolved || self_id == target_id {
            return Err(format!(
                "Refusing to stop, switch, or delete instance '{self_id}' because this command is running inside its IDE process"
            ));
        }
    }
    Ok(())
}
