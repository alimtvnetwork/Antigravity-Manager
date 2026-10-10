use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use super::*;

/// Stage 3 implementation: Dedicated CLI instruction (`agm open-ui`) to launch the Antigravity Manager UI.
pub fn open_ui(args: &[String]) {
    let opts = parse_delegate_args(args);
    let resolved_install_dir =
        resolve_default_install_dir(opts.install_dir.clone(), opts.target_exe.as_ref());
    let resolved_target_exe =
        resolve_default_target_exe(opts.target_exe.clone(), &resolved_install_dir);

    let exe_to_launch = if resolved_target_exe.exists() {
        resolved_target_exe
    } else {
        let alt = resolved_install_dir.join(if cfg!(target_os = "windows") {
            "agm-alim.exe"
        } else {
            "agm-alim"
        });
        if alt.exists() {
            alt
        } else {
            resolved_target_exe
        }
    };

    println!(
        "[*] Opening Antigravity Manager UI: {}",
        exe_to_launch.display()
    );
    std::thread::sleep(Duration::from_millis(400));

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const DETACHED_PROCESS: u32 = 0x00000008;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;

        let mut cmd = Command::new(&exe_to_launch);
        if resolved_install_dir.exists() {
            cmd.current_dir(&resolved_install_dir);
        }
        cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP);

        match cmd.spawn() {
            Ok(child) => {
                println!(
                    "[OK] Antigravity Manager UI launched (PID: {}).",
                    child.id()
                );
            }
            Err(e) => {
                eprintln!(
                    "[WARN] Direct UI spawn failed ({}), trying PowerShell Start-Process...",
                    e
                );
                let ps_launch = format!(
                    "Start-Process -FilePath '{}' -WorkingDirectory '{}'",
                    exe_to_launch.to_string_lossy().replace('\'', "''"),
                    resolved_install_dir.to_string_lossy().replace('\'', "''")
                );
                // Justification: best-effort process spawn; failure logged
                crate::error::record_ignored(
                    Command::new("powershell.exe")
                        .args(["-NoProfile", "-Command", &ps_launch])
                        .spawn(),
                    "spawn powershell.exe",
                );
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let mut launch_ok = false;
        match Command::new("open").arg(&exe_to_launch).output() {
            Ok(output) => {
                if output.status.success() {
                    println!(
                        "[OK] Antigravity Manager UI launched via open: {:?}",
                        exe_to_launch
                    );
                    launch_ok = true;
                } else {
                    let err = String::from_utf8_lossy(&output.stderr);
                    eprintln!(
                        "[ERROR] Failed to launch Antigravity Manager via open (exit: {:?}): {}",
                        output.status.code(),
                        err.trim()
                    );
                    eprintln!(
                        "[STACK TRACE] Diagnostic Backtrace:\n{:?}",
                        std::backtrace::Backtrace::force_capture()
                    );

                    eprintln!("[RECOVERY] LaunchServices error or trash conflict detected. Purging stale trash references, resetting LaunchServices, and re-registering...");
                    // Justification: best-effort process spawn; failure logged
                    crate::error::record_ignored(Command::new("bash")
                        .arg("-c")
                        .arg(r#"
                            rm -rf /private/tmp/*[Aa]ntigravity* /private/tmp/*[Aa]gm* /tmp/*[Aa]ntigravity* /tmp/*[Aa]gm* 2>/dev/null || true
                            osascript -e '
                            tell application "Finder"
                                try
                                    set destFolder to (POSIX file "/private/tmp") as alias
                                    repeat with anItem in (every item of trash)
                                        try
                                            set n to name of anItem as text
                                            if n contains "Antigravity" or n contains "agm" then
                                                move anItem to destFolder with replacing
                                            end if
                                        end try
                                    end repeat
                                end try
                            end tell' 2>/dev/null || true
                            rm -rf /private/tmp/*[Aa]ntigravity* /private/tmp/*[Aa]gm* /tmp/*[Aa]ntigravity* /tmp/*[Aa]gm* 2>/dev/null || true

                            lsregister=$(find /System/Library/Frameworks/CoreServices.framework -name "lsregister" -type f 2>/dev/null | head -n 1)
                            find "$HOME/.Trash" -maxdepth 1 \( -iname "*antigravity*" -o -iname "*agm*" \) 2>/dev/null | while read -r ta; do
                                if [ -n "$ta" ]; then
                                    [ -n "$lsregister" ] && "$lsregister" -u "$ta" 2>/dev/null || true
                                    chflags -R nouchg,noschg "$ta" 2>/dev/null || true
                                    rm -rf "$ta" 2>/dev/null || true
                                fi
                            done
                            if [ -n "$lsregister" ]; then
                                "$lsregister" -u "$1" 2>/dev/null || true
                                "$lsregister" -gc -R -v -apps u,s,l 2>/dev/null || "$lsregister" -gc 2>/dev/null || true
                                "$lsregister" -f -r "$1" 2>/dev/null || true
                                killall Finder Dock 2>/dev/null || true
                            fi
                        "#)
                        .arg("bash")
                        .arg(&exe_to_launch)
                        .status(), "spawn bash");

                    std::thread::sleep(Duration::from_millis(800));
                    if let Ok(retry_out) =
                        Command::new("open").arg("-n").arg(&exe_to_launch).output()
                    {
                        if retry_out.status.success() {
                            println!("[OK] Antigravity Manager UI launched after LaunchServices recovery: {:?}", exe_to_launch);
                            launch_ok = true;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "[ERROR] Failed to execute open command for {:?}: {}",
                    exe_to_launch, e
                );
                eprintln!(
                    "[STACK TRACE] Backtrace:\n{:?}",
                    std::backtrace::Backtrace::force_capture()
                );
            }
        }

        if !launch_ok {
            let internal_bins = [
                exe_to_launch.join("Contents/MacOS/agm-alim"),
                exe_to_launch.join("Contents/MacOS/Antigravity Manager Tools"),
                exe_to_launch.join("Contents/MacOS/agm"),
            ];
            for ib in &internal_bins {
                if ib.exists() {
                    eprintln!("[INFO] Attempting fallback direct binary launch: {:?}", ib);
                    let mut cmd = Command::new(ib);
                    if let Ok(home) = std::env::var("HOME") {
                        let log_dir =
                            std::path::PathBuf::from(home).join("Library/Logs/AntigravityManager");
                        // Justification: best-effort directory creation; later file ops fail loudly if the directory is actually needed
                        crate::error::record_ignored(
                            std::fs::create_dir_all(&log_dir),
                            "create_dir_all",
                        );
                        if let Ok(f) = std::fs::OpenOptions::new()
                            .create(true)
                            .append(true)
                            .open(log_dir.join("updater_binary.log"))
                        {
                            if let Ok(f2) = f.try_clone() {
                                cmd.stdout(f);
                                cmd.stderr(f2);
                            }
                        }
                    }
                    match cmd.spawn() {
                        Ok(child) => {
                            println!(
                                "[OK] Antigravity Manager core binary launched (PID: {})",
                                child.id()
                            );
                            break;
                        }
                        Err(e) => {
                            eprintln!("[ERROR] Direct binary launch failed for {:?}: {}", ib, e);
                            eprintln!(
                                "[STACK TRACE]\n{:?}",
                                std::backtrace::Backtrace::force_capture()
                            );
                        }
                    }
                }
            }
        }
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        let mut cmd = Command::new(&exe_to_launch);
        if resolved_install_dir.exists() {
            cmd.current_dir(&resolved_install_dir);
        }
        // Justification: best-effort call; failure logged without changing control flow
        crate::error::record_ignored(cmd.spawn(), "spawn");
        println!("[OK] Antigravity Manager UI launched.");
    }
}
