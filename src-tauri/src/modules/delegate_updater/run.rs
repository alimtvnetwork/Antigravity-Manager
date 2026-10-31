use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use super::*;

#[cfg(target_os = "windows")]
extern "system" {
    fn AttachConsole(dw_process_id: u32) -> i32;
    fn AllocConsole() -> i32;
    fn FreeConsole() -> i32;
}

#[cfg(target_os = "windows")]
mod win32_console {
    use std::os::raw::c_void;

    const STD_OUTPUT_HANDLE: u32 = 0xFFFFFFF5;
    const STD_ERROR_HANDLE: u32 = 0xFFFFFFF4;
    const ENABLE_PROCESSED_OUTPUT: u32 = 0x0001;
    const ENABLE_WRAP_AT_EOL_OUTPUT: u32 = 0x0002;
    const ENABLE_VIRTUAL_TERMINAL_PROCESSING: u32 = 0x0004;
    const DISABLE_NEWLINE_AUTO_RETURN: u32 = 0x0008;

    #[repr(C)]
    #[allow(dead_code)]
    struct COORD {
        x: i16,
        y: i16,
    }

    #[repr(C)]
    #[allow(dead_code)]
    struct CONSOLE_FONT_INFOEX {
        cb_size: u32,
        n_font: u32,
        dw_font_size: COORD,
        font_family: u32,
        font_weight: u32,
        face_name: [u16; 32],
    }

    #[repr(C)]
    struct SMALL_RECT {
        left: i16,
        top: i16,
        right: i16,
        bottom: i16,
    }

    extern "system" {
        fn GetStdHandle(n_std_handle: u32) -> *mut c_void;
        fn GetConsoleMode(h_console_handle: *mut c_void, lp_mode: *mut u32) -> i32;
        fn SetConsoleMode(h_console_handle: *mut c_void, dw_mode: u32) -> i32;
        fn SetCurrentConsoleFontEx(
            h_console_output: *mut c_void,
            b_maximum_window: i32,
            lp_console_current_font_ex: *mut CONSOLE_FONT_INFOEX,
        ) -> i32;
        fn SetConsoleCP(w_code_page_id: u32) -> i32;
        fn SetConsoleOutputCP(w_code_page_id: u32) -> i32;
        fn SetConsoleScreenBufferSize(h_console_output: *mut c_void, dw_size: COORD) -> i32;
        fn SetConsoleWindowInfo(
            h_console_output: *mut c_void,
            b_absolute: i32,
            lp_console_window: *const SMALL_RECT,
        ) -> i32;
        fn SetConsoleTitleW(lp_console_title: *const u16) -> i32;
    }

    pub fn set_updater_title() {
        unsafe {
            let title_utf16: Vec<u16> = "Antigravity Manager — CLI Updater\0"
                .encode_utf16()
                .collect();
            SetConsoleTitleW(title_utf16.as_ptr());
        }
    }

    /// Configure console handles for standard CRLF line breaks and ANSI VT processing.
    /// CRITICAL: DISABLE_NEWLINE_AUTO_RETURN (0x0008) must NEVER be enabled.
    pub fn ensure_clean_newline_mode() -> bool {
        unsafe {
            SetConsoleOutputCP(65001);
            SetConsoleCP(65001);

            let mut all_ok = true;
            for &handle_id in &[STD_OUTPUT_HANDLE, STD_ERROR_HANDLE] {
                let handle = GetStdHandle(handle_id);
                if handle.is_null() {
                    continue;
                }
                let mut mode: u32 = 0;
                if GetConsoleMode(handle, &mut mode) != 0 {
                    let mode_with_vt = (mode
                        | ENABLE_PROCESSED_OUTPUT
                        | ENABLE_WRAP_AT_EOL_OUTPUT
                        | ENABLE_VIRTUAL_TERMINAL_PROCESSING)
                        & !DISABLE_NEWLINE_AUTO_RETURN;

                    if SetConsoleMode(handle, mode_with_vt) == 0 {
                        let mode_fallback =
                            (mode | ENABLE_PROCESSED_OUTPUT | ENABLE_WRAP_AT_EOL_OUTPUT)
                                & !DISABLE_NEWLINE_AUTO_RETURN;
                        if SetConsoleMode(handle, mode_fallback) == 0 {
                            all_ok = false;
                        }
                    }
                }
            }
            all_ok
        }
    }

    /// Setup the dedicated updater console window appearance (font, title, buffer size).
    pub fn setup_dedicated_updater_console() -> bool {
        unsafe {
            ensure_clean_newline_mode();
            set_updater_title();

            let handle = GetStdHandle(STD_OUTPUT_HANDLE);
            if handle.is_null() {
                return false;
            }

            let mut font_info: CONSOLE_FONT_INFOEX = std::mem::zeroed();
            font_info.cb_size = std::mem::size_of::<CONSOLE_FONT_INFOEX>() as u32;
            font_info.font_family = 54; // FF_MODERN (0x30) | FIXED_PITCH (0x06)
            font_info.dw_font_size.y = 18;
            font_info.font_weight = 600;
            let face_name_utf16: Vec<u16> = "Consolas\0".encode_utf16().collect();
            for (i, &c) in face_name_utf16.iter().enumerate() {
                if i < font_info.face_name.len() {
                    font_info.face_name[i] = c;
                }
            }
            if SetCurrentConsoleFontEx(handle, 0, &mut font_info) == 0 {
                // Fallback to Lucida Console if Consolas fails
                let mut fallback_font: CONSOLE_FONT_INFOEX = std::mem::zeroed();
                fallback_font.cb_size = std::mem::size_of::<CONSOLE_FONT_INFOEX>() as u32;
                fallback_font.font_family = 54;
                fallback_font.dw_font_size.y = 18;
                fallback_font.font_weight = 600;
                let lucida_utf16: Vec<u16> = "Lucida Console\0".encode_utf16().collect();
                for (i, &c) in lucida_utf16.iter().enumerate() {
                    if i < fallback_font.face_name.len() {
                        fallback_font.face_name[i] = c;
                    }
                }
                SetCurrentConsoleFontEx(handle, 0, &mut fallback_font);
            }

            // A short window makes long paths wrap into the next log line.
            let _ = SetConsoleWindowInfo(
                handle,
                1,
                &SMALL_RECT {
                    left: 0,
                    top: 0,
                    right: 20,
                    bottom: 8,
                },
            );
            let _ = SetConsoleScreenBufferSize(handle, COORD { x: 120, y: 400 });
            let _ = SetConsoleWindowInfo(
                handle,
                1,
                &SMALL_RECT {
                    left: 0,
                    top: 0,
                    right: 119,
                    bottom: 34,
                },
            );
            true
        }
    }
}

/// Enable console colors and clean newlines without altering window size or title.
#[cfg(windows)]
pub fn enable_modern_console() -> bool {
    win32_console::ensure_clean_newline_mode()
}

#[cfg(not(windows))]
pub fn enable_modern_console() -> bool {
    true
}

fn print_update_field(label: &str, value: &str) {
    println!("    {:<14} : {}", label, value);
}

pub fn run(args: &[String]) {
    #[cfg(target_os = "windows")]
    unsafe {
        // The parent console belongs to the manager and is full of tracing
        // codes. A new console keeps the update text on its own.
        if AttachConsole(0xFFFFFFFF) != 0 {
            FreeConsole();
        }
        AllocConsole();
        let _ = win32_console::setup_dedicated_updater_console();
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

    println!();
    println!("  ╔════════════════════════════════════════════════════════════╗");
    println!("  ║        Antigravity Manager — Standalone CLI Updater        ║");
    println!("  ╚════════════════════════════════════════════════════════════╝");
    println!();
    print_update_field("CLI Binary", &current_exe.display().to_string());
    print_update_field("Install Path", &resolved_install_dir.display().to_string());
    print_update_field("Target App", &resolved_target_exe.display().to_string());
    if let Some(pid) = opts.wait_pid {
        print_update_field("Waiting PID", &pid.to_string());
    }
    print_update_field("Relaunch UI", if opts.is_relaunch { "Yes" } else { "No" });
    println!("  ────────────────────────────────────────────────────────────");
    println!();

    // STAGE 1: Wait for monitored UI process (and any sibling UI instances) to shut down and release file locks
    if let Some(pid) = opts.wait_pid {
        println!(
            "  [*] Stage 1/3: Waiting for Antigravity Manager UI (PID {}) to exit cleanly...",
            pid
        );
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(10);
        while is_pid_alive(pid) && start.elapsed() < timeout {
            std::thread::sleep(Duration::from_millis(250));
        }

        if is_pid_alive(pid) {
            println!(
                "  [!] UI process (PID {}) still active after 10s. Terminating to release file locks...",
                pid
            );
            force_kill_pid(pid);
            std::thread::sleep(Duration::from_millis(800));
        } else {
            println!("  [OK] UI process exited cleanly.");
        }
    }

    // Ensure no other agm-alim.exe UI process holds locks in the target directory
    let my_pid = std::process::id();
    if is_ui_process_running_excluding(my_pid) {
        println!(
            "  [*] Closing remaining Antigravity Manager UI instances before binary replacement..."
        );
        kill_other_ui_processes(my_pid);
        std::thread::sleep(Duration::from_millis(700));
    }

    // STAGE 2: Delegate to `<self_exe> update --force --no-launch --install-dir <DIR>`
    println!(
        "  [*] Stage 2/3: Running CLI update command (`{} update --force --no-launch`)...",
        current_exe
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "agm-update-cli".to_string())
    );

    let is_pinned = opts.is_pinned
        || opts.target_version.is_some()
        || args
            .iter()
            .any(|a| a.eq_ignore_ascii_case("-isPinned") || a.eq_ignore_ascii_case("--is-pinned"));

    let mut update_args = vec![
        "--force".to_string(),
        "--no-launch".to_string(),
        "--install-dir".to_string(),
        resolved_install_dir.to_string_lossy().to_string(),
    ];
    if let Some(ref ver) = opts.target_version {
        let norm_ver = normalize_version_string(ver);
        update_args.push("--version".to_string());
        update_args.push(norm_ver);
    }
    if is_pinned {
        update_args.push("-isPinned".to_string());
    }

    let update_status = Command::new(&current_exe)
        .arg("update")
        .args(&update_args)
        .status();

    let update_ok = match update_status {
        Ok(s) if s.success() => true,
        Ok(_) => run_cli_update(&update_args),
        Err(e) => {
            // Justification: child CLI spawn failed; falling back to in-process run_cli_update execution
            crate::error::record_ignored(Err::<(), _>(e), "spawn child update command");
            run_cli_update(&update_args)
        }
    };

    #[cfg(target_os = "windows")]
    {
        win32_console::ensure_clean_newline_mode();
        win32_console::set_updater_title();
    }

    // STAGE 3: Delegate to `<self_exe> open-ui --target-exe <EXE> --install-dir <DIR>`
    if opts.is_relaunch {
        println!(
            "  [*] Stage 3/3: Reopening Antigravity Manager UI (`{} open-ui`)...",
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

        match open_status {
            Ok(s) if s.success() => {}
            Ok(_) => open_ui(&open_args),
            Err(e) => {
                // Justification: child open-ui spawn failed; falling back to in-process open_ui execution
                crate::error::record_ignored(Err::<(), _>(e), "spawn child open-ui command");
                open_ui(&open_args);
            }
        }
    }

    println!();
    println!("  ────────────────────────────────────────────────────────────");
    if update_ok {
        println!("  [OK] Update complete. The manager window was reopened.");
    } else {
        println!("  [!] Update finished with warnings. Read the lines above.");
    }
    println!("  ────────────────────────────────────────────────────────────");
    println!();
    std::thread::sleep(Duration::from_millis(1200));
}

/// Stage 2 implementation: Execute the CLI update (`agm update --force --no-launch --install-dir <DIR>`)
pub fn run_cli_update(args: &[String]) -> bool {
    let opts = parse_delegate_args(args);
    let raw_install_dir =
        resolve_default_install_dir(opts.install_dir.clone(), opts.target_exe.as_ref());
    let resolved_install_dir = {
        let stripped = strip_instance_sandbox_local_appdata(&raw_install_dir);
        if stripped != raw_install_dir {
            stripped.join("Programs").join("agm-alim")
        } else {
            raw_install_dir
        }
    };
    let normalized_target_version = opts.target_version.as_deref().map(normalize_version_string);

    println!(
        "[*] Executing AGM CLI Update for directory: {}",
        resolved_install_dir.display()
    );

    let mut update_ok = false;

    #[cfg(target_os = "windows")]
    {
        let dir_arg = format!("-InstallDir \"{}\"", resolved_install_dir.to_string_lossy());
        let ver_arg = normalized_target_version
            .as_ref()
            .map(|v| format!("-Version \"{}\"", v))
            .unwrap_or_default();

        let is_pinned = opts.is_pinned
            || normalized_target_version.is_some()
            || args.iter().any(|a| {
                a.eq_ignore_ascii_case("-isPinned") || a.eq_ignore_ascii_case("--is-pinned")
            });

        let is_pinned_arg = if is_pinned { "-isPinned" } else { "" };

        let local_script = resolved_install_dir.join("install.ps1");
        let cwd_script = Path::new("install.ps1");

        let ps_script = if local_script.exists() {
            format!(
                "& \"{}\" -Update -Force -NoLaunch {} {} {}",
                local_script.display(),
                dir_arg,
                ver_arg,
                is_pinned_arg
            )
        } else if cwd_script.exists() {
            format!(
                ".\\install.ps1 -Update -Force -NoLaunch {} {} {}",
                dir_arg, ver_arg, is_pinned_arg
            )
        } else {
            format!(
                "$s = irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1; & ([scriptblock]::Create($s)) -Update -Force -NoLaunch {} {} {}",
                dir_arg, ver_arg, is_pinned_arg
            )
        };

        let status = Command::new("powershell.exe")
            .args([
                "-NoLogo",
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &ps_script,
            ])
            .status();

        #[cfg(target_os = "windows")]
        {
            win32_console::ensure_clean_newline_mode();
            win32_console::set_updater_title();
        }

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
                // Justification: PowerShell spawn failed; logged and falling back to gitmap agm update
                crate::error::record_ignored(
                    Err::<(), _>(&e),
                    "spawn powershell installer in run_cli_update",
                );
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
        let ver_arg = normalized_target_version
            .as_ref()
            .map(|v| format!("--version \"{}\"", v))
            .unwrap_or_default();
        let sh_script = if Path::new("install.sh").exists() {
            format!(
                "bash ./install.sh --update --no-launch {} {}",
                dir_arg, ver_arg
            )
        } else {
            format!(
                "curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash -s -- --update --no-launch {} {}",
                dir_arg, ver_arg
            )
        };

        let status = Command::new("bash").args(["-c", &sh_script]).status();
        match status {
            Ok(s) if s.success() => {
                update_ok = true;
                println!("[OK] Official shell installer update completed successfully!");
            }
            Ok(_) => {}
            Err(e) => {
                // Justification: bash installer spawn failed; falling back to gitmap agm update
                crate::error::record_ignored(Err::<(), _>(e), "spawn bash installer");
            }
        }
    }

    if !update_ok && normalized_target_version.is_none() {
        let gitmap_status = Command::new("gitmap")
            .args(["agm", "update", "-y"])
            .status();
        match gitmap_status {
            Ok(s) if s.success() => {
                println!("[OK] GitMap executed Antigravity Manager update successfully!");
                update_ok = true;
            }
            Ok(_) => {}
            Err(e) => {
                // Justification: optional gitmap fallback is best-effort when gitmap is not installed
                crate::error::record_ignored(Err::<(), _>(e), "spawn gitmap agm update fallback");
            }
        }
    }

    // Synchronize %LOCALAPPDATA%\agm-cli\agm.exe & agm-update-cli.exe if updated agm.exe exists in install_dir
    #[cfg(target_os = "windows")]
    if update_ok {
        let installed_agm = resolved_install_dir.join("agm.exe");
        let installed_ui = resolved_install_dir.join("agm-alim.exe");
        if let Ok(local) = env::var("LOCALAPPDATA") {
            let real_local = strip_instance_sandbox_local_appdata(Path::new(&local));
            let cli_dir = real_local.join("agm-cli");
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
