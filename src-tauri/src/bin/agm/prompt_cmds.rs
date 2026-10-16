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
        "reset-tree" | "reset-db" => cmd_prompts_reset_tree(args),
        "inspect" | "observe" => cmd_prompts_inspect(args),
        "send" | "dispatch" => cmd_prompts_send(args),
        "running" | "wpr" => cmd_prompts_running(args),
        "queue" | "enqueue" | "qe" => cmd_prompts_queue(args),
        "history" => cmd_prompts_history(args),
        "export" => crate::prompt_import_export::cmd_prompts_export(args),
        "export-compact" | "compact-export" => crate::compact_cmds::cmd_export_compact(args),
        "import-compact" | "compact-import" => crate::compact_cmds::cmd_import_compact(args),
        "backup" | "brp" => cmd_prompts_backup(args),
        "backup-queued" | "bqp" => crate::prompt_resend_queue_cmds::cmd_backup_queued_prompts(args),
        "restore" | "rrp" => cmd_prompts_restore(args),
        "resend-queued" | "rqp" => crate::prompt_resend_queue_cmds::cmd_resend_queued_prompts(args),
        "resend" | "rrc" | "resend-running" => {
            crate::prompt_resend::cmd_resend_running_commands(args)
        }
        "copy" | "cp" => cmd_prompts_copy(args),
        "purge" | "clean" => cmd_prompts_purge(args),
        "help" | "--help" | "-h" => cmd_prompts_help(args),
        _ => crate::common::handle_unknown_domain_command("prompts", subcommand, args),
    }
}

pub(crate) fn cmd_prompts_help(_args: &[String]) {
    println!("AGM Prompts Domain Commands:");
    println!("  agm prompt send [target] \"<text>\"     Immediately dispatch prompt (bypasses FIFO queue)");
    println!("  agm prompt queue [target] \"<text>\"    Enqueue prompt into SQLite FIFO queue");
    println!("  agm queue [target] \"<text>\"           Top-level shortcut to enqueue prompt");
    println!("  agm enqueue [target] \"<text>\"         Top-level alias to enqueue prompt");
    println!("  agm prompt [target] \"<text>\" [--queue] Dispatch prompt immediately (or queue if --queue)");
    println!("  agm prompt list                       List active prompts");
    println!("  agm prompt tree                       Render project/conversation tree");
    println!("  agm prompt running                    Show currently running prompts");
    println!("  agm prompt backup                     Snapshot active prompts");
    println!("  agm prompt backup-queued              Snapshot queued prompts only (FIFO order)");
    println!("  agm prompt restore                    Restore and re-enqueue in-flight prompts");
    println!("  agm prompt resend-queued              Re-send all enqueued prompts in FIFO order");
    println!("  agm prompt inspect                    Observe prompt goals and liveness");
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

pub(crate) fn cmd_prompts_reset_tree(args: &[String]) {
    let ctx = CliContext::parse(args);
    let target = ctx.resolve_target_instance().ok();
    repo_db::invalidate_prompt_tree_cache(target.as_deref());
    if ctx.json_output {
        CliEnvelope::ok(
            "prompts reset-tree",
            target.clone(),
            serde_json::json!({ "cleared": true, "instance": target }),
        )
        .print_and_exit();
    }
    println!(
        "Prompt tree cache cleared for {}",
        target.as_deref().unwrap_or("all instances")
    );
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

fn resolve_destination_session(args: &[String], repo_path: &str) -> Option<String> {
    let mut conv_arg: Option<String> = None;
    let mut is_explicit_new = false;
    let mut is_explicit_existing = false;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if (arg == "--conversation" || arg == "-c" || arg == "--conv" || arg == "--session")
            && i + 1 < args.len()
        {
            conv_arg = Some(args[i + 1].clone());
            i += 2;
            continue;
        } else if arg.starts_with("--conversation=")
            || arg.starts_with("--conv=")
            || arg.starts_with("--session=")
        {
            if let Some((_, val)) = arg.split_once('=') {
                conv_arg = Some(val.to_string());
            }
        } else if arg == "--new" || arg == "--new-conversation" || arg == "--new-session" {
            is_explicit_new = true;
        } else if arg == "--existing" || arg == "--existing-conversation" || arg == "--resume" {
            is_explicit_existing = true;
        }
        i += 1;
    }

    if let Some(conv) = conv_arg {
        return Some(conv);
    }
    if is_explicit_new {
        return Some(format!("conv-{}", uuid::Uuid::new_v4()));
    }
    if is_explicit_existing {
        if let Ok(conn) = repo_db::connect_db() {
            let session_found: Option<String> = conn
                .query_row(
                    "SELECT session_id FROM active_prompts 
                     WHERE (repo_path = ?1 OR project_id = ?1) 
                       AND session_id IS NOT NULL 
                       AND session_id != '' 
                     ORDER BY created_at DESC, id DESC 
                     LIMIT 1",
                    rusqlite::params![repo_path],
                    |row| row.get(0),
                )
                .ok();
            if session_found.is_some() {
                return session_found;
            }
        }
    }
    None
}

pub(crate) fn cmd_prompts_send(args: &[String]) {
    let ctx = CliContext::parse(args);
    let mut resolved_seq: Option<repo_db::AgmSequenceResolution> = None;
    let mut prompt_tokens = ctx.positional_args.clone();

    if !prompt_tokens.is_empty() {
        let first = prompt_tokens[0].trim();
        let upper = first.trim_start_matches('#').to_uppercase();
        let looks_like_seq = upper.starts_with("AGM:")
            || upper.starts_with("GM:")
            || ((upper.starts_with('C') || upper.starts_with('P'))
                && upper[1..].chars().all(|c| c.is_ascii_digit())
                && !upper[1..].is_empty());

        if looks_like_seq {
            if let Some(res) = repo_db::resolve_agm_sequence_target(first) {
                resolved_seq = Some(res);
                prompt_tokens.remove(0);
            }
        } else if ctx.instance_id.is_none()
            && prompt_tokens.len() >= 2
            && (first.starts_with('#')
                || first.starts_with("inst-")
                || first == "default"
                || instance::resolve_instance_id(first).is_ok())
        {
            prompt_tokens.remove(0);
        }
    }

    let (target_instance, repo_path, session_id) = if let Some(seq) = resolved_seq {
        let inst = ctx.instance_id.clone().unwrap_or(seq.instance_id);
        let repo = ctx.repo_path.clone().unwrap_or(seq.repo_path);
        let sess = seq
            .conversation_id
            .or_else(|| resolve_destination_session(args, &repo));
        (inst, repo, sess)
    } else {
        let inst = ctx
            .resolve_target_instance()
            .unwrap_or_else(|_| "default".to_string());
        let repo = ctx.repo_path.clone().unwrap_or_else(|| {
            std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "scratch/test-repo".to_string())
        });
        let sess = resolve_destination_session(args, &repo);
        (inst, repo, sess)
    };

    let resolved_inst =
        instance::resolve_instance_id(&target_instance).unwrap_or_else(|_| target_instance.clone());

    let raw_prompt = prompt_tokens.join(" ");
    let mut final_prompt = raw_prompt;
    if let Some(ref pref) = ctx.prefix {
        final_prompt = format!("{} {}", pref, final_prompt);
    }
    if let Some(ref suff) = ctx.suffix {
        final_prompt = format!("{} {}", final_prompt, suff);
    }

    if final_prompt.trim().is_empty() {
        if ctx.json_output {
            CliEnvelope::<()>::err(
                "prompt send",
                Some(target_instance),
                "EMPTY_PROMPT",
                "Prompt content cannot be empty.",
            )
            .print_and_exit();
        }
        eprintln!("[ERROR] Prompt content cannot be empty.");
        std::process::exit(1);
    }

    let session_id = session_id.or_else(|| Some(format!("conv-{}", uuid::Uuid::new_v4())));

    // Query smart process cache / OS process table
    let cached_proc = instance::get_or_detect_instance_process(&resolved_inst);
    let is_already_alive = cached_proc.is_some();
    let process_action = if is_already_alive {
        "cache_hit_dispatched"
    } else {
        "cold_launched"
    };

    let active_pid =
        match instance::ensure_instance_running_for_dispatch(&resolved_inst, Some(&repo_path)) {
            Ok(pid) => Some(pid),
            Err(_) => cached_proc.map(|r| r.pid),
        };

    match repo_db::send_prompt_now_for_instance(
        &resolved_inst,
        &repo_path,
        &final_prompt,
        session_id.as_deref(),
    ) {
        Ok(dispatched) => {
            antigravity_tools_lib::modules::audit::record_if_active(
                "cli",
                "cli",
                "send_prompt_now",
                &format!("instance:{resolved_inst}"),
                serde_json::json!({
                    "instanceId": resolved_inst,
                    "repoPath": repo_path,
                    "conversationId": dispatched.session_id,
                    "promptLen": final_prompt.len(),
                }),
                "ok",
                &format!("Dispatched prompt {}", dispatched.id),
            );
            let res_data = serde_json::json!({
                "id": dispatched.id,
                "project_id": dispatched.project_id,
                "instance_id": dispatched.instance_id,
                "repo_path": dispatched.repo_path,
                "prompt_content": dispatched.prompt_content,
                "model": dispatched.model,
                "session_id": dispatched.session_id,
                "status": dispatched.status,
                "created_at": dispatched.created_at,
                "updated_at": dispatched.updated_at,
                "image_payload": dispatched.image_payload,
                "pid": active_pid,
                "process_action": process_action,
            });

            if ctx.json_output {
                CliEnvelope::ok("prompt send", Some(target_instance), res_data).print_and_exit();
            }

            println!(
                "[SUCCESS] Prompt dispatched to instance '{}' (id: {}, conv: {:?}, pid: {:?}).",
                target_instance, dispatched.id, dispatched.session_id, active_pid
            );
        }
        Err(e) => {
            if ctx.json_output {
                CliEnvelope::<()>::err("prompt send", Some(target_instance), "DISPATCH_FAILED", &e)
                    .print_and_exit();
            }
            eprintln!("[ERROR] Failed to dispatch prompt: {}", e);
            std::process::exit(1);
        }
    }
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
    let mut resolved_seq: Option<repo_db::AgmSequenceResolution> = None;
    let mut prompt_tokens = ctx.positional_args.clone();

    if !prompt_tokens.is_empty() {
        let first = prompt_tokens[0].trim();
        let upper = first.trim_start_matches('#').to_uppercase();
        let looks_like_seq = upper.starts_with("AGM:")
            || upper.starts_with("GM:")
            || ((upper.starts_with('C') || upper.starts_with('P'))
                && upper[1..].chars().all(|c| c.is_ascii_digit())
                && !upper[1..].is_empty());

        if looks_like_seq {
            if let Some(res) = repo_db::resolve_agm_sequence_target(first) {
                resolved_seq = Some(res);
                prompt_tokens.remove(0);
            }
        } else if ctx.instance_id.is_none()
            && prompt_tokens.len() >= 2
            && (first.starts_with('#')
                || first.starts_with("inst-")
                || first == "default"
                || instance::resolve_instance_id(first).is_ok())
        {
            prompt_tokens.remove(0);
        }
    }

    let (target_instance, repo_path, session_id) = if let Some(seq) = resolved_seq {
        let inst = ctx.instance_id.clone().unwrap_or(seq.instance_id);
        let repo = ctx.repo_path.clone().unwrap_or(seq.repo_path);
        let sess = seq
            .conversation_id
            .or_else(|| resolve_destination_session(args, &repo));
        (inst, repo, sess)
    } else {
        let inst = ctx
            .resolve_target_instance()
            .unwrap_or_else(|_| "default".to_string());
        let repo = ctx.repo_path.clone().unwrap_or_else(|| {
            std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| "scratch/test-repo".to_string())
        });
        let sess = resolve_destination_session(args, &repo);
        (inst, repo, sess)
    };

    let resolved_inst =
        instance::resolve_instance_id(&target_instance).unwrap_or_else(|_| target_instance.clone());

    let raw_prompt = prompt_tokens.join(" ");
    let mut final_prompt = raw_prompt;
    if let Some(ref pref) = ctx.prefix {
        final_prompt = format!("{} {}", pref, final_prompt);
    }
    if let Some(ref suff) = ctx.suffix {
        final_prompt = format!("{} {}", final_prompt, suff);
    }

    if final_prompt.trim().is_empty() {
        if ctx.json_output {
            CliEnvelope::<()>::err(
                "prompt queue",
                Some(target_instance),
                "EMPTY_PROMPT",
                "Prompt content cannot be empty.",
            )
            .print_and_exit();
        }
        eprintln!("[ERROR] Prompt content cannot be empty.");
        std::process::exit(1);
    }

    match repo_db::enqueue_prompt_for_instance(
        &resolved_inst,
        &repo_path,
        &final_prompt,
        session_id.as_deref(),
    ) {
        Ok(queued) => {
            antigravity_tools_lib::modules::audit::record_if_active(
                "cli",
                "cli",
                "enqueue_prompt",
                &format!("instance:{resolved_inst}"),
                serde_json::json!({
                    "instanceId": resolved_inst,
                    "repoPath": repo_path,
                    "conversationId": queued.session_id,
                    "promptLen": final_prompt.len(),
                }),
                "ok",
                &format!("Enqueued prompt {}", queued.id),
            );
            let queue_position = repo_db::count_enqueued_prompts(Some(&resolved_inst));
            let data = serde_json::json!({
                "id": queued.id,
                "instance_id": queued.instance_id,
                "repo_path": queued.repo_path,
                "prompt_content": queued.prompt_content,
                "session_id": queued.session_id,
                "status": queued.status,
                "queue_position": queue_position,
                "created_at": queued.created_at,
                "fifo_ordered": true,
            });

            if ctx.json_output {
                CliEnvelope::ok("prompt queue", Some(target_instance), data).print_and_exit();
            }

            println!(
                "[SUCCESS] Prompt queued for instance '{}' at position #{} (id: {}, conv: {:?}).",
                target_instance, queue_position, queued.id, queued.session_id
            );
        }
        Err(e) => {
            if ctx.json_output {
                CliEnvelope::<()>::err("prompt queue", Some(target_instance), "QUEUE_FAILED", &e)
                    .print_and_exit();
            }
            eprintln!("[ERROR] Failed to queue prompt: {}", e);
            std::process::exit(1);
        }
    }
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

pub(crate) fn cmd_prompts_copy(args: &[String]) {
    let ctx = CliContext::parse(args);
    let target_instance = ctx
        .resolve_target_instance()
        .unwrap_or_else(|_| "default".to_string());
    let resolved_inst =
        instance::resolve_instance_id(&target_instance).unwrap_or_else(|_| target_instance.clone());
    let selector = ctx.positional_args.first().map(|s| s.trim().to_string());

    let all_prompts = repo_db::list_all_prompts().unwrap_or_default();
    let matched_active = all_prompts
        .iter()
        .rev()
        .find(|p| {
            let inst_ok = p.instance_id == resolved_inst
                || (resolved_inst == "default"
                    && (p.instance_id == "__default__" || p.instance_id.is_empty()));
            if !inst_ok {
                return false;
            }
            match selector.as_deref() {
                Some(sel) if !sel.is_empty() => {
                    p.id.eq_ignore_ascii_case(sel)
                        || p.session_id.as_deref() == Some(sel)
                        || p.project_id.contains(sel)
                }
                _ => true,
            }
        })
        .cloned();

    let (prompt_id, conv_id, repo_path, prompt_text) = if let Some(p) = matched_active {
        (p.id, p.session_id, p.repo_path, p.prompt_content)
    } else {
        let tree =
            repo_db::get_project_conversation_tree_cached(Some(&resolved_inst), 2000, false, false);
        let found = tree.iter().find_map(|proj| {
            proj.conversations.iter().find_map(|c| {
                let text = match c.full_prompt_text.as_deref() {
                    Some(full) if !full.trim().is_empty() => full.to_string(),
                    _ => c.prompt_preview_200w.clone(),
                };
                if text.trim().is_empty() {
                    return None;
                }
                let matches = match selector.as_deref() {
                    Some(sel) if !sel.is_empty() => {
                        c.conversation_id.eq_ignore_ascii_case(sel)
                            || c.short_id.eq_ignore_ascii_case(sel)
                            || c.seq_code.eq_ignore_ascii_case(sel)
                            || proj.seq_code.eq_ignore_ascii_case(sel)
                    }
                    _ => true,
                };
                if matches {
                    Some((
                        c.seq_code.clone(),
                        Some(c.conversation_id.clone()),
                        proj.repo_path.clone(),
                        text,
                    ))
                } else {
                    None
                }
            })
        });
        match found {
            Some(tuple) => tuple,
            None => {
                if ctx.json_output {
                    CliEnvelope::<()>::err(
                        "prompts copy",
                        Some(target_instance),
                        "PROMPT_NOT_FOUND",
                        "No prompt found to copy for target instance",
                    )
                    .print_and_exit();
                }
                eprintln!(
                    "[ERROR] No prompt found to copy for instance '{}'.",
                    target_instance
                );
                std::process::exit(1);
            }
        }
    };

    let copied_to_clipboard = copy_text_to_system_clipboard(&prompt_text);
    let data = serde_json::json!({
        "id": prompt_id,
        "instance_id": resolved_inst,
        "conversation_id": conv_id,
        "repo_path": repo_path,
        "copied_to_clipboard": copied_to_clipboard,
        "char_count": prompt_text.chars().count(),
        "prompt_content": prompt_text,
    });

    if ctx.json_output {
        CliEnvelope::ok("prompts copy", Some(target_instance), data).print_and_exit();
    }

    println!(
        "[SUCCESS] Copied prompt '{}' ({} chars, clipboard={}) for instance '{}':\n{}",
        prompt_id,
        prompt_text.chars().count(),
        copied_to_clipboard,
        target_instance,
        prompt_text
    );
}

fn copy_text_to_system_clipboard(text: &str) -> bool {
    use std::io::Write;
    use std::process::{Command, Stdio};

    #[cfg(target_os = "windows")]
    {
        if let Ok(mut child) = Command::new("clip")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            return child.wait().map(|s| s.success()).unwrap_or(false);
        }
        false
    }
    #[cfg(target_os = "macos")]
    {
        if let Ok(mut child) = Command::new("pbcopy")
            .stdin(Stdio::piped())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
        {
            if let Some(mut stdin) = child.stdin.take() {
                let _ = stdin.write_all(text.as_bytes());
            }
            return child.wait().map(|s| s.success()).unwrap_or(false);
        }
        false
    }
    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        for (bin, bin_args) in [
            ("wl-copy", &[][..]),
            ("xclip", &["-selection", "clipboard"][..]),
            ("xsel", &["--clipboard", "--input"][..]),
        ] {
            if let Ok(mut child) = Command::new(bin)
                .args(bin_args)
                .stdin(Stdio::piped())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
            {
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(text.as_bytes());
                }
                if child.wait().map(|s| s.success()).unwrap_or(false) {
                    return true;
                }
            }
        }
        false
    }
}
