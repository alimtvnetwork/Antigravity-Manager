//! running_prompts_cmds — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::repo_db::ActivePrompt;
use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use chrono::Utc;
use std::fs;
use uuid::Uuid;

pub(crate) fn cmd_running_prompts(args: &[String]) {
    if let Some(first) = args.first() {
        let first_lower = first.to_lowercase();
        if first_lower == "help" || first_lower == "--help" || first_lower == "-h" {
            println!("AGM Running Prompts Management:");
            println!("  agm running-prompts [ls] [--limit <Y>] [--words <N>] [--full] [--json]");
            println!("  agm running-prompts backup [ls|clean] [-f <path.db>] [--json]");
            println!("  agm running-prompts restore [--keep] [--json] [-f <path.db>]");
            println!("  agm running-prompts export [-f <path>] [--wc <N>]");
            println!("  agm running-prompts import [-f <path>] [--wc <N>]");
            println!("\nDescription:");
            println!(
                "  Parallel inspection, snapshotting, and restoration of active running prompts"
            );
            println!("  across all Antigravity workspace projects.");
            println!("\nAliases: agm running-prompts, agm prompts, agm wpr");
            println!("\nSubcommands:");
            println!(
                "  ls, list            List running prompts across active workspaces (default)"
            );
            println!(
                "  backup, brp         Snapshot all active running prompts to split SQLite DB"
            );
            println!(
                "  restore, rrp        Restore and re-inject saved prompts into workspace queue"
            );
            println!("  export, pe          Export prompts database to file or JSON");
            println!("  import, pi          Import prompts from file into local execution queue");
            println!("\nOptions:");
            println!("    --limit, -l <Y>     Limit number of prompts displayed (default: 8)");
            println!("    --words, --wc <N>   Maximum words to display per prompt snippet (default: 100)");
            println!("    --full              Display full prompt text without truncation");
            println!("    --json              Format output as structured JSON");
            println!("    --keep, -k          Preserve unrestored state during restore");
            println!("    -f, --file <path>   Specify custom SQLite storage file path");
            println!("\nExamples:");
            println!("  agm running-prompts                 # List active running prompts");
            println!("  agm running-prompts --words 200     # Query prompts with 200-word preview");
            println!("  agm running-prompts backup          # Parallel snapshot running prompts before switch");
            println!(
                "  agm running-prompts restore         # Re-inject backed-up prompts post-switch"
            );
            println!("  agm running-prompts export -f b.db  # Export running prompts to file");
            return;
        }
        if first_lower == "backup" {
            crate::running_backup_cmds::cmd_backup_running_prompts(&args[1..]);
            return;
        }
        if first_lower == "restore" {
            crate::running_backup_cmds::cmd_restore_running_prompts(&args[1..]);
            return;
        }
        if first_lower == "export" {
            cmd_running_prompts_export(&args[1..]);
            return;
        }
        if first_lower == "import" {
            cmd_running_prompts_import(&args[1..]);
            return;
        }
    }

    // Default to running-prompts ls
    let is_json = args.iter().any(|a| a == "--json");
    let is_full = args.iter().any(|a| a == "--full");
    let mut limit_y: usize = 8;
    let mut max_words: usize = 100;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "ls" || arg == "list" {
            i += 1;
            continue;
        }
        if (arg == "--limit" || arg == "-l") && i + 1 < args.len() {
            if let Ok(y) = args[i + 1].parse::<usize>() {
                limit_y = y.max(1);
            }
            i += 2;
            continue;
        }
        if (arg == "--words" || arg == "--wordcount" || arg == "--wc" || arg == "-w")
            && i + 1 < args.len()
        {
            if let Ok(w) = args[i + 1].parse::<usize>() {
                max_words = w.max(1);
            }
            i += 2;
            continue;
        } else if arg.starts_with("--words=")
            || arg.starts_with("--wordcount=")
            || arg.starts_with("--wc=")
        {
            if let Some(val) = arg.split('=').nth(1) {
                if let Ok(w) = val.parse::<usize>() {
                    max_words = w.max(1);
                }
            }
            i += 1;
            continue;
        }
        if !arg.starts_with('-') {
            if let Ok(n) = arg.parse::<usize>() {
                limit_y = n.max(1);
            }
        }
        i += 1;
    }

    let all_prompts = repo_db::list_all_prompts().unwrap_or_default();
    let mut running_prompts: Vec<_> = all_prompts
        .into_iter()
        .filter(|p| {
            p.status == "running"
                || p.status == "queued"
                || p.status == "dispatched"
                || p.status == "backed_up"
        })
        .collect();

    running_prompts.truncate(limit_y);
    running_prompts.reverse();

    let mut items = Vec::new();
    for (idx, p) in running_prompts.iter().enumerate() {
        let (snippet, word_count) = if is_full {
            (
                p.prompt_content.clone(),
                p.prompt_content.split_whitespace().count(),
            )
        } else {
            crate::common::truncate_words(&p.prompt_content, max_words)
        };

        items.push(serde_json::json!({
            "seq": idx + 1,
            "id": p.id,
            "project_id": p.project_id,
            "instance_id": p.instance_id,
            "repo_path": p.repo_path,
            "status": p.status,
            "word_count": word_count,
            "prompt": snippet,
            "has_images": p.image_payload.is_some(),
            "updated_at": p.updated_at,
        }));
    }

    if is_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&items).unwrap_or_else(|_| "[]".to_string())
        );
        return;
    }

    println!(
        "\n=== Running Prompts (Limit: {}, Words: {}, Mode: {}) ===",
        limit_y,
        if is_full {
            "Full".to_string()
        } else {
            max_words.to_string()
        },
        if is_full { "Full Prompt" } else { "Truncated" }
    );
    if items.is_empty() {
        println!("No running or queued prompts found.");
        return;
    }

    println!(
        "{:<5} {:<10} {:<22} {:<12} {:<10} PROMPT SNIPPET (ASC STACK)",
        "SEQ", "ID", "PROJECT", "STATUS", "WORDS"
    );
    println!("{}", "-".repeat(110));
    for it in &items {
        let seq = it["seq"].as_u64().unwrap_or(0);
        let sid: String = it["id"].as_str().unwrap_or("-").chars().take(8).collect();
        let proj: String = it["project_id"]
            .as_str()
            .unwrap_or("-")
            .chars()
            .take(20)
            .collect();
        let status = it["status"].as_str().unwrap_or("-");
        let wc = it["word_count"].as_u64().unwrap_or(0);
        let prompt_txt = it["prompt"].as_str().unwrap_or("");
        println!(
            "#{:<4} {:<10} {:<22} {:<12} {:<10} {}",
            seq, sid, proj, status, wc, prompt_txt
        );
    }
    println!();
}

pub(crate) fn cmd_running_prompts_export(args: &[String]) {
    let mut file_path = "agm-running-prompts.db".to_string();
    let mut word_limit: Option<usize> = None;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if (arg == "-f" || arg == "--file" || arg == "-file") && i + 1 < args.len() {
            file_path = args[i + 1].clone();
            i += 2;
            continue;
        }
        if (arg == "--wc" || arg == "-w" || arg == "--wordcount") && i + 1 < args.len() {
            word_limit = args[i + 1].parse().ok();
            i += 2;
            continue;
        }
        if !arg.starts_with('-') && i == 0 {
            file_path = arg.clone();
        }
        i += 1;
    }

    if file_path.ends_with(".json") {
        let prompts = repo_db::list_all_prompts().unwrap_or_default();
        let mut export_items = Vec::new();
        for (idx, p) in prompts.iter().enumerate() {
            let prompt_text = if let Some(wl) = word_limit {
                crate::common::truncate_words(&p.prompt_content, wl).0
            } else {
                p.prompt_content.clone()
            };
            export_items.push(serde_json::json!({
                "sequence": idx + 1,
                "id": p.id,
                "project_id": p.project_id,
                "repo_path": p.repo_path,
                "prompt": prompt_text,
                "has_images": p.image_payload.is_some(),
                "images_payload": p.image_payload,
                "status": p.status,
            }));
        }
        let json_str = serde_json::to_string_pretty(&export_items).unwrap_or_default();
        if let Err(e) = fs::write(&file_path, json_str) {
            eprintln!("[ERROR] Failed to write JSON export: {}", e);
            std::process::exit(1);
        }
        println!(
            "[SUCCESS] Exported {} prompts to JSON file: {}",
            export_items.len(),
            file_path
        );
    } else {
        match backup_prompts_db::backup_active_running_prompts(Some("default"), Some(&file_path)) {
            Ok((batch, records)) => {
                println!(
                    "[SUCCESS] Exported {} prompts to SQLite DB: {}",
                    records.len(),
                    batch.file_path
                );
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to export prompts to SQLite DB: {}", e);
                std::process::exit(1);
            }
        }
    }
}

pub(crate) fn cmd_running_prompts_import(args: &[String]) {
    let mut file_path = "agm-running-prompts.db".to_string();

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if (arg == "-f" || arg == "--file" || arg == "-file") && i + 1 < args.len() {
            file_path = args[i + 1].clone();
            i += 2;
            continue;
        }
        if !arg.starts_with('-') && i == 0 {
            file_path = arg.clone();
        }
        i += 1;
    }

    if file_path.ends_with(".json") {
        let content = match fs::read_to_string(&file_path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "[ERROR] Failed to read JSON import file '{}': {}",
                    file_path, e
                );
                std::process::exit(1);
            }
        };
        let items: Vec<serde_json::Value> = serde_json::from_str(&content).unwrap_or_default();
        let now = Utc::now().timestamp();
        for item in &items {
            let id = item["id"]
                .as_str()
                .unwrap_or(&Uuid::new_v4().to_string())
                .to_string();
            let proj_id = item["project_id"]
                .as_str()
                .unwrap_or("imported")
                .to_string();
            let repo_path = item["repo_path"].as_str().unwrap_or("").to_string();
            let prompt_text = item["prompt"].as_str().unwrap_or("").to_string();
            let images_payload = item["images_payload"].as_str().map(|s| s.to_string());
            let active_p = ActivePrompt {
                id,
                project_id: proj_id,
                instance_id: "default".to_string(),
                repo_path,
                prompt_content: prompt_text,
                model: Some("gemini-3.8-flash-high".to_string()),
                session_id: None,
                status: "queued".to_string(),
                created_at: now,
                updated_at: now,
                image_payload: images_payload,
            };
            let _ = repo_db::save_or_requeue_prompt(&active_p);
        }
        println!(
            "[SUCCESS] Imported and enqueued {} prompt(s) from JSON: {}",
            items.len(),
            file_path
        );
    } else {
        match backup_prompts_db::restore_running_prompts(Some("default"), true, Some(&file_path)) {
            Ok(records) => {
                println!(
                    "[SUCCESS] Imported and enqueued {} prompt(s) from SQLite DB: {}",
                    records.len(),
                    file_path
                );
            }
            Err(e) => {
                eprintln!("[ERROR] Failed to import from SQLite DB: {}", e);
                std::process::exit(1);
            }
        }
    }
}
