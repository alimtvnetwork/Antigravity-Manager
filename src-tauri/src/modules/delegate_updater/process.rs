use std::process::Command;

use super::*;

pub fn is_pid_alive(pid: u32) -> bool {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("tasklist.exe")
            .args(["/FI", &format!("PID eq {}", pid), "/NH"])
            .output();
        if let Ok(out) = output {
            let text = String::from_utf8_lossy(&out.stdout);
            let trimmed = text.trim();
            if trimmed.is_empty() || trimmed.to_lowercase().starts_with("info:") {
                return false;
            }
            trimmed.contains(&pid.to_string()) && !trimmed.contains("No tasks")
        } else {
            false
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let output = Command::new("kill").args(["-0", &pid.to_string()]).output();
        matches!(output, Ok(o) if o.status.success())
    }
}

pub fn force_kill_pid(pid: u32) {
    #[cfg(target_os = "windows")]
    {
        // Justification: best-effort process spawn; failure logged
        crate::error::record_ignored(
            Command::new("taskkill.exe")
                .args(["/F", "/PID", &pid.to_string()])
                .output(),
            "spawn taskkill.exe",
        );
    }
    #[cfg(not(target_os = "windows"))]
    {
        // Justification: best-effort process spawn; failure logged
        crate::error::record_ignored(
            Command::new("kill").args(["-9", &pid.to_string()]).output(),
            "spawn kill",
        );
    }
}

pub fn is_ui_process_running() -> bool {
    is_ui_process_running_excluding(0)
}

pub fn is_ui_process_running_excluding(exclude_pid: u32) -> bool {
    #[cfg(target_os = "windows")]
    {
        let process_names = ["agm-alim.exe", "antigravity-tools.exe"];
        for proc_name in process_names {
            let output = Command::new("tasklist.exe")
                .args(["/FI", &format!("IMAGENAME eq {}", proc_name), "/NH"])
                .output();
            if let Ok(out) = output {
                let text = String::from_utf8_lossy(&out.stdout);
                for line in text.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty()
                        || trimmed.to_lowercase().starts_with("info:")
                        || trimmed.contains("No tasks")
                    {
                        continue;
                    }
                    if trimmed.contains(proc_name) {
                        if exclude_pid > 0 && trimmed.contains(&exclude_pid.to_string()) {
                            continue;
                        }
                        return true;
                    }
                }
            }
        }
        false
    }
    #[cfg(not(target_os = "windows"))]
    {
        let output = Command::new("pgrep").args(["-f", "agm-alim"]).output();
        if let Ok(out) = output {
            let text = String::from_utf8_lossy(&out.stdout);
            for line in text.lines() {
                if let Ok(pid) = line.trim().parse::<u32>() {
                    if pid != exclude_pid && pid != std::process::id() {
                        return true;
                    }
                }
            }
        }
        false
    }
}

pub fn kill_other_ui_processes(exclude_pid: u32) {
    #[cfg(target_os = "windows")]
    {
        let filter = if exclude_pid > 0 {
            format!("PID ne {}", exclude_pid)
        } else {
            "PID gt 0".to_string()
        };
        // Justification: best-effort process spawn; failure logged
        crate::error::record_ignored(
            Command::new("taskkill.exe")
                .args(["/F", "/IM", "agm-alim.exe", "/FI", &filter])
                .output(),
            "spawn taskkill.exe",
        );
        // Justification: best-effort process spawn; failure logged
        crate::error::record_ignored(
            Command::new("taskkill.exe")
                .args(["/F", "/IM", "antigravity-tools.exe", "/FI", &filter])
                .output(),
            "spawn taskkill.exe",
        );
    }
    #[cfg(not(target_os = "windows"))]
    {
        // Justification: intentionally unused on non-Windows (only the Windows branch consumes it); suppresses the unused-variable warning.
        let _ = exclude_pid;
        // Justification: best-effort process spawn; failure logged
        crate::error::record_ignored(
            Command::new("pkill").args(["-f", "agm-alim"]).output(),
            "spawn pkill",
        );
    }
}
