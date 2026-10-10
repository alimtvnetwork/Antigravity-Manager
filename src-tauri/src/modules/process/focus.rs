use std::process::Command;
use sysinfo::System;

use super::*;

/// Bring any instance window in the process list to the foreground
#[cfg(target_os = "windows")]
pub fn focus_instance_pids(pids: &[u32]) -> bool {
    if pids.is_empty() {
        return false;
    }
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let pid_list = pids
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join(",");

    let ps_cmd = format!(
        r#"$pids = @({});
$ws = New-Object -ComObject WScript.Shell;
$act = $false;
foreach ($p in $pids) {{
    if ($ws.AppActivate($p)) {{ $act = $true; break }}
}}
if (-not $act) {{
    foreach ($p in $pids) {{
        $proc = Get-Process -Id $p -ErrorAction SilentlyContinue;
        if ($proc -and $proc.MainWindowHandle -ne 0) {{
            Add-Type '[DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd); [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);' -Name 'Win32Focus' -Namespace 'Antigravity';
            [Antigravity.Win32Focus]::ShowWindow($proc.MainWindowHandle, 9);
            [Antigravity.Win32Focus]::SetForegroundWindow($proc.MainWindowHandle);
            $act = $true;
            break;
        }}
    }}
}}
exit $(if ($act) {{ 0 }} else {{ 1 }})"#,
        pid_list
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps_cmd])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    match output {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
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
    const CREATE_NO_WINDOW: u32 = 0x08000000;
    let pid_list = pids
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join(",");

    let safe_repo = repo_name.replace('\'', "''").replace('"', "");

    let ps_cmd = format!(
        r#"$pids = @({});
$repo = '{}';
$act = $false;

if (-not ([System.Management.Automation.PSTypeName]'Antigravity.Win32WorkspaceWindow').Type) {{
    Add-Type @'
    using System;
    using System.Runtime.InteropServices;
    using System.Text;

    namespace Antigravity {{
        public class Win32WorkspaceWindow {{
            [DllImport("user32.dll")]
            public static extern bool SetForegroundWindow(IntPtr hWnd);

            [DllImport("user32.dll")]
            public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);

            [DllImport("user32.dll")]
            public static extern bool BringWindowToTop(IntPtr hWnd);

            [DllImport("user32.dll")]
            public static extern int GetWindowText(IntPtr hWnd, StringBuilder lpString, int nMaxCount);

            [DllImport("user32.dll")]
            public static extern uint GetWindowThreadProcessId(IntPtr hWnd, out uint lpdwProcessId);

            [DllImport("user32.dll")]
            public static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);

            public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);
        }}
    }}
'@;
}}

# 1. First pass: Search top-level windows belonging to target PIDs matching repo name
[Antigravity.Win32WorkspaceWindow]::EnumWindows({{
    param($hwnd, $lparam)
    $procId = 0;
    [Antigravity.Win32WorkspaceWindow]::GetWindowThreadProcessId($hwnd, [ref]$procId);
    if ($pids -contains $procId) {{
        $sb = New-Object System.Text.StringBuilder 512;
        [Antigravity.Win32WorkspaceWindow]::GetWindowText($hwnd, $sb, 512) | Out-Null;
        $title = $sb.ToString();
        if ($title -and ($title -like "*$repo*")) {{
            [Antigravity.Win32WorkspaceWindow]::ShowWindow($hwnd, 9);
            [Antigravity.Win32WorkspaceWindow]::BringWindowToTop($hwnd);
            [Antigravity.Win32WorkspaceWindow]::SetForegroundWindow($hwnd);
            $global:act = $true;
            $script:act = $true;
            return $false;
        }}
    }}
    return $true;
}}, [IntPtr]::Zero);

# 2. Second pass: Fallback to any MainWindowHandle if exact repo title not found
if (-not $global:act -and -not $script:act -and -not $act) {{
    foreach ($p in $pids) {{
        $proc = Get-Process -Id $p -ErrorAction SilentlyContinue;
        if ($proc -and $proc.MainWindowHandle -ne 0) {{
            [Antigravity.Win32WorkspaceWindow]::ShowWindow($proc.MainWindowHandle, 9);
            [Antigravity.Win32WorkspaceWindow]::BringWindowToTop($proc.MainWindowHandle);
            [Antigravity.Win32WorkspaceWindow]::SetForegroundWindow($proc.MainWindowHandle);
            $global:act = $true;
            $script:act = $true;
            $act = $true;
            break;
        }}
    }}
}}
exit $(if ($global:act -or $script:act -or $act) {{ 0 }} else {{ 1 }})"#,
        pid_list, safe_repo
    );

    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", &ps_cmd])
        .creation_flags(CREATE_NO_WINDOW)
        .output();

    match output {
        Ok(out) => out.status.success(),
        Err(_) => false,
    }
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
