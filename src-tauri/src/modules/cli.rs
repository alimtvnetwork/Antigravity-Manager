use crate::modules::instance;

#[cfg(target_os = "windows")]
fn attach_parent_console() {
    #[link(name = "Kernel32")]
    extern "system" {
        fn AttachConsole(dw_process_id: u32) -> i32;
    }
    const ATTACH_PARENT_PROCESS: u32 = 0xFFFFFFFF;
    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

/// Check and handle CLI arguments passed from terminal.
/// Returns true if a CLI command was handled (caller should exit).
pub fn handle_cli_arguments() -> bool {
    let args: Vec<String> = std::env::args().collect();
    if args.len() <= 1 {
        return false;
    }

    #[cfg(target_os = "windows")]
    attach_parent_console();

    let cmd = args[1].as_str();

    match cmd {
        "--create-profile" | "-cp" | "create-profile" => {
            let name = if args.len() > 2 {
                args[2..].join(" ")
            } else {
                eprintln!("Error: Profile name required. Usage: antigravity-manager --create-profile <name>");
                std::process::exit(1);
            };

            match instance::create_instance(name) {
                Ok(config) => {
                    println!("[CLI] Successfully created profile:");
                    println!("  ID:       {}", config.id);
                    println!("  Name:     {}", config.name);
                    println!("  Data Dir: {}", config.data_dir);
                    println!("  Default:  {}", config.is_default);
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("Error creating profile: {}", e);
                    std::process::exit(1);
                }
            }
        }

        "--list-profiles" | "-lp" | "list-profiles" => {
            match instance::list_instances() {
                Ok(instances) => {
                    println!("\nRegistered Profiles ({} total):", instances.len());
                    println!("{:<16} {:<18} {:<16} {:<24} {}", "ID", "NAME", "STATUS", "BOUND EMAIL", "DATA DIR");
                    println!("{}", "-".repeat(95));
                    for inst in instances {
                        let status_str = if inst.is_running {
                            format!("Running (PID: {})", inst.pid.unwrap_or(0))
                        } else {
                            "Idle".to_string()
                        };
                        let email = inst.config.bound_email.unwrap_or_else(|| "-".to_string());
                        println!(
                            "{:<16} {:<18} {:<16} {:<24} {}",
                            inst.config.id, inst.config.name, status_str, email, inst.config.data_dir
                        );
                    }
                    println!();
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("Error listing profiles: {}", e);
                    std::process::exit(1);
                }
            }
        }

        "--copy-profile" | "-dp" | "copy-profile" => {
            if args.len() < 4 {
                eprintln!("Error: Usage: antigravity-manager --copy-profile <source_id> <new_name>");
                std::process::exit(1);
            }
            let source_id = &args[2];
            let target_name = args[3..].join(" ");

            match instance::copy_instance(source_id, target_name) {
                Ok(new_config) => {
                    println!("[CLI] Successfully cloned profile:");
                    println!("  ID:       {}", new_config.id);
                    println!("  Name:     {}", new_config.name);
                    println!("  Data Dir: {}", new_config.data_dir);
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("Error copying profile: {}", e);
                    std::process::exit(1);
                }
            }
        }

        "--delete-profile" | "delete-profile" => {
            if args.len() < 3 {
                eprintln!("Error: Usage: antigravity-manager --delete-profile <profile_id>");
                std::process::exit(1);
            }
            let profile_id = &args[2];
            match instance::delete_instance(profile_id) {
                Ok(_) => {
                    println!("[CLI] Profile '{}' deleted successfully.", profile_id);
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("Error deleting profile: {}", e);
                    std::process::exit(1);
                }
            }
        }

        "--run-profile" | "run-profile" => {
            if args.len() < 3 {
                eprintln!("Error: Usage: antigravity-manager --run-profile <profile_id>");
                std::process::exit(1);
            }
            let profile_id = &args[2];
            match instance::launch_instance(profile_id) {
                Ok(_) => {
                    println!("[CLI] Launched Antigravity with profile '{}'.", profile_id);
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("Error launching profile: {}", e);
                    std::process::exit(1);
                }
            }
        }

        "--profile-help" | "--help-profile" => {
            println!("Antigravity Multi-Profile CLI Commands:");
            println!("  --create-profile <name>           Create a new isolated profile");
            println!("  --list-profiles                   List all registered profiles and running PIDs");
            println!("  --copy-profile <src_id> <name>    Clone an existing profile to a new one");
            println!("  --delete-profile <id>             Delete a profile directory and registry entry");
            println!("  --run-profile <id>                Launch Antigravity using the specified profile");
            std::process::exit(0);
        }

        _ => false,
    }
}
