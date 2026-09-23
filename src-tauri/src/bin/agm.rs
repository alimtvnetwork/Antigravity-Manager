//! AGM - Antigravity-Manager Native Terminal CLI
//! Autonomous terminal companion for Antigravity-Manager:
//! Status monitoring, multi-instance management, fast-forward switching,
//! PATH self-installation, GitHub auto-updates, and SSH remote machine management.

use antigravity_tools_lib::modules::{account, auto_switcher, email_watcher, instance};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() <= 1 {
        print_banner();
        print_help();
        return;
    }

    let subcommand = args[1].to_lowercase();
    match subcommand.as_str() {
        "status" => cmd_status(),
        "instances" | "instance" | "ls" => cmd_instances(),
        "ff" | "smart-switch" | "fast-forward" => cmd_fast_forward(),
        "install" => cmd_install(),
        "update" => cmd_update(),
        "ssh" => {
            let ssh_args = if args.len() > 2 {
                args[2..].to_vec()
            } else {
                Vec::new()
            };
            cmd_ssh(&ssh_args);
        }
        "version" | "--version" | "-v" => {
            println!("agm v{}", VERSION);
        }
        "help" | "--help" | "-h" => {
            print_banner();
            print_help();
        }
        _ => {
            eprintln!("Unknown command: '{}'", args[1]);
            eprintln!("Run 'agm help' for available commands.");
            std::process::exit(1);
        }
    }
}

fn print_banner() {
    println!("================================================================================");
    println!("             AGM - Antigravity-Manager Native Terminal CLI                      ");
    println!(
        "             Version: v{}                                                        ",
        VERSION
    );
    println!("================================================================================");
}

fn print_help() {
    println!("Usage:");
    println!("  agm <command> [arguments] [options]");
    println!();
    println!("Commands:");
    println!("  status                      Show current node status, proxy, active profile & IP");
    println!("  instances, ls               List all registered sandbox profiles and running PIDs");
    println!("  ff, smart-switch            Trigger fast-forward rotation to freshest account");
    println!("  install                     Install 'agm' executable into system PATH and profile");
    println!("  update                      Check GitHub releases and update AGM binary");
    println!(
        "  ssh <target> [options]      Connect to remote VM via SSH or run remote auto-update"
    );
    println!("  version, -v                 Print agm CLI version");
    println!("  help, -h                    Display this help manual");
    println!();
    println!("SSH Options:");
    println!("  agm ssh <[user@]host> [-p port] [--password <pwd>] [--update]");
    println!("  --password <pwd>            Provide SSH password non-interactively");
    println!("  --update                    Auto-update or install AGM on the remote VM via SSH");
    println!("  -p <port>                   Specify custom SSH port (default: 22)");
    println!();
    println!("Examples:");
    println!("  agm status");
    println!("  agm instances");
    println!("  agm ff");
    println!("  agm ssh root@192.168.1.50");
    println!("  agm ssh 192.168.1.50 --update");
    println!();
}

fn cmd_status() {
    let machine_name = email_watcher::detect_machine_name();
    let local_ip = email_watcher::detect_local_ip();

    println!("[*] Antigravity-Manager Node Status:");
    println!("    Machine Name:    {}", machine_name);
    println!("    Local IP:        {}", local_ip);
    println!("    CLI Version:     v{}", VERSION);

    // Active Account
    match account::get_current_account() {
        Ok(Some(acc)) => {
            println!("    Active Account:  {}", acc.email);
            println!(
                "    Account Name:    {}",
                acc.name.as_deref().unwrap_or("-")
            );
        }
        Ok(None) => {
            println!("    Active Account:  (None / Default)");
        }
        Err(e) => {
            println!("    Active Account:  Error querying ({})", e);
        }
    }

    // Instances Count
    match instance::list_instances() {
        Ok(list) => {
            let running_count = list.iter().filter(|i| i.is_running).count();
            println!(
                "    Sandbox Profiles: {} configured ({} currently running)",
                list.len(),
                running_count
            );
        }
        Err(e) => {
            println!("    Sandbox Profiles: Error querying ({})", e);
        }
    }
}

fn cmd_instances() {
    match instance::list_instances() {
        Ok(instances) => {
            if instances.is_empty() {
                println!("No sandbox profiles registered yet.");
                println!("Create one via AGM GUI or 'agm-alim --create-profile <name>'.");
                return;
            }

            println!("\nRegistered Sandbox Profiles ({} total):", instances.len());
            println!(
                "{:<16} {:<20} {:<18} {:<24} {}",
                "ID", "NAME", "STATUS", "BOUND ACCOUNT", "DATA DIR"
            );
            println!("{}", "-".repeat(100));

            for inst in instances {
                let status_str = if inst.is_running {
                    format!("Running (PID: {})", inst.pid.unwrap_or(0))
                } else {
                    "Idle".to_string()
                };
                let email = inst.config.bound_email.unwrap_or_else(|| "-".to_string());
                println!(
                    "{:<16} {:<20} {:<18} {:<24} {}",
                    inst.config.id, inst.config.name, status_str, email, inst.config.data_dir
                );
            }
            println!();
        }
        Err(e) => {
            eprintln!("Error querying instances: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_fast_forward() {
    println!("[*] Triggering fast-forward account rotation...");
    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to initialize async runtime: {}", e);
            std::process::exit(1);
        }
    };

    match rt.block_on(auto_switcher::trigger_manual_rotation()) {
        Ok(result) => {
            println!("[OK] Fast-forward completed: {}", result);
            if let Ok(Some(current)) = account::get_current_account() {
                println!("     New Active Profile: {}", current.email);
            }
        }
        Err(e) => {
            eprintln!("[ERROR] Fast-forward failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn cmd_install() {
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

        // Create agm.cmd helper wrapper
        let cmd_wrapper = target_dir.join("agm.cmd");
        let cmd_content = "@echo off\r\n\"%~dp0agm.exe\" %*\r\n";
        let _ = fs::write(&cmd_wrapper, cmd_content);

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
        register_powershell_profile_function(&target_exe);

        println!("[SUCCESS] AGM CLI installed successfully!");
        println!("          You can now run 'agm' from any Command Prompt or PowerShell window.");
    }

    #[cfg(not(target_os = "windows"))]
    {
        let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let target_dir = PathBuf::from(home).join(".local").join("bin");
        let _ = fs::create_dir_all(&target_dir);
        let target_exe = target_dir.join("agm");
        if let Err(e) = fs::copy(&current_exe, &target_exe) {
            eprintln!("[ERROR] Failed to copy binary to {:?}: {}", target_exe, e);
            std::process::exit(1);
        }
        println!("[SUCCESS] AGM CLI installed to {:?}", target_exe);
    }
}

#[cfg(target_os = "windows")]
fn register_powershell_profile_function(exe_path: &Path) {
    let script = format!(
        "$profilePath = $PROFILE; \
         if ($profilePath -and (Test-Path -Path $profilePath)) {{ \
             $content = Get-Content -LiteralPath $profilePath -Raw; \
             if ($content -notmatch 'function agm\\b') {{ \
                 $entry = \"`n# agm command wrapper`nfunction agm {{ & '{exe}' @args }}`n\"; \
                 Add-Content -LiteralPath $profilePath -Value $entry; \
             }} \
         }}",
        exe = exe_path.to_string_lossy().replace('\'', "''")
    );
    let _ = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output();
}

fn cmd_update() {
    println!("[*] Checking GitHub for AGM updates...");
    let client = match reqwest::blocking::Client::builder()
        .user_agent("AGM-CLI-Updater")
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to initialize HTTP client: {}", e);
            return;
        }
    };

    let url = "https://api.github.com/repos/alimtvnetwork/Antigravity-Manager/releases/latest";
    let resp = match client.get(url).send() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to connect to GitHub releases API: {}", e);
            return;
        }
    };

    if !resp.status().is_success() {
        eprintln!("GitHub API returned HTTP status: {}", resp.status());
        return;
    }

    let json: serde_json::Value = match resp.json() {
        Ok(j) => j,
        Err(e) => {
            eprintln!("Failed to parse release response: {}", e);
            return;
        }
    };

    let tag_name = json["tag_name"]
        .as_str()
        .unwrap_or("")
        .trim_start_matches('v');
    println!("    Current Version: v{}", VERSION);
    println!("    Latest Release:  v{}", tag_name);

    if tag_name == VERSION {
        println!("[OK] AGM is already at the latest release (v{}).", VERSION);
        return;
    }

    println!(
        "[*] A new version is available: v{} -> v{}",
        VERSION, tag_name
    );
    println!("[*] Triggering automatic update installation...");

    #[cfg(target_os = "windows")]
    {
        let ps_cmd = "irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex";
        let status = Command::new("powershell")
            .args([
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                ps_cmd,
            ])
            .status();
        match status {
            Ok(s) => {
                let code = s.code().unwrap_or(1);
                if code == 0 {
                    println!("[OK] Update completed successfully!");
                } else {
                    eprintln!("[ERROR] Update script exited with code {}", code);
                }
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to run update script: {}", e);
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        let sh_cmd = "curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash";
        let _ = Command::new("sh").args(["-c", sh_cmd]).status();
    }
}

fn cmd_ssh(args: &[String]) {
    if args.is_empty() {
        eprintln!("Usage: agm ssh <[user@]host> [-p port] [--password <pwd>] [--update] [cmd...]");
        std::process::exit(1);
    }

    let mut target = String::new();
    let mut port = "22".to_string();
    let mut password: Option<String> = None;
    let mut is_update = false;
    let mut remote_cmd: Vec<String> = Vec::new();

    let mut idx = 0;
    while idx < args.len() {
        let arg = &args[idx];
        if arg == "-p" || arg == "--port" {
            if idx + 1 < args.len() {
                port = args[idx + 1].clone();
                idx += 2;
                continue;
            }
        } else if arg == "--password" {
            if idx + 1 < args.len() {
                password = Some(args[idx + 1].clone());
                idx += 2;
                continue;
            }
        } else if arg == "--update" {
            is_update = true;
            idx += 1;
            continue;
        } else if target.is_empty() && !arg.starts_with('-') {
            target = arg.clone();
            idx += 1;
            continue;
        } else {
            remote_cmd.push(arg.clone());
            idx += 1;
        }
    }

    if target.is_empty() {
        eprintln!("[ERROR] Missing SSH host target.");
        std::process::exit(1);
    }

    println!(
        "[*] Connecting to SSH target '{}' (port {})...",
        target, port
    );

    // If --update was specified, formulate remote update command
    let final_cmd = if is_update {
        println!("[*] Auto-update mode active: will execute AGM update on remote VM.");
        // Try Linux install.sh or Windows install.ps1 detection
        "curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh 2>/dev/null | bash || powershell -Command \"irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex\"".to_string()
    } else if !remote_cmd.is_empty() {
        remote_cmd.join(" ")
    } else {
        String::new()
    };

    // If password not given, check if passwordless SSH key works
    let effective_password = if password.is_some() {
        password
    } else {
        let test_res = Command::new("ssh")
            .args([
                "-o",
                "BatchMode=yes",
                "-o",
                "ConnectTimeout=4",
                "-p",
                &port,
                &target,
                "exit",
            ])
            .output();

        let needs_password = match test_res {
            Ok(out) => out.status.code().unwrap_or(1) != 0,
            Err(_) => true,
        };

        if needs_password {
            print!("Enter SSH password for '{}': ", target);
            let _ = io::stdout().flush();
            let pwd = read_password_masked();
            Some(pwd)
        } else {
            None
        }
    };

    // Execute SSH
    let mut ssh = Command::new("ssh");
    ssh.arg("-p").arg(&port);

    if let Some(ref pwd) = effective_password {
        #[cfg(target_os = "windows")]
        {
            // Set SSH_ASKPASS or invoke via sshpass if installed
            env::set_var("SSH_PASSWORD", pwd);
        }
        #[cfg(not(target_os = "windows"))]
        {
            // On Unix, check if sshpass is installed
            if Command::new("sshpass").arg("-V").output().is_ok() {
                let mut pass_cmd = Command::new("sshpass");
                pass_cmd
                    .arg("-p")
                    .arg(pwd)
                    .arg("ssh")
                    .arg("-p")
                    .arg(&port)
                    .arg(&target);
                if !final_cmd.is_empty() {
                    pass_cmd.arg(&final_cmd);
                }
                let _ = pass_cmd.status();
                return;
            }
        }
    }

    ssh.arg(&target);
    if !final_cmd.is_empty() {
        ssh.arg(&final_cmd);
    }

    ssh.stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    match ssh.status() {
        Ok(s) => {
            let code = s.code().unwrap_or(0);
            if code != 0 {
                eprintln!("[*] SSH session exited with code: {}", code);
            }
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to execute 'ssh': {}", e);
            eprintln!("Ensure OpenSSH client is installed and accessible in your system PATH.");
        }
    }
}

fn read_password_masked() -> String {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "$p = Read-Host -Prompt '' -AsSecureString; \
                 [Runtime.InteropServices.Marshal]::PtrToStringAuto([Runtime.InteropServices.Marshal]::SecureStringToBSTR($p))",
            ])
            .output();

        if let Ok(out) = output {
            let res = String::from_utf8_lossy(&out.stdout).trim().to_string();
            return res;
        }
    }

    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
    input.trim().to_string()
}
