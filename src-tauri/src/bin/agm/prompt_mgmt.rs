//! prompt_mgmt — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::repo_db::ActivePrompt;
use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use std::env;
use std::path::{Path, PathBuf};

pub(crate) fn cmd_prompts(args: &[String]) {
    if let Some(first) = args.first() {
        let first_lower = first.to_lowercase();
        if first_lower == "help" || first_lower == "--help" || first_lower == "-h" {
            println!("AGM Prompts Management & SQLite Caching:");
            println!("  agm prompts [ls] [N] [--words <W>] [--running] [--json]");
            println!(
                "  agm prompts query [term] [--words <W>] [--limit <N>] [--status <S>] [--json]"
            );
            println!("  agm prompts show <id|seq> [--json]");
            println!("  agm prompts backup [ls|clean] [-f <path.db>] [--json]");
            println!("  agm prompts restore [--keep] [--json] [-f <path.db>]");
            println!("  agm prompts status [--json]");
            println!("  agm prompts export [-f <file.db>] [--wc <N>]");
            println!("  agm prompts import [-f <file.db>]");
            println!("\nDescription:");
            println!(
                "  Inspects, snapshots, exports, imports, queries, and restores active and queued prompts"
            );
            println!(
                "  across all running Antigravity workspace projects with SQLite database caching."
            );
            println!("\nAliases: agm prompts, agm running-prompts");
            println!("\nSubcommands:");
            println!(
                "  ls, list            List running prompts in ASC stack order (oldest to newest)"
            );
            println!("  query, search       Query cached prompts in SQLite with ≥200-word preview");
            println!("  show                Display detailed prompt record and full word content");
            println!(
                "  backup, brp         Parallel snapshot of active prompts to split SQLite DB"
            );
            println!(
                "  restore, rrp        Restore and re-inject saved prompts into workspace queue"
            );
            println!(
                "  status, wpr         Display which projects and prompts are actively executing"
            );
            println!("  export, pe          Export prompts database to file or JSON");
            println!("  import, pi          Import prompts from file into local execution queue");
            println!("\nExamples:");
            println!("  agm prompts query                   # Query recent prompts with 200-word previews");
            println!("  agm prompts query \"cicd\"            # Search prompts containing 'cicd'");
            println!(
                "  agm prompts query --words 250 -n 5  # Show top 5 prompts with 250 words preview"
            );
            println!(
                "  agm prompts query --status running  # Query only actively executing prompts"
            );
            println!(
                "  agm prompts show P001               # Show detailed prompt for sequence P001"
            );
            println!("  agm prompts                         # List latest 10 running prompts");
            println!("  agm prompts ls 5                    # Show latest 5 running prompts in ASC stack");
            println!("  agm prompts backup                  # Snapshot all running prompts before rotation");
            println!("  agm prompts restore                 # Re-inject backed-up prompts into workspaces");
            println!("  agm prompts status                  # View active project prompts execution status");
            println!(
                "  agm prompts export -f backup.db     # Export prompts to specific SQLite file"
            );
            return;
        }
        if first_lower == "query" || first_lower == "search" || first_lower == "find" {
            cmd_prompts_query(&args[1..]);
            return;
        }
        if first_lower == "show" {
            cmd_prompts_show(&args[1..]);
            return;
        }
        if first_lower == "backup" || first_lower == "brp" {
            crate::running_backup_cmds::cmd_backup_running_prompts(&args[1..]);
            return;
        }
        if first_lower == "restore" || first_lower == "rrp" {
            crate::running_backup_cmds::cmd_restore_running_prompts(&args[1..]);
            return;
        }
        if first_lower == "status" || first_lower == "wpr" || first_lower == "running" {
            crate::prompt_resend::cmd_which_prompts_running(&args[1..]);
            return;
        }
        if first_lower == "export" || first_lower == "pe" {
            crate::prompt_import_export::cmd_prompts_export(&args[1..]);
            return;
        }
        if first_lower == "import" || first_lower == "pi" {
            crate::prompt_import_export::cmd_prompts_import(&args[1..]);
            return;
        }
        if first_lower == "goal-worker" || first_lower == "gw" {
            crate::prompt_goals::cmd_prompt_goal_worker(&args[1..]);
            return;
        }
        if first_lower == "start-goal" || first_lower == "sg" {
            crate::prompt_goals::cmd_prompt_start_goal(&args[1..]);
            return;
        }
        if first_lower == "check-goal" || first_lower == "cg" {
            crate::prompt_goals::cmd_prompt_check_goal(&args[1..]);
            return;
        }
    }

    let is_ls = args
        .first()
        .map(|a| a.eq_ignore_ascii_case("ls") || a.eq_ignore_ascii_case("list"))
        .unwrap_or(false);
    let is_json = args.iter().any(|a| a == "--json");
    let running_only = args.iter().any(|a| a == "--running") || is_ls;

    let mut limit_n: usize = 10;
    let mut max_words: usize = 100;

    let mut i = if is_ls { 1 } else { 0 };
    while i < args.len() {
        let arg = &args[i];
        if arg == "--words" || arg == "-w" {
            if i + 1 < args.len() {
                max_words = args[i + 1].parse().unwrap_or(100);
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

    let cwd_opt = env::current_dir()
        .ok()
        .map(|p| p.to_string_lossy().to_lowercase().replace('\\', "/"));

    let mut all_prompts = repo_db::list_all_prompts().unwrap_or_default();

    // Filter to current repo if we are inside a repo folder that has tracked prompts
    if let Some(ref cwd) = cwd_opt {
        let repo_matches: Vec<repo_db::ActivePrompt> = all_prompts
            .iter()
            .filter(|p| {
                let rp = p.repo_path.to_lowercase().replace('\\', "/");
                !rp.is_empty() && (cwd.starts_with(&rp) || rp.starts_with(cwd))
            })
            .cloned()
            .collect();
        if !repo_matches.is_empty() {
            all_prompts = repo_matches;
        }
    }

    if running_only {
        let running_filtered: Vec<repo_db::ActivePrompt> = all_prompts
            .iter()
            .filter(|p| {
                p.status == "running" || p.status == "dispatched" || p.status == "backed_up"
            })
            .cloned()
            .collect();
        if !running_filtered.is_empty() {
            all_prompts = running_filtered;
        }
    }

    // Take top N most recent and reverse into ASC stack order (oldest -> newest in the N window)
    all_prompts.truncate(limit_n);
    all_prompts.reverse();

    let mut stack_items = Vec::new();
    for (idx, p) in all_prompts.iter().enumerate() {
        let (snippet, word_count) = crate::common::truncate_words(&p.prompt_content, max_words);
        stack_items.push(serde_json::json!({
            "seq": idx + 1,
            "id": p.id,
            "project_id": p.project_id,
            "instance_id": p.instance_id,
            "repo_path": p.repo_path,
            "status": p.status,
            "model": p.model.clone().unwrap_or_else(|| "default".to_string()),
            "word_count": word_count,
            "words_limit": max_words,
            "prompt": snippet,
            "has_image": p.image_payload.is_some(),
            "updated_at": p.updated_at,
        }));
    }

    if is_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&stack_items).unwrap_or_else(|_| "[]".to_string())
        );
        return;
    }

    println!(
        "\n[Table Mode: Displaying {} prompt(s) in ASC stack order (truncated to {} words; pass --json for raw JSON)]",
        stack_items.len(),
        max_words
    );

    if stack_items.is_empty() {
        println!("No active prompt tasks tracked in repo_prompts.db.");
    } else {
        println!(
            "{:<5} {:<10} {:<22} {:<12} {:<10} PROMPT SNIPPET (ASC STACK)",
            "SEQ", "ID", "PROJECT", "STATUS", "WORDS"
        );
        println!("{}", "-".repeat(110));
        for item in &stack_items {
            let seq = item["seq"].as_u64().unwrap_or(0);
            let short_id: String = item["id"].as_str().unwrap_or("-").chars().take(8).collect();
            let proj: String = item["project_id"]
                .as_str()
                .unwrap_or("-")
                .chars()
                .take(20)
                .collect();
            let status = item["status"].as_str().unwrap_or("-");
            let wc = item["word_count"].as_u64().unwrap_or(0);
            let prompt_txt = item["prompt"].as_str().unwrap_or("");
            println!(
                "#{:<4} {:<10} {:<22} {:<12} {:<10} {}",
                seq, short_id, proj, status, wc, prompt_txt
            );
        }
        println!();
    }

    if !is_ls {
        crate::prompt_dispatch::scan_prompt_templates();
    }
}

pub(crate) fn cmd_prompts_query(args: &[String]) {
    let mut search_term: Option<String> = None;
    let mut max_words: usize = 200;
    let mut limit_n: usize = 10;
    let mut status_filter: Option<String> = None;
    let is_json = args.iter().any(|a| a == "--json" || a == "-j");

    let is_help = args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help");
    if is_help {
        println!("AGM Prompts Query (SQLite Cached Prompts):");
        println!("  agm prompts query [term] [--words <W>] [--limit <N>] [--status <S>] [--json]");
        println!("\nDescription:");
        println!("  Queries cached prompts in SQLite repo_prompts.db and backup_prompts.db");
        println!("  with word-count previews (default: 200 words), filtering by status, term, or project.");
        println!("\nOptions:");
        println!("    --words, -w <W>     Preview word count (default: 200 words)");
        println!("    --limit, -n <N>     Maximum number of results to display (default: 10)");
        println!("    --status, -s <S>    Filter by status (running, queued, dispatched, backed_up, all)");
        println!("    --json, -j          Output pure JSON payload");
        println!("\nExamples:");
        println!(
            "  agm prompts query                   # View latest prompts with 200-word preview"
        );
        println!("  agm prompts query \"pipeline\"        # Search prompts containing 'pipeline'");
        println!(
            "  agm prompts query --words 250 -n 5  # Show top 5 prompts with 250 words preview"
        );
        println!("  agm prompts query -s running        # Query only actively executing prompts");
        return;
    }

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if (arg == "--words" || arg == "-w") && i + 1 < args.len() {
            max_words = args[i + 1].parse().unwrap_or(200);
            i += 2;
            continue;
        } else if (arg == "--limit" || arg == "-n") && i + 1 < args.len() {
            limit_n = args[i + 1].parse().unwrap_or(10);
            i += 2;
            continue;
        } else if (arg == "--status" || arg == "-s") && i + 1 < args.len() {
            status_filter = Some(args[i + 1].to_lowercase());
            i += 2;
            continue;
        } else if arg == "--json" || arg == "-j" {
            i += 1;
            continue;
        } else if !arg.starts_with('-') && search_term.is_none() {
            search_term = Some(arg.clone());
        }
        i += 1;
    }

    let all_prompts = repo_db::list_all_prompts().unwrap_or_default();
    let term_lower = search_term.as_ref().map(|s| s.to_lowercase());

    let mut filtered: Vec<_> = all_prompts
        .into_iter()
        .filter(|p| {
            if let Some(ref st) = status_filter {
                if st != "all" && !p.status.to_lowercase().contains(st) {
                    return false;
                }
            }
            if let Some(ref term) = term_lower {
                let in_content = p.prompt_content.to_lowercase().contains(term);
                let in_proj = p.project_id.to_lowercase().contains(term)
                    || p.repo_path.to_lowercase().contains(term);
                let in_id = p.id.to_lowercase().contains(term);
                let in_model = p
                    .model
                    .as_deref()
                    .unwrap_or("")
                    .to_lowercase()
                    .contains(term);
                if !in_content && !in_proj && !in_id && !in_model {
                    return false;
                }
            }
            true
        })
        .collect();

    filtered.truncate(limit_n);

    let mut results = Vec::new();
    for (idx, p) in filtered.iter().enumerate() {
        let (preview, word_count) =
            repo_db::extract_prompt_words_preview(&p.prompt_content, max_words);
        results.push(serde_json::json!({
            "seq": idx + 1,
            "id": p.id,
            "project_id": p.project_id,
            "instance_id": p.instance_id,
            "repo_path": p.repo_path,
            "status": p.status,
            "model": p.model,
            "word_count": word_count,
            "preview_words_limit": max_words,
            "prompt_preview": preview,
            "has_images": p.image_payload.is_some(),
            "updated_at": p.updated_at,
        }));
    }

    if is_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&results).unwrap_or_else(|_| "[]".to_string())
        );
        return;
    }

    if results.is_empty() {
        println!("No prompts found matching query criteria.");
        return;
    }

    println!(
        "\n[AGM Prompts Query: Found {} prompt(s) in SQLite (≥{} words preview)]",
        results.len(),
        max_words
    );
    println!(
        "{:<5} {:<10} {:<20} {:<12} {:<8} PROMPT PREVIEW (≥{} WORDS)",
        "SEQ", "ID", "PROJECT", "STATUS", "WORDS", max_words
    );
    println!("{}", "-".repeat(120));
    for r in &results {
        let seq = r["seq"].as_u64().unwrap_or(0);
        let short_id: String = r["id"].as_str().unwrap_or("-").chars().take(8).collect();
        let proj: String = r["project_id"]
            .as_str()
            .unwrap_or("-")
            .chars()
            .take(18)
            .collect();
        let status = r["status"].as_str().unwrap_or("-");
        let wc = r["word_count"].as_u64().unwrap_or(0);
        let preview = r["prompt_preview"].as_str().unwrap_or("");
        println!(
            "#{:<4} {:<10} {:<20} {:<12} {:<8} {}",
            seq, short_id, proj, status, wc, preview
        );
    }
    println!();
}

pub(crate) fn cmd_prompts_show(args: &[String]) {
    let is_json = args.iter().any(|a| a == "--json" || a == "-j");
    let query_id = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .map(|s| s.as_str());

    let Some(target) = query_id else {
        eprintln!("Usage: agm prompts show <id|sequence> [--json]");
        return;
    };

    let all_prompts = repo_db::list_all_prompts().unwrap_or_default();
    let q_lower = target.to_lowercase();

    let matched = all_prompts.into_iter().find(|p| {
        p.id.to_lowercase() == q_lower
            || p.id.to_lowercase().starts_with(&q_lower)
            || p.project_id.to_lowercase() == q_lower
            || p.project_id.to_lowercase().contains(&q_lower)
    });

    if let Some(p) = matched {
        let word_count = p.prompt_content.split_whitespace().count();
        if is_json {
            let out = serde_json::json!({
                "id": p.id,
                "project_id": p.project_id,
                "instance_id": p.instance_id,
                "repo_path": p.repo_path,
                "status": p.status,
                "model": p.model,
                "session_id": p.session_id,
                "word_count": word_count,
                "created_at": p.created_at,
                "updated_at": p.updated_at,
                "has_images": p.image_payload.is_some(),
                "prompt_content": p.prompt_content,
            });
            println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
        } else {
            println!("\n=== Prompt Details ({}) ===", p.id);
            println!("  Project:     {}", p.project_id);
            println!("  Instance:    {}", p.instance_id);
            println!("  Repo Path:   {}", p.repo_path);
            println!("  Status:      {}", p.status);
            println!("  Model:       {}", p.model.as_deref().unwrap_or("default"));
            println!("  Word Count:  {} words", word_count);
            println!(
                "  Updated:     {}",
                chrono::DateTime::from_timestamp(p.updated_at, 0)
                    .map(|d| d.to_rfc3339())
                    .unwrap_or_default()
            );
            println!("  Has Image:   {}", p.image_payload.is_some());
            println!("\n--- Full Prompt Content ---\n{}\n", p.prompt_content);
        }
    } else {
        eprintln!(
            "[ERROR] Prompt with ID or project '{}' not found in repo_prompts.db",
            target
        );
    }
}
