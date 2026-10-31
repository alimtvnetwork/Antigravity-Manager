//! Imp-66: Native Conscious PID Verification & Clean Termination Subsystem
//! Prevents accidentally terminating caller IDEs, host developer terminals, or unrelated background processes.

use crate::error::AppError;
use sysinfo::System;

/// Verifies that a PID genuinely belongs to the Antigravity IDE process family and is NOT an AGM process or developer tooling.
pub fn is_verified_antigravity_pid(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    let mut sys = System::new();
    let target_pid = sysinfo::Pid::from_u32(pid);
    sys.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::Some(&[target_pid]),
        sysinfo::ProcessRefreshKind::new()
            .with_cmd(sysinfo::UpdateKind::Always)
            .with_exe(sysinfo::UpdateKind::Always),
    );

    let Some(proc) = sys.process(target_pid) else {
        return false;
    };

    let name = proc.name().to_string_lossy().to_lowercase();
    let exe = proc
        .exe()
        .map(|p| p.to_string_lossy().to_lowercase().replace('\\', "/"))
        .unwrap_or_default();

    if name.contains("agm")
        || exe.contains("agm")
        || name.contains("webview")
        || exe.contains("webview")
    {
        return false;
    }

    if crate::modules::process::is_non_ide_binary(&name, &exe, "") {
        return false;
    }

    name.contains("antigravity")
        || exe.ends_with("/antigravity.exe")
        || exe.ends_with("/antigravity")
        || exe.contains("/antigravity-")
}

/// Verifies whether `child_pid` is a descendant of `parent_pid` in the OS process hierarchy.
pub fn verify_parent_child_hierarchy(parent_pid: u32, child_pid: u32) -> bool {
    if parent_pid == 0 || child_pid == 0 || parent_pid == child_pid {
        return false;
    }
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All);

    let mut curr_pid = sysinfo::Pid::from_u32(child_pid);
    let target_parent = sysinfo::Pid::from_u32(parent_pid);

    for _ in 0..32 {
        if let Some(proc) = sys.process(curr_pid) {
            if let Some(parent) = proc.parent() {
                if parent == target_parent {
                    return true;
                }
                curr_pid = parent;
            } else {
                break;
            }
        } else {
            break;
        }
    }
    false
}

/// Safely terminates a process only after verifying it is a genuine Antigravity PID.
/// Respects caller IDE preservation.
pub fn safe_terminate_conscious_pid(pid: u32, force: bool) -> Result<bool, AppError> {
    if !is_verified_antigravity_pid(pid) {
        return Err(AppError::Process(format!(
            "PID {} rejected: Not a verified Antigravity process",
            pid
        )));
    }

    let mut sys = System::new();
    let target = sysinfo::Pid::from_u32(pid);
    sys.refresh_processes_specifics(
        sysinfo::ProcessesToUpdate::Some(&[target]),
        sysinfo::ProcessRefreshKind::new(),
    );

    if let Some(proc) = sys.process(target) {
        if force {
            let killed = proc.kill_with(sysinfo::Signal::Kill).unwrap_or(false);
            Ok(killed)
        } else {
            let terminated = proc.kill_with(sysinfo::Signal::Term).unwrap_or(false);
            Ok(terminated)
        }
    } else {
        Ok(true) // already terminated
    }
}
