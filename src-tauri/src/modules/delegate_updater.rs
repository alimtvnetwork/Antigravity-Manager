//! Delegated Out-of-Process Updater for Antigravity-Manager
//! Handles seamless zero-file-lock background/foreground updates and automatic relaunch.

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

#[cfg(target_os = "windows")]
extern "system" {
    fn AllocConsole() -> i32;
    fn AttachConsole(dwProcessId: u32) -> i32;
}

pub fn run(args: &[String]) {
    // 0. On Windows, if running as a GUI process without a console, allocate or attach console
    #[cfg(target_os = "windows")]
    unsafe {
        if AttachConsole(0xFFFFFFFF) == 0 {
            AllocConsole();
        }
    }

    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        print_help();
        return;
    }

    let mut wait_pid: Option<u32> = None;
    let mut relaunch = true;
    let mut target_exe: Option<PathBuf> = None;
    let mut install_dir: Option<PathBuf> = None;
    let mut target_version: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--wait-pid" => {
                if i + 1 < args.len() {
                    wait_pid = args[i + 1].parse::<u32>().ok();
                    i += 1;
                }
            }
            "--relaunch" => {
                relaunch = true;
            }
            "--no-relaunch" => {
                relaunch = false;
            }
            "--target-exe" => {
                if i + 1 < args.len() {
                    target_exe = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            "--install-dir" => {
                if i + 1 < args.len() {
                    install_dir = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            "--version" => {
                if i + 1 < args.len() {
                    target_version = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }

    // Resolve install directory if not explicitly provided
    let resolved_install_dir = install_dir
        .or_else(|| {
            target_exe
                .as_ref()
                .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        })
        .unwrap_or_else(|| {
            #[cfg(target_os = "windows")]
            {
                if let Ok(local) = env::var("LOCALAPPDATA") {
                    let cand1 = PathBuf::from(&local).join("Programs").join("agm-alim");
                    if cand1.exists() {
                        return cand1;
                    }
                    let cand2 = PathBuf::from(&local)
                        .join("Programs")
                        .join("Antigravity-Tools");
                    if cand2.exists() {
                        return cand2;
                    }
                    cand1
                } else {
                    PathBuf::from(".")
                }
            }
            #[cfg(not(target_os = "windows"))]
            {
                PathBuf::from("/Applications/Antigravity Manager Tools.app")
            }
        });

    let resolved_target_exe = target_exe.unwrap_or_else(|| {
        #[cfg(target_os = "windows")]
        {
            let exe_cand = resolved_install_dir.join("agm-alim.exe");
            if exe_cand.exists() {
                exe_cand
            } else {
                PathBuf::from("agm-alim.exe")
            }
        }
        #[cfg(target_os = "macos")]
        {
            PathBuf::from("/Applications/Antigravity Manager Tools.app")
        }
        #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
        {
            resolved_install_dir.join("agm-alim")
        }
    });

    println!("================================================================================");
    println!("   ANTIGRAVITY MANAGER - DELEGATED UPDATE ENGINE");
    println!("================================================================================");
    println!("  ● Mode:             Out-of-Process Delegated Self-Update");
    println!("  ● Target Dir:       {}", resolved_install_dir.display());
    println!("  ● Target Exe:       {}", resolved_target_exe.display());
    if let Some(pid) = wait_pid {
        println!("  ● Monitored UI PID: {}", pid);
    }
    println!("  ● Auto-Relaunch:    {}", relaunch);
    println!("--------------------------------------------------------------------------------");

    // 1. Wait for monitored UI process to shut down and release file locks
    if let Some(pid) = wait_pid {
        println!(
            "[*] Waiting for Antigravity Manager UI (PID {}) to exit cleanly...",
            pid
        );
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(15);
        while is_pid_alive(pid) && start.elapsed() < timeout {
            std::thread::sleep(Duration::from_millis(250));
        }

        if is_pid_alive(pid) {
            println!(
                "[!] UI process (PID {}) did not exit within 15s. Terminating to release file locks...",
                pid
            );
            force_kill_pid(pid);
            std::thread::sleep(Duration::from_millis(800));
        } else {
            println!("[OK] UI process has exited. All file locks released.");
        }
    }

    // 2. Extra safety buffer for OS file handles to be released
    std::thread::sleep(Duration::from_millis(600));

    // 3. Execute update
    println!("[*] Running update via GitMap or official installer...");
    let mut update_ok = false;

    // Check if gitmap is available in PATH
    let gitmap_status = Command::new("gitmap")
        .args(["agm", "update", "-y"])
        .status();

    if let Ok(s) = gitmap_status {
        if s.success() {
            println!("[OK] GitMap executed Antigravity Manager update successfully!");
            update_ok = true;
        }
    }

    if !update_ok {
        #[cfg(target_os = "windows")]
        {
            println!("[*] Running official PowerShell installer (install.ps1)...");
            let dir_arg = format!("-InstallDir \"{}\"", resolved_install_dir.to_string_lossy());
            let ver_arg = target_version
                .as_ref()
                .map(|v| format!("-Version \"{}\"", v))
                .unwrap_or_default();

            let ps_script = if resolved_install_dir.join("install.ps1").exists() {
                format!(
                    "& \"{}\" -Update -NoLaunch {} {}",
                    resolved_install_dir.join("install.ps1").display(),
                    dir_arg,
                    ver_arg
                )
            } else if Path::new("install.ps1").exists() {
                format!(".\\install.ps1 -Update -NoLaunch {} {}", dir_arg, ver_arg)
            } else {
                format!(
                    "$s = irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1; & ([scriptblock]::Create($s)) -Update -NoLaunch {} {}",
                    dir_arg,
                    ver_arg
                )
            };

            let status = Command::new("powershell.exe")
                .args([
                    "-NoProfile",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-Command",
                    &ps_script,
                ])
                .status();

            if let Ok(s) = status {
                if s.success() {
                    update_ok = true;
                    println!("[OK] PowerShell installer completed successfully!");
                } else {
                    eprintln!(
                        "[ERROR] PowerShell installer exited with code: {:?}",
                        s.code()
                    );
                }
            } else if let Err(e) = status {
                eprintln!("[ERROR] Failed to execute PowerShell installer: {}", e);
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            println!("[*] Running official Unix shell installer (install.sh)...");
            let dir_arg = format!(
                "--install-dir \"{}\"",
                resolved_install_dir.to_string_lossy()
            );
            let sh_script = if Path::new("install.sh").exists() {
                format!("bash ./install.sh --update --no-launch {}", dir_arg)
            } else {
                format!(
                    "curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash -s -- --update --no-launch {}",
                    dir_arg
                )
            };

            let status = Command::new("bash").args(["-c", &sh_script]).status();
            if let Ok(s) = status {
                if s.success() {
                    update_ok = true;
                    println!("[OK] Shell installer completed successfully!");
                }
            }
        }
    }

    // 4. Post-update relaunch
    if relaunch {
        let exe_to_launch = if resolved_target_exe.exists() {
            resolved_target_exe.clone()
        } else {
            let alt = resolved_install_dir.join("agm-alim.exe");
            if alt.exists() {
                alt
            } else {
                resolved_target_exe.clone()
            }
        };

        if !is_ui_process_running() {
            println!(
                "[*] Launching updated Antigravity Manager Tools ({})...",
                exe_to_launch.display()
            );
            std::thread::sleep(Duration::from_millis(600));

            #[cfg(target_os = "windows")]
            {
                let _ = Command::new("cmd.exe")
                    .args(["/c", "start", "", &exe_to_launch.to_string_lossy()])
                    .spawn();
            }
            #[cfg(target_os = "macos")]
            {
                let _ = Command::new("open").arg(&exe_to_launch).spawn();
            }
            #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
            {
                let _ = Command::new(&exe_to_launch).spawn();
            }
            println!("[OK] Antigravity Manager Tools UI launched successfully!");
        } else {
            println!("[OK] Antigravity Manager Tools UI is already active.");
        }
    }

    println!("================================================================================");
    if update_ok {
        println!("   UPDATE COMPLETE - Reopened Antigravity Manager.");
    } else {
        println!("   UPDATE FINISHED WITH WARNINGS - Check logs above.");
    }
    println!("================================================================================");
    std::thread::sleep(Duration::from_millis(1500));
}

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
        let _ = Command::new("taskkill.exe")
            .args(["/F", "/PID", &pid.to_string()])
            .output();
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = Command::new("kill").args(["-9", &pid.to_string()]).output();
    }
}

pub fn is_ui_process_running() -> bool {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("tasklist.exe")
            .args(["/FI", "IMAGENAME eq agm-alim.exe", "/NH"])
            .output();
        if let Ok(out) = output {
            let text = String::from_utf8_lossy(&out.stdout);
            let trimmed = text.trim();
            if trimmed.is_empty() || trimmed.to_lowercase().starts_with("info:") {
                return false;
            }
            trimmed.contains("agm-alim.exe") && !trimmed.contains("No tasks")
        } else {
            false
        }
    }
    #[cfg(not(target_os = "windows"))]
    {
        let output = Command::new("pgrep").args(["-f", "agm-alim"]).output();
        matches!(output, Ok(o) if o.status.success())
    }
}

fn print_help() {
    println!("AGM Delegated Out-of-Process Updater:");
    println!("  agm delegate-update [--wait-pid <PID>] [--install-dir <PATH>] [--target-exe <PATH>] [--relaunch]");
    println!();
    println!("Description:");
    println!("  Executes isolated, zero-file-lock application update outside of the installation directory.");
    println!("  Waits for the GUI process to terminate, runs GitMap or official installer in-place, and relaunches the UI.");
    println!();
    println!("Options:");
    println!("  --wait-pid <PID>     Wait for specified GUI process ID to exit before updating");
    println!("  --install-dir <PATH> Exact application installation directory to update");
    println!("  --target-exe <PATH>  Specific UI executable path to launch after update");
    println!("  --relaunch           Relaunch UI upon update completion (default: true)");
    println!("  --no-relaunch        Do not relaunch UI upon update completion");
    println!("  --version <VER>      Target release version to install");
}
