use crate::modules::{account, auto_switcher, config, instance, repo_db};
use chrono::Utc;

use super::*;

// -----------------------------------------------------------------------------
// Instances Subcommand Handler
// -----------------------------------------------------------------------------

pub(crate) fn handle_instance_subcommand(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        print_instance_cli_help();
        std::process::exit(0);
    }

    if args.is_empty() || args[0] == "ls" || args[0] == "list" {
        let is_json = is_flag_present(args, &["--json", "-j"]);
        match instance::list_instances() {
            Ok(instances) => {
                if is_json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&instances).unwrap_or_default()
                    );
                    std::process::exit(0);
                }
                println!(
                    "\nRegistered Antigravity Profiles ({} total):",
                    instances.len()
                );
                println!(
                    "{:<5} {:<18} {:<20} {:<18} {:<28} {}",
                    "#", "ID", "NAME", "STATUS", "BOUND EMAIL", "DATA DIR"
                );
                println!("{}", "-".repeat(115));
                for (idx, inst) in instances.iter().enumerate() {
                    let seq = idx + 1;
                    let status_str = if inst.is_running {
                        format!("Running (PID: {})", inst.pid.unwrap_or(0))
                    } else {
                        "Idle".to_string()
                    };
                    let email = inst.config.bound_email.as_deref().unwrap_or("-");
                    println!(
                        "#{:<4} {:<18} {:<20} {:<18} {:<28} {}",
                        seq,
                        inst.config.id,
                        inst.config.name,
                        status_str,
                        email,
                        inst.config.data_dir
                    );
                }
                println!();
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("[ERROR] Error listing profiles: {}", e);
                std::process::exit(1);
            }
        }
    }

    let sub = args[0].to_lowercase();
    match sub.as_str() {
        "create" | "add" | "new" => {
            handle_instance_create(&args[1..]);
        }
        "switch" | "use" | "swtich" => {
            if args.len() < 3 {
                eprintln!(
                    "[ERROR] Usage: antigravity-manager instance switch <instance> <account>"
                );
                std::process::exit(1);
            }
            handle_switch_command(&[args[1].clone(), args[2].clone()]);
        }
        "launch" | "start" | "run" => {
            if args.len() < 2 {
                eprintln!("[ERROR] Usage: antigravity-manager instance launch <instance_id>");
                std::process::exit(1);
            }
            let target = &args[1];
            let resolved = instance::resolve_instance_id(target).unwrap_or_else(|_| target.clone());
            match instance::launch_instance(&resolved) {
                Ok(_) => {
                    println!("[SUCCESS] Launched instance window for '{}'.", resolved);
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to launch instance '{}': {}", resolved, e);
                    std::process::exit(1);
                }
            }
        }
        "stop" | "close" | "kill" => {
            if args.len() < 2 {
                eprintln!("[ERROR] Usage: antigravity-manager instance stop <instance_id>");
                std::process::exit(1);
            }
            let target = &args[1];
            let resolved = instance::resolve_instance_id(target).unwrap_or_else(|_| target.clone());
            match instance::stop_instance(&resolved) {
                Ok(_) => {
                    println!("[SUCCESS] Stopped instance '{}'.", resolved);
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to stop instance '{}': {}", resolved, e);
                    std::process::exit(1);
                }
            }
        }
        "delete" | "rm" | "remove" => {
            if args.len() < 2 {
                eprintln!("[ERROR] Usage: antigravity-manager instance delete <instance_id>");
                std::process::exit(1);
            }
            let target = &args[1];
            let resolved = instance::resolve_instance_id(target).unwrap_or_else(|_| target.clone());
            match instance::delete_instance(&resolved) {
                Ok(_) => {
                    println!(
                        "[SUCCESS] Deleted instance '{}' and cleaned storage.",
                        resolved
                    );
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to delete instance '{}': {}", resolved, e);
                    std::process::exit(1);
                }
            }
        }
        "copy" | "clone" => {
            if args.len() < 3 {
                eprintln!(
                    "[ERROR] Usage: antigravity-manager instance copy <source_id> <new_name>"
                );
                std::process::exit(1);
            }
            let source_id = &args[1];
            let resolved_src =
                instance::resolve_instance_id(source_id).unwrap_or_else(|_| source_id.clone());
            let target_name = args[2..].join(" ");
            match instance::copy_instance(&resolved_src, target_name, None) {
                Ok(new_config) => {
                    println!(
                        "[SUCCESS] Cloned profile '{}' -> '{}' ({})",
                        source_id, new_config.name, new_config.id
                    );
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to copy instance: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "ff" | "fast-forward" | "rotate" => {
            let target = args.get(1).map(|s| s.as_str());
            let rt = tokio::runtime::Runtime::new().expect("Failed to initialize tokio runtime");
            match rt.block_on(auto_switcher::trigger_manual_rotation_for_instance(target)) {
                Ok(msg) => {
                    println!("[SUCCESS] {}", msg);
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("[ERROR] Fast-forward failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("[ERROR] Unknown instance subcommand '{}'.", sub);
            println!("Run 'antigravity-manager instance --help' for usage instructions.");
            std::process::exit(1);
        }
    }
}

pub(crate) fn handle_instance_create(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("Antigravity Instance Create Command:");
        println!("  antigravity-manager instance create <name> [options]");
        println!("\nOptions:");
        println!("  --account, -a <email|id>    Bind specific account (defaults to next available unbound)");
        println!("  --from, -f <source_id>      Clone configuration and extensions from an existing instance");
        println!("  --launch, -l                Immediately launch instance window after creation");
        println!("  --data-only, --do           Create isolated data directory only without cloning binary");
        println!(
            "  --json, -j                  Output created instance details in structured JSON"
        );
        println!("\nExamples:");
        println!(
            "  antigravity-manager instance create \"Work-Project\" -a \"work@gmail.com\" --launch"
        );
        println!("  antigravity-manager instance create \"Dev-Sandbox\"");
        println!("  antigravity-manager instance create \"Client-Clone\" --from \"Work-Project\"");
        std::process::exit(0);
    }

    let is_json = is_flag_present(args, &["--json", "-j"]);
    let should_launch = is_flag_present(args, &["--launch", "-l"]);
    let is_data_only = is_flag_present(args, &["--data-only", "-data-only", "--do", "-do"]);

    let mut target_account: Option<String> = None;
    let mut from_instance: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        let arg_lower = args[i].to_lowercase();
        if (arg_lower == "--account" || arg_lower == "-a" || arg_lower == "--acc")
            && i + 1 < args.len()
        {
            target_account = Some(args[i + 1].clone());
            i += 2;
            continue;
        }
        if (arg_lower == "--from" || arg_lower == "-f") && i + 1 < args.len() {
            from_instance = Some(args[i + 1].clone());
            i += 2;
            continue;
        }
        i += 1;
    }

    let non_flags: Vec<&String> = args
        .iter()
        .filter(|a| {
            !a.starts_with('-')
                && !target_account
                    .as_ref()
                    .map(|acc| acc == *a)
                    .unwrap_or(false)
                && !from_instance.as_ref().map(|src| src == *a).unwrap_or(false)
                && !a.eq_ignore_ascii_case("create")
                && !a.eq_ignore_ascii_case("add")
                && !a.eq_ignore_ascii_case("new")
        })
        .collect();

    let name = if !non_flags.is_empty() {
        non_flags
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    } else {
        format!("Instance-{}", chrono::Utc::now().timestamp() % 1000)
    };

    println!("[*] Creating instance profile '{}'...", name);
    let create_res = if let Some(ref source) = from_instance {
        let resolved_src = instance::resolve_instance_id(source).unwrap_or_else(|_| source.clone());
        instance::copy_instance(&resolved_src, name.clone(), Some("full"))
    } else {
        instance::create_instance_with_account(name.clone(), target_account.as_deref())
    };

    match create_res {
        Ok(mut cfg) => {
            if !is_data_only {
                if let Ok(exe_path) = instance::clone_instance_executable(&cfg.id) {
                    cfg.executable_path = Some(exe_path);
                }
            }

            if let Some(ref acc_query) = target_account {
                if from_instance.is_some() {
                    let rt =
                        tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
                    if let Ok(index) = account::load_account_index() {
                        let q_lower = acc_query.to_lowercase();
                        if let Some(target) = index.accounts.iter().find(|a| {
                            a.id == *acc_query
                                || a.email.to_lowercase() == q_lower
                                || a.email.to_lowercase().contains(&q_lower)
                        }) {
                            // Justification: best-effort blocking wait on an async op; failure logged
                            crate::error::record_ignored(
                                rt.block_on(instance::switch_account_to_instance(
                                    &target.id,
                                    Some(&cfg.id),
                                )),
                                "block_on",
                            );
                            cfg.bound_account_id = Some(target.id.clone());
                            cfg.bound_email = Some(target.email.clone());
                        }
                    }
                }
            }

            if should_launch {
                // Justification: best-effort call; failure logged without changing control flow
                crate::error::record_ignored(instance::launch_instance(&cfg.id), "launch_instance");
            }

            if is_json {
                println!("{}", serde_json::to_string_pretty(&cfg).unwrap_or_default());
            } else {
                println!(
                    "[SUCCESS] Created profile #{}: '{}' (ID: {}, bound: {}, data_only: {}, dir: {})",
                    cfg.seq_num.unwrap_or(1),
                    cfg.name,
                    cfg.id,
                    cfg.bound_email.as_deref().unwrap_or("unbound"),
                    is_data_only,
                    cfg.data_dir
                );
                if should_launch {
                    println!("  [->] Antigravity IDE window launched immediately.");
                }
            }
            std::process::exit(0);
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to create instance: {}", e);
            std::process::exit(1);
        }
    }
}
