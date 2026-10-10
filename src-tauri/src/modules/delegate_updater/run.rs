use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use super::*;

pub fn run(args: &[String]) {
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

    let opts = parse_delegate_args(args);
    let resolved_install_dir =
        resolve_default_install_dir(opts.install_dir.clone(), opts.target_exe.as_ref());
    let resolved_target_exe =
        resolve_default_target_exe(opts.target_exe.clone(), &resolved_install_dir);

    // Self-delegation check: if invoked directly from the install directory (not yet copied to %TEMP%\agm-updater),
    // copy self to %TEMP%\agm-updater\agm-update-cli-<pid>.exe, delegate the task to that copy, and exit immediately.
    let current_exe = env::current_exe().unwrap_or_default();
    let is_running_from_temp = current_exe
        .to_string_lossy()
        .to_lowercase()
        .contains("agm-updater")
        || opts.is_delegated_worker;

    if !is_running_from_temp {
        let my_pid = std::process::id();
        println!("[*] Self-delegating update task to isolated temp CLI copy...");
        match prepare_isolated_update_cli(my_pid) {
            Ok(temp_cli) => {
                let wait_target = opts.wait_pid.unwrap_or(my_pid);
                match spawn_delegated_update_cli(
                    &temp_cli,
                    wait_target,
                    &resolved_install_dir,
                    &resolved_target_exe,
                    opts.is_relaunch,
                    opts.target_version.as_deref(),
                ) {
                    Ok(child_pid) => {
                        println!(
                            "[OK] Delegated to {:?} (PID {}). Exiting caller so update can proceed.",
                            temp_cli, child_pid
                        );
                        return;
                    }
                    Err(e) => {
                        eprintln!(
                            "[WARN] Could not spawn delegated copy ({}), continuing in-process...",
                            e
                        );
                    }
                }
            }
            Err(e) => {
                eprintln!(
                    "[WARN] Could not prepare isolated CLI copy ({}), continuing in-process...",
                    e
                );
            }
        }
    }

    println!("================================================================================");
    println!("   ANTIGRAVITY MANAGER - DELEGATED UPDATE CLI ENGINE");
    println!("================================================================================");
    println!("  ● Update CLI Exe:   {}", current_exe.display());
    println!("  ● Target Dir:       {}", resolved_install_dir.display());
    println!("  ● Target UI Exe:    {}", resolved_target_exe.display());
    if let Some(pid) = opts.wait_pid {
        println!("  ● Monitored UI PID: {}", pid);
    }
    println!("  ● Auto-Relaunch UI: {}", opts.is_relaunch);
    println!("--------------------------------------------------------------------------------");

    // STAGE 1: Wait for monitored UI process (and any sibling UI instances) to shut down and release file locks
    if let Some(pid) = opts.wait_pid {
        println!(
            "[*] Stage 1/3: Waiting for Antigravity Manager UI (PID {}) to exit cleanly...",
            pid
        );
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(10);
        while is_pid_alive(pid) && start.elapsed() < timeout {
            std::thread::sleep(Duration::from_millis(250));
        }

        if is_pid_alive(pid) {
            println!(
                "[!] UI process (PID {}) still active after 10s. Terminating to release file locks...",
                pid
            );
            force_kill_pid(pid);
            std::thread::sleep(Duration::from_millis(800));
        } else {
            println!("[OK] UI process exited cleanly.");
        }
    }

    // Ensure no other agm-alim.exe UI process holds locks in the target directory
    let my_pid = std::process::id();
    if is_ui_process_running_excluding(my_pid) {
        println!(
            "[*] Closing remaining Antigravity Manager UI instances before binary replacement..."
        );
        kill_other_ui_processes(my_pid);
        std::thread::sleep(Duration::from_millis(700));
    }

    // STAGE 2: Delegate to `<self_exe> update --force --no-launch --install-dir <DIR>`
    println!(
        "[*] Stage 2/3: Running CLI update command (`{} update --force --no-launch`)...",
        current_exe
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "agm-update-cli".to_string())
    );

    let mut update_args = vec![
        "--force".to_string(),
        "--no-launch".to_string(),
        "--install-dir".to_string(),
        resolved_install_dir.to_string_lossy().to_string(),
    ];
    if let Some(ref ver) = opts.target_version {
        update_args.push("--version".to_string());
        update_args.push(ver.clone());
    }

    let update_status = Command::new(&current_exe)
        .arg("update")
        .args(&update_args)
        .status();

    let update_ok = match update_status {
        Ok(s) if s.success() => true,
        _ => {
            // Direct in-process fallback if child invocation failed
            run_cli_update(&update_args)
        }
    };

    // STAGE 3: Delegate to `<self_exe> open-ui --target-exe <EXE> --install-dir <DIR>`
    if opts.is_relaunch {
        println!(
            "[*] Stage 3/3: Running CLI instruction to reopen UI (`{} open-ui`)...",
            current_exe
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "agm-update-cli".to_string())
        );

        let open_args = vec![
            "--target-exe".to_string(),
            resolved_target_exe.to_string_lossy().to_string(),
            "--install-dir".to_string(),
            resolved_install_dir.to_string_lossy().to_string(),
        ];

        let open_status = Command::new(&current_exe)
            .arg("open-ui")
            .args(&open_args)
            .status();

        if !matches!(open_status, Ok(s) if s.success()) {
            open_ui(&open_args);
        }
    }

    println!("================================================================================");
    if update_ok {
        println!("   UPDATE COMPLETE - Antigravity Manager Updated & Reopened.");
    } else {
        println!("   UPDATE FINISHED WITH WARNINGS - Check logs above.");
    }
    println!("================================================================================");
    std::thread::sleep(Duration::from_millis(1200));
}

/// Stage 2 implementation: Execute the CLI update (`agm update --force --no-launch --install-dir <DIR>`)
pub fn run_cli_update(args: &[String]) -> bool {
    let opts = parse_delegate_args(args);
    let resolved_install_dir =
        resolve_default_install_dir(opts.install_dir.clone(), opts.target_exe.as_ref());

    println!(
        "[*] Executing AGM CLI Update for directory: {}",
        resolved_install_dir.display()
    );

    let mut update_ok = false;

    #[cfg(target_os = "windows")]
    {
        let dir_arg = format!("-InstallDir \"{}\"", resolved_install_dir.to_string_lossy());
        let ver_arg = opts
            .target_version
            .as_ref()
            .map(|v| format!("-Version \"{}\"", v))
            .unwrap_or_default();

        let local_script = resolved_install_dir.join("install.ps1");
        let cwd_script = Path::new("install.ps1");

        let ps_script = if local_script.exists() {
            format!(
                "& \"{}\" -Update -Force -NoLaunch {} {}",
                local_script.display(),
                dir_arg,
                ver_arg
            )
        } else if cwd_script.exists() {
            format!(
                ".\\install.ps1 -Update -Force -NoLaunch {} {}",
                dir_arg, ver_arg
            )
        } else {
            format!(
                "$s = irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1; & ([scriptblock]::Create($s)) -Update -Force -NoLaunch {} {}",
                dir_arg, ver_arg
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

        match status {
            Ok(s) if s.success() => {
                update_ok = true;
                println!("[OK] Official PowerShell installer update completed successfully!");
            }
            Ok(s) => {
                eprintln!(
                    "[WARN] PowerShell installer exited with code: {:?}. Attempting gitmap agm update fallback...",
                    s.code()
                );
            }
            Err(e) => {
                eprintln!(
                    "[WARN] Failed to execute PowerShell installer ({}). Attempting gitmap agm update fallback...",
                    e
                );
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
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
                println!("[OK] Official shell installer update completed successfully!");
            }
        }
    }

    if !update_ok {
        let gitmap_status = Command::new("gitmap")
            .args(["agm", "update", "-y"])
            .status();
        if let Ok(s) = gitmap_status {
            if s.success() {
                println!("[OK] GitMap executed Antigravity Manager update successfully!");
                update_ok = true;
            }
        }
    }

    // Synchronize %LOCALAPPDATA%\agm-cli\agm.exe & agm-update-cli.exe if updated agm.exe exists in install_dir
    #[cfg(target_os = "windows")]
    {
        let installed_agm = resolved_install_dir.join("agm.exe");
        let installed_ui = resolved_install_dir.join("agm-alim.exe");
        if let Ok(local) = env::var("LOCALAPPDATA") {
            let cli_dir = PathBuf::from(local).join("agm-cli");
            // Justification: best-effort directory creation; later file ops fail loudly if the directory is actually needed
            crate::error::record_ignored(fs::create_dir_all(&cli_dir), "create_dir_all");
            if installed_agm.exists() {
                // Justification: best-effort file copy; logged for diagnosis
                crate::error::record_ignored(
                    fs::copy(&installed_agm, cli_dir.join("agm.exe")),
                    "fs::copy",
                );
                // Justification: best-effort file copy; logged for diagnosis
                crate::error::record_ignored(
                    fs::copy(&installed_agm, cli_dir.join("agm-update-cli.exe")),
                    "fs::copy",
                );
            } else if installed_ui.exists() {
                // Justification: best-effort file copy; logged for diagnosis
                crate::error::record_ignored(
                    fs::copy(&installed_ui, cli_dir.join("agm-update-cli.exe")),
                    "fs::copy",
                );
            }
        }
    }

    update_ok
}
