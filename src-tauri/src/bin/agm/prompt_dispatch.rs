//! prompt_dispatch — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::repo_db::ActivePrompt;
use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use chrono::Utc;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use uuid::Uuid;

pub(crate) fn cmd_prompt_dispatch(args: &[String]) {
    if args.is_empty() {
        crate::prompt_mgmt::cmd_prompts(args);
        return;
    }

    if let Some(first) = args.first() {
        let first_lower = first.to_lowercase();
        if first_lower == "help" || first_lower == "--help" || first_lower == "-h" {
            println!("AGM Prompt Dispatch (By AGM Seq ID, Instance, Project, or Node):");
            println!("  agm prompt [C001|P001|project] <text> [--instance <id>] [--node <alias>] [--prefix <cat>] [--suffix <cat>]");
            println!("\nDescription:");
            println!("  Dispatches a prompt to a specific conversation sequence (C001), project sequence (P001),");
            println!("  instance (--instance <id>), or remote SSH node (--node <alias>), with automatic git pull");
            println!("  and optional canonical prompt template framing from 01-prompts/.");
            println!("\nAliases: agm prompt");
            println!("\nOptions:");
            println!("    --instance, -i <id> Target a specific sandbox instance (default: active/default)");
            println!("    --node, -n <alias>  Dispatch prompt to a remote cluster machine via GitMap SSH");
            println!(
                "    --prefix <cat>      Prepend template from 01-prompts/<cat> to the prompt"
            );
            println!("    --suffix <cat>      Append template from 01-prompts/<cat> to the prompt");
            println!("\nExamples:");
            println!("  agm prompt C001 \"Is it done?\"                      # Target conversation sequence C001");
            println!("  agm prompt P001 --instance default \"Run tests\"     # Target project sequence P001 on instance");
            println!("  agm prompt C001 --node vm-01 \"Check status\"        # Target C001 on remote SSH node");
            println!("  agm prompt \"Audit DB\" --prefix coding-standards    # Frame prompt with template");
            println!("  agm prompt query [term]                            # Search SQLite cached prompts");
            println!(
                "  agm prompt show <id|seq>                           # Show full prompt record"
            );
            return;
        }
        if first_lower == "query" || first_lower == "search" || first_lower == "find" {
            crate::prompt_mgmt::cmd_prompts_query(&args[1..]);
            return;
        }
        if first_lower == "show" {
            crate::prompt_mgmt::cmd_prompts_show(&args[1..]);
            return;
        }
        if first_lower == "ls" || first_lower == "list" || first_lower == "--running" {
            crate::prompt_mgmt::cmd_prompts(args);
            return;
        }
        if first_lower == "tree" {
            crate::misc_cmds::cmd_tree(&args[1..]);
            return;
        }
        if first_lower == "backup" || first_lower == "backpack" {
            crate::running_backup_cmds::cmd_backup_running_prompts(&args[1..]);
            return;
        }
        if first_lower == "restore" {
            crate::running_backup_cmds::cmd_restore_running_prompts(&args[1..]);
            return;
        }
    }

    let mut is_queue_mode = false;
    let mut prefix_cat: Option<String> = None;
    let mut suffix_cat: Option<String> = None;
    let mut explicit_instance: Option<String> = None;
    let mut explicit_node: Option<String> = None;
    let mut explicit_seq_or_target: Option<String> = None;
    let mut base64_payload: Option<String> = None;
    let mut text_parts: Vec<String> = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--queue" || arg == "-q" || arg == "--enqueue" {
            is_queue_mode = true;
            i += 1;
            continue;
        } else if arg == "--base64" || arg == "-b64" || arg == "--b64" {
            if i + 1 < args.len() {
                base64_payload = Some(args[i + 1].clone());
                i += 2;
                continue;
            }
        } else if let Some(b64) = arg.strip_prefix("--base64=") {
            base64_payload = Some(b64.to_string());
            i += 1;
            continue;
        } else if arg == "--prefix" || arg == "-prefix" {
            if i + 1 < args.len() {
                prefix_cat = Some(args[i + 1].clone());
                i += 2;
                continue;
            }
        } else if arg == "--suffix" || arg == "-suffix" {
            if i + 1 < args.len() {
                suffix_cat = Some(args[i + 1].clone());
                i += 2;
                continue;
            }
        } else if arg == "--instance" || arg == "-i" {
            if i + 1 < args.len() {
                explicit_instance = Some(args[i + 1].clone());
                i += 2;
                continue;
            }
        } else if let Some(ins) = arg
            .strip_prefix("instance:")
            .or_else(|| arg.strip_prefix("ins:"))
        {
            explicit_instance = Some(ins.to_string());
            i += 1;
            continue;
        } else if arg == "--node" || arg == "-n" {
            if i + 1 < args.len() {
                explicit_node = Some(args[i + 1].clone());
                i += 2;
                continue;
            }
        } else if arg == "--seq" || arg == "--conv" || arg == "-c" {
            if i + 1 < args.len() {
                explicit_seq_or_target = Some(args[i + 1].clone());
                i += 2;
                continue;
            }
        } else {
            text_parts.push(arg.clone());
        }
        i += 1;
    }

    // Check if the first positional token is an AGM or GitMap Sequence ID (e.g. C001, P001, AGM:C001, GM:#1, GM:<cid>)
    let mut resolved_seq: Option<repo_db::AgmSequenceResolution> = None;
    if let Some(ref seq_tok) = explicit_seq_or_target {
        resolved_seq = repo_db::resolve_agm_sequence_target(seq_tok);
    } else if !text_parts.is_empty() {
        let first_tok = text_parts[0].trim();
        let upper = first_tok.trim_start_matches('#').to_uppercase();
        let looks_like_seq = upper.starts_with("AGM:")
            || upper.starts_with("GM:")
            || ((upper.starts_with('C') || upper.starts_with('P'))
                && upper[1..].chars().all(|c| c.is_ascii_digit())
                && !upper[1..].is_empty());
        if looks_like_seq {
            if let Some(res) = repo_db::resolve_agm_sequence_target(first_tok) {
                resolved_seq = Some(res);
                text_parts.remove(0);
            }
        } else if base64_payload.is_some() {
            explicit_seq_or_target = Some(text_parts.remove(0));
        }
    }

    let raw_text = if let Some(ref b64_str) = base64_payload {
        use base64::Engine;
        let clean_b64 = b64_str.trim().trim_matches('"').trim_matches('\'');
        match base64::engine::general_purpose::STANDARD.decode(clean_b64.as_bytes()) {
            Ok(bytes) => String::from_utf8(bytes).unwrap_or_else(|_| text_parts.join(" ")),
            Err(_) => text_parts.join(" "),
        }
    } else {
        text_parts.join(" ")
    };
    let final_prompt =
        wrap_prompt_with_templates(&raw_text, prefix_cat.as_deref(), suffix_cat.as_deref());

    if final_prompt.trim().is_empty() {
        eprintln!("[ERROR] Prompt content cannot be empty.");
        std::process::exit(1);
    }

    // Remote SSH Node Delegation if `--node <alias>` was specified
    if let Some(node_alias) = explicit_node {
        let target_arg = resolved_seq
            .as_ref()
            .map(|s| s.seq_code.clone())
            .or(explicit_seq_or_target.clone())
            .unwrap_or_else(|| "default".to_string());
        let inst_arg = explicit_instance
            .as_deref()
            .map(|ins| format!(" --instance {}", ins))
            .unwrap_or_default();
        use base64::Engine;
        let b64_prompt = base64::engine::general_purpose::STANDARD.encode(final_prompt.as_bytes());
        let remote_cmd = format!(
            "agm prompt {}{} --base64 \"{}\"",
            target_arg, inst_arg, b64_prompt
        );
        println!(
            "[*] Dispatching prompt to remote node '{}' via GitMap cluster SSH...",
            node_alias
        );
        let status = Command::new("gitmap")
            .args(["cluster", "exec", &node_alias, &remote_cmd])
            .status();
        match status {
            Ok(s) if s.success() => {
                println!(
                    "[SUCCESS] Remote prompt dispatched to node '{}' (target: {}).",
                    node_alias, target_arg
                );
                return;
            }
            _ => {
                eprintln!(
                    "[WARN] gitmap cluster exec did not succeed; falling back to Telegram/Supabase queue dispatch..."
                );
                let rt = tokio::runtime::Runtime::new().unwrap();
                let reply = rt.block_on(telegram_inbound::execute_prompt_injection(&format!(
                    "{} {} {}",
                    node_alias, target_arg, final_prompt
                )));
                println!("{}", reply);
                return;
            }
        }
    }

    // Pull latest changes before dispatching prompt locally
    if Path::new(".git").exists() {
        println!("[*] Synchronizing repository via git pull before prompt dispatch...");
        let _ = Command::new("git").args(["pull"]).status();
    }

    let resolved_explicit_inst = explicit_instance
        .as_deref()
        .map(|s| instance::resolve_instance_id(s).unwrap_or_else(|_| s.to_string()));

    let (slug, cwd_str, inst_id, session_id, seq_label) = if let Some(seq) = resolved_seq {
        let inst = resolved_explicit_inst.unwrap_or(seq.instance_id);
        let sess = seq
            .conversation_id
            .unwrap_or_else(|| seq.project_id.clone());
        (
            seq.project_id,
            seq.repo_path,
            inst,
            sess,
            Some(seq.seq_code),
        )
    } else {
        let target_str = explicit_seq_or_target.as_deref().unwrap_or("default");
        let projects = repo_db::get_live_project_execution_info();
        let matched = projects.iter().find(|p| {
            p.project_id.eq_ignore_ascii_case(target_str)
                || p.repo_name.eq_ignore_ascii_case(target_str)
        });
        let (s, c) = if let Some(p) = matched {
            (p.project_id.clone(), p.repo_path.clone())
        } else if Path::new(target_str).is_dir() {
            (target_str.to_string(), target_str.to_string())
        } else {
            let s = crate::common::derive_current_repo_slug();
            let c = env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_else(|_| ".".to_string());
            (s, c)
        };
        let i = resolved_explicit_inst.unwrap_or_else(|| {
            instance::get_active_instance_id().unwrap_or_else(|_| "default".to_string())
        });
        let sess = s.clone();
        (s, c, i, sess, None)
    };

    if is_queue_mode {
        match repo_db::enqueue_prompt_for_instance(
            &inst_id,
            &cwd_str,
            &final_prompt,
            Some(&session_id),
        ) {
            Ok(queued) => {
                let queue_pos = repo_db::count_enqueued_prompts(Some(&inst_id));
                antigravity_tools_lib::modules::audit::record_if_active(
                    "cli",
                    "cli",
                    "enqueue_prompt",
                    &format!("instance:{inst_id}"),
                    serde_json::json!({
                        "instanceId": inst_id,
                        "repoPath": cwd_str,
                        "conversationId": session_id,
                        "promptLen": final_prompt.len(),
                        "queuePosition": queue_pos,
                    }),
                    "ok",
                    &format!("Enqueued prompt {} to {slug}", queued.id),
                );
                if let Some(seq_code) = seq_label {
                    println!(
                        "[SUCCESS] Prompt queued for AGM Seq [{}] -> instance '{}' at position #{} (id: {}, conv: {}).",
                        seq_code, inst_id, queue_pos, queued.id, session_id
                    );
                } else {
                    println!(
                        "[SUCCESS] Prompt queued for instance '{}' at position #{} (conv: {}).",
                        inst_id, queue_pos, session_id
                    );
                }
                return;
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to queue prompt: {}", e);
                std::process::exit(1);
            }
        }
    }

    let now = chrono::Utc::now().timestamp();
    let prompt_id = uuid::Uuid::new_v4().to_string();

    let active_p = repo_db::ActivePrompt {
        id: prompt_id.clone(),
        project_id: slug.clone(),
        instance_id: inst_id.clone(),
        repo_path: cwd_str.clone(),
        prompt_content: final_prompt.clone(),
        model: Some("gemini-3.8-flash-high".to_string()),
        session_id: Some(session_id.clone()),
        status: "running".to_string(),
        created_at: now,
        updated_at: now,
        image_payload: None,
    };
    let _ = repo_db::save_or_requeue_prompt(&active_p);

    let task_file = PathBuf::from(&cwd_str).join(".antigravity_resume_task.json");
    let inst_task_file =
        PathBuf::from(&cwd_str).join(format!(".antigravity_resume_task.{}.json", inst_id));
    let payload = serde_json::json!({
        "prompt_id": prompt_id,
        "agm_seq_id": seq_label,
        "project_id": slug,
        "conversation_id": session_id,
        "instance_id": inst_id,
        "repo_path": cwd_str,
        "prompt_content": final_prompt,
        "prefix_template": prefix_cat,
        "suffix_template": suffix_cat,
        "dispatched_at": now,
    });
    match serde_json::to_string_pretty(&payload) {
        Ok(js) => {
            if let Err(write_err) = fs::write(&task_file, &js) {
                eprintln!("[ERROR] Could not write the resume task file: {write_err}");
                std::process::exit(1);
            }
            // Justification: per-instance resume snapshot is a convenience file alongside the primary task_file
            let _ = fs::write(&inst_task_file, &js);
        }
        Err(json_err) => {
            eprintln!("[ERROR] Could not encode the resume task: {json_err}");
            std::process::exit(1);
        }
    }

    if let Err(spawn_err) = repo_db::spawn_or_replace_prompt_via_agy(&active_p, true) {
        let clean_prompt = repo_db::extract_clean_user_prompt(&final_prompt);
        let cdp_ok =
            antigravity_tools_lib::modules::instance::cdp_onboarding::submit_prompt_via_cdp_for_instance(
                &inst_id,
                &clean_prompt,
            );
        if cdp_ok {
            antigravity_tools_lib::modules::audit::record_if_active(
                "cli",
                "cli",
                "cdp_dispatch_fallback",
                &format!("instance:{inst_id}"),
                serde_json::json!({
                    "promptId": active_p.id,
                    "instanceId": inst_id,
                    "repoPath": cwd_str,
                    "fallback": "cdp_direct",
                }),
                "ok",
                "Prompt dispatched via live DevTools CDP fallback",
            );
            println!(
                "[SUCCESS] Direct CDP fallback succeeded for prompt '{}' on instance '{}'.",
                active_p.id, inst_id
            );
        } else {
            antigravity_tools_lib::modules::audit::record_if_active(
                "cli",
                "cli",
                "send_prompt_now",
                &format!("instance:{inst_id}"),
                serde_json::json!({
                    "instanceId": inst_id,
                    "repoPath": cwd_str,
                    "conversationId": session_id,
                    "promptLen": final_prompt.len(),
                }),
                "error",
                &spawn_err,
            );
            eprintln!("[ERROR] agy did not accept the prompt and CDP fallback failed: {spawn_err}");
            std::process::exit(1);
        }
    }

    antigravity_tools_lib::modules::audit::record_if_active(
        "cli",
        "cli",
        "send_prompt_now",
        &format!("instance:{inst_id}"),
        serde_json::json!({
            "instanceId": inst_id,
            "repoPath": cwd_str,
            "conversationId": session_id,
            "promptLen": final_prompt.len(),
        }),
        "ok",
        &format!("Dispatched prompt {} to {slug}", active_p.id),
    );

    if let Some(seq_code) = seq_label {
        println!(
            "[SUCCESS] Dispatched prompt ({} chars) to AGM Seq [{}] -> workspace '{}' (conv: {}, instance: {}).",
            final_prompt.len(),
            seq_code,
            slug,
            session_id,
            inst_id
        );
    } else {
        println!(
            "[SUCCESS] Dispatched prompt ({} chars) to workspace '{}' [Instance: {}].",
            final_prompt.len(),
            slug,
            inst_id
        );
    }
}

pub(crate) fn scan_prompt_templates() {
    let prompts_dir = Path::new("01-prompts");
    if prompts_dir.exists() {
        if let Ok(entries) = fs::read_dir(prompts_dir) {
            let mut categories = Vec::new();
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    categories.push(entry.file_name().to_string_lossy().to_string());
                }
            }
            if !categories.is_empty() {
                println!(
                    "Available Prompt Categories in 01-prompts/ ({} found):",
                    categories.len()
                );
                for cat in categories.iter().take(10) {
                    println!("  - 01-prompts/{}", cat);
                }
                if categories.len() > 10 {
                    println!("  ... and {} more categories.", categories.len() - 10);
                }
                println!();
            }
        }
    }
}

pub(crate) fn resolve_prompt_template(category_or_name: &str) -> Option<String> {
    let query = category_or_name.trim().to_lowercase();
    if query.is_empty() {
        return None;
    }

    let mut search_roots = vec![PathBuf::from("01-prompts")];
    if let Ok(cwd) = env::current_dir() {
        search_roots.push(cwd.join("01-prompts"));
        if let Some(parent) = cwd.parent() {
            search_roots.push(parent.join("coding-guidelines").join("01-prompts"));
            search_roots.push(parent.join("Antigravity-Manager").join("01-prompts"));
        }
    }

    for root in search_roots {
        if !root.exists() {
            continue;
        }
        if let Ok(entries) = fs::read_dir(&root) {
            for entry in entries.flatten() {
                let path = entry.path();
                let fname = entry.file_name().to_string_lossy().to_lowercase();
                if fname.contains(&query) {
                    if path.is_file() {
                        if let Ok(content) = fs::read_to_string(&path) {
                            return Some(content.trim().to_string());
                        }
                    } else if path.is_dir() {
                        if let Ok(sub_entries) = fs::read_dir(&path) {
                            let mut md_files: Vec<PathBuf> = sub_entries
                                .flatten()
                                .map(|e| e.path())
                                .filter(|p| p.is_file())
                                .collect();
                            md_files.sort();
                            if let Some(first_file) = md_files.first() {
                                if let Ok(content) = fs::read_to_string(first_file) {
                                    return Some(content.trim().to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

pub(crate) fn wrap_prompt_with_templates(
    raw_prompt: &str,
    prefix_cat: Option<&str>,
    suffix_cat: Option<&str>,
) -> String {
    let mut parts = Vec::new();
    if let Some(pref) = prefix_cat {
        if let Some(tpl) = resolve_prompt_template(pref) {
            parts.push(tpl);
        } else {
            parts.push(format!("[Template Prefix: {}]", pref));
        }
    }
    if !raw_prompt.trim().is_empty() {
        parts.push(raw_prompt.trim().to_string());
    }
    if let Some(suff) = suffix_cat {
        if let Some(tpl) = resolve_prompt_template(suff) {
            parts.push(tpl);
        } else {
            parts.push(format!("[Template Suffix: {}]", suff));
        }
    }
    parts.join("\n\n")
}
