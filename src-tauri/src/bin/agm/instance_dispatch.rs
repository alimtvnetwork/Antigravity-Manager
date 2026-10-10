//! instance_dispatch — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};

pub(crate) fn cmd_instances(args: &[String]) {
    let non_flag_args: Vec<String> = args
        .iter()
        .filter(|a| !a.starts_with('-'))
        .cloned()
        .collect();
    if non_flag_args
        .first()
        .map(|s| s.eq_ignore_ascii_case("export"))
        .unwrap_or(false)
    {
        crate::instance_import_export::cmd_instances_export(args);
        return;
    }

    if non_flag_args
        .first()
        .map(|s| s.eq_ignore_ascii_case("import") || s.eq_ignore_ascii_case("load-json"))
        .unwrap_or(false)
    {
        crate::instance_import_export::cmd_instances_import(args);
        return;
    }

    let is_help = args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help");

    if is_help
        && non_flag_args
            .first()
            .map(|s| s.eq_ignore_ascii_case("create") || s.eq_ignore_ascii_case("add"))
            .unwrap_or(false)
    {
        println!("AGM Instance Create CLI:");
        println!("  agm instances create <name> [options]");
        println!("  agm create <name> [options]");
        println!("\nDescription:");
        println!("  Creates an isolated Antigravity IDE profile directory with its own SQLite token store,");
        println!("  machine fingerprints, extensions, and configuration without cross-contaminating Default or sibling instances.");
        println!("\nOptions:");
        println!("  --account, -a <email|id>          Bind a specific account by email or ID (defaults to next available unbound)");
        println!("  --from, -f <source_instance>      Clone from default or another instance. Omit for a new empty instance");
        println!("  --data-only, --do                 Create isolated data directory structure without cloning executable");
        println!("  --launch, -l                      Immediately launch the instance window after creation");
        println!("  --json, -j                        Output result in structured JSON format");
        println!("\nExamples:");
        println!("  agm instances create \"Worker-2\"                      # Create instance with next available account");
        println!("  agm instances create \"QA-Test\" -a dev@gmail.com     # Create instance bound to dev@gmail.com");
        println!("  agm instances create \"Stage-Clone\" --from #1         # Clone settings from instance #1");
        println!("  agm instances create \"Fast-Worker\" -a dev@gmail.com -l # Create and immediately launch");
        println!(
            "  agm create \"Backend-Dev\" --data-only                # Create data-only profile"
        );
        return;
    }

    if is_help
        && non_flag_args
            .first()
            .map(|s| s.eq_ignore_ascii_case("switch") || s.eq_ignore_ascii_case("use"))
            .unwrap_or(false)
    {
        println!("AGM Instance Switch Account CLI:");
        println!("  agm instances switch <instance> <account> [--json]");
        println!("  agm switch <instance> <account>");
        println!("\nDescription:");
        println!("  Switches an instance profile's bound account credentials directly without GUI intervention.");
        println!("  Snapshots and restores active running prompts for the target instance.");
        println!("\nArguments:");
        println!("  <instance>                        Target instance name, ID, or sequence number (#1, #2)");
        println!("  <account>                         Account email, prefix, ID, or account list number (#1, #2)");
        println!("\nExamples:");
        println!("  agm instances switch #2 dev2@gmail.com                # Switch instance #2 to dev2@gmail.com");
        println!("  agm instances switch Worker-1 dev2@gmail.com          # Switch Worker-1 to dev2@gmail.com");
        println!("  agm instances switch a-6650 acc-12345                 # Switch by IDs");
        return;
    }

    if is_help
        && non_flag_args
            .first()
            .map(|s| {
                s.eq_ignore_ascii_case("auto-switch")
                    || s.eq_ignore_ascii_case("auto")
                    || s.eq_ignore_ascii_case("autoswitch")
            })
            .unwrap_or(false)
    {
        crate::scheduler_cmds::cmd_auto_switch(&["--help".to_string()]);
        return;
    }

    if is_help
        && non_flag_args
            .first()
            .map(|s| {
                s.eq_ignore_ascii_case("ff")
                    || s.eq_ignore_ascii_case("fast-forward")
                    || s.eq_ignore_ascii_case("rotate")
            })
            .unwrap_or(false)
    {
        println!("AGM Instance Fast-Forward Account Rotation CLI:");
        println!("  agm instances ff [instance]");
        println!("  agm instances all ff");
        println!("\nDescription:");
        println!("  Evaluates rolling 4-hour quota windows and automatically rotates the instance");
        println!("  to the freshest account in the pool with maximum remaining quota and longest refill runway.");
        println!("  Immediately snapshots active prompts before rotation and restores them upon completion.");
        println!("\nArguments:");
        println!("  [instance]                        Target instance name, ID, sequence number (#1, #2), or 'all' (defaults to active)");
        println!("\nExamples:");
        println!(
            "  agm instances ff #2               # Fast-forward rotate account for instance #2"
        );
        println!("  agm instances ff Worker-1         # Fast-forward rotate account for Worker-1");
        println!("  agm instances all ff              # Fast-forward rotate accounts for ALL running instances");
        return;
    }

    if is_help {
        println!("AGM Multi-Instance & Profile CLI:");
        println!("  agm instances [ls] [--json]");
        println!("  agm instances create <name> [--account <email|id>] [--from <inst>] [--data-only] [--launch]");
        println!("  agm instances switch <instance> <account>");
        println!("  agm instances ff [instance]");
        println!("  agm instances all ff");
        println!("  agm instances auto-switch [status|enable|disable|toggle|run]");
        println!("  agm instances launch <instance>");
        println!("  agm instances stop <instance>");
        println!("  agm instances rm <instance> [--force]");
        println!("  agm instances rm-all [--force]");
        println!("  agm instances assign <instance> <repo_paths...>");
        println!("\nDescription:");
        println!(
            "  Creates, lists, manages, launches, switches accounts, auto-rotates, and isolates"
        );
        println!(
            "  Antigravity multi-instance IDE profiles with dedicated configuration, keychain,"
        );
        println!("  and state databases without cross-contaminating Default or sibling instances.");
        println!("\nAliases: agm instances, agm instance, agm ls");
        println!("\nSubcommands:");
        println!("  ls, list                          List all registered instance profiles, statuses, and bound emails (default)");
        println!(
            "  create, add <name> [options]      Create a new isolated sandbox instance profile"
        );
        println!("  duplicate, clone <src> <new>      Duplicate instance profile with optional project copying");
        println!("  count [--json]                    Display summary of total, active, and running instance counts");
        println!("  copy-projects --from <s> --to <d> Copy workspace projects and recent paths between instances");
        println!("  copy-settings --from <s> --to <d> Deep-merge theme, Antigravity, and policy settings");
        println!("  sync-settings <src> <dst>         Synchronize and deep-merge settings between instances");
        println!("  theme set <theme-id> [options]    Set workbench.colorTheme for active or specified instance");
        println!("  settings <action> [options]       Configure defaults, turboMode, planReview, export, or import");
        println!("  switch, use <inst> <account>      Switch an instance profile's bound account credentials directly");
        println!("  ff, rotate [inst]                 Fast-forward / smart-rotate account for an instance (or all)");
        println!("  auto-switch [action]              Inspect or configure background auto-profile switcher");
        println!("  launch, start <inst>              Launch Antigravity IDE for the specified instance profile");
        println!("  stop, kill, close <inst>          Safely close the running process for the specified instance");
        println!("  rm, delete <inst> [--force]       Remove an instance profile, its data directory, and executable");
        println!("  rm-all [--force]                  Remove all non-default sandbox instances");
        println!("  assign, bind <inst> <paths...>    Bind one or more project workspace folders to an instance");
        println!("  export [--file <path>]            Export sandbox instances wrapped in standard JSON envelope");
        println!("  import <path>                     Import sandbox instances from standard JSON envelope file");
        println!("\nCreate Options:");
        println!("  --account, -a <email|id>          Bind a specific account by email or ID (defaults to next available unbound)");
        println!("  --from, -f <source_instance>      Clone from default or another instance. Omit for a new empty instance");
        println!("  --data-only, --do                 Create isolated data directory structure without cloning executable");
        println!("  --launch, -l                      Immediately launch the instance window after creation");
        println!("\nDuplicate / Clone Options:");
        println!("  --copy-projects, -cp              Copy open workspace projects and recent paths to cloned instance");
        println!("  --launch, -l                      Immediately launch the instance window after duplication");
        println!("\nGeneral Options:");
        println!("  --json, -j                        Output result in structured JSON format");
        println!("  --force, -f                       Bypass confirmation prompt for destructive actions");
        println!("\nExamples:");
        println!("  agm instances                                          # List all instances and running statuses");
        println!("  agm instances create \"backend-dev\"                     # Create instance with next available account");
        println!("  agm instances duplicate #1 \"Worker-2\" --copy-projects  # Duplicate instance #1 including open projects");
        println!("  agm instances count                                    # Count total, active, and running instances");
        println!("  agm instances copy-projects --from #1 --to #2          # Copy workspace projects from #1 to #2");
        println!("  agm instances copy-settings --from #1 --to #2          # Deep-merge settings from #1 to #2");
        println!("  agm instances settings set-turbo --enable              # Enable turboMode across all instances");
        println!("  agm instances settings enforce-defaults                # Enforce performance baseline defaults");
        println!("  agm instances create \"qa-test\" -a dev@gmail.com       # Create instance bound to dev@gmail.com");
        println!("  agm instances create \"stage-clone\" --from a-6650       # Clone settings from a-6650");
        println!("  agm instances switch #2 dev2@gmail.com                 # Switch instance #2 to dev2@gmail.com");
        println!("  agm instances switch a-6650 acc-12345                  # Switch instance a-6650 to acc-12345");
        println!("  agm instances ff #2                                    # Fast-forward rotate account for instance #2");
        println!("  agm instances auto-switch status                       # Check auto-switcher daemon status");
        println!("  agm instances auto-switch toggle                       # Toggle auto-switcher daemon");
        println!("  agm instances launch #2                                # Launch instance #2");
        println!(
            "  agm instances stop #2                                  # Safely close instance #2"
        );
        println!("  agm instances rm qa-test --force                       # Force delete qa-test");
        println!("  agm instances all ff                                   # Fast-forward switch all running instances");
        return;
    }

    let is_json = args.iter().any(|a| a == "--json" || a == "-j");

    if crate::instance_routes_a::route_instances_duplicate(args, &non_flag_args, is_json) {
        return;
    }

    if crate::instance_routes_a::route_instances_count(args, &non_flag_args, is_json) {
        return;
    }

    if crate::instance_routes_a::route_instances_copy_projects(args, &non_flag_args, is_json) {
        return;
    }

    if crate::instance_routes_a::route_instances_copy_settings(args, &non_flag_args, is_json) {
        return;
    }

    if crate::instance_routes_a::route_instances_theme(args, &non_flag_args, is_json) {
        return;
    }

    if crate::instance_routes_a::route_instances_settings(args, &non_flag_args, is_json) {
        return;
    }

    if crate::instance_routes_a::route_instances_observe(args, &non_flag_args, is_json) {
        return;
    }

    if crate::instance_routes_a::route_instances_test_flow(args, &non_flag_args, is_json) {
        return;
    }

    if crate::instance_routes_a::route_instances_auto_switch(args, &non_flag_args, is_json) {
        return;
    }

    if crate::instance_routes_a::route_instances_assign(args, &non_flag_args, is_json) {
        return;
    }

    if crate::instance_routes_a::route_instances_all_ff(args, &non_flag_args, is_json) {
        return;
    }

    if crate::instance_routes_a::route_instances_rm_all(args, &non_flag_args, is_json) {
        return;
    }

    if crate::instance_routes_a::route_instances_create(args, &non_flag_args, is_json) {
        return;
    }

    if crate::instance_routes_b::route_instances_launch(args, &non_flag_args, is_json) {
        return;
    }

    if crate::instance_routes_a::route_instances_stop(args, &non_flag_args, is_json) {
        return;
    }

    if crate::instance_routes_b::route_instances_rm(args, &non_flag_args, is_json) {
        return;
    }

    if crate::instance_routes_b::route_instances_switch(args, &non_flag_args, is_json) {
        return;
    }

    if crate::instance_routes_b::route_instances_ff(args, &non_flag_args, is_json) {
        return;
    }

    // Default: agm instances [ls] [--json]
    match instance::list_instances() {
        Ok(instances) => {
            let node_alias = supabase_sync::load_config()
                .map(|c| c.node_alias)
                .unwrap_or_else(|_| email_watcher::detect_machine_name());
            let local_ip = supabase_sync::get_local_ip();

            if is_json {
                let items: Vec<serde_json::Value> = instances
                    .iter()
                    .enumerate()
                    .map(|(idx, inst)| {
                        let seq = inst.config.seq_num.unwrap_or((idx + 1) as u32);
                        let eff_email = inst.config.bound_email.clone().or_else(|| {
                            if inst.config.is_default || inst.config.id == "default" {
                                account::get_current_account()
                                    .ok()
                                    .flatten()
                                    .map(|a| a.email)
                            } else {
                                None
                            }
                        });
                        let seq_name = format!("#{} {}", seq, inst.config.name);
                        let file_path = inst
                            .config
                            .executable_path
                            .clone()
                            .unwrap_or_else(|| inst.config.data_dir.clone());
                        serde_json::json!({
                            "seq": seq,
                            "sequence_name": seq_name,
                            "id": inst.config.id,
                            "name": inst.config.name,
                            "is_default": inst.config.is_default,
                            "is_running": inst.is_running,
                            "pid": inst.pid,
                            "bound_account_id": inst.config.bound_account_id,
                            "bound_email": eff_email,
                            "node": node_alias,
                            "local_ip": local_ip,
                            "data_dir": inst.config.data_dir,
                            "executable_path": inst.config.executable_path,
                            "file_path": file_path,
                        })
                    })
                    .collect();
                let envelope = json_envelope::JsonEnvelope::new("agm/instances-export", items);
                println!(
                    "{}",
                    serde_json::to_string_pretty(&envelope).unwrap_or_else(|_| "{}".to_string())
                );
                return;
            }

            if instances.is_empty() {
                println!("No sandbox profiles registered yet.");
                return;
            }

            println!(
                "\nRegistered Sandbox Profiles ({} total) [Node: {} | IP: {}]:",
                instances.len(),
                node_alias,
                local_ip
            );
            println!(
                "{:<6} {:<10} {:<24} {:<18} {:<24} FILE / EXE PATH",
                "#", "PID", "SEQUENCE NAME", "STATUS", "BOUND ACCOUNT"
            );
            println!("{}", "-".repeat(115));

            for (idx, inst) in instances.iter().enumerate() {
                let seq = inst.config.seq_num.unwrap_or((idx + 1) as u32);
                let seq_str = format!("#{}", seq);
                let seq_name = format!("#{} {}", seq, inst.config.name);
                let pid_str = inst
                    .pid
                    .map(|p| p.to_string())
                    .unwrap_or_else(|| "-".to_string());
                let status_str = if inst.is_running {
                    format!("Running ({})", pid_str)
                } else {
                    "Idle".to_string()
                };
                let email = inst
                    .config
                    .bound_email
                    .clone()
                    .or_else(|| {
                        if inst.config.is_default || inst.config.id == "default" {
                            account::get_current_account()
                                .ok()
                                .flatten()
                                .map(|a| a.email)
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| "-".to_string());
                let file_path = inst
                    .config
                    .executable_path
                    .as_deref()
                    .unwrap_or(&inst.config.data_dir);
                println!(
                    "{:<6} {:<10} {:<24} {:<18} {:<24} {}",
                    seq_str, pid_str, seq_name, status_str, email, file_path
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
