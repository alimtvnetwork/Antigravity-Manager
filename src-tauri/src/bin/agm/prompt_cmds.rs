//! prompt_cmds — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::cli::{
    forward_to_local_rest, is_daemon_running, CliContext, CliEnvelope,
};
use antigravity_tools_lib::modules::repo_db::ActivePrompt;
use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub(crate) fn dispatch_prompt_domain_from_args(args: &[String]) {
    if args.is_empty() {
        cmd_prompts_list(&[]);
        return;
    }
    let sub = args[0]
        .trim_start_matches('/')
        .trim_start_matches('-')
        .to_lowercase();
    if args[0].starts_with('-') {
        cmd_prompts_list(args);
        return;
    }
    dispatch_prompt_domain(&sub, &args[1..]);
}

pub(crate) fn dispatch_prompt_domain(subcommand: &str, args: &[String]) {
    match subcommand {
        "list" | "ls" => cmd_prompts_list(args),
        "tree" => cmd_prompts_tree(args),
        "inspect" | "observe" => cmd_prompts_inspect(args),
        "send" | "dispatch" => cmd_prompts_send(args),
        "running" | "wpr" => cmd_prompts_running(args),
        "queue" => cmd_prompts_queue(args),
        "history" => cmd_prompts_history(args),
        "export" => crate::prompt_import_export::cmd_prompts_export(args),
        "backup" | "brp" => cmd_prompts_backup(args),
        "restore" | "rrp" => cmd_prompts_restore(args),
        "purge" | "clean" => cmd_prompts_purge(args),
        _ => crate::common::handle_unknown_domain_command("prompts", subcommand, args),
    }
}

pub(crate) fn cmd_prompts_list(args: &[String]) {
    let ctx = CliContext::parse(args);
    if ctx.json_output {
        if is_daemon_running() {
            if let Ok(resp) =
                forward_to_local_rest::<serde_json::Value>(reqwest::Method::GET, "/prompts", None)
            {
                if let Some(data) = resp.get("data") {
                    CliEnvelope::ok("prompts list", None, data.clone()).print_and_exit();
                }
            }
        }
        let prompts = repo_db::list_active_prompts().unwrap_or_default();
        CliEnvelope::ok("prompts list", None, prompts).print_and_exit();
    }
    crate::prompt_mgmt::cmd_prompts(args);
}

pub(crate) fn cmd_prompts_tree(args: &[String]) {
    let ctx = CliContext::parse(args);
    let target_instance = ctx
        .resolve_target_instance()
        .unwrap_or_else(|_| "default".to_string());
    let repo_filter = ctx.repo_path.as_deref();
    if ctx.json_output {
        if is_daemon_running() {
            let path = format!("/prompts/tree?instanceId={}", target_instance);
            if let Ok(resp) =
                forward_to_local_rest::<serde_json::Value>(reqwest::Method::GET, &path, None)
            {
                if let Some(data) = resp.get("data") {
                    CliEnvelope::ok("prompts tree", Some(target_instance), data.clone())
                        .print_and_exit();
                }
            }
        }
        let tree =
            repo_db::get_project_conversation_tree_for_instance(&target_instance, repo_filter)
                .unwrap_or_default();
        CliEnvelope::ok("prompts tree", Some(target_instance), tree).print_and_exit();
    }
    crate::misc_cmds::cmd_tree(args);
}

pub(crate) fn cmd_prompts_inspect(args: &[String]) {
    crate::prompt_goals::cmd_observe(args);
}

pub(crate) fn cmd_prompts_send(args: &[String]) {
    let ctx = CliContext::parse(args);
    let target_instance = ctx
        .resolve_target_instance()
        .unwrap_or_else(|_| "default".to_string());
    let resolved_inst =
        instance::resolve_instance_id(&target_instance).unwrap_or_else(|_| target_instance.clone());
    let repo_path = ctx
        .repo_path
        .clone()
        .unwrap_or_else(|| "scratch/test-repo".to_string());

    let raw_prompt = if ctx.instance_id.is_none() && ctx.positional_args.len() >= 2 {
        let first = &ctx.positional_args[0];
        if first.starts_with('#')
            || first.starts_with("inst-")
            || first == "default"
            || instance::resolve_instance_id(first).is_ok()
        {
            ctx.positional_args[1..].join(" ")
        } else {
            ctx.positional_args.join(" ")
        }
    } else {
        ctx.positional_args.join(" ")
    };

    let mut final_prompt = raw_prompt;
    if let Some(ref pref) = ctx.prefix {
        final_prompt = format!("{} {}", pref, final_prompt);
    }
    if let Some(ref suff) = ctx.suffix {
        final_prompt = format!("{} {}", final_prompt, suff);
    }

    let now = chrono::Utc::now().timestamp();
    let prompt_id = format!("prompt-{}", uuid::Uuid::new_v4());
    let model = ctx
        .model
        .clone()
        .unwrap_or_else(|| "gemini-3.8-flash-high".to_string());

    // Query smart process cache / OS process table
    let cached_proc = instance::get_or_detect_instance_process(&resolved_inst);
    let is_already_alive = cached_proc.is_some();
    let process_action = if is_already_alive {
        "cache_hit_dispatched"
    } else {
        "cold_launched"
    };

    // Guarantee instance running and focused without relaunching existing IDE windows
    let active_pid =
        match instance::ensure_instance_running_for_dispatch(&resolved_inst, Some(&repo_path)) {
            Ok(pid) => Some(pid),
            Err(_) => cached_proc.map(|r| r.pid),
        };

    let active_prompt = ActivePrompt {
        id: prompt_id.clone(),
        project_id: repo_path.clone(),
        instance_id: resolved_inst.clone(),
        repo_path: repo_path.clone(),
        prompt_content: final_prompt.clone(),
        model: Some(model),
        session_id: Some(format!("conv-{}", uuid::Uuid::new_v4())),
        status: "in_flight".to_string(),
        created_at: now,
        updated_at: now,
        image_payload: None,
    };

    let _ = repo_db::insert_active_prompt(&active_prompt);

    let target_dir = PathBuf::from(&repo_path);
    let _ = fs::create_dir_all(&target_dir);
    let task_file = target_dir.join(".antigravity_resume_task.json");
    let task_payload = serde_json::json!({
        "prompt_id": prompt_id,
        "instance_id": resolved_inst,
        "repo_path": repo_path,
        "prompt_content": final_prompt,
        "auto_boot": true,
        "dispatched_at": now,
        "pid": active_pid,
        "process_action": process_action,
    });
    let _ = fs::write(
        task_file,
        serde_json::to_string_pretty(&task_payload).unwrap_or_default(),
    );

    let res_data = serde_json::json!({
        "id": active_prompt.id,
        "project_id": active_prompt.project_id,
        "instance_id": active_prompt.instance_id,
        "repo_path": active_prompt.repo_path,
        "prompt_content": active_prompt.prompt_content,
        "model": active_prompt.model,
        "session_id": active_prompt.session_id,
        "status": active_prompt.status,
        "created_at": active_prompt.created_at,
        "updated_at": active_prompt.updated_at,
        "image_payload": active_prompt.image_payload,
        "pid": active_pid,
        "process_action": process_action,
    });

    if ctx.json_output {
        CliEnvelope::ok("prompt send", Some(target_instance), res_data).print_and_exit();
    }

    println!(
        "[SUCCESS] Prompt dispatched to instance '{}' (id: {}, action: {}, pid: {:?}).",
        target_instance, prompt_id, process_action, active_pid
    );
}

pub(crate) fn cmd_prompts_running(args: &[String]) {
    let ctx = CliContext::parse(args);
    let target_instance = ctx
        .resolve_target_instance()
        .unwrap_or_else(|_| "default".to_string());
    let resolved_inst =
        instance::resolve_instance_id(&target_instance).unwrap_or_else(|_| target_instance.clone());

    // Filter candidate prompts against dead PIDs and completed transcripts using is_terminal_done
    let proc_opt = instance::get_or_detect_instance_process(&resolved_inst);
    let is_instance_proc_alive = proc_opt.is_some() || {
        let instances = instance::list_instances().unwrap_or_default();
        instances
            .iter()
            .find(|i| {
                i.config.id == resolved_inst || (resolved_inst == "default" && i.config.is_default)
            })
            .and_then(|i| i.pid)
            .map(instance::is_pid_alive_targeted)
            .unwrap_or(false)
    };

    let running: Vec<repo_db::ActivePrompt> = if !is_instance_proc_alive {
        Vec::new()
    } else {
        let raw_running =
            repo_db::list_running_prompts_for_instance(&resolved_inst).unwrap_or_default();
        raw_running
            .into_iter()
            .filter(|p| {
                let is_active =
                    repo_db::is_prompt_running_for_project(&p.repo_path, &resolved_inst)
                        || repo_db::is_prompt_running_for_project(&p.project_id, &resolved_inst);
                let is_fresh_in_flight =
                    p.status == "in_flight" && (chrono::Utc::now().timestamp() - p.updated_at) < 60;
                is_active || is_fresh_in_flight
            })
            .collect()
    };

    if ctx.json_output {
        CliEnvelope::ok("prompt running", Some(target_instance), running).print_and_exit();
    }

    if running.is_empty() {
        println!("Instance '{}': 0 active/running prompts.", target_instance);
    } else {
        println!(
            "Instance '{}': {} active/running prompt(s):",
            target_instance,
            running.len()
        );
        for (i, p) in running.iter().enumerate() {
            println!(
                "  [{}] {} ({}) - {}",
                i + 1,
                p.id,
                p.status,
                p.prompt_content.chars().take(60).collect::<String>()
            );
        }
    }
}

pub(crate) fn cmd_prompts_queue(args: &[String]) {
    let ctx = CliContext::parse(args);
    let target_instance = ctx
        .resolve_target_instance()
        .unwrap_or_else(|_| "default".to_string());
    let resolved_inst =
        instance::resolve_instance_id(&target_instance).unwrap_or_else(|_| target_instance.clone());
    let repo_path = ctx
        .repo_path
        .clone()
        .unwrap_or_else(|| "scratch/test-repo".to_string());

    let raw_prompt = if ctx.instance_id.is_none() && ctx.positional_args.len() >= 2 {
        let first = &ctx.positional_args[0];
        if first.starts_with('#')
            || first.starts_with("inst-")
            || first == "default"
            || instance::resolve_instance_id(first).is_ok()
        {
            ctx.positional_args[1..].join(" ")
        } else {
            ctx.positional_args.join(" ")
        }
    } else {
        ctx.positional_args.join(" ")
    };

    let mut final_prompt = raw_prompt;
    if let Some(ref pref) = ctx.prefix {
        final_prompt = format!("{} {}", pref, final_prompt);
    }
    if let Some(ref suff) = ctx.suffix {
        final_prompt = format!("{} {}", final_prompt, suff);
    }

    let now = chrono::Utc::now().timestamp();
    let prompt_id = format!("queued-{}-{}", resolved_inst, uuid::Uuid::new_v4());
    let model = ctx
        .model
        .clone()
        .unwrap_or_else(|| "gemini-3.8-flash-high".to_string());

    let conn_res = repo_db::connect_db();
    if let Ok(ref conn) = conn_res {
        let _ = conn.execute(
            "INSERT INTO active_prompts (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, NULL, 'queued', ?7, ?8)",
            rusqlite::params![
                &prompt_id,
                &repo_path,
                &resolved_inst,
                &repo_path,
                &final_prompt,
                &model,
                now,
                now,
            ],
        );
    }

    let queue_position: usize = if let Ok(ref conn) = conn_res {
        conn.query_row(
            "SELECT COUNT(*) FROM active_prompts
             WHERE (instance_id = ?1 OR (?1 = 'default' AND (instance_id = 'default' OR instance_id = '__default__' OR instance_id = '')))
               AND status IN ('queued', 'pending', 'backed_up')
               AND (created_at < ?2 OR (created_at = ?2 AND id <= ?3))",
            rusqlite::params![&resolved_inst, now, &prompt_id],
            |r| r.get(0),
        )
        .unwrap_or(1)
    } else {
        1
    };

    let data = serde_json::json!({
        "id": prompt_id,
        "instance_id": resolved_inst,
        "repo_path": repo_path,
        "prompt_content": final_prompt,
        "status": "queued",
        "queue_position": queue_position,
        "created_at": now,
        "fifo_ordered": true,
    });

    if ctx.json_output {
        CliEnvelope::ok("prompt queue", Some(target_instance), data).print_and_exit();
    }

    println!(
        "[SUCCESS] Prompt queued for instance '{}' at position #{} (id: {}).",
        target_instance, queue_position, prompt_id
    );
}

pub(crate) fn cmd_prompts_history(args: &[String]) {
    let ctx = CliContext::parse(args);
    if ctx.json_output {
        let hist = repo_db::list_active_prompts().unwrap_or_default();
        CliEnvelope::ok("prompts history", None, hist).print_and_exit();
    }
    crate::misc_cmds::cmd_history(args);
}

pub(crate) fn cmd_prompts_backup(args: &[String]) {
    let ctx = CliContext::parse(args);
    let target_instance = ctx
        .resolve_target_instance()
        .unwrap_or_else(|_| "default".to_string());
    if ctx.json_output {
        if is_daemon_running() {
            let payload = serde_json::json!({ "instanceId": target_instance });
            let _ = forward_to_local_rest::<serde_json::Value>(
                reqwest::Method::POST,
                "/prompts/backup",
                Some(payload),
            );
        }
        let _ = repo_db::backup_running_prompts(&target_instance);
        let data = serde_json::json!({ "status": "backed_up", "instance_id": target_instance });
        CliEnvelope::ok("prompts backup", Some(target_instance), data).print_and_exit();
    }
    crate::running_backup_cmds::cmd_backup_running_prompts(args);
}

pub(crate) fn cmd_prompts_restore(args: &[String]) {
    let ctx = CliContext::parse(args);
    let target_instance = ctx
        .resolve_target_instance()
        .unwrap_or_else(|_| "default".to_string());
    if ctx.json_output {
        if is_daemon_running() {
            let payload = serde_json::json!({ "instanceId": target_instance });
            let _ = forward_to_local_rest::<serde_json::Value>(
                reqwest::Method::POST,
                "/prompts/restore",
                Some(payload),
            );
        }
        let _ = repo_db::resend_running_commands_for_instance(Some(&target_instance), 20);
        let data = serde_json::json!({ "status": "restored", "instance_id": target_instance });
        CliEnvelope::ok("prompts restore", Some(target_instance), data).print_and_exit();
    }
    crate::running_backup_cmds::cmd_restore_running_prompts(args);
}

pub(crate) fn cmd_prompts_purge(args: &[String]) {
    crate::cache_cmds::cmd_clear_cache(args);
}
