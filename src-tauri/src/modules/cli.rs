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

        "--list-profiles" | "-lp" | "list-profiles" => match instance::list_instances() {
            Ok(instances) => {
                println!("\nRegistered Profiles ({} total):", instances.len());
                println!(
                    "{:<16} {:<18} {:<16} {:<24} {}",
                    "ID", "NAME", "STATUS", "BOUND EMAIL", "DATA DIR"
                );
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
        },

        "--copy-profile" | "-dp" | "copy-profile" => {
            if args.len() < 4 {
                eprintln!(
                    "Error: Usage: antigravity-manager --copy-profile <source_id> <new_name>"
                );
                std::process::exit(1);
            }
            let source_id = &args[2];
            let target_name = args[3..].join(" ");

            match instance::copy_instance(source_id, target_name, None) {
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

        "--fast-forward" | "-ff" | "fast-forward" => {
            println!("[CLI] Initiating fast-forward profile rotation...");
            let rt = match tokio::runtime::Runtime::new() {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Error creating async runtime: {}", e);
                    std::process::exit(1);
                }
            };

            match rt.block_on(crate::modules::auto_switcher::trigger_manual_rotation()) {
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

        "--profile-help" | "--help-profile" => {
            print_profile_help();
            std::process::exit(0);
        }

        "help" | "--help" | "-h" => {
            print_agy_cli_help();
            std::process::exit(0);
        }

        "agy" | "agi" | "cli" => {
            let sub_args = &args[2..];
            if sub_args.is_empty() {
                print_agy_cli_help();
                std::process::exit(0);
            }
            handle_agy_subcommand(sub_args);
            true
        }

        "cache-clear" | "--cache-clear" | "clean-agy" | "clear-agy" => {
            handle_clear_action(&args[2..], 10);
            true
        }

        "cache-clear-keep-one" | "ccko" => {
            handle_clear_action(&args[2..], 1);
            true
        }

        "cache-clear-keep-five" | "cckf" => {
            handle_clear_action(&args[2..], 5);
            true
        }

        "undo" | "--undo" => {
            let tx_id = args.get(2).map(|s| s.as_str());
            execute_agy_undo(tx_id);
            true
        }

        _ => false,
    }
}

fn print_profile_help() {
    println!("Antigravity Multi-Profile CLI Commands:");
    println!("  --create-profile <name>           Create a new isolated profile");
    println!("  --list-profiles                   List all registered profiles and running PIDs");
    println!("  --copy-profile <src_id> <name>    Clone an existing profile to a new one");
    println!("  --delete-profile <id>             Delete a profile directory and registry entry");
    println!("  --run-profile <id>                Launch Antigravity using the specified profile");
    println!("  --fast-forward, -ff               Rotate immediately to next best profile with healthy credits");
}

fn print_agy_cli_help() {
    println!("================================================================================");
    println!("             Antigravity-Manager: AGY Cache & Retention CLI                      ");
    println!("================================================================================");
    println!("Usage:");
    println!("  antigravity-manager agy <command> [options]");
    println!("  .\\run.ps1 agy <command> [options]");
    println!();
    println!("Commands:");
    println!("  agy cache-clear [--keep <N>]      Prune older conversations (default: keep 10) & clear caches");
    println!(
        "  agy cache-clear-keep-one (ccko)   Keep only the 1 latest conversation, prune the rest"
    );
    println!(
        "  agy cache-clear-keep-five (cckf)  Keep only the 5 latest conversations, prune the rest"
    );
    println!("  agy undo [tx_id]                  Revert the last (or specific) conversation pruning transaction");
    println!();
    println!("Options & Flags:");
    println!("  --precheck, --pre, --preflight    Simulate and preview what would be pruned without modifying files");
    println!("  -y, --yes                         Bypass interactive confirmation prompt for automated scripting");
    println!("  --keep <N>, -k <N>                Specify number of latest conversations to retain intact");
    println!();
    println!("Temporary Staging & Safety:");
    println!(
        "  Pruned conversations are safely staged in the OS temporary directory before removal."
    );
    println!("  Run 'agy undo' to immediately restore them.");
    println!("  [NOTE] Temporary directories may be pruned by the OS over time; revert promptly if needed.");
    println!();
    print_profile_help();
}

fn is_flag_present(args: &[String], flags: &[&str]) -> bool {
    args.iter().any(|a| flags.iter().any(|f| a == f))
}

fn parse_keep_count(args: &[String], default_val: usize) -> usize {
    for (idx, arg) in args.iter().enumerate() {
        let is_keep_flag = arg == "--keep" || arg == "-k";
        if is_keep_flag {
            if let Some(val_str) = args.get(idx + 1) {
                if let Ok(parsed) = val_str.parse::<usize>() {
                    return parsed;
                }
            }
        }
        if let Ok(num) = arg.parse::<usize>() {
            return num;
        }
    }
    default_val
}

fn handle_agy_subcommand(sub_args: &[String]) {
    let sub = sub_args[0].as_str();
    match sub {
        "cache-clear" | "clear" | "clean" | "cache-clean" => {
            handle_clear_action(&sub_args[1..], 10);
        }
        "cache-clear-keep-one" | "ccko" => {
            handle_clear_action(&sub_args[1..], 1);
        }
        "cache-clear-keep-five" | "cckf" => {
            handle_clear_action(&sub_args[1..], 5);
        }
        "undo" => {
            let tx_id = sub_args.get(1).map(|s| s.as_str());
            execute_agy_undo(tx_id);
        }
        "help" | "--help" | "-h" => {
            print_agy_cli_help();
            std::process::exit(0);
        }
        _ => {
            eprintln!("Unknown AGY command: {}", sub);
            eprintln!("Run 'antigravity-manager agy help' for usage instructions.");
            std::process::exit(1);
        }
    }
}

fn handle_clear_action(extra_args: &[String], default_keep: usize) {
    let keep = parse_keep_count(extra_args, default_keep);
    let is_preflight = is_flag_present(extra_args, &["--precheck", "--pre", "--preflight", "-p"]);
    let is_yes = is_flag_present(extra_args, &["-y", "--yes", "-f"]);
    execute_agy_clear(keep, is_preflight, is_yes);
}

fn execute_agy_clear(keep_count: usize, is_preflight: bool, is_yes: bool) {
    if is_preflight {
        let report = crate::modules::agy_cleaner::preflight_check(keep_count);
        print_preflight_report(&report);
        std::process::exit(0);
    }

    if !is_yes {
        println!();
        println!("  [!] Antigravity Conversation & Cache Pruner");
        println!(
            "  Retention Plan: Keeping top {} recent conversations intact.",
            keep_count
        );
        println!(
            "  Older conversations will be safely staged in OS temp storage for undo recovery."
        );
        print!("  Proceed with pruning? [y/N]: ");
        use std::io::{self, Write};
        let _ = io::stdout().flush();
        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            println!("Aborted.");
            std::process::exit(1);
        }
        let trimmed = input.trim().to_lowercase();
        let is_confirmed = trimmed == "y" || trimmed == "yes";
        if !is_confirmed {
            println!("  [--] Operation cancelled by user. (Use -y to run non-interactively)");
            std::process::exit(0);
        }
    }

    println!(
        "  [*] Pruning older conversations and clearing cache (keep {})...",
        keep_count
    );
    match crate::modules::agy_cleaner::prune_and_clean(keep_count) {
        Ok(result) => {
            println!("  [OK] Pruning completed successfully!");
            println!("    Transaction ID      : {}", result.transaction_id);
            println!("    Conversations Kept  : {}", result.preserved_count);
            println!(
                "    Conversations Pruned: {} ({:.2} MB)",
                result.pruned_count,
                result.pruned_bytes as f64 / 1024.0 / 1024.0
            );
            println!(
                "    Cache Cleared       : {:.2} MB",
                result.cache_cleared_bytes as f64 / 1024.0 / 1024.0
            );
            println!(
                "    Total Space Freed   : {:.2} MB",
                result.total_freed_bytes as f64 / 1024.0 / 1024.0
            );
            println!("    Staging Backup Path : {}", result.staging_dir);
            println!();
            println!("  [TIP] To revert this operation, run: antigravity-manager agy undo");
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("  [ERROR] Cleanup failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn execute_agy_undo(tx_id: Option<&str>) {
    println!("  [*] Reverting conversation pruning transaction from temporary storage...");
    match crate::modules::agy_cleaner::undo_prune(tx_id) {
        Ok(result) => {
            println!("  [OK] Reversion successful!");
            println!("    Transaction ID        : {}", result.transaction_id);
            println!(
                "    Restored Conversations: {}",
                result.restored_conversations
            );
            println!(
                "    Restored Size         : {:.2} MB",
                result.restored_bytes as f64 / 1024.0 / 1024.0
            );
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("  [ERROR] Undo failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn print_preflight_report(report: &crate::modules::agy_cleaner::PreflightReport) {
    println!("================================================================================");
    println!(" [==] Antigravity Optimizer & Conversation Pre-Flight Report");
    println!("================================================================================");
    println!(
        " Total Conversations Scanned : {} ({:.2} MB)",
        report.total_conversations,
        report.total_conversation_bytes as f64 / 1024.0 / 1024.0
    );
    println!(
        " Retention Policy            : Keeping latest {} conversations intact",
        report.keep_count
    );
    println!(" Preserved Recent Convs      : {}", report.preserved_count);
    println!(
        " Older Convs to Prune        : {} (will be staged to temporary storage)",
        report.pruned_count
    );
    println!(
        " Projected Prune Reclamation : {:.2} MB",
        report.projected_reclaimed_bytes as f64 / 1024.0 / 1024.0
    );
    println!(
        " Application Cache Targets   : {} folders ({:.2} MB)",
        report.cache_paths_count,
        report.cache_bytes as f64 / 1024.0 / 1024.0
    );
    println!(
        " Total Projected Reclamation : ~{:.2} MB",
        (report.projected_reclaimed_bytes + report.cache_bytes) as f64 / 1024.0 / 1024.0
    );
    println!(" Temporary Staging Directory : {}", report.staging_dir);
    println!("--------------------------------------------------------------------------------");
    println!(" [TIP] Undo / Rollback Capability:");
    println!("  Pruned conversation steps are staged in the temporary directory.");
    println!("  To revert any operation, run: agy undo");
    println!("  [NOTE] Temporary directories may be pruned by the OS over time; revert promptly if needed.");
    println!("================================================================================");
    println!(" [NOTE] Pre-flight mode active. No files modified. No processes terminated.");
    println!("================================================================================");
}
