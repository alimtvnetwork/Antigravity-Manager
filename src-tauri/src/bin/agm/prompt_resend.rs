//! prompt_resend — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::repo_db::ActivePrompt;
use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use std::fs;

pub(crate) fn cmd_resend_running_commands(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("AGM Resend Running Commands:");
        println!("  agm resend-running-commands [N] [--json] [-f <file.json>]");
        println!("\nDescription:");
        println!("  Captures active in-flight running commands across workspaces into SQLite");
        println!("  and re-dispatches them with image paths and task state synchronized into .antigravity_resume_task.json.");
        println!("\nAliases: agm resend-running-commands, agm rrc, agm resend-running, agm resend");
        println!("\nOptions:");
        println!(
            "    [N]                 Maximum number of active prompts to process (default: 20)"
        );
        println!("    --json              Output pure JSON array of resent commands");
        println!("    -f, --file <path>   Export resent commands JSON payload to disk");
        println!("\nExamples:");
        println!("  agm resend-running-commands         # Resend up to 20 active commands");
        println!("  agm rrc 5                           # Resend top 5 active commands");
        println!("  agm rrc --json                      # Output structured JSON of resent items");
        return;
    }

    let is_json = args.iter().any(|a| a == "--json");
    let mut limit_n = 20usize;
    let mut file_out: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "-f" || arg == "--file" {
            if i + 1 < args.len() {
                file_out = Some(args[i + 1].clone());
                i += 2;
                continue;
            }
        } else if !arg.starts_with('-') {
            if let Ok(n) = arg.parse::<usize>() {
                limit_n = n.max(1);
            }
        }
        i += 1;
    }

    // Step 1: Backup current in-flight prompts from workspaceStorage into SQLite before resend
    let active_inst = instance::get_active_instance_id().unwrap_or_else(|_| "default".to_string());
    let _ = repo_db::backup_running_prompts(&active_inst);

    // Step 2: Resend running commands from SQLite DB and write .antigravity_resume_task.json
    let resent_prompts = match repo_db::resend_all_running_commands(limit_n) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[ERROR] Failed to resend running commands: {}", e);
            std::process::exit(1);
        }
    };

    let mut items = Vec::new();
    for (idx, p) in resent_prompts.iter().enumerate() {
        let (extracted_img, img_paths) = repo_db::extract_image_payload_or_path(&p.prompt_content);
        let final_img = p.image_payload.clone().or(extracted_img);
        let has_image = final_img.is_some() || !img_paths.is_empty();
        let (snippet, word_count) = crate::common::truncate_words(&p.prompt_content, 100);

        items.push(serde_json::json!({
            "seq": idx + 1,
            "id": p.id,
            "project": p.project_id,
            "instance_id": p.instance_id,
            "repo_path": p.repo_path,
            "status": "running",
            "prompt": snippet,
            "word_count": word_count,
            "has_images": has_image,
            "image_paths": img_paths,
            "image_payload": final_img,
            "resent_via": ".antigravity_resume_task.json",
            "resend_status": "success",
            "updated_at": p.updated_at,
        }));
    }

    if let Some(ref path) = file_out {
        let target_path = if path.trim().is_empty() {
            let m_name = email_watcher::detect_machine_name();
            format!("agm-{}-resend.json", m_name.to_lowercase())
        } else {
            path.clone()
        };
        if let Ok(js_str) = serde_json::to_string_pretty(&items) {
            let _ = fs::write(&target_path, js_str);
            if !is_json {
                println!(
                    "[SUCCESS] Saved resend commands payload to \"{}\"",
                    target_path
                );
            }
        }
    }

    if is_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&items).unwrap_or_else(|_| "[]".to_string())
        );
        return;
    }

    println!("\n================================================================================");
    println!("  AGM RESEND RUNNING COMMANDS (RRC)");
    println!("================================================================================");
    println!(
        "[Table Mode: Resending {} running command(s) across active project(s) via .antigravity_resume_task.json]\n",
        items.len()
    );

    if items.is_empty() {
        println!("No active or previously running commands tracked in SQLite database.");
        return;
    }

    println!(
        "{:<5} {:<10} {:<24} {:<10} {:<18} PROMPT SNIPPET",
        "SEQ", "ID", "PROJECT", "STATUS", "IMAGES"
    );
    println!("{}", "-".repeat(110));

    for item in &items {
        let seq = item["seq"].as_u64().unwrap_or(0);
        let id_str = item["id"].as_str().unwrap_or("");
        let short_id = if id_str.len() > 8 {
            &id_str[..8]
        } else {
            id_str
        };
        let proj = item["project"].as_str().unwrap_or("-");
        let status = item["status"].as_str().unwrap_or("running");
        let has_img = item["has_images"].as_bool().unwrap_or(false);
        let img_paths = item["image_paths"].as_array();
        let img_label = if has_img {
            let count = img_paths.map(|a| a.len()).unwrap_or(1).max(1);
            format!("Yes ({} file(s))", count)
        } else {
            "None".to_string()
        };
        let prompt_txt = item["prompt"].as_str().unwrap_or("");
        println!(
            "#{:<4} {:<10} {:<24} {:<10} {:<18} {}",
            seq, short_id, proj, status, img_label, prompt_txt
        );
    }
    println!();
    println!(
        "[SUCCESS] Resent and queued {} command(s) for execution. SQLite DB synchronized with image file paths.",
        items.len()
    );
}

pub(crate) fn cmd_which_prompts_running(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("AGM Which Prompts Running:");
        println!("  agm which-prompts-running [--json]");
        println!("\nDescription:");
        println!(
            "  Inspects all registered workspaces and Antigravity conversation queues to detect"
        );
        println!("  actively executing, queued, and in-flight prompts with associated friendly project names.");
        println!("\nAliases: agm which-prompts-running, agm wpr");
        println!("\nOptions:");
        println!("    --json, -j          Output running prompts and project metadata as JSON");
        println!("\nExamples:");
        println!(
            "  agm which-prompts-running           # Display formatted table of running prompts"
        );
        println!("  agm wpr --json                      # Output active prompt queues as JSON");
        return;
    }

    let is_json = args.iter().any(|a| a == "--json");

    // Refresh live projects across registered instances
    if let Ok(reg) = instance::load_registry() {
        for inst in &reg.instances {
            let _ = repo_db::detect_running_projects(&inst.id);
        }
    }

    let projects = repo_db::list_running_projects().unwrap_or_default();
    let all_prompts = repo_db::list_all_prompts().unwrap_or_default();
    let conversations = agy_cleaner::scan_conversations(100);

    let mut rows = Vec::new();
    let mut seq = 0usize;

    for proj in &projects {
        let proj_prompts: Vec<&repo_db::ActivePrompt> = all_prompts
            .iter()
            .filter(|p| {
                (p.project_id == proj.id || p.repo_path.eq_ignore_ascii_case(&proj.repo_path))
                    && (p.status == "running"
                        || p.status == "backed_up"
                        || p.status == "dispatched")
            })
            .collect();

        if !proj.is_running && proj_prompts.is_empty() {
            continue;
        }

        seq += 1;
        let repo_norm = proj.repo_path.to_lowercase().replace('\\', "/");
        let matched_conv = conversations.iter().find(|c| {
            let uris_norm = c.workspace_uris.to_lowercase().replace('\\', "/");
            (!repo_norm.is_empty() && uris_norm.contains(&repo_norm))
                || (!proj.repo_name.is_empty()
                    && uris_norm.contains(&proj.repo_name.to_lowercase()))
        });

        let conv_id = matched_conv
            .map(|c| c.conversation_id.clone())
            .or_else(|| proj_prompts.first().and_then(|p| p.session_id.clone()))
            .unwrap_or_else(|| "-".to_string());

        let conv_name = matched_conv
            .and_then(|c| {
                if c.title.trim().is_empty() {
                    None
                } else {
                    Some(c.title.clone())
                }
            })
            .unwrap_or_else(|| proj.repo_name.clone());

        let queue_count = if proj_prompts.is_empty() && proj.is_running {
            1usize
        } else {
            proj_prompts.len()
        };

        rows.push(serde_json::json!({
            "seq": seq,
            "project": proj.repo_name,
            "id": proj.id,
            "instance_id": proj.instance_id,
            "repo_path": proj.repo_path,
            "conv_id": conv_id,
            "conv_name": conv_name,
            "prompts_count": queue_count,
            "is_running": proj.is_running,
        }));
    }

    // Also include any active/running prompts whose project wasn't listed in projects registry
    for p in &all_prompts {
        if p.status != "running" && p.status != "backed_up" && p.status != "dispatched" {
            continue;
        }
        let already_included = rows.iter().any(|r| {
            r["id"].as_str() == Some(&p.project_id) || r["repo_path"].as_str() == Some(&p.repo_path)
        });
        if !already_included {
            seq += 1;
            let conv_id = p.session_id.clone().unwrap_or_else(|| "-".to_string());
            let conv_name = p.project_id.clone();
            rows.push(serde_json::json!({
                "seq": seq,
                "project": p.project_id,
                "id": p.project_id,
                "instance_id": p.instance_id,
                "repo_path": p.repo_path,
                "conv_id": conv_id,
                "conv_name": conv_name,
                "prompts_count": 1,
                "is_running": true,
            }));
        }
    }

    if is_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&rows).unwrap_or_else(|_| "[]".to_string())
        );
        return;
    }

    if rows.is_empty() {
        println!("No projects currently have running or queued prompts.");
        return;
    }

    println!(
        "\nProjects with Running / Queued Prompts ({} active):",
        rows.len()
    );
    println!(
        "{:<5} {:<22} {:<24} {:<16} {:<26} PROMPTS (QUEUE)",
        "SEQ", "PROJECT", "ID", "CONV ID", "CONV NAME"
    );
    println!("{}", "-".repeat(110));

    for item in &rows {
        let s = item["seq"].as_u64().unwrap_or(0);
        let proj = item["project"].as_str().unwrap_or("-");
        let id: String = item["id"]
            .as_str()
            .unwrap_or("-")
            .chars()
            .take(22)
            .collect();
        let cid: String = item["conv_id"]
            .as_str()
            .unwrap_or("-")
            .chars()
            .take(14)
            .collect();
        let cname: String = item["conv_name"]
            .as_str()
            .unwrap_or("-")
            .chars()
            .take(24)
            .collect();
        let qcount = item["prompts_count"].as_u64().unwrap_or(0);
        println!(
            "#{:<4} {:<22} {:<24} {:<16} {:<26} {}",
            s, proj, id, cid, cname, qcount
        );
    }
    println!();
}
