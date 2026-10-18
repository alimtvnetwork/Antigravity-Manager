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
    let remove_lock_entry = |path: &Path| {
        if path.exists() {
            #[cfg(target_os = "windows")]
            {
                if let Ok(metadata) = fs::metadata(path) {
                    let mut permissions = metadata.permissions();
                    if permissions.readonly() {
                        permissions.set_readonly(false);
                        // Justification: clearing readonly flag to allow lockfile removal
                        crate::error::record_ignored(
                            fs::set_permissions(path, permissions),
                            "clear readonly permission",
                        );
                    }
                }
            }
            if path.is_dir() {
                // Justification: cleanup of an optional lock directory; absence is the normal case
                crate::error::record_ignored(fs::remove_dir_all(path), "remove lock directory");
            } else {
                // Justification: cleanup of an optional lock file; absence is the normal case
                crate::error::record_ignored(fs::remove_file(path), "remove file");
            }
        }
    };

    remove_lock_entry(&target_data_path.join("lockfile"));
    remove_lock_entry(&target_data_path.join("code.lock"));
    remove_lock_entry(&target_data_path.join("DevToolsActivePort"));

    if let Ok(entries) = fs::read_dir(target_data_path) {
        for entry in entries.flatten() {
            let path = entry.path();
            let fname = entry.file_name().to_string_lossy().to_lowercase();
            let is_stale_lock = fname == "lockfile"
                || fname.starts_with("singleton")
                || fname.ends_with(".lock")
                || fname == "code.lock"
                || fname == "devtoolsactiveport";
            if is_stale_lock {
                remove_lock_entry(&path);
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn get_8_3_short_path(path: &str) -> Option<String> {
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
fn get_8_3_short_path(_path: &str) -> Option<String> {
    None
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
    current_pid: Option<u32>,
) -> bool {
    let id = instance_id.to_lowercase();
    let exe_l = exe.to_lowercase().replace('\\', "/");
    let name_l = name.to_lowercase();
    let args = args_str.to_lowercase().replace('\\', "/");
    if name_l.contains("cursor")
        || exe_l.contains("cursor")
        || name_l.contains("agm")
        || exe_l.contains("agm-alim")
        || name_l.contains("antigravity-manager")
        || exe_l.contains("antigravity-manager")
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
    if !clean_data.is_empty()
        && (args.contains(clean_data) || args.contains(&clean_data.replace('/', "\\")))
    {
        return true;
    }
    if let Some(canonical) = canonical_data {
        let canon = canonical.to_lowercase();
        if !canon.is_empty() && (args.contains(&canon) || args.contains(&canon.replace('/', "\\")))
        {
            return true;
        }
    }
    if let Some(short) = get_8_3_short_path(clean_data) {
        if !short.is_empty() && (args.contains(&short) || args.contains(&short.replace('/', "\\")))
        {
            return true;
        }
    }
    if clean_data.contains("administrator") {
        let alt = clean_data.replace("administrator", "admini~1");
        if args.contains(&alt) || args.contains(&alt.replace('/', "\\")) {
            return true;
        }
    } else if clean_data.contains("admini~1") {
        let alt = clean_data.replace("admini~1", "administrator");
        if args.contains(&alt) || args.contains(&alt.replace('/', "\\")) {
            return true;
        }
    }
    let marker = format!("antigravity-{id}");
    if exe_l.contains(&marker)
        || name_l.contains(&marker)
        || args.contains(&format!("/instances/{id}"))
        || args.contains(&format!("\\instances\\{id}"))
    {
        return true;
    }
    if is_default || id == "default" {
        let file = exe_l.rsplit('/').next().unwrap_or(&exe_l);
        if file == "antigravity.exe" {
            return true;
        }
    }

    #[cfg(target_os = "windows")]
    {
        let is_antigravity = name_l.contains("antigravity")
            || exe_l.contains("antigravity")
            || exe_l.ends_with("/apprun")
            || name_l == "apprun"
            || name_l.starts_with("code");

        if (args.is_empty() || !args.contains("--user-data-dir")) && is_antigravity {
            let recorded_pid = get_instance_saved_pid(instance_id).or_else(|| {
                load_registry().ok().and_then(|r| {
                    r.instances
                        .into_iter()
                        .find(|i| i.id == instance_id)
                        .and_then(|i| i.pid)
                })
            });

            let pid_matches = match (current_pid, recorded_pid) {
                (Some(c), Some(r)) => c == r,
                (None, Some(_)) => true,
                _ => false,
            };

            let in_cache_or_db = current_pid.map_or(false, |cpid| {
                let in_cache = INSTANCE_PROCESS_CACHE.read().ok().map_or(false, |cache| {
                    cache.get(instance_id).map_or(false, |rec| {
                        rec.pid == cpid
                            || rec.primary_pid == Some(cpid)
                            || rec.pids.contains(&cpid)
                    })
                });
                if in_cache {
                    return true;
                }
                open_instance_db().ok().map_or(false, |conn| {
                    conn.query_row(
                        "SELECT 1 FROM instance_processes WHERE instance_id = ?1 AND pid = ?2 AND is_active = 1",
                        rusqlite::params![instance_id, cpid as i64],
                        |_| Ok(true),
                    )
                    .unwrap_or(false)
                })
            });

            if pid_matches || in_cache_or_db {
                let target_pid = current_pid.or(recorded_pid).unwrap_or(0);
                let (protected_pids, markers) = other_instance_protection(instance_id);
                if !should_spare_pid(
                    target_pid,
                    &args,
                    &exe_l,
                    &name_l,
                    &protected_pids,
                    &markers,
                ) {
                    return true;
                }
            }
        }
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
            .filter(|&pid| is_pid_active_checked(pid))
            .collect::<Vec<u32>>();
        if let Some(pid) = saved_pid {
            if is_pid_active_checked(pid) && !pids.contains(&pid) {
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
                        Some(pid),
                    ) {
                        pids.push(pid);
                    }
                }
            }
        }
        if pids.is_empty() {
            let is_default = instance_id == "default" || instance_id == "__default__";
            let wpid = find_pids_for_data_dir(&data_dir, is_default)
                .into_iter()
                .find(|&pid| is_pid_active_checked(pid) && saved_pid_matches(pid));
            if let Some(wpid) = wpid {
                if !pids.contains(&wpid) {
                    pids.push(wpid);
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
        if attempt > 0 && !targeted.is_empty() {
            let mut kill_audit = crate::modules::task_history_db::AuditTask::start(
                crate::modules::audit_action::AuditAction::KillInstance,
                &format!("Force kill stragglers for instance {}", instance_id),
                Some(instance_id),
            );
            kill_audit.succeed(&format!(
                "Dispatched force-close pass for instance '{}' (PIDs: {:?})",
                instance_id, targeted
            ));
        }
        // close_instance only errors on registry/instance problems (both
        // validated above); the kill itself is best-effort per pass and the
        // verification poll below is what decides success.
        if let Err(e) = close_instance(instance_id) {
            crate::modules::logger::log_warn(&format!(
                "[CLOSE_JOURNAL] instance='{}' stage=kill_error attempt={} error='{}'",
                instance_id, attempt, e
            ));
        }
        let effective_timeout = (timeout / 2).max(std::time::Duration::from_millis(2500));
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
                crate::modules::repo_db::invalidate_prompt_tree_cache(Some(instance_id));
                let target_data_path = PathBuf::from(&data_dir);
                clear_stale_instance_lockfiles(&target_data_path);
                // Settle delay allowing Windows kernel & NTFS file handles to release
                std::thread::sleep(std::time::Duration::from_millis(200));
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
    crate::modules::repo_db::invalidate_prompt_tree_cache(Some(instance_id));
    let target_data_path = PathBuf::from(&data_dir);
    clear_stale_instance_lockfiles(&target_data_path);
    std::thread::sleep(std::time::Duration::from_millis(200));
    Ok(false)
}

#[cfg(target_os = "windows")]
pub fn force_terminate_pid_win32(pid: u32) -> bool {
    #[link(name = "kernel32")]
    extern "system" {
        fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut std::ffi::c_void;
        fn TerminateProcess(handle: *mut std::ffi::c_void, exit_code: u32) -> i32;
        fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
    }
    const PROCESS_TERMINATE: u32 = 0x0001;
    unsafe {
        let handle = OpenProcess(PROCESS_TERMINATE, 0, pid);
        if handle.is_null() {
            return false;
        }
        let ok = TerminateProcess(handle, 1);
        CloseHandle(handle);
        ok != 0
    }
}

#[cfg(target_os = "windows")]
pub fn is_pid_active_win32(pid: u32) -> bool {
    #[link(name = "kernel32")]
    extern "system" {
        fn OpenProcess(access: u32, inherit: i32, pid: u32) -> *mut std::ffi::c_void;
        fn GetExitCodeProcess(handle: *mut std::ffi::c_void, code: *mut u32) -> i32;
        fn CloseHandle(handle: *mut std::ffi::c_void) -> i32;
    }
    const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
    const STILL_ACTIVE: u32 = 259;
    if pid == 0 {
        return false;
    }
    unsafe {
        let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if handle.is_null() {
            return false;
        }
        let mut code = 0u32;
        let ok = GetExitCodeProcess(handle, &mut code);
        CloseHandle(handle);
        ok != 0 && code == STILL_ACTIVE
    }
}

#[cfg(not(target_os = "windows"))]
pub fn is_pid_active_win32(pid: u32) -> bool {
    is_pid_alive_targeted(pid)
}

#[inline]
pub(crate) fn is_pid_active_checked(pid: u32) -> bool {
    #[cfg(test)]
    if let Ok(guard) = MOCK_PID_ALIVE.read() {
        if let Some(set) = guard.as_ref() {
            return set.contains(&pid);
        }
    }
    is_pid_active_win32(pid)
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

    let mut audit = crate::modules::task_history_db::AuditTask::start(
        crate::modules::audit_action::AuditAction::StopInstance,
        &format!("Stop instance {}", instance_id),
        Some(instance_id),
    );

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
    let short_data = get_8_3_short_path(&config.data_dir);
    let admin_alt = if clean_data.contains("administrator") {
        Some(clean_data.replace("administrator", "admini~1"))
    } else if clean_data.contains("admini~1") {
        Some(clean_data.replace("admini~1", "administrator"))
    } else {
        None
    };

    let (protected_pids, markers) = other_instance_protection(instance_id);
    let saved_pid_opt = config.pid.or_else(|| get_instance_saved_pid(instance_id));
    if let Some(saved_pid) = saved_pid_opt {
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
                let is_protected = should_spare_pid(
                    saved_pid,
                    &args_str,
                    &proc_exe,
                    &proc_name,
                    &protected_pids,
                    &markers,
                );
                if !is_protected {
                    pids.push(saved_pid);
                } else {
                    crate::modules::logger::log_warn(&format!(
                        "[Instance] Saved PID {} matches another instance's protected markers, skipping to protect other sessions",
                        saved_pid
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
            let norm_args = args.to_lowercase().replace('\\', "/");
            let norm_exe = exe_path.to_lowercase().replace('\\', "/");
            let norm_name = p_name.to_lowercase();

            norm_args.contains(clean_data)
                || canonical_data.as_ref().map_or(false, |c| {
                    !c.is_empty() && (norm_args.contains(c) || args.contains(&c.replace('/', "\\")))
                })
                || short_data.as_ref().map_or(false, |s| {
                    !s.is_empty() && (norm_args.contains(s) || args.contains(&s.replace('/', "\\")))
                })
                || admin_alt.as_ref().map_or(false, |a| {
                    !a.is_empty() && (norm_args.contains(a) || args.contains(&a.replace('/', "\\")))
                })
                || norm_args.contains(&inst_marker_slash)
                || norm_args.contains(&inst_marker_bslash)
                || args.contains(&inst_marker_slash)
                || args.contains(&inst_marker_bslash)
                || norm_exe.contains(&inst_id_lower)
                || norm_name.contains(&inst_id_lower)
        };

        pids.retain(|&pid| {
            if Some(pid) == saved_pid_opt {
                return true;
            }
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

                #[cfg(target_os = "windows")]
                {
                    if proc.cmd().is_empty() {
                        let configured = config
                            .executable_path
                            .as_deref()
                            .unwrap_or("")
                            .to_lowercase()
                            .replace('\\', "/");
                        let exe_matches = if !configured.is_empty() {
                            let file = configured.rsplit('/').next().unwrap_or(&configured);
                            p_exe == configured
                                || (!file.is_empty()
                                    && (p_exe.ends_with(&format!("/{file}"))
                                        || p_exe.ends_with(file)))
                        } else {
                            p_exe.ends_with("antigravity.exe")
                                || p_name == "antigravity.exe"
                                || p_name.contains("antigravity")
                        };

                        let in_process_cache =
                            INSTANCE_PROCESS_CACHE.read().ok().map_or(false, |cache| {
                                cache.get(instance_id).map_or(false, |rec| {
                                    rec.pid == pid
                                        || rec.primary_pid == Some(pid)
                                        || rec.pids.contains(&pid)
                                })
                            });

                        let in_db = open_instance_db().ok().map_or(false, |conn| {
                            conn.query_row(
                                "SELECT 1 FROM instance_processes WHERE instance_id = ?1 AND pid = ?2 AND is_active = 1",
                                rusqlite::params![instance_id, pid as i64],
                                |_| Ok(true),
                            )
                            .unwrap_or(false)
                        });

                        if exe_matches || in_process_cache || in_db {
                            return true;
                        }
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
            Some(pid),
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
        let fallback_pid = config.pid.or_else(|| get_instance_saved_pid(instance_id));
        if let Some(fpid) = fallback_pid {
            if is_pid_alive_targeted(fpid) {
                let mut f_sys = System::new();
                f_sys.refresh_processes_specifics(
                    sysinfo::ProcessesToUpdate::Some(&[sysinfo::Pid::from_u32(fpid)]),
                    sysinfo::ProcessRefreshKind::new()
                        .with_cmd(sysinfo::UpdateKind::Always)
                        .with_exe(sysinfo::UpdateKind::Always),
                );
                if let Some(proc) = f_sys.process(sysinfo::Pid::from_u32(fpid)) {
                    let name = proc.name().to_string_lossy().to_lowercase();
                    let exe = proc
                        .exe()
                        .map(|p| p.to_string_lossy().to_lowercase())
                        .unwrap_or_default();
                    let is_editor_or_manager = name.contains("cursor")
                        || exe.contains("cursor")
                        || name.contains("agm")
                        || exe.contains("agm-alim")
                        || name.contains("antigravity-manager")
                        || exe.contains("antigravity-manager");
                    if !is_editor_or_manager {
                        crate::modules::logger::log_warn(&format!(
                            "[Instance] Conscious kill: candidate list was empty but saved/cached PID {} for instance '{}' is alive. Enqueuing for tree termination.",
                            fpid, instance_id
                        ));
                        pids.push(fpid);
                    }
                }
            }
        }
    }

    if pids.is_empty() {
        let is_default = is_default_inst;
        let wpid = find_pids_for_data_dir(&config.data_dir, is_default)
            .into_iter()
            .find(|&pid| is_pid_alive_targeted(pid) && saved_pid_matches(pid));
        if let Some(wpid) = wpid {
            if !pids.contains(&wpid) {
                crate::modules::logger::log_warn(&format!(
                    "[Instance] Window kill: candidate list was empty but window PID {} for instance '{}' is alive. Enqueuing for tree termination.",
                    wpid, instance_id
                ));
                pids.push(wpid);
            }
        }
    }

    if pids.is_empty() {
        let target_data_path = PathBuf::from(&config.data_dir);
        clear_stale_instance_lockfiles(&target_data_path);
        // Justification: registry flag update; live state is re-derived from the OS on next refresh
        crate::error::record_ignored(mark_instance_stopped(instance_id), "mark instance stopped");
        audit.succeed(&format!(
            "Instance {} was already stopped (no active PIDs)",
            instance_id
        ));
        std::thread::sleep(std::time::Duration::from_millis(200));
        return Ok(());
    }

    // Ensure descendant child processes (renderers, helpers, utility) are gathered and terminated
    let root_pids_set: std::collections::HashSet<u32> = pids.iter().copied().collect();
    let mut descendant_pids = Vec::new();
    for (spid, sproc) in system.processes() {
        let cand_pid = spid.as_u32();
        if root_pids_set.contains(&cand_pid) {
            continue;
        }
        let p_name = sproc.name().to_string_lossy().to_lowercase();
        let p_exe = sproc
            .exe()
            .map(|p| p.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let is_editor_or_manager = p_name.contains("cursor")
            || p_exe.contains("cursor")
            || p_name.contains("agm")
            || p_exe.contains("agm-alim")
            || p_name.contains("antigravity-manager")
            || p_exe.contains("antigravity-manager");
        if is_editor_or_manager {
            continue;
        }

        let mut curr = cand_pid;
        for _ in 0..10 {
            if let Some(parent) = system
                .process(sysinfo::Pid::from_u32(curr))
                .and_then(|pr| pr.parent())
                .map(|pp| pp.as_u32())
            {
                if root_pids_set.contains(&parent) {
                    descendant_pids.push(cand_pid);
                    break;
                }
                curr = parent;
            } else {
                break;
            }
        }
    }
    for dpid in &descendant_pids {
        if !pids.contains(dpid) {
            crate::modules::logger::log_info(&format!(
                "[Instance] Including descendant child process PID {} (renderer/helper/utility) for termination",
                dpid
            ));
            pids.push(*dpid);
        }
    }

    crate::modules::logger::log_info(&format!(
        "[Instance] Terminating {} process(es) for instance '{}' (Root PIDs: {:?}, Descendants: {:?})",
        pids.len(),
        instance_id,
        root_pids_set,
        descendant_pids
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
    let max_graceful = std::time::Duration::from_millis(1500);

    loop {
        let has_alive = pids.iter().any(|&pid| is_pid_active_checked(pid));

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
                .filter(|&pid| is_pid_active_checked(pid))
                .collect();

            if !still.is_empty() {
                crate::modules::logger::log_warn(&format!(
                    "[Instance] Graceful termination pending for '{}', executing direct force kill on surviving PIDs: {:?}",
                    instance_id, still
                ));

                let mut kill_audit = crate::modules::task_history_db::AuditTask::start(
                    crate::modules::audit_action::AuditAction::KillInstance,
                    &format!("Direct kernel kill for instance {}", instance_id),
                    Some(instance_id),
                );
                kill_audit.succeed(&format!(
                    "Executed direct kernel force kill on lingering PIDs: {:?}",
                    still
                ));

                for pid in &still {
                    #[cfg(target_os = "windows")]
                    {
                        force_terminate_pid_win32(*pid);
                        if is_pid_active_checked(*pid) {
                            // Justification: powershell Stop-Process fallback when Win32 TerminateProcess fails
                            crate::error::record_ignored(
                                Command::new("powershell")
                                    .args([
                                        "-NoProfile",
                                        "-NonInteractive",
                                        "-Command",
                                        &format!("Stop-Process -Id {} -Force", pid),
                                    ])
                                    .creation_flags(0x08000000)
                                    .output(),
                                "powershell Stop-Process fallback",
                            );
                        }
                    }
                    #[cfg(not(target_os = "windows"))]
                    {
                        crate::error::record_ignored(
                            Command::new("kill").args(["-9", &pid.to_string()]).output(),
                            "kill -9 fallback",
                        );
                    }
                }

                // Stage 3: Bounded wait polling (up to 1000ms additional, 50ms interval)
                let force_wait_start = std::time::Instant::now();
                while force_wait_start.elapsed() < std::time::Duration::from_millis(1000) {
                    if !still.iter().any(|&pid| is_pid_active_checked(pid)) {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
            }

            let lingering: Vec<u32> = pids
                .iter()
                .copied()
                .filter(|&pid| is_pid_active_checked(pid))
                .collect();

            if !lingering.is_empty() {
                crate::modules::logger::log_warn(&format!(
                    "[CLOSE_JOURNAL] instance='{}' stage=lingering_stragglers pids={:?}",
                    instance_id, lingering
                ));
            }
            break;
        }

        std::thread::sleep(std::time::Duration::from_millis(50));
    }

    // Clean any orphaned lock files in data_dir
    let target_data_path = PathBuf::from(&config.data_dir);
    let dt_port = target_data_path.join("DevToolsActivePort");
    if dt_port.exists() {
        // Justification: cleanup of an optional file; absence is the normal case
        crate::error::record_ignored(fs::remove_file(&dt_port), "remove file");
    }
    clear_stale_instance_lockfiles(&target_data_path);
    crate::modules::process::sweep_orphan_language_servers();

    // Justification: registry flag update; live state is re-derived from the OS on next refresh
    crate::error::record_ignored(mark_instance_stopped(instance_id), "mark instance stopped");
    invalidate_instance_process_cache(instance_id);
    crate::modules::repo_db::invalidate_prompt_tree_cache(Some(instance_id));

    // Mandatory 200ms kernel settle delay allowing Windows kernel & NTFS file handles to release
    std::thread::sleep(std::time::Duration::from_millis(200));

    let payload = serde_json::json!({
        "instance_id": instance_id,
        "terminated_pids": pids,
        "data_dir": config.data_dir,
    })
    .to_string();
    audit.succeed_with_payload(
        &format!("Instance {} closed successfully", instance_id),
        &payload,
    );

    Ok(())
}

/// Force kill all processes associated with this instance
pub fn kill_instance(instance_id: &str) -> Result<(), String> {
    refuse_if_caller_instance(instance_id)?;
    force_refresh_process_cache();
    let registry = load_registry()?;
    let config = registry
        .instances
        .iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| format!("Instance {} not found", instance_id))?;

    let mut audit = crate::modules::task_history_db::AuditTask::start(
        crate::modules::audit_action::AuditAction::KillInstance,
        &format!("Kill instance {}", instance_id),
        Some(instance_id),
    );

    let is_default_inst = config.is_default || instance_id == "default";
    let mut pids = find_pids_for_data_dir(&config.data_dir, is_default_inst);
    if let Some(saved) = config.pid.or_else(|| get_instance_saved_pid(instance_id)) {
        if is_pid_alive_targeted(saved) && !pids.contains(&saved) {
            pids.push(saved);
        }
    }

    crate::modules::logger::log_info(&format!(
        "[Instance] Force killing {} process(es) for instance '{}' (PIDs: {:?})",
        pids.len(),
        instance_id,
        pids
    ));

    for pid in &pids {
        #[cfg(target_os = "windows")]
        {
            crate::error::record_ignored(
                Command::new("taskkill")
                    .args(["/F", "/T", "/PID", &pid.to_string()])
                    .creation_flags(0x08000000)
                    .output(),
                "taskkill process tree",
            );
            force_terminate_pid_win32(*pid);
            if is_pid_active_checked(*pid) {
                // Justification: powershell Stop-Process fallback when Win32 TerminateProcess fails
                crate::error::record_ignored(
                    Command::new("powershell")
                        .args([
                            "-NoProfile",
                            "-NonInteractive",
                            "-Command",
                            &format!("Stop-Process -Id {} -Force", pid),
                        ])
                        .creation_flags(0x08000000)
                        .output(),
                    "powershell Stop-Process fallback",
                );
            }
        }
        #[cfg(not(target_os = "windows"))]
        {
            crate::error::record_ignored(
                Command::new("kill").args(["-9", &pid.to_string()]).output(),
                "kill process",
            );
        }
    }

    let target_data_path = PathBuf::from(&config.data_dir);
    clear_stale_instance_lockfiles(&target_data_path);
    crate::error::record_ignored(mark_instance_stopped(instance_id), "mark instance stopped");
    invalidate_instance_process_cache(instance_id);
    crate::modules::repo_db::invalidate_prompt_tree_cache(Some(instance_id));
    std::thread::sleep(std::time::Duration::from_millis(200));

    let payload = serde_json::json!({
        "instance_id": instance_id,
        "killed_pids": pids,
        "data_dir": config.data_dir,
    })
    .to_string();
    audit.succeed_with_payload(&format!("Instance {} killed", instance_id), &payload);

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
    if norm1.replace("admini~1", "administrator") == norm2.replace("admini~1", "administrator") {
        return true;
    }
    if let (Some(s1), Some(s2)) = (get_8_3_short_path(arg_dir), get_8_3_short_path(inst_dir)) {
        if s1 == s2 {
            return true;
        }
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
    if let Ok(dir) = std::env::var("ANTIGRAVITY_APP_DATA_DIR") {
        let trimmed = dir.trim();
        if !trimmed.is_empty() {
            if let Ok(ref reg) = load_registry() {
                for inst in &reg.instances {
                    if path_matches_dir(&trimmed, &inst.data_dir) {
                        return Some(inst.id.clone());
                    }
                }
            }
        }
    }
    if let Ok(dir) = std::env::var("ANTIGRAVITY_APP_DIR") {
        let trimmed = dir.trim();
        if !trimmed.is_empty() {
            if let Ok(ref reg) = load_registry() {
                for inst in &reg.instances {
                    if path_matches_dir(&trimmed, &inst.data_dir) {
                        return Some(inst.id.clone());
                    }
                }
            }
        }
    }
    if let Ok(vpid_str) = std::env::var("VSCODE_PID") {
        if let Ok(vpid) = vpid_str.trim().parse::<u32>() {
            if let Ok(ref reg) = load_registry() {
                for inst in &reg.instances {
                    if inst.pid == Some(vpid) {
                        return Some(inst.id.clone());
                    }
                }
            }
            if let Ok(conn) = open_instance_db() {
                if let Ok(inst_id) = conn.query_row(
                    "SELECT instance_id FROM instance_processes WHERE pid = ?1 AND is_active = 1",
                    rusqlite::params![vpid as i64],
                    |row| row.get::<_, String>(0),
                ) {
                    return Some(inst_id);
                }
            }
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

        // Check if process exe matches antigravity.exe, antigravity, or cloned binaries
        let exe_name = proc
            .exe()
            .and_then(|p| p.file_name())
            .map(|f| f.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        let proc_name = proc.name().to_string_lossy().to_lowercase();
        let is_manager = exe_name.starts_with("antigravity-manager")
            || proc_name.starts_with("antigravity-manager")
            || exe_name.contains("agm")
            || proc_name.contains("agm")
            || exe_name.contains("cursor")
            || proc_name.contains("cursor");
        let is_antigravity = !is_manager
            && (exe_name.starts_with("antigravity") || proc_name.starts_with("antigravity"));

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
                if let Some(ref reg) = registry {
                    for inst in &reg.instances {
                        let marker = format!("antigravity-{}", inst.id.to_lowercase());
                        if exe_name.contains(&marker) || proc_name.contains(&marker) {
                            return Some(inst.id.clone());
                        }
                    }
                }
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
    if std::env::var("AGM_FORCE_CALLER_OVERRIDE")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        return Ok(());
    }
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
