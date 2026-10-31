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

// -----------------------------------------------------------------------------
// IDE Subcommand Handler (Export, Import, Purge-Reinstall, Install)
// -----------------------------------------------------------------------------

pub fn handle_ide_subcommand(args: &[String]) {
    if args.is_empty()
        || args
            .iter()
            .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        print_ide_cli_help();
        std::process::exit(0);
    }

    let sub = args[0].to_lowercase();
    let rest = &args[1..];

    match sub.as_str() {
        "export" | "exp" => {
            let is_json = is_flag_present(rest, &["--json", "-j"]);
            let out_flag = extract_flag_value(rest, &["--out", "-o"]);

            let non_flags: Vec<&String> = rest
                .iter()
                .filter(|a| {
                    !a.starts_with('-') && !out_flag.as_ref().map(|o| o == *a).unwrap_or(false)
                })
                .collect();

            let target_inst = non_flags.first().map(|s| s.as_str()).unwrap_or("default");

            let default_filename = format!("{}-profile.agmide", target_inst);
            let out_path_buf = out_flag
                .map(std::path::PathBuf::from)
                .unwrap_or_else(|| std::path::PathBuf::from(default_filename));

            match crate::modules::instance::export_ide_profile_bundle(
                target_inst,
                Some(&out_path_buf),
            ) {
                Ok(bundle) => {
                    if is_json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&bundle).unwrap_or_default()
                        );
                    } else {
                        println!(
                            "\n[SUCCESS] Exported IDE profile bundle for '{}':",
                            target_inst
                        );
                        println!("  File:            {}", out_path_buf.display());
                        println!("  Bundle Version:  {}", bundle.agm_bundle_version);
                        println!(
                            "  Workspaces:      {}",
                            bundle
                                .metadata
                                .item_counts
                                .get("workspaces")
                                .copied()
                                .unwrap_or(0)
                        );
                        println!(
                            "  Pinned Projects: {}",
                            bundle
                                .metadata
                                .item_counts
                                .get("pinned_projects")
                                .copied()
                                .unwrap_or(0)
                        );
                        println!(
                            "  Active Prompts:  {} (Strict FIFO)",
                            bundle
                                .metadata
                                .item_counts
                                .get("active_prompts")
                                .copied()
                                .unwrap_or(0)
                        );
                        println!("  SHA-256 Checksum: {}", bundle.metadata.bundle_checksum);
                        println!();
                    }
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("[ERROR] Bundle export failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "import" | "imp" => {
            let target_flag = extract_flag_value(rest, &["--target", "-t"]);
            let new_name_flag = extract_flag_value(rest, &["--new-name", "-n", "--name"]);
            let force = is_flag_present(rest, &["--force", "-f"]);

            let non_flags: Vec<&String> = rest
                .iter()
                .filter(|a| {
                    !a.starts_with('-')
                        && !target_flag.as_ref().map(|t| t == *a).unwrap_or(false)
                        && !new_name_flag.as_ref().map(|n| n == *a).unwrap_or(false)
                })
                .collect();

            if non_flags.is_empty() {
                eprintln!("[ERROR] Missing bundle file path.");
                eprintln!("Usage: antigravity-manager ide import <bundle_file.agmide> [--target <id>] [--new-name <name>] [--force]");
                std::process::exit(1);
            }

            let bundle_path_str = non_flags[0];
            let bundle_path = std::path::Path::new(bundle_path_str);

            let options = crate::modules::instance::bundle_types::ImportOptions {
                target_instance_id: target_flag,
                new_instance_name: new_name_flag,
                force,
            };

            match crate::modules::instance::import_ide_profile_bundle(bundle_path, options) {
                Ok(res) => {
                    println!(
                        "\n[SUCCESS] Imported IDE profile bundle into '{}' ({}):",
                        res.instance_id, res.instance_name
                    );
                    println!("  Settings & Keybindings applied: {}", res.settings_applied);
                    println!("  Projects & Sequences restored: {}", res.projects_restored);
                    println!(
                        "  Active Prompts queued:          {} (FIFO)",
                        res.prompts_queued
                    );
                    println!();
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("[ERROR] Bundle import failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "purge-reinstall" | "purge" | "clean-reinstall" => {
            let wipe_all = is_flag_present(rest, &["--wipe-all", "--wipe", "--all"]);
            let keep_accounts =
                !wipe_all && !is_flag_present(rest, &["--no-accounts", "--wipe-accounts"]);
            let force = is_flag_present(rest, &["--force", "-f"]);
            let is_json = is_flag_present(rest, &["--json", "-j"]);
            let uninstall_binaries =
                wipe_all || is_flag_present(rest, &["--uninstall-binaries", "--uninstall"]);

            let options = crate::modules::instance::purge_reinstall::PurgeOptions {
                keep_accounts,
                purge_instances: true,
                uninstall_binaries,
                force,
            };

            println!("[*] Initiating safe 4-stage Antigravity clean purge & re-provisioning...");
            match crate::modules::instance::purge_reinstall::execute_purge_protocol(options) {
                Ok(summary) => {
                    if is_json {
                        println!(
                            "{}",
                            serde_json::to_string_pretty(&summary).unwrap_or_default()
                        );
                    } else {
                        println!("\n[SUCCESS] Full purge & clean re-provisioning completed successfully:");
                        println!("  Processes Terminated:   {}", summary.processes_terminated);
                        println!(
                            "  Directories Purged:     {}",
                            summary.directories_purged.len()
                        );
                        println!(
                            "  Bytes Reclaimed:        {:.2} MB",
                            (summary.bytes_reclaimed as f64) / (1024.0 * 1024.0)
                        );
                        println!(
                            "  Accounts Safeguarded:   {}",
                            if summary.accounts_safeguarded {
                                "YES"
                            } else {
                                "NO"
                            }
                        );
                        println!();
                    }
                    std::process::exit(0);
                }
                Err(e) => {
                    eprintln!("[ERROR] Purge protocol failed: {}", e);
                    std::process::exit(1);
                }
            }
        }
        "install" | "inst" => {
            let dry_run = is_flag_present(rest, &["--dry-run", "-d"]);
            let verbose = is_flag_present(rest, &["--verbose", "-v"]);

            let non_flags: Vec<&String> = rest.iter().filter(|a| !a.starts_with('-')).collect();
            let target_str = non_flags
                .first()
                .map(|s| s.as_str())
                .unwrap_or("antigravity");

            let target = match crate::modules::ide_installer::types::IdeTarget::from_str(target_str)
            {
                Some(t) => t,
                None => {
                    eprintln!("[ERROR] Unknown IDE target '{}'.", target_str);
                    eprintln!("Valid options: antigravity, cursor, vscode");
                    std::process::exit(1);
                }
            };

            let options = crate::modules::ide_installer::types::InstallOptions { dry_run, verbose };

            println!(
                "[*] Resolving and executing installer for '{}'...",
                target.display_name()
            );

            match crate::modules::ide_installer::install_ide(target, options) {
                Ok(res) => {
                    if res.success {
                        println!(
                            "\n[SUCCESS] {} installed successfully.",
                            target.display_name()
                        );
                        println!("  Command Executed: {}", res.command_executed);
                        if let Some(ref path) = res.installed_path {
                            println!("  Detected Path:    {}", path);
                        }
                        if verbose && !res.output.is_empty() {
                            println!("  Output:\n{}", res.output);
                        }
                        println!();
                        std::process::exit(0);
                    } else {
                        eprintln!("\n[ERROR] Installation exited with failure.");
                        eprintln!("  Command: {}", res.command_executed);
                        eprintln!("  Output:\n{}", res.output);
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("[ERROR] Failed to execute IDE installation: {}", e);
                    std::process::exit(1);
                }
            }
        }
        _ => {
            eprintln!("[ERROR] Unknown ide subcommand '{}'.", sub);
            print_ide_cli_help();
            std::process::exit(1);
        }
    }
}

pub fn print_ide_cli_help() {
    println!("\nAntigravity Manager IDE Lifecycle Engine (`agm ide`):");
    println!("  agm ide export [spec] [--out <path>] [--json]");
    println!("  agm ide import <bundle_path> [--target <id>] [--new-name <name>] [--force]");
    println!("  agm ide purge-reinstall [--keep-accounts] [--wipe-all] [--force] [--json]");
    println!("  agm ide install [antigravity|cursor|vscode] [--dry-run] [--verbose]");
    println!("\nSubcommands:");
    println!("  export           Package IDE settings, projects, and FIFO prompts into an .agmide bundle");
    println!("  import           Restore and unpack an .agmide profile bundle into a new or existing profile");
    println!("  purge-reinstall  Execute 4-stage deep purge of ~/.gemini/antigravity and caches with caller guard");
    println!("  install          Automated platform package installation for Antigravity, Cursor, or VS Code");
    println!("\nOptions:");
    println!("  --out, -o <path>       Specify destination path for profile bundle export");
    println!("  --target, -t <id>      Specify target instance profile for import or merge");
    println!(
        "  --new-name, -n <name>  Specify new instance name when creating profile from bundle"
    );
    println!("  --keep-accounts        Preserve accounts.json and secure token vaults across purge (default: true)");
    println!("  --wipe-all             Complete wipe removing all credentials, caches, and installed binaries");
    println!("  --force, -f            Bypass Caller IDE Self-Preservation protection");
    println!("  --dry-run, -d          Simulate installation command without executing");
    println!("  --json, -j             Output structured JSON format");
    println!();
}
