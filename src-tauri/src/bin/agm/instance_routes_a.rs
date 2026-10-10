//! instance_routes_a — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use chrono::Utc;

pub(crate) fn route_instances_duplicate(
    args: &[String],
    non_flag_args: &[String],
    is_json: bool,
) -> bool {
    // Subcommand: agm instances duplicate / clone
    if !non_flag_args.is_empty()
        && (non_flag_args[0].eq_ignore_ascii_case("duplicate")
            || non_flag_args[0].eq_ignore_ascii_case("clone"))
    {
        crate::instance_import_export::cmd_instance_duplicate_or_clone(args);
        return true;
    }
    false
}

pub(crate) fn route_instances_count(
    args: &[String],
    non_flag_args: &[String],
    is_json: bool,
) -> bool {
    // Subcommand: agm instances count
    if !non_flag_args.is_empty() && non_flag_args[0].eq_ignore_ascii_case("count") {
        crate::instance_import_export::cmd_instance_count(args);
        return true;
    }
    false
}

pub(crate) fn route_instances_copy_projects(
    args: &[String],
    non_flag_args: &[String],
    is_json: bool,
) -> bool {
    // Subcommand: agm instances copy-projects
    if !non_flag_args.is_empty()
        && (non_flag_args[0].eq_ignore_ascii_case("copy-projects")
            || non_flag_args[0].eq_ignore_ascii_case("copy_projects"))
    {
        crate::instance_import_export::cmd_instance_copy_projects(args);
        return true;
    }
    false
}

pub(crate) fn route_instances_copy_settings(
    args: &[String],
    non_flag_args: &[String],
    is_json: bool,
) -> bool {
    // Subcommand: agm instances copy-settings / sync-settings
    if !non_flag_args.is_empty()
        && (non_flag_args[0].eq_ignore_ascii_case("copy-settings")
            || non_flag_args[0].eq_ignore_ascii_case("copy_settings")
            || non_flag_args[0].eq_ignore_ascii_case("sync-settings")
            || non_flag_args[0].eq_ignore_ascii_case("sync_settings"))
    {
        crate::instance_import_export::cmd_instance_copy_settings(args);
        return true;
    }
    false
}

pub(crate) fn route_instances_theme(
    args: &[String],
    non_flag_args: &[String],
    is_json: bool,
) -> bool {
    // Subcommand: agm instances theme / profile-theme
    if !non_flag_args.is_empty()
        && (non_flag_args[0].eq_ignore_ascii_case("theme")
            || non_flag_args[0].eq_ignore_ascii_case("profile-theme"))
    {
        crate::instance_settings::cmd_theme(args);
        return true;
    }
    false
}

pub(crate) fn route_instances_settings(
    args: &[String],
    non_flag_args: &[String],
    is_json: bool,
) -> bool {
    // Subcommand: agm instances settings
    if !non_flag_args.is_empty()
        && (non_flag_args[0].eq_ignore_ascii_case("settings")
            || non_flag_args[0].eq_ignore_ascii_case("setting"))
    {
        crate::instance_settings::cmd_instance_settings(args);
        return true;
    }
    false
}

pub(crate) fn route_instances_observe(
    args: &[String],
    non_flag_args: &[String],
    is_json: bool,
) -> bool {
    // Subcommand: agm instances observe [target]
    if !non_flag_args.is_empty()
        && (non_flag_args[0].eq_ignore_ascii_case("observe")
            || non_flag_args[0].eq_ignore_ascii_case("inspect")
            || non_flag_args[0].eq_ignore_ascii_case("watch"))
    {
        crate::prompt_goals::cmd_observe(&args[1..]);
        return true;
    }
    false
}

pub(crate) fn route_instances_test_flow(
    args: &[String],
    non_flag_args: &[String],
    is_json: bool,
) -> bool {
    // Subcommand: agm instances test-flow
    if !non_flag_args.is_empty()
        && (non_flag_args[0].eq_ignore_ascii_case("test-flow")
            || non_flag_args[0].eq_ignore_ascii_case("test-instance")
            || non_flag_args[0].eq_ignore_ascii_case("test-switching"))
    {
        crate::test_flow::cmd_test_instance_flow(&args[1..]);
        return true;
    }
    false
}

pub(crate) fn route_instances_auto_switch(
    args: &[String],
    non_flag_args: &[String],
    is_json: bool,
) -> bool {
    // Subcommand: agm instances auto-switch [sub]
    if !non_flag_args.is_empty()
        && (non_flag_args[0].eq_ignore_ascii_case("auto-switch")
            || non_flag_args[0].eq_ignore_ascii_case("auto")
            || non_flag_args[0].eq_ignore_ascii_case("autoswitch"))
    {
        crate::scheduler_cmds::cmd_auto_switch(&args[1..]);
        return true;
    }
    false
}

pub(crate) fn route_instances_assign(
    args: &[String],
    non_flag_args: &[String],
    is_json: bool,
) -> bool {
    // Subcommand: agm instances assign <target_inst> <repo_path...>
    if non_flag_args.len() >= 3
        && (non_flag_args[0].eq_ignore_ascii_case("assign")
            || non_flag_args[0].eq_ignore_ascii_case("bind")
            || non_flag_args[1].eq_ignore_ascii_case("assign")
            || non_flag_args[1].eq_ignore_ascii_case("bind"))
    {
        let (inst_spec, paths_slice) = if non_flag_args[0].eq_ignore_ascii_case("assign")
            || non_flag_args[0].eq_ignore_ascii_case("bind")
        {
            (non_flag_args[1].as_str(), &non_flag_args[2..])
        } else {
            (non_flag_args[0].as_str(), &non_flag_args[2..])
        };
        for repo_path in paths_slice {
            match instance::assign_project_to_instance(inst_spec, repo_path) {
                Ok(msg) => println!("[SUCCESS] {}", msg),
                Err(e) => {
                    eprintln!("[ERROR] Failed to assign project '{}': {}", repo_path, e);
                    std::process::exit(1);
                }
            }
        }
        return true;
    }
    false
}

pub(crate) fn route_instances_all_ff(
    args: &[String],
    non_flag_args: &[String],
    is_json: bool,
) -> bool {
    // Subcommand: agm instances all ff
    if non_flag_args.len() >= 2
        && non_flag_args[0].eq_ignore_ascii_case("all")
        && (non_flag_args[1].eq_ignore_ascii_case("ff")
            || non_flag_args[1].eq_ignore_ascii_case("fast-forward")
            || non_flag_args[1].eq_ignore_ascii_case("switch")
            || non_flag_args[1].eq_ignore_ascii_case("rotate"))
    {
        crate::instance_import_export::cmd_instances_all(args);
        return true;
    }
    false
}

pub(crate) fn route_instances_rm_all(
    args: &[String],
    non_flag_args: &[String],
    is_json: bool,
) -> bool {
    // Subcommand: agm instances rm-all
    if args
        .first()
        .map(|s| s.eq_ignore_ascii_case("rm-all") || s.eq_ignore_ascii_case("remove-all"))
        .unwrap_or(false)
    {
        let instances = instance::list_instances().unwrap_or_default();
        let mut removed = 0usize;
        for inst in instances {
            if inst.config.is_default || inst.config.id == "default" {
                continue;
            }
            if instance::delete_instance(&inst.config.id).is_ok() {
                println!(
                    "  [✓] Removed instance '{}' ({})",
                    inst.config.name, inst.config.id
                );
                removed += 1;
            }
        }
        println!(
            "[SUCCESS] Removed {} non-default instance(s). Default profile preserved.",
            removed
        );
        return true;
    }
    false
}

pub(crate) fn route_instances_create(
    args: &[String],
    non_flag_args: &[String],
    is_json: bool,
) -> bool {
    // Subcommand: agm instances create <name> [options]
    if non_flag_args
        .first()
        .map(|s| {
            s.eq_ignore_ascii_case("create")
                || s.eq_ignore_ascii_case("add")
                || s.eq_ignore_ascii_case("new")
        })
        .unwrap_or(false)
    {
        let is_data_only = args.iter().any(|a| {
            a.eq_ignore_ascii_case("--data-only")
                || a.eq_ignore_ascii_case("-data-only")
                || a.eq_ignore_ascii_case("--do")
                || a.eq_ignore_ascii_case("-do")
                || a.eq_ignore_ascii_case("do")
        });

        let should_launch = args.iter().any(|a| {
            a.eq_ignore_ascii_case("--launch")
                || a.eq_ignore_ascii_case("-launch")
                || a.eq_ignore_ascii_case("-l")
        });

        // Parse optional account query: --account <email|id> or -a <email|id>
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

        let name = non_flag_args
            .iter()
            .skip(1)
            .find(|s| {
                !s.eq_ignore_ascii_case("do")
                    && !target_account
                        .as_ref()
                        .map(|a| a.eq_ignore_ascii_case(s))
                        .unwrap_or(false)
                    && !from_instance
                        .as_ref()
                        .map(|f| f.eq_ignore_ascii_case(s))
                        .unwrap_or(false)
            })
            .map(|s| (*s).clone())
            .unwrap_or_else(|| format!("Instance-{}", chrono::Utc::now().timestamp() % 1000));

        let create_res = if let Some(ref source) = from_instance {
            let resolved_src =
                instance::resolve_instance_id(source).unwrap_or_else(|_| source.clone());
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
                        // Reseed cloned instance with explicit account
                        let rt =
                            tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
                        if let Ok(index) = account::load_account_index() {
                            let q_lower = acc_query.to_lowercase();
                            if let Some(target) = index.accounts.iter().find(|a| {
                                a.id == *acc_query
                                    || a.email.to_lowercase() == q_lower
                                    || a.email.to_lowercase().contains(&q_lower)
                            }) {
                                let _ = rt.block_on(instance::switch_account_to_instance(
                                    &target.id,
                                    Some(&cfg.id),
                                ));
                                cfg.bound_account_id = Some(target.id.clone());
                                cfg.bound_email = Some(target.email.clone());
                            }
                        }
                    }
                }

                if should_launch {
                    let _ = instance::launch_instance(&cfg.id);
                }

                if is_json {
                    println!("{}", serde_json::to_string_pretty(&cfg).unwrap_or_default());
                } else {
                    println!(
                        "[SUCCESS] Created instance #{}: '{}' (ID: {}, bound: {}, data_only: {}, dir: {})",
                        cfg.seq_num.unwrap_or(1),
                        cfg.name,
                        cfg.id,
                        cfg.bound_email.as_deref().unwrap_or("none"),
                        is_data_only,
                        cfg.data_dir
                    );
                    if should_launch {
                        println!("  [✓] Launched instance '{}' window", cfg.id);
                    }
                }
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to create instance: {}", e);
                std::process::exit(1);
            }
        }
        return true;
    }
    false
}

pub(crate) fn route_instances_stop(
    args: &[String],
    non_flag_args: &[String],
    is_json: bool,
) -> bool {
    // Subcommand: agm instances stop <target> OR agm instances kill <target> OR agm instances close <target>
    if non_flag_args.len() >= 2
        && (non_flag_args[0].eq_ignore_ascii_case("stop")
            || non_flag_args[0].eq_ignore_ascii_case("kill")
            || non_flag_args[0].eq_ignore_ascii_case("close")
            || non_flag_args[1].eq_ignore_ascii_case("stop")
            || non_flag_args[1].eq_ignore_ascii_case("kill")
            || non_flag_args[1].eq_ignore_ascii_case("close"))
    {
        let target_spec = if non_flag_args[0].eq_ignore_ascii_case("stop")
            || non_flag_args[0].eq_ignore_ascii_case("kill")
            || non_flag_args[0].eq_ignore_ascii_case("close")
        {
            non_flag_args[1].clone()
        } else {
            non_flag_args[0].clone()
        };

        let resolved_id = match instance::resolve_instance_id(&target_spec) {
            Ok(id) => id,
            Err(e) => {
                eprintln!(
                    "[ERROR] Could not resolve instance '{}': {}",
                    target_spec, e
                );
                std::process::exit(1);
            }
        };

        println!(
            "[*] Stopping instance '{}' (resolved from '{}')...",
            resolved_id, target_spec
        );
        match instance::close_instance(&resolved_id) {
            Ok(_) => {
                println!("[SUCCESS] Stopped instance '{}' process(es).", resolved_id);
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to stop instance '{}': {}", resolved_id, e);
                std::process::exit(1);
            }
        }
        return true;
    }
    false
}
