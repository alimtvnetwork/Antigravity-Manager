//! install_cmds — CLI command handlers, split from agm.rs.

use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub(crate) fn cmd_install(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("AGM CLI System Installation:");
        println!("  agm install");
        println!("\nDescription:");
        println!(
            "  Installs the agm executable into the user system PATH (%LOCALAPPDATA%\\agm-cli on Windows,"
        );
        println!("  ~/.local/bin on Linux/macOS) and configures the 'agm' global command.");
        println!("\nExamples:");
        println!("  agm install                         # Install agm to system PATH");
        return;
    }

    println!("[*] Installing AGM CLI into system PATH...");

    let current_exe = match env::current_exe() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[ERROR] Failed to locate current executable: {}", e);
            std::process::exit(1);
        }
    };

    #[cfg(target_os = "windows")]
    {
        let local_app_data = match env::var("LOCALAPPDATA") {
            Ok(v) => PathBuf::from(v),
            Err(_) => {
                let user_profile = env::var("USERPROFILE").unwrap_or_else(|_| "C:\\".to_string());
                PathBuf::from(user_profile).join("AppData").join("Local")
            }
        };

        let target_dir = local_app_data.join("agm-cli");
        if let Err(e) = fs::create_dir_all(&target_dir) {
            eprintln!("[ERROR] Failed to create directory {:?}: {}", target_dir, e);
            std::process::exit(1);
        }

        let target_exe = target_dir.join("agm.exe");
        if let Err(e) = fs::copy(&current_exe, &target_exe) {
            eprintln!("[ERROR] Failed to copy binary to {:?}: {}", target_exe, e);
            std::process::exit(1);
        }
        println!("  [OK] Binary copied to {:?}", target_exe);

        // Create agm.cmd and adm.cmd helper wrappers
        let cmd_wrapper = target_dir.join("agm.cmd");
        let adm_wrapper = target_dir.join("adm.cmd");
        let cmd_content = "@echo off\r\n\"%~dp0agm.exe\" %*\r\n";
        let _ = fs::write(&cmd_wrapper, cmd_content);
        let _ = fs::write(&adm_wrapper, cmd_content);

        // Add to User PATH via registry if missing
        let target_dir_str = target_dir.to_string_lossy().to_string();
        let path_script = format!(
            "$dir = '{}'; \
             $old = [Environment]::GetEnvironmentVariable('Path', 'User'); \
             if ($old -notlike \"*$dir*\") {{ \
                 [Environment]::SetEnvironmentVariable('Path', \"$old;$dir\", 'User'); \
                 Write-Host 'PATH updated'; \
             }} else {{ Write-Host 'Already in PATH'; }}",
            target_dir_str.replace('\'', "''")
        );
        let _ = Command::new("powershell")
            .args(["-NoProfile", "-Command", &path_script])
            .output();

        // Register function in PowerShell profile
        crate::update_helpers::register_powershell_profile_function(&target_exe);

        println!("[SUCCESS] AGM & ADM CLI installed successfully!");
        println!("          You can now run 'agm' or 'adm' from any Command Prompt or PowerShell window.");
    }

    #[cfg(not(target_os = "windows"))]
    {
        let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let target_dir = PathBuf::from(home).join(".local").join("bin");
        let _ = fs::create_dir_all(&target_dir);
        let target_exe = target_dir.join("agm");
        let adm_exe = target_dir.join("adm");
        if let Err(e) = fs::copy(&current_exe, &target_exe) {
            eprintln!("[ERROR] Failed to copy binary to {:?}: {}", target_exe, e);
            std::process::exit(1);
        }
        let _ = fs::copy(&current_exe, &adm_exe);
        println!("[SUCCESS] AGM & ADM CLI installed to {:?}", target_exe);
    }

    println!("\n  💡 Next Steps & Setup Suggestions:");
    println!("    • Verify Installation:       agm version");
    println!("    • Inspect System Health:     agm doctor");
    println!("    • Explore Fleet SSH Nodes:   agm ssh nodes");
    println!("    • Clear Terminal Session:    agm clear-terminal\n");
}
