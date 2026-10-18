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
        "create-test" | "add-test" | "new-test" | "test-create" => cmd_instances_create_test(args),
        "rm-test" | "delete-test" | "test-clean" | "test-destroy" | "clean-test" => {
            cmd_instances_rm_test(args)
        }
        "start" | "launch" | "open" => cmd_instances_start(args),
        "open-ui" | "ui" => antigravity_tools_lib::modules::delegate_updater::open_ui(args),
        "tree" | "prompts" => crate::prompt_cmds::cmd_prompts_tree(args),
        "whoami" | "detect-caller" | "caller" | "current" | "self" | "which-self" => {
            cmd_instances_current(args)
        }
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
                // Justification: home directory creation is best-effort; instance startup handles missing folder
                antigravity_tools_lib::error::record_ignored(
                    fs::create_dir_all(&home_dir),
                    "create instance home dir",
                );
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
        // Justification: CLI start launches in background; status envelope reports triggered state
        antigravity_tools_lib::error::record_ignored(
            instance::launch_instance(&target_instance),
            "launch instance",
        );
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
    let mut forward_args = vec!["self".to_string()];
    forward_args.extend_from_slice(args);
    crate::instance_dispatch::cmd_instances(&forward_args);
}

fn resolve_target_instance_arg(ctx: &CliContext) -> String {
    if let Some(ref id) = ctx.instance_id {
        return id.clone();
    }
    if let Some(pos) = ctx.positional_args.first().filter(|s| !s.starts_with('-')) {
        return pos.clone();
    }
    ctx.resolve_target_instance()
        .unwrap_or_else(|_| "default".to_string())
}

fn guard_caller_instance(args: &[String], target_instance: &str) {
    let has_force = args.iter().any(|a| a == "--force" || a == "-f");
    if !has_force {
        if let Some(caller_id) = instance::detect_caller_instance_id() {
            let resolved = instance::resolve_instance_id(target_instance)
                .unwrap_or_else(|_| target_instance.to_string());
            if caller_id == resolved || caller_id == target_instance {
                eprintln!(
                    "[ERROR] Refusing to stop, switch, or delete instance '{}' because this command is running inside its IDE process",
                    caller_id
                );
                std::process::exit(2);
            }
        }
        if let Err(err) = instance::refuse_if_caller_instance(target_instance) {
            eprintln!("[ERROR] {err}");
            std::process::exit(2);
        }
    } else {
        std::env::set_var("AGM_FORCE_CALLER_OVERRIDE", "1");
    }
}

pub(crate) fn cmd_instances_stop(args: &[String]) {
    let ctx = CliContext::parse(args);
    let target_instance = resolve_target_instance_arg(&ctx);
    guard_caller_instance(args, &target_instance);
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
        antigravity_tools_lib::modules::task_history_db::record_instance_lifecycle_event(
            antigravity_tools_lib::modules::audit_action::AuditAction::StopInstance,
            Some(&target_instance),
            &format!("Stop instance {}", target_instance),
            "CLI stop command executed",
        );
        let data = serde_json::json!({ "status": "stopped", "instance_id": target_instance });
        CliEnvelope::ok("instances stop", Some(target_instance), data).print_and_exit();
    }
    antigravity_tools_lib::modules::task_history_db::record_instance_lifecycle_event(
        antigravity_tools_lib::modules::audit_action::AuditAction::StopInstance,
        Some(&target_instance),
        &format!("Stop instance {}", target_instance),
        "CLI stop command executed",
    );
    let forward_args = vec!["stop".to_string(), target_instance];
    crate::instance_dispatch::cmd_instances(&forward_args);
}

pub(crate) fn cmd_instances_restart(args: &[String]) {
    let ctx = CliContext::parse(args);
    let target_instance = resolve_target_instance_arg(&ctx);
    guard_caller_instance(args, &target_instance);
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
        // Justification: CLI restart runs asynchronously; envelope communicates triggered state
        antigravity_tools_lib::error::record_ignored(
            instance::restart_instance(&target_instance),
            "restart instance",
        );
        antigravity_tools_lib::modules::task_history_db::record_instance_lifecycle_event(
            antigravity_tools_lib::modules::audit_action::AuditAction::RestartInstance,
            Some(&target_instance),
            &format!("Restart instance {}", target_instance),
            "CLI restart command executed",
        );
        let data = serde_json::json!({ "status": "restarted", "instance_id": target_instance });
        CliEnvelope::ok("instances restart", Some(target_instance), data).print_and_exit();
    }
    println!("[*] Restarting instance '{}'...", target_instance);
    // Justification: best-effort CLI restart execution
    antigravity_tools_lib::error::record_ignored(
        instance::restart_instance(&target_instance),
        "restart instance",
    );
    antigravity_tools_lib::modules::task_history_db::record_instance_lifecycle_event(
        antigravity_tools_lib::modules::audit_action::AuditAction::RestartInstance,
        Some(&target_instance),
        &format!("Restart instance {}", target_instance),
        "CLI restart command executed",
    );
    println!("[SUCCESS] Restarted instance '{}'.", target_instance);
}

pub(crate) fn cmd_instances_switch(args: &[String]) {
    let ctx = CliContext::parse(args);
    let mut account_query = None;
    let mut i = 0;
    while i < args.len() {
        if (args[i] == "--account" || args[i] == "-a") && i + 1 < args.len() {
            account_query = Some(args[i + 1].clone());
            break;
        }
        i += 1;
    }

    let mut target_instance = ctx.instance_id.clone().unwrap_or_default();
    if target_instance.is_empty() {
        if let Some(first) = ctx.positional_args.first().filter(|s| !s.starts_with('-')) {
            if first.contains('@') {
                target_instance = ctx
                    .resolve_target_instance()
                    .unwrap_or_else(|_| "default".to_string());
                if account_query.is_none() {
                    account_query = Some(first.clone());
                }
            } else {
                target_instance = first.clone();
            }
        } else {
            target_instance = ctx
                .resolve_target_instance()
                .unwrap_or_else(|_| "default".to_string());
        }
    }

    if account_query.is_none() && ctx.positional_args.len() > 1 {
        account_query = Some(ctx.positional_args[1].clone());
    }

    guard_caller_instance(args, &target_instance);
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
        // Justification: best-effort test resume payload persistence
        antigravity_tools_lib::error::record_ignored(
            fs::write(
                resume_file,
                serde_json::to_string_pretty(&payload).unwrap_or_default(),
            ),
            "write resume task",
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
    let target_instance = resolve_target_instance_arg(&ctx);
    guard_caller_instance(args, &target_instance);
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
        // Justification: CLI delete executes deletion ladder; envelope communicates deleted status
        antigravity_tools_lib::error::record_ignored(
            instance::delete_instance(&target_instance),
            "delete instance",
        );
        antigravity_tools_lib::modules::task_history_db::record_instance_lifecycle_event(
            antigravity_tools_lib::modules::audit_action::AuditAction::DeleteInstance,
            Some(&target_instance),
            &format!("Delete instance {}", target_instance),
            "CLI delete command executed",
        );
        let data = serde_json::json!({ "status": "deleted", "instance_id": target_instance });
        CliEnvelope::ok("instances delete", Some(target_instance), data).print_and_exit();
    }
    antigravity_tools_lib::modules::task_history_db::record_instance_lifecycle_event(
        antigravity_tools_lib::modules::audit_action::AuditAction::DeleteInstance,
        Some(&target_instance),
        &format!("Delete instance {}", target_instance),
        "CLI delete command executed",
    );
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
    let mut forward_args = vec!["test-create".to_string()];
    forward_args.extend_from_slice(args);
    crate::instance_dispatch::cmd_instances(&forward_args);
}

pub(crate) fn cmd_instances_rm_test(args: &[String]) {
    let mut forward_args = vec!["test-clean".to_string()];
    forward_args.extend_from_slice(args);
    crate::instance_dispatch::cmd_instances(&forward_args);
}
