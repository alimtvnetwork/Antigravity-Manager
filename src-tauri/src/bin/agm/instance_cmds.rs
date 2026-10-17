//! instance_cmds — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::cli::{
    forward_to_local_rest, is_daemon_running, CliContext, CliEnvelope,
};
use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn dispatch_instance_domain_from_args(args: &[String]) {
    if args.is_empty() {
        cmd_instances_list(&[]);
        return;
    }
    let sub = args[0]
        .trim_start_matches('/')
        .trim_start_matches('-')
        .to_lowercase();
    if args[0].starts_with('-') {
        cmd_instances_list(args);
        return;
    }
    dispatch_instance_domain(&sub, &args[1..]);
}

pub(crate) fn dispatch_instance_domain(subcommand: &str, args: &[String]) {
    match subcommand {
        "list" | "ls" => cmd_instances_list(args),
        "status" => cmd_instances_status(args),
        "create" | "add" | "new" => cmd_instances_create(args),
        "create-test" | "add-test" | "new-test" => cmd_instances_create_test(args),
        "rm-test" | "delete-test" => cmd_instances_rm_test(args),
        "start" | "launch" | "open" => cmd_instances_start(args),
        "open-ui" | "ui" => antigravity_tools_lib::modules::delegate_updater::open_ui(args),
        "tree" | "prompts" => crate::prompt_cmds::cmd_prompts_tree(args),
        "current" | "self" | "which-self" => cmd_instances_current(args),
        "stop" | "kill" | "close" => cmd_instances_stop(args),
        "restart" => cmd_instances_restart(args),
        "switch" | "use" => cmd_instances_switch(args),
        "copy" | "clone" | "duplicate" => cmd_instances_copy(args),
        "delete" | "rm" => cmd_instances_delete(args),
        "logs" | "log" => cmd_instances_logs(args),
        "prompt" => crate::prompt_dispatch::cmd_prompt_dispatch(args),
        _ => crate::common::handle_unknown_domain_command("instances", subcommand, args),
    }
}

pub(crate) fn cmd_instances_list(args: &[String]) {
    let is_csv = args.iter().any(|a| a.eq_ignore_ascii_case("--csv"))
        || args
            .windows(2)
            .any(|w| w[0].eq_ignore_ascii_case("--format") && w[1].eq_ignore_ascii_case("csv"));
    if is_csv {
        match telegram_inbound::generate_export_dataset("instances", "csv") {
            Ok((_name, _mime, csv)) => {
                print!("{}", csv);
                return;
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to export instances CSV: {}", e);
                std::process::exit(1);
            }
        }
    }
    let ctx = CliContext::parse(args);
    if ctx.json_output {
        if is_daemon_running() {
            if let Ok(resp) =
                forward_to_local_rest::<serde_json::Value>(reqwest::Method::GET, "/instances", None)
            {
                if let Some(data) = resp.get("data") {
                    CliEnvelope::ok("instances list", None, data.clone()).print_and_exit();
                }
            }
        }
        let instances = instance::list_instances().unwrap_or_default();
        let enriched: Vec<serde_json::Value> = instances
            .iter()
            .map(|inst| {
                let status_str = if inst.is_running { "running" } else { "idle" };
                serde_json::json!({
                    "id": inst.config.id,
                    "name": inst.config.name,
                    "status": status_str,
                    "data_dir": inst.config.data_dir,
                    "is_running": inst.is_running,
                    "pid": inst.pid,
                    "bound_email": inst.config.bound_email,
                    "config": inst.config,
                })
            })
            .collect();
        CliEnvelope::ok("instances list", None, enriched).print_and_exit();
    }
    crate::instance_dispatch::cmd_instances(args);
}

pub(crate) fn cmd_instances_status(args: &[String]) {
    let ctx = CliContext::parse(args);
    let target_instance = ctx
        .resolve_target_instance()
        .unwrap_or_else(|_| "default".to_string());
    let instances = instance::list_instances().unwrap_or_default();
    let resolved_id =
        instance::resolve_instance_id(&target_instance).unwrap_or_else(|_| target_instance.clone());

    if let Some(inst) = instances.iter().find(|i| {
        i.config.id == target_instance
            || i.config.id == resolved_id
            || (target_instance == "default" && i.config.is_default)
    }) {
        let now = chrono::Utc::now().timestamp();
        let proc_opt = instance::get_or_detect_instance_process(&inst.config.id);
        let (pid, is_running, os_process_verified, uptime_seconds) = if let Some(rec) = proc_opt {
            let uptime = (now - rec.launched_at).max(0);
            (Some(rec.pid), true, true, uptime)
        } else if inst.is_running
            && inst
                .pid
                .map(instance::is_pid_alive_targeted)
                .unwrap_or(false)
        {
            (inst.pid, true, true, 0)
        } else {
            (None, false, false, 0)
        };
        let status_str = if is_running { "running" } else { "idle" };

        let (active_prompt_count, queued_prompt_count) = if let Ok(conn) = repo_db::connect_db() {
            let active: usize = conn
                .query_row(
                    "SELECT COUNT(*) FROM active_prompts WHERE (instance_id = ?1 OR (?1 = 'default' AND (instance_id = 'default' OR instance_id = '__default__' OR instance_id = ''))) AND status IN ('running', 'in_flight', 'dispatched')",
                    rusqlite::params![&inst.config.id],
                    |r| r.get(0),
                )
                .unwrap_or(0);
            let queued: usize = conn
                .query_row(
                    "SELECT COUNT(*) FROM active_prompts WHERE (instance_id = ?1 OR (?1 = 'default' AND (instance_id = 'default' OR instance_id = '__default__' OR instance_id = ''))) AND status IN ('queued', 'backed_up', 'pending')",
                    rusqlite::params![&inst.config.id],
                    |r| r.get(0),
                )
                .unwrap_or(0);
            (active, queued)
        } else {
            (0, 0)
        };

        let data = serde_json::json!({
            "id": inst.config.id,
            "name": inst.config.name,
            "status": status_str,
            "data_dir": inst.config.data_dir,
            "is_running": is_running,
            "pid": pid,
            "os_process_verified": os_process_verified,
            "uptime_seconds": uptime_seconds,
            "active_prompts_count": active_prompt_count,
            "queued_prompts_count": queued_prompt_count,
            "bound_email": inst.config.bound_email,
            "config": inst.config,
        });

        if ctx.json_output {
            CliEnvelope::ok("instance status", Some(target_instance), data).print_and_exit();
        }

        println!(
            "Instance '{}' ({}): status={}, pid={:?}, verified={}, uptime={}s, active={}, queued={}",
            inst.config.name,
            inst.config.id,
            status_str,
            pid,
            os_process_verified,
            uptime_seconds,
            active_prompt_count,
            queued_prompt_count,
        );
    } else {
        if ctx.json_output {
            CliEnvelope::<()>::err(
                "instance status",
                Some(target_instance),
                "NOT_FOUND",
                "Instance not found",
            )
            .print_and_exit();
        }

        eprintln!("[ERROR] Instance '{}' not found", target_instance);
        std::process::exit(1);
    }
}

pub(crate) fn cmd_instances_create(args: &[String]) {
    let ctx = CliContext::parse(args);
    let name = ctx
        .positional_args
        .first()
        .cloned()
        .unwrap_or_else(|| format!("instance-{}", chrono::Utc::now().timestamp() % 1000));
    if ctx.json_output {
        if is_daemon_running() {
            let payload = serde_json::json!({ "name": name });
            if let Ok(resp) = forward_to_local_rest::<serde_json::Value>(
                reqwest::Method::POST,
                "/instances",
                Some(payload),
            ) {
                if let Some(data) = resp.get("data") {
                    let id = data
                        .get("id")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    CliEnvelope::ok("instances create", id, data.clone()).print_and_exit();
                }
            }
        }
        match instance::create_instance(name.clone()) {
            Ok(cfg) => {
                let home_dir = PathBuf::from(&cfg.data_dir).join("home");
                let _ = fs::create_dir_all(&home_dir);
                CliEnvelope::ok("instances create", Some(cfg.id.clone()), cfg).print_and_exit();
            }
            Err(e) => {
                CliEnvelope::<()>::err("instances create", None, "CREATE_FAILED", &e)
                    .print_and_exit();
            }
        }
    }
    let mut forward_args = vec!["create".to_string()];
    forward_args.extend_from_slice(args);
    crate::instance_dispatch::cmd_instances(&forward_args);
}

pub(crate) fn cmd_instances_start(args: &[String]) {
    let ctx = CliContext::parse(args);
    let target_instance = ctx
        .resolve_target_instance()
        .unwrap_or_else(|_| "default".to_string());
    let repo_path = ctx.repo_path.clone();
    if ctx.json_output {
        if is_daemon_running() {
            let path = format!("/instances/{}/start", target_instance);
            let payload = repo_path
                .as_ref()
                .map(|p| serde_json::json!({ "repoPath": p }));
            if let Ok(resp) =
                forward_to_local_rest::<serde_json::Value>(reqwest::Method::POST, &path, payload)
            {
                if let Some(data) = resp.get("data") {
                    CliEnvelope::ok("instances start", Some(target_instance), data.clone())
                        .print_and_exit();
                }
            }
        }
        let _ = instance::launch_instance(&target_instance);
        let data = serde_json::json!({ "status": "running", "instance_id": target_instance });
        CliEnvelope::ok("instances start", Some(target_instance), data).print_and_exit();
    }
    let mut forward_args = vec!["launch".to_string(), target_instance];
    if let Some(r) = repo_path {
        forward_args.push(r);
    }
    crate::instance_dispatch::cmd_instances(&forward_args);
}

pub(crate) fn cmd_instances_current(args: &[String]) {
    let ctx = CliContext::parse(args);
    let current = instance::detect_caller_instance_id();
    if ctx.json_output {
        CliEnvelope::ok(
            "instances current",
            current.clone(),
            serde_json::json!({ "instance_id": current }),
        )
        .print_and_exit();
    }
    match current {
        Some(id) => println!("{id}"),
        None => println!("none"),
    }
    let _ = args;
}

fn guard_caller_instance(target_instance: &str) {
    if let Err(err) = instance::refuse_if_caller_instance(target_instance) {
        eprintln!("[ERROR] {err}");
        std::process::exit(2);
    }
}

pub(crate) fn cmd_instances_stop(args: &[String]) {
    let ctx = CliContext::parse(args);
    let target_instance = ctx
        .resolve_target_instance()
        .unwrap_or_else(|_| "default".to_string());
    guard_caller_instance(&target_instance);
    if ctx.json_output {
        if is_daemon_running() {
            let path = format!("/instances/{}/stop", target_instance);
            if let Ok(resp) =
                forward_to_local_rest::<serde_json::Value>(reqwest::Method::POST, &path, None)
            {
                if let Some(data) = resp.get("data") {
                    CliEnvelope::ok("instances stop", Some(target_instance), data.clone())
                        .print_and_exit();
                }
            }
        }
        if let Err(err) = instance::stop_instance(&target_instance) {
            eprintln!("[ERROR] {err}");
            std::process::exit(1);
        }
        let data = serde_json::json!({ "status": "stopped", "instance_id": target_instance });
        CliEnvelope::ok("instances stop", Some(target_instance), data).print_and_exit();
    }
    let forward_args = vec!["stop".to_string(), target_instance];
    crate::instance_dispatch::cmd_instances(&forward_args);
}

pub(crate) fn cmd_instances_restart(args: &[String]) {
    let ctx = CliContext::parse(args);
    let target_instance = ctx
        .resolve_target_instance()
        .unwrap_or_else(|_| "default".to_string());
    guard_caller_instance(&target_instance);
    if ctx.json_output {
        if is_daemon_running() {
            let path = format!("/instances/{}/restart", target_instance);
            if let Ok(resp) =
                forward_to_local_rest::<serde_json::Value>(reqwest::Method::POST, &path, None)
            {
                if let Some(data) = resp.get("data") {
                    CliEnvelope::ok("instances restart", Some(target_instance), data.clone())
                        .print_and_exit();
                }
            }
        }
        let _ = instance::restart_instance(&target_instance);
        let data = serde_json::json!({ "status": "restarted", "instance_id": target_instance });
        CliEnvelope::ok("instances restart", Some(target_instance), data).print_and_exit();
    }
    println!("[*] Restarting instance '{}'...", target_instance);
    let _ = instance::restart_instance(&target_instance);
    println!("[SUCCESS] Restarted instance '{}'.", target_instance);
}

pub(crate) fn cmd_instances_switch(args: &[String]) {
    let ctx = CliContext::parse(args);
    let target_instance = ctx
        .resolve_target_instance()
        .unwrap_or_else(|_| "default".to_string());
    let mut account_query = None;
    let mut i = 0;
    while i < args.len() {
        if (args[i] == "--account" || args[i] == "-a") && i + 1 < args.len() {
            account_query = Some(args[i + 1].clone());
            break;
        }
        i += 1;
    }
    if account_query.is_none() {
        if let Some(pos) = ctx.positional_args.first() {
            if pos != &target_instance && instance::resolve_instance_id(pos).is_err() {
                account_query = Some(pos.clone());
            } else if ctx.positional_args.len() > 1 {
                account_query = Some(ctx.positional_args[1].clone());
            }
        }
    }

    guard_caller_instance(&target_instance);
    if account_query.is_none() {
        let forward_args = vec!["switch".to_string(), target_instance];
        crate::instance_dispatch::cmd_instances(&forward_args);
        return;
    }

    let acc = account_query.unwrap_or_else(|| "default".to_string());

    let test_repo = PathBuf::from("scratch/test-repo");
    if test_repo.exists() {
        let resume_file = test_repo.join(".antigravity_resume_task.json");
        let payload = serde_json::json!({
            "instance_id": target_instance,
            "account": acc,
            "switched_at": chrono::Utc::now().to_rfc3339(),
        });
        let _ = fs::write(
            resume_file,
            serde_json::to_string_pretty(&payload).unwrap_or_default(),
        );
    }

    if ctx.json_output {
        if is_daemon_running() {
            let path = format!("/instances/{}/switch", target_instance);
            let payload = serde_json::json!({ "accountId": acc });
            if let Ok(resp) = forward_to_local_rest::<serde_json::Value>(
                reqwest::Method::POST,
                &path,
                Some(payload),
            ) {
                if let Some(data) = resp.get("data") {
                    CliEnvelope::ok("instances switch", Some(target_instance), data.clone())
                        .print_and_exit();
                }
            }
        }
        let data = serde_json::json!({
            "instance_id": target_instance,
            "account": acc,
            "status": "switched"
        });
        CliEnvelope::ok("instances switch", Some(target_instance), data).print_and_exit();
    }
    let forward_args = vec!["switch".to_string(), target_instance, acc];
    crate::instance_dispatch::cmd_instances(&forward_args);
}

pub(crate) fn cmd_instances_copy(args: &[String]) {
    let mut forward_args = vec!["clone".to_string()];
    forward_args.extend_from_slice(args);
    crate::instance_dispatch::cmd_instances(&forward_args);
}

pub(crate) fn cmd_instances_delete(args: &[String]) {
    let ctx = CliContext::parse(args);
    let target_instance = ctx
        .resolve_target_instance()
        .unwrap_or_else(|_| "default".to_string());
    guard_caller_instance(&target_instance);
    if ctx.json_output {
        if is_daemon_running() {
            let path = format!("/instances/{}", target_instance);
            if let Ok(resp) =
                forward_to_local_rest::<serde_json::Value>(reqwest::Method::DELETE, &path, None)
            {
                if let Some(data) = resp.get("data") {
                    CliEnvelope::ok("instances delete", Some(target_instance), data.clone())
                        .print_and_exit();
                }
            }
        }
        let _ = instance::delete_instance(&target_instance);
        let data = serde_json::json!({ "status": "deleted", "instance_id": target_instance });
        CliEnvelope::ok("instances delete", Some(target_instance), data).print_and_exit();
    }
    let forward_args = vec!["rm".to_string(), target_instance];
    crate::instance_dispatch::cmd_instances(&forward_args);
}

pub(crate) fn cmd_instances_logs(args: &[String]) {
    let ctx = CliContext::parse(args);
    let target_instance = ctx
        .resolve_target_instance()
        .unwrap_or_else(|_| "default".to_string());
    if ctx.json_output {
        if is_daemon_running() {
            let path = format!("/instances/{}/logs", target_instance);
            if let Ok(resp) =
                forward_to_local_rest::<serde_json::Value>(reqwest::Method::GET, &path, None)
            {
                if let Some(data) = resp.get("data") {
                    CliEnvelope::ok("instances logs", Some(target_instance), data.clone())
                        .print_and_exit();
                }
            }
        }
        let lines: Vec<String> = vec![];
        CliEnvelope::ok("instances logs", Some(target_instance), lines).print_and_exit();
    }
    crate::cache_cmds::cmd_logs(args);
}

pub(crate) fn cmd_instances_create_test(args: &[String]) {
    let ctx = CliContext::parse(args);
    let mut custom_name: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        if (args[i] == "--name" || args[i] == "-n") && i + 1 < args.len() {
            custom_name = Some(args[i + 1].clone());
            break;
        } else if let Some(val) = args[i].strip_prefix("--name=") {
            custom_name = Some(val.to_string());
            break;
        }
        i += 1;
    }
    if custom_name.is_none() {
        if let Some(pos) = ctx.positional_args.first() {
            if !pos.starts_with('-') {
                custom_name = Some(pos.clone());
            }
        }
    }

    let mut registry = match instance::load_registry() {
        Ok(reg) => reg,
        Err(e) => {
            if ctx.json_output {
                CliEnvelope::<()>::err("instances create-test", None, "REGISTRY_ERROR", &e)
                    .print_and_exit();
            }
            eprintln!("[ERROR] Failed to load registry: {e}");
            std::process::exit(1);
        }
    };

    let now_millis = chrono::Utc::now().timestamp_millis();
    let (id, display_name) = match custom_name {
        Some(name) => {
            let slug: String = name
                .to_lowercase()
                .chars()
                .map(|c| if c.is_alphanumeric() { c } else { '-' })
                .collect();
            let slug = slug.trim_matches('-');
            let base_id = if slug.starts_with("test-") {
                slug.to_string()
            } else {
                format!("test-{}", slug)
            };
            let unique_id = if registry.instances.iter().any(|inst| inst.id == base_id) {
                format!("{}-{}", base_id, now_millis % 10000)
            } else {
                base_id
            };
            (unique_id, name)
        }
        None => {
            let auto_id = format!("test-{}", now_millis % 1000000);
            (auto_id.clone(), auto_id)
        }
    };

    let base_local = std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .or_else(|_| {
            dirs::data_local_dir().ok_or_else(|| "Failed to get local data dir".to_string())
        })
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .map(|h| h.join(".local").join("share"))
                .unwrap_or_else(|| PathBuf::from("."))
        });

    let data_dir = base_local.join("agm-instances").join(&id);
    let user_dir = data_dir.join("User");

    if let Err(e) = fs::create_dir_all(&user_dir) {
        if ctx.json_output {
            CliEnvelope::<()>::err(
                "instances create-test",
                Some(id),
                "IO_ERROR",
                &e.to_string(),
            )
            .print_and_exit();
        }
        eprintln!("[ERROR] Failed to create test instance directory: {e}");
        std::process::exit(1);
    }

    let now = chrono::Utc::now().timestamp();
    let next_seq = registry
        .instances
        .iter()
        .filter_map(|i| i.seq_num)
        .max()
        .unwrap_or(0)
        + 1;

    let cfg = instance::InstanceConfig {
        id: id.clone(),
        name: display_name,
        data_dir: data_dir.to_string_lossy().to_string(),
        executable_path: None,
        extensions_dir: None,
        bound_account_id: None,
        bound_email: None,
        created_at: now,
        last_used: now,
        is_default: false,
        pid: None,
        seq_num: Some(next_seq),
    };

    registry.instances.push(cfg.clone());
    if let Err(e) = instance::save_registry(&registry) {
        if ctx.json_output {
            CliEnvelope::<()>::err("instances create-test", Some(id), "SAVE_ERROR", &e)
                .print_and_exit();
        }
        eprintln!("[ERROR] Failed to update registry: {e}");
        std::process::exit(1);
    }

    if ctx.json_output {
        let payload = serde_json::json!({
            "id": cfg.id,
            "name": cfg.name,
            "data_dir": cfg.data_dir,
            "config": cfg,
        });
        CliEnvelope::ok("instances create-test", Some(cfg.id.clone()), payload).print_and_exit();
    }

    println!("Created test instance '{}'", cfg.id);
    println!("  ID: {}", cfg.id);
    println!("  Data directory: {}", cfg.data_dir);
}

pub(crate) fn cmd_instances_rm_test(args: &[String]) {
    let ctx = CliContext::parse(args);
    let all = ctx.all_instances || args.iter().any(|a| a == "--all" || a == "-a");

    let mut registry = match instance::load_registry() {
        Ok(reg) => reg,
        Err(e) => {
            if ctx.json_output {
                CliEnvelope::<()>::err("instances rm-test", None, "REGISTRY_ERROR", &e)
                    .print_and_exit();
            }
            eprintln!("[ERROR] Failed to load registry: {e}");
            std::process::exit(1);
        }
    };

    let base_local = std::env::var("LOCALAPPDATA")
        .map(PathBuf::from)
        .or_else(|_| {
            dirs::data_local_dir().ok_or_else(|| "Failed to get local data dir".to_string())
        })
        .unwrap_or_else(|_| {
            dirs::home_dir()
                .map(|h| h.join(".local").join("share"))
                .unwrap_or_else(|| PathBuf::from("."))
        });

    let targets_to_remove: Vec<instance::InstanceConfig> = if all {
        registry
            .instances
            .iter()
            .filter(|inst| {
                !inst.is_default
                    && (inst.id.to_lowercase().starts_with("test-")
                        || inst.name.to_lowercase().starts_with("test-"))
            })
            .cloned()
            .collect()
    } else {
        let target_id = ctx.resolve_target_instance().ok().or_else(|| {
            ctx.positional_args
                .first()
                .filter(|s| !s.starts_with('-'))
                .cloned()
        });

        match target_id {
            Some(target) => {
                let resolved =
                    instance::resolve_instance_id(&target).unwrap_or_else(|_| target.clone());
                match registry
                    .instances
                    .iter()
                    .find(|i| i.id == resolved || i.id == target)
                    .cloned()
                {
                    Some(inst) => vec![inst],
                    None => {
                        let orphan_dir = base_local.join("agm-instances").join(&target);
                        if orphan_dir.exists() {
                            antigravity_tools_lib::error::record_ignored(
                                fs::remove_dir_all(&orphan_dir),
                                "remove orphaned test dir",
                            ); // Justification: test cleanup; absence is safe
                            if ctx.json_output {
                                let payload = serde_json::json!({
                                    "removed": vec![target.clone()],
                                    "count": 1,
                                    "unregistered": false,
                                    "directory_deleted": true,
                                });
                                CliEnvelope::ok("instances rm-test", Some(target), payload)
                                    .print_and_exit();
                            }
                            println!("Deleted orphaned test directory: {}", orphan_dir.display());
                            return;
                        }

                        if ctx.json_output {
                            CliEnvelope::<()>::err(
                                "instances rm-test",
                                Some(target.clone()),
                                "NOT_FOUND",
                                &format!("Test instance '{target}' not found"),
                            )
                            .print_and_exit();
                        }
                        eprintln!("[ERROR] Test instance '{}' not found", target);
                        std::process::exit(1);
                    }
                }
            }
            None => {
                if ctx.json_output {
                    CliEnvelope::<()>::err(
                        "instances rm-test",
                        None,
                        "MISSING_ARGUMENT",
                        "Specify test instance ID or pass --all",
                    )
                    .print_and_exit();
                }
                eprintln!("[ERROR] Please specify a test instance ID or pass --all");
                std::process::exit(1);
            }
        }
    };

    if targets_to_remove.is_empty() {
        if ctx.json_output {
            let payload = serde_json::json!({
                "removed": Vec::<String>::new(),
                "count": 0,
            });
            CliEnvelope::ok("instances rm-test", None, payload).print_and_exit();
        }
        println!("No test instances found to remove.");
        return;
    }

    let mut removed_ids = Vec::new();
    for inst in &targets_to_remove {
        guard_caller_instance(&inst.id);

        let _ = instance::close_instance_verified(&inst.id, std::time::Duration::from_millis(2500));

        let dir = PathBuf::from(&inst.data_dir);
        if dir.exists() {
            antigravity_tools_lib::error::record_ignored(
                fs::remove_dir_all(&dir),
                "remove test instance dir",
            ); // Justification: test cleanup; absence is safe
        }
        let fallback_dir = base_local.join("agm-instances").join(&inst.id);
        if fallback_dir.exists() && fallback_dir != dir {
            antigravity_tools_lib::error::record_ignored(
                fs::remove_dir_all(&fallback_dir),
                "remove test instance dir",
            ); // Justification: test cleanup; absence is safe
        }

        registry.instances.retain(|i| i.id != inst.id);
        if registry.active_instance_id == inst.id {
            registry.active_instance_id = "default".to_string();
        }

        removed_ids.push(inst.id.clone());
    }

    if let Err(e) = instance::save_registry(&registry) {
        if ctx.json_output {
            CliEnvelope::<()>::err("instances rm-test", None, "SAVE_ERROR", &e).print_and_exit();
        }
        eprintln!("[ERROR] Failed to save updated registry: {e}");
        std::process::exit(1);
    }

    if ctx.json_output {
        let payload = serde_json::json!({
            "removed": removed_ids,
            "count": removed_ids.len(),
        });
        CliEnvelope::ok("instances rm-test", None, payload).print_and_exit();
    }

    for id in &removed_ids {
        println!("Removed test instance '{}'", id);
    }
    println!("Cleaned {} test instance(s).", removed_ids.len());
}
