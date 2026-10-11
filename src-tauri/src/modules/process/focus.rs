#[cfg(not(target_os = "windows"))]
use std::process::Command;

use super::*;

#[cfg(target_os = "windows")]
mod win32 {
    use std::ffi::c_void;

    pub type HWND = *mut c_void;
    pub type BOOL = i32;
    pub type LPARAM = isize;
    pub type WNDENUMPROC = unsafe extern "system" fn(HWND, LPARAM) -> BOOL;

    #[link(name = "user32")]
    extern "system" {
        pub fn SetForegroundWindow(hWnd: HWND) -> BOOL;
        pub fn ShowWindow(hWnd: HWND, nCmdShow: i32) -> BOOL;
        pub fn BringWindowToTop(hWnd: HWND) -> BOOL;
        pub fn SwitchToThisWindow(hWnd: HWND, fAltTab: BOOL);
        pub fn GetWindowTextW(hWnd: HWND, lpString: *mut u16, nMaxCount: i32) -> i32;
        pub fn GetWindowThreadProcessId(hWnd: HWND, lpdwProcessId: *mut u32) -> u32;
        pub fn EnumWindows(lpEnumFunc: WNDENUMPROC, lParam: LPARAM) -> BOOL;
        pub fn IsWindowVisible(hWnd: HWND) -> BOOL;
        pub fn IsIconic(hWnd: HWND) -> BOOL;
        pub fn OpenIcon(hWnd: HWND) -> BOOL;
        pub fn GetCurrentThreadId() -> u32;
        pub fn GetForegroundWindow() -> HWND;
        pub fn AttachThreadInput(idAttach: u32, idAttachTo: u32, fAttach: BOOL) -> BOOL;
    }

    pub unsafe fn activate_hwnd(hwnd: HWND) -> bool {
        if hwnd.is_null() {
            return false;
        }

        let is_minimized = IsIconic(hwnd) != 0;
        if is_minimized {
            OpenIcon(hwnd);
            ShowWindow(hwnd, 9); // SW_RESTORE
        } else {
            ShowWindow(hwnd, 5); // SW_SHOW
        }
        BringWindowToTop(hwnd);

        let fg_hwnd = GetForegroundWindow();
        let cur_thread_id = GetCurrentThreadId();
        let fg_thread_id = if !fg_hwnd.is_null() {
            GetWindowThreadProcessId(fg_hwnd, std::ptr::null_mut())
        } else {
            0
        };
        let target_thread_id = GetWindowThreadProcessId(hwnd, std::ptr::null_mut());

        let has_attached_fg = if fg_thread_id != 0 && fg_thread_id != cur_thread_id {
            AttachThreadInput(cur_thread_id, fg_thread_id, 1) != 0
        } else {
            false
        };
        let has_attached_target = if target_thread_id != 0 && target_thread_id != cur_thread_id {
            AttachThreadInput(cur_thread_id, target_thread_id, 1) != 0
        } else {
            false
        };

        let is_foreground = SetForegroundWindow(hwnd) != 0;
        SwitchToThisWindow(hwnd, 1);

        if has_attached_fg {
            AttachThreadInput(cur_thread_id, fg_thread_id, 0);
        }
        if has_attached_target {
            AttachThreadInput(cur_thread_id, target_thread_id, 0);
        }

        is_foreground || !hwnd.is_null()
    }
}

/// Bring any instance window in the process list to the foreground
#[cfg(target_os = "windows")]
pub fn focus_instance_pids(pids: &[u32]) -> bool {
    if pids.is_empty() {
        return false;
    }

    struct PidSearchContext<'a> {
        target_pids: &'a [u32],
        best_hwnd: Option<win32::HWND>,
        fallback_hwnd: Option<win32::HWND>,
    }

    unsafe extern "system" fn enum_pid_proc(
        hwnd: win32::HWND,
        lparam: win32::LPARAM,
    ) -> win32::BOOL {
        let ctx = &mut *(lparam as *mut PidSearchContext);
        if win32::IsWindowVisible(hwnd) == 0 {
            return 1;
        }

        let mut proc_id = 0u32;
        win32::GetWindowThreadProcessId(hwnd, &mut proc_id);
        if ctx.target_pids.contains(&proc_id) {
            let mut title_buf = [0u16; 512];
            let len = win32::GetWindowTextW(hwnd, title_buf.as_mut_ptr(), 512);
            if len > 0 {
                ctx.best_hwnd = Some(hwnd);
                return 0; // Stop enumeration: found visible window with title belonging to PID
            } else if ctx.fallback_hwnd.is_none() {
                ctx.fallback_hwnd = Some(hwnd);
            }
        }
        1
    }

    let mut ctx = PidSearchContext {
        target_pids: pids,
        best_hwnd: None,
        fallback_hwnd: None,
    };

    unsafe {
        win32::EnumWindows(
            enum_pid_proc,
            &mut ctx as *mut PidSearchContext as win32::LPARAM,
        );
        if let Some(hwnd) = ctx.best_hwnd.or(ctx.fallback_hwnd) {
            return win32::activate_hwnd(hwnd);
        }
    }

    false
}

/// Bring an instance window to the foreground by PID
#[cfg(target_os = "windows")]
pub fn focus_instance_process(pid: u32) -> bool {
    focus_instance_pids(&[pid])
}

#[cfg(target_os = "macos")]
pub fn focus_instance_pids(pids: &[u32]) -> bool {
    if let Some(&pid) = pids.first() {
        focus_instance_process(pid)
    } else {
        false
    }
}

#[cfg(target_os = "macos")]
pub fn focus_instance_process(pid: u32) -> bool {
    let script = format!(
        "tell application \"System Events\" to set frontmost of (first process whose unix id is {}) to true",
        pid
    );
    let output = Command::new("osascript").args(["-e", &script]).output();
    match output {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}

#[cfg(target_os = "linux")]
pub fn focus_instance_pids(pids: &[u32]) -> bool {
    if let Some(&pid) = pids.first() {
        focus_instance_process(pid)
    } else {
        false
    }
}

#[cfg(target_os = "linux")]
pub fn focus_instance_process(pid: u32) -> bool {
    let output = Command::new("xdotool")
        .args(["search", "--pid", &pid.to_string(), "windowactivate"])
        .output();
    match output {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn focus_instance_pids(_pids: &[u32]) -> bool {
    false
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn focus_instance_process(_pid: u32) -> bool {
    false
}

/// Bring Antigravity IDE/Classic window to the foreground and ensure it is visible/restored
pub fn focus_antigravity_window(target_ide: Option<&str>) -> bool {
    let pids = get_antigravity_pids(target_ide);
    focus_instance_pids(&pids)
}

/// Bring an instance window with matching workspace title to the foreground
#[cfg(target_os = "windows")]
pub fn focus_instance_workspace_window(pids: &[u32], repo_name: &str) -> bool {
    if pids.is_empty() {
        return false;
    }

    struct WorkspaceSearchContext<'a> {
        target_pids: &'a [u32],
        repo_name: &'a str,
        matched_title_hwnd: Option<win32::HWND>,
        fallback_hwnd: Option<win32::HWND>,
    }

    unsafe extern "system" fn enum_workspace_proc(
        hwnd: win32::HWND,
        lparam: win32::LPARAM,
    ) -> win32::BOOL {
        let ctx = &mut *(lparam as *mut WorkspaceSearchContext);
        if win32::IsWindowVisible(hwnd) == 0 {
            return 1;
        }

        let mut proc_id = 0u32;
        win32::GetWindowThreadProcessId(hwnd, &mut proc_id);
        if ctx.target_pids.contains(&proc_id) {
            let mut title_buf = [0u16; 512];
            let len = win32::GetWindowTextW(hwnd, title_buf.as_mut_ptr(), 512);
            if len > 0 {
                let title = String::from_utf16_lossy(&title_buf[..len as usize]);
                let is_matching_repo = title.to_lowercase().contains(&ctx.repo_name.to_lowercase());
                if is_matching_repo {
                    ctx.matched_title_hwnd = Some(hwnd);
                    return 0; // Stop enumeration: found exact workspace title match!
                }
                if ctx.fallback_hwnd.is_none() {
                    ctx.fallback_hwnd = Some(hwnd);
                }
            } else if ctx.fallback_hwnd.is_none() {
                ctx.fallback_hwnd = Some(hwnd);
            }
        }
        1
    }

    let mut ctx = WorkspaceSearchContext {
        target_pids: pids,
        repo_name,
        matched_title_hwnd: None,
        fallback_hwnd: None,
    };

    unsafe {
        win32::EnumWindows(
            enum_workspace_proc,
            &mut ctx as *mut WorkspaceSearchContext as win32::LPARAM,
        );
        if let Some(hwnd) = ctx.matched_title_hwnd.or(ctx.fallback_hwnd) {
            return win32::activate_hwnd(hwnd);
        }
    }

    false
}

#[cfg(target_os = "macos")]
pub fn focus_instance_workspace_window(pids: &[u32], repo_name: &str) -> bool {
    let script = format!(
        r#"tell application "System Events"
            set matched to false
            repeat with p in {{{}}}
                set procList to (processes whose unix id is p)
                if (count of procList) > 0 then
                    set theProc to item 1 of procList
                    tell theProc
                        repeat with w in windows
                            if name of w contains "{}" then
                                set frontmost to true
                                perform action "AXRaise" of w
                                set matched to true
                                exit repeat
                            end if
                        end repeat
                    end tell
                    if matched then exit repeat
                end if
            end repeat
            if not matched and (count of pids) > 0 then
                set frontmost of (first process whose unix id is (item 1 of pids)) to true
            end if
        end tell"#,
        pids.iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(","),
        repo_name.replace('"', "\\\"")
    );
    let output = Command::new("osascript").args(["-e", &script]).output();
    match output {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
}

#[cfg(target_os = "linux")]
pub fn focus_instance_workspace_window(pids: &[u32], repo_name: &str) -> bool {
    let wm_status = Command::new("wmctrl")
        .args(["-a", repo_name])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    if wm_status {
        return true;
    }
    if let Some(&pid) = pids.first() {
        Command::new("xdotool")
            .args(["search", "--pid", &pid.to_string(), "windowactivate"])
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
    } else {
        false
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub fn focus_instance_workspace_window(_pids: &[u32], _repo_name: &str) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_empty_pids() {
        assert!(!focus_instance_pids(&[]));
        assert!(!focus_instance_workspace_window(&[], "test-repo"));
    }
}
