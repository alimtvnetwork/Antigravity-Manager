use crate::modules::{account, auto_switcher, config, instance, repo_db};

use super::*;

/// Check and handle CLI arguments passed from terminal.
/// Returns true if a CLI command was handled (caller should exit).
pub fn handle_cli_arguments() -> bool {
    let args: Vec<String> = std::env::args().collect();
    if args.len() <= 1 {
        return false;
    }

    #[cfg(target_os = "windows")]
    attach_parent_console();

    let raw_cmd = &args[1];
    let cmd = raw_cmd
        .trim_start_matches('/')
        .trim_start_matches('-')
        .to_lowercase();
    let rest_args = if args.len() > 2 {
        args[2..].to_vec()
    } else {
        Vec::new()
    };

    match cmd.as_str() {
        // Multi-Instance & Profile commands
        "instances" | "instance" | "intrance" | "intrances" | "profile" | "profiles" | "ls" => {
            handle_instance_subcommand(&rest_args);
            true
        }

        // Direct create commands (e.g. agm create, instance-create, intrance-create, create-profile)
        "create" | "create-instance" | "create_instance" | "instance-create"
        | "intrance-create" | "intrance_create" | "create-profile" | "cp" => {
            handle_instance_create(&rest_args);
            true
        }

        // Direct switch commands (e.g. agm switch, switch-account, swtich, swtich-account)
        "switch" | "switch-account" | "switch_account" | "account-switch" | "swtich"
        | "swtich-account" | "swtich_account" | "account-swtich" => {
            handle_switch_command(&rest_args);
            true
        }

        // Direct auto-switch daemon commands
        "auto-switch" | "auto-swtich" | "autoswitch" | "auto_switch" | "auto" | "switcher" => {
            handle_auto_switch_command(&rest_args);
            true
        }

        "list-profiles" | "lp" => {
            handle_instance_subcommand(&["list".to_string()]);
            true
        }

        "copy-profile" | "dp" => {
            if rest_args.len() < 2 {
                eprintln!("Error: Usage: antigravity-manager copy-profile <source_id> <new_name>");
                std::process::exit(1);
            }
            let source_id = &rest_args[0];
            let resolved_src =
                instance::resolve_instance_id(source_id).unwrap_or_else(|_| source_id.clone());
            let target_name = rest_args[1..].join(" ");
            match instance::copy_instance(&resolved_src, target_name, None) {
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

        "delete-profile" => {
            if rest_args.is_empty() {
                eprintln!("Error: Usage: antigravity-manager delete-profile <profile_id>");
                std::process::exit(1);
            }
            let profile_id = &rest_args[0];
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

        "run-profile" => {
            if rest_args.is_empty() {
                eprintln!("Error: Usage: antigravity-manager run-profile <profile_id>");
                std::process::exit(1);
            }
            let profile_id = &rest_args[0];
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

        "fast-forward" | "ff" => {
            println!("[CLI] Initiating fast-forward profile rotation...");
            let rt = match tokio::runtime::Runtime::new() {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Error creating async runtime: {}", e);
                    std::process::exit(1);
                }
            };

            let target_spec = rest_args.first().map(|s| s.as_str());
            match rt.block_on(auto_switcher::trigger_manual_rotation_for_instance(
                target_spec,
            )) {
                Ok(msg) => {
                    println!("[CLI] Fast-forward success: {}", msg);
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("[CLI] Fast-forward failed: {}", e);
                    std::process::exit(1);
                }
            }
        }

        "profile-help" | "help-profile" => {
            print_instance_cli_help();
            std::process::exit(0);
        }

        "help" | "h" => {
            if let Some(sub) = rest_args.first() {
                let sub_low = sub.trim_start_matches('-').to_lowercase();
                match sub_low.as_str() {
                    "instance" | "instances" | "intrance" | "create" => print_instance_cli_help(),
                    "switch" | "switch-account" | "account" => print_switch_cli_help(),
                    "auto-switch" | "autoswitch" | "auto" => print_auto_switch_cli_help(),
                    "agy" | "cache" | "clean" => print_agy_cli_help(),
                    "supabase" | "sb" => print_supabase_cli_help(),
                    "prompts" | "prompt" | "tree" => print_prompts_cli_help(),
                    "doctor" | "check" | "health" => print_doctor_cli_help(),
                    _ => print_all_cli_help(),
                }
            } else {
                print_all_cli_help();
            }
            std::process::exit(0);
        }

        "agy" | "agi" | "cli" => {
            if rest_args.is_empty() {
                print_agy_cli_help();
                std::process::exit(0);
            }
            handle_agy_subcommand(&rest_args);
            true
        }

        "cache-clear" | "clean-agy" | "clear-agy" => {
            handle_clear_action(&rest_args, 10);
            true
        }

        "cache-clear-keep-one" | "ccko" => {
            handle_clear_action(&rest_args, 1);
            true
        }

        "cache-clear-keep-five" | "cckf" => {
            handle_clear_action(&rest_args, 5);
            true
        }

        "undo" => {
            let tx_id = rest_args.first().map(|s| s.as_str());
            execute_agy_undo(tx_id);
            true
        }

        "delegate-update" | "update-ui" | "ui-update-runner" => {
            crate::modules::delegate_updater::run(&rest_args);
            std::process::exit(0);
        }

        "update" | "update-all" | "ua" => {
            let ok = crate::modules::delegate_updater::run_cli_update(&rest_args);
            std::process::exit(if ok { 0 } else { 1 });
        }

        "open-ui" | "ui" | "launch-ui" | "start-ui" => {
            crate::modules::delegate_updater::open_ui(&rest_args);
            std::process::exit(0);
        }

        "supabase" | "sb" => {
            handle_supabase_subcommand(&rest_args);
            true
        }

        // Prompt management commands
        "prompts" | "prompt" | "tree" => {
            handle_prompts_subcommand(&rest_args, cmd.as_str());
            true
        }

        // System diagnostic doctor command
        "doctor" | "check" | "health" => {
            handle_doctor_subcommand(&rest_args);
            true
        }

        _ => false,
    }
}
