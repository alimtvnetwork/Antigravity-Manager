//! Delegated Out-of-Process Updater for Antigravity-Manager
//! Implements the 3-Stage Self-Delegating CLI Update Pipeline:
//!   Stage 1: Copy CLI to isolated `%TEMP%\agm-updater\agm-update-cli-<pid>.exe` & wait for UI exit
//!   Stage 2: Execute `<agm-update-cli> update --force --no-launch --install-dir <DIR>`
//!   Stage 3: Execute `<agm-update-cli> open-ui --target-exe <EXE> --install-dir <DIR>`

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

#[cfg(target_os = "windows")]
extern "system" {
    fn AllocConsole() -> i32;
    fn AttachConsole(dwProcessId: u32) -> i32;
}

#[derive(Debug, Clone, Default)]
pub struct DelegateUpdateOptions {
    pub wait_pid: Option<u32>,
    pub is_relaunch: bool,
    pub target_exe: Option<PathBuf>,
    pub install_dir: Option<PathBuf>,
    pub target_version: Option<String>,
    pub is_delegated_worker: bool,
}

pub fn parse_delegate_args(args: &[String]) -> DelegateUpdateOptions {
    let mut opts = DelegateUpdateOptions {
        wait_pid: None,
        is_relaunch: true,
        target_exe: None,
        install_dir: None,
        target_version: None,
        is_delegated_worker: false,
    };

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--wait-pid" => {
                if i + 1 < args.len() {
                    opts.wait_pid = args[i + 1].parse::<u32>().ok();
                    i += 1;
                }
            }
            "--relaunch" => {
                opts.is_relaunch = true;
            }
            "--no-relaunch" | "--no-launch" => {
                opts.is_relaunch = false;
            }
            "--target-exe" => {
                if i + 1 < args.len() {
                    opts.target_exe = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            "--install-dir" => {
                if i + 1 < args.len() {
                    opts.install_dir = Some(PathBuf::from(&args[i + 1]));
                    i += 1;
                }
            }
            "--version" => {
                if i + 1 < args.len() {
                    opts.target_version = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            "--delegated-worker" => {
                opts.is_delegated_worker = true;
            }
            _ => {}
        }
        i += 1;
    }

    opts
}

pub fn resolve_default_install_dir(
    install_dir: Option<PathBuf>,
    target_exe: Option<&PathBuf>,
) -> PathBuf {
    install_dir
        .or_else(|| {
            target_exe
                .and_then(|p| p.parent().map(|d| d.to_path_buf()))
                .filter(|d| !d.to_string_lossy().to_lowercase().contains("agm-updater"))
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
        })
}

pub fn resolve_default_target_exe(
    target_exe: Option<PathBuf>,
    resolved_install_dir: &Path,
) -> PathBuf {
    if let Some(exe) = target_exe {
        if !exe
            .to_string_lossy()
            .to_lowercase()
            .contains("agm-update-cli")
            && !exe.to_string_lossy().to_lowercase().contains("agm-updater")
        {
            return exe;
        }
    }

    #[cfg(target_os = "windows")]
    {
        let exe_cand = resolved_install_dir.join("agm-alim.exe");
        if exe_cand.exists() {
            exe_cand
        } else {
            resolved_install_dir.join("agm-alim.exe")
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
}

/// Locate the best CLI source binary (`agm.exe` / `agm`) and copy it to both:
/// 1. Persistent dedicated Update CLI: `%LOCALAPPDATA%\agm-cli\agm-update-cli.exe`
/// 2. Isolated per-run temp Update CLI: `%TEMP%\agm-updater\agm-update-cli-<pid>.exe`
pub fn prepare_isolated_update_cli(caller_pid: u32) -> Result<PathBuf, String> {
    let current_exe =
        env::current_exe().map_err(|e| format!("Cannot locate current executable: {}", e))?;
    let current_dir = current_exe
        .parent()
        .map(|d| d.to_path_buf())
        .unwrap_or_else(|| PathBuf::from("."));

    let bin_name = if cfg!(target_os = "windows") {
        "agm.exe"
    } else {
        "agm"
    };

    let mut candidates: Vec<PathBuf> = Vec::new();
    candidates.push(current_dir.join(bin_name));

    #[cfg(target_os = "windows")]
    if let Ok(local) = env::var("LOCALAPPDATA") {
        let local_path = PathBuf::from(&local);
        candidates.push(local_path.join("agm-cli").join("agm-update-cli.exe"));
        candidates.push(local_path.join("Programs").join("agm-alim").join("agm.exe"));
        candidates.push(local_path.join("agm-cli").join("agm.exe"));
    }

    #[cfg(not(target_os = "windows"))]
    if let Ok(home) = env::var("HOME") {
        let home_path = PathBuf::from(&home);
        candidates.push(home_path.join(".local").join("bin").join("agm-update-cli"));
        candidates.push(home_path.join(".local").join("bin").join("agm"));
    }

    let current_mtime = fs::metadata(&current_exe).and_then(|m| m.modified()).ok();

    let updater_src = candidates
        .into_iter()
        .find(|p| {
            if !p.exists() {
                return false;
            }
            if let (Some(cur_t), Ok(cand_meta)) = (current_mtime, fs::metadata(p)) {
                if let Ok(cand_t) = cand_meta.modified() {
                    if cur_t.duration_since(cand_t).unwrap_or_default() > Duration::from_secs(300) {
                        return false;
                    }
                }
            }
            true
        })
        .unwrap_or_else(|| current_exe.clone());

    // Maintain persistent dedicated `agm-update-cli` (and `agm.exe` if source is `agm.exe`) in agm-cli folder
    #[cfg(target_os = "windows")]
    if let Ok(local) = env::var("LOCALAPPDATA") {
        let cli_dir = PathBuf::from(local).join("agm-cli");
        let _ = fs::create_dir_all(&cli_dir);
        let dedicated_cli = cli_dir.join("agm-update-cli.exe");
        if updater_src != dedicated_cli {
            let _ = fs::copy(&updater_src, &dedicated_cli);
        }
        let sibling_agm = current_dir.join("agm.exe");
        let global_agm = cli_dir.join("agm.exe");
        if sibling_agm.exists() && sibling_agm != global_agm {
            let _ = fs::copy(&sibling_agm, &global_agm);
        }
    }

    let temp_dir = env::temp_dir().join("agm-updater");
    fs::create_dir_all(&temp_dir)
        .map_err(|e| format!("Failed to create temp updater dir {:?}: {}", temp_dir, e))?;

    let temp_name = if cfg!(target_os = "windows") {
        format!("agm-update-cli-{}.exe", caller_pid)
    } else {
        format!("agm-update-cli-{}", caller_pid)
    };
    let temp_cli_path = temp_dir.join(temp_name);

    fs::copy(&updater_src, &temp_cli_path).map_err(|e| {
        format!(
            "Failed to copy updater {:?} -> {:?}: {}",
            updater_src, temp_cli_path, e
        )
    })?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&temp_cli_path, fs::Permissions::from_mode(0o755));
    }

    Ok(temp_cli_path)
}

/// Spawn the isolated `agm-update-cli` process in its own console / session without `cmd.exe /c start` quote mangling.
pub fn spawn_delegated_update_cli(
    temp_cli_path: &Path,
    wait_pid: u32,
    install_dir: &Path,
    target_exe: &Path,
    is_relaunch: bool,
    target_version: Option<&str>,
) -> Result<u32, String> {
    let mut cmd = Command::new(temp_cli_path);
    cmd.arg("delegate-update")
        .arg("--delegated-worker")
        .arg("--wait-pid")
        .arg(wait_pid.to_string())
        .arg("--install-dir")
        .arg(install_dir.to_string_lossy().as_ref())
        .arg("--target-exe")
        .arg(target_exe.to_string_lossy().as_ref());

    if is_relaunch {
        cmd.arg("--relaunch");
    } else {
        cmd.arg("--no-relaunch");
    }

    if let Some(ver) = target_version {
        if !ver.trim().is_empty() {
            cmd.arg("--version").arg(ver.trim());
        }
    }

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NEW_CONSOLE: u32 = 0x00000010;
        const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;
        cmd.creation_flags(CREATE_NEW_CONSOLE | CREATE_NEW_PROCESS_GROUP);
    }

    let child = cmd
        .spawn()
        .map_err(|e| format!("Failed to spawn delegated update CLI: {}", e))?;

    Ok(child.id())
}

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
            let _ = fs::create_dir_all(&cli_dir);
            if installed_agm.exists() {
                let _ = fs::copy(&installed_agm, cli_dir.join("agm.exe"));
                let _ = fs::copy(&installed_agm, cli_dir.join("agm-update-cli.exe"));
            } else if installed_ui.exists() {
                let _ = fs::copy(&installed_ui, cli_dir.join("agm-update-cli.exe"));
            }
        }
    }

    update_ok
}

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
                let _ = Command::new("powershell.exe")
                    .args(["-NoProfile", "-Command", &ps_launch])
                    .spawn();
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        let _ = Command::new("open").arg(&exe_to_launch).spawn();
        println!("[OK] Antigravity Manager UI launched via open.");
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        let mut cmd = Command::new(&exe_to_launch);
        if resolved_install_dir.exists() {
            cmd.current_dir(&resolved_install_dir);
        }
        let _ = cmd.spawn();
        println!("[OK] Antigravity Manager UI launched.");
    }
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
    is_ui_process_running_excluding(0)
}

pub fn is_ui_process_running_excluding(exclude_pid: u32) -> bool {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("tasklist.exe")
            .args(["/FI", "IMAGENAME eq agm-alim.exe", "/NH"])
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
                if trimmed.contains("agm-alim.exe") {
                    if exclude_pid > 0 && trimmed.contains(&exclude_pid.to_string()) {
                        continue;
                    }
                    return true;
                }
            }
            false
        } else {
            false
        }
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
        let _ = Command::new("taskkill.exe")
            .args(["/F", "/IM", "agm-alim.exe", "/FI", &filter])
            .output();
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = exclude_pid;
        let _ = Command::new("pkill").args(["-f", "agm-alim"]).output();
    }
}

fn print_help() {
    println!("AGM Delegated Out-of-Process Update CLI:");
    println!("  agm delegate-update [--wait-pid <PID>] [--install-dir <PATH>] [--target-exe <PATH>] [--relaunch]");
    println!("  agm open-ui [--target-exe <PATH>] [--install-dir <PATH>]");
    println!();
    println!("Description:");
    println!("  3-Stage Self-Delegating CLI Update Pipeline:");
    println!("  1. Copies CLI to isolated %TEMP%\\agm-updater\\agm-update-cli-<pid>.exe and waits for UI exit.");
    println!("  2. Executes `agm-update-cli update --force --no-launch --install-dir <PATH>`.");
    println!("  3. Executes `agm-update-cli open-ui --target-exe <PATH>` to automatically reopen the updated UI.");
    println!();
    println!("Options:");
    println!("  --wait-pid <PID>     Wait for specified GUI process ID to exit before updating");
    println!("  --install-dir <PATH> Exact application installation directory to update");
    println!("  --target-exe <PATH>  Specific UI executable path to launch after update");
    println!("  --relaunch           Relaunch UI upon update completion (default: true)");
    println!("  --no-relaunch        Do not relaunch UI upon update completion");
    println!("  --version <VER>      Target release version to install");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_delegate_args() {
        let args = vec![
            "--wait-pid".to_string(),
            "4321".to_string(),
            "--install-dir".to_string(),
            "C:\\Users\\Test\\AppData\\Local\\Programs\\agm-alim".to_string(),
            "--target-exe".to_string(),
            "C:\\Users\\Test\\AppData\\Local\\Programs\\agm-alim\\agm-alim.exe".to_string(),
            "--relaunch".to_string(),
            "--delegated-worker".to_string(),
        ];
        let opts = parse_delegate_args(&args);
        assert_eq!(opts.wait_pid, Some(4321));
        assert!(opts.is_relaunch);
        assert!(opts.is_delegated_worker);
        assert_eq!(
            opts.install_dir.unwrap().to_string_lossy(),
            "C:\\Users\\Test\\AppData\\Local\\Programs\\agm-alim"
        );
        assert_eq!(
            opts.target_exe.unwrap().to_string_lossy(),
            "C:\\Users\\Test\\AppData\\Local\\Programs\\agm-alim\\agm-alim.exe"
        );
    }

    #[test]
    fn test_resolve_default_target_exe_filters_temp_updater() {
        let install_dir = PathBuf::from("C:\\Users\\Test\\AppData\\Local\\Programs\\agm-alim");
        let temp_exe = Some(PathBuf::from(
            "C:\\Temp\\agm-updater\\agm-update-cli-999.exe",
        ));
        let resolved = resolve_default_target_exe(temp_exe, &install_dir);
        assert!(resolved.to_string_lossy().contains("agm-alim"));
        assert!(!resolved.to_string_lossy().contains("agm-update-cli"));
    }

    #[test]
    fn test_prepare_isolated_update_cli_creates_temp_binary() {
        let res = prepare_isolated_update_cli(987654);
        assert!(res.is_ok(), "prepare_isolated_update_cli failed: {:?}", res);
        let path = res.unwrap();
        assert!(path.exists());
        assert!(path.to_string_lossy().contains("agm-update-cli-987654"));
        let _ = fs::remove_file(&path);
    }
}
