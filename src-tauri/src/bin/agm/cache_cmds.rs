//! cache_cmds — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};

pub(crate) fn cmd_clear_cache(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("AGM Cache & Conversation Pruner (Safety Gated):");
        println!("  agm prune [--keep <N>] [--dry-run] [--undo [TX]] [--json]");
        println!("  agm pr [N] [--dry-run] [--undo]");
        println!("  agm clear-cache [--keep <N>] [--preflight] [--undo] [--json]");
        println!("  agm clean [--keep <N>] [--preflight]");
        println!("\nDescription:");
        println!("  Safely prunes older conversation steps, developer logs, and build artifacts,");
        println!("  staging conversations into OS temp storage for undo recovery.");
        println!("  Safety Invariant: Active/running prompts and up to 5 latest project sessions are strictly protected.");
        println!(
            "\nAliases: agm prune, agm pr, agm clear-cache, agm cache-clear, agm clean, agm purge"
        );
        println!("\nOptions:");
        println!(
            "    --keep, -k <N>      Number of recent conversations to preserve (default: 10)"
        );
        println!("    --preflight, -p     Preview space reclamation without deleting files");
        println!("    --dry-run           Alias for --preflight preview");
        println!(
            "    --undo [TX]         Rollback the most recent or specified pruning transaction"
        );
        println!("    --json, -j          Output pure machine-readable JSON");
        println!("    -y, --yes           Bypass interactive confirmation prompt");
        println!("\nExamples:");
        println!(
            "  agm prune                           # Prune cache keeping 10 latest conversations"
        );
        println!(
            "  agm prune --keep 5                  # Prune cache keeping 5 latest conversations"
        );
        println!(
            "  agm prune --dry-run                 # Dry-run inspection without deleting files"
        );
        println!(
            "  agm prune --undo                    # Rollback the most recent pruning operation"
        );
        println!(
            "  agm pr 5                            # Shorthand to prune keeping 5 conversations"
        );
        println!(
            "  agm clean                           # Prune cache keeping 10 latest conversations"
        );
        println!(
            "  agm clean --preflight               # Dry-run preview of space to be reclaimed"
        );
        return;
    }

    let is_json = args.iter().any(|a| a == "--json" || a == "-j");
    let is_undo = args.iter().any(|a| a == "--undo" || a == "undo");
    if is_undo {
        let undo_target = args
            .iter()
            .position(|a| a == "--undo" || a == "undo")
            .and_then(|idx| args.get(idx + 1))
            .filter(|s| !s.starts_with('-'))
            .map(|s| s.as_str());

        match agy_cleaner::undo_prune(undo_target) {
            Ok(res) => {
                if is_json {
                    println!("{}", serde_json::to_string_pretty(&res).unwrap_or_default());
                } else {
                    println!(
                        "[✓] Successfully rolled back transaction: {}",
                        res.transaction_id
                    );
                    println!("    Restored conversations: {}", res.restored_conversations);
                    println!(
                        "    Restored data: {:.2} MB",
                        res.restored_bytes as f64 / 1024.0 / 1024.0
                    );
                    if !res.errors.is_empty() {
                        println!("    Encountered warnings: {:?}", res.errors);
                    }
                }
            }
            Err(e) => {
                if is_json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&serde_json::json!({ "error": e }))
                            .unwrap_or_default()
                    );
                } else {
                    eprintln!("[ERROR] Failed to undo prune: {}", e);
                }
                std::process::exit(1);
            }
        }
        return;
    }

    let mut keep_count: usize = 10;
    let is_preflight = args
        .iter()
        .any(|a| a == "--preflight" || a == "-p" || a == "--dry-run" || a == "dry-run");

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        let arg_lower = arg.to_lowercase();
        if arg_lower == "--preflight"
            || arg_lower == "-p"
            || arg_lower == "--dry-run"
            || arg_lower == "dry-run"
        {
            i += 1;
            continue;
        }
        if arg_lower == "--keep"
            || arg_lower == "-k"
            || arg_lower == "k"
            || arg_lower == "-keep"
            || arg_lower == "keep"
            || arg_lower == "--keep/k"
            || arg_lower == "-keep/k"
            || arg_lower == "keep/k"
        {
            if i + 1 < args.len() {
                keep_count = args[i + 1].parse::<usize>().unwrap_or(10);
                i += 2;
                continue;
            }
        } else if arg_lower.starts_with("--keep=")
            || arg_lower.starts_with("-k=")
            || arg_lower.starts_with("k=")
            || arg_lower.starts_with("--keep/k=")
            || arg_lower.starts_with("-keep/k=")
        {
            if let Some(val) = arg.split('=').nth(1) {
                keep_count = val.parse::<usize>().unwrap_or(10);
            }
        } else if (arg_lower.starts_with("-k") && arg_lower.len() > 2)
            || (arg_lower.starts_with('k')
                && arg_lower.len() > 1
                && arg_lower[1..].chars().all(|c| c.is_ascii_digit()))
        {
            let num_str = arg_lower.trim_start_matches("-k").trim_start_matches('k');
            if let Ok(n) = num_str.parse::<usize>() {
                keep_count = n;
            }
        } else if !arg.starts_with('-') {
            if let Ok(n) = arg.parse::<usize>() {
                keep_count = n;
            }
        }
        i += 1;
    }

    if is_preflight {
        let report = agy_cleaner::preflight_check(keep_count);
        if is_json {
            println!(
                "{}",
                serde_json::to_string_pretty(&report).unwrap_or_default()
            );
        } else {
            println!("\n=== AGM Conversation Prune Preflight Preview ===");
            println!("  Keep Count:             {}", report.keep_count);
            println!("  Total Conversations:    {}", report.total_conversations);
            println!("  Preserved Count:        {}", report.preserved_count);
            println!("  Pruned Count:           {}", report.pruned_count);
            println!(
                "  Projected Conversation Space Freed: {:.2} MB",
                report.projected_reclaimed_bytes as f64 / 1024.0 / 1024.0
            );
            println!(
                "  Projected Cache Space Freed:        {:.2} MB ({} targets)",
                report.cache_bytes as f64 / 1024.0 / 1024.0,
                report.cache_paths_count
            );
            let total_projected = report.projected_reclaimed_bytes + report.cache_bytes;
            println!(
                "  Total Projected Reclamation:        {:.2} MB",
                total_projected as f64 / 1024.0 / 1024.0
            );
            println!(
                "  Safe Temp Staging Dir:              {}",
                report.staging_dir
            );
            if !report.conversations_to_prune.is_empty() {
                println!("\n  Conversations to be staged & pruned:");
                for c in report.conversations_to_prune.iter().take(10) {
                    let short_id: String = c.conversation_id.chars().take(8).collect();
                    println!(
                        "    - [{}] {} ({:.2} KB)",
                        short_id,
                        c.title,
                        c.file_size as f64 / 1024.0
                    );
                }
                if report.conversations_to_prune.len() > 10 {
                    println!(
                        "    ... and {} more",
                        report.conversations_to_prune.len() - 10
                    );
                }
            }
            println!(
                "\n  (Run 'agm clean' or 'agm prune' without --dry-run/--preflight to execute)"
            );
        }
        return;
    }

    if !is_json {
        println!(
            "[*] Pruning old conversations (keeping {} most recent) and cleaning application caches...",
            keep_count
        );
    }

    match agy_cleaner::prune_and_clean(keep_count) {
        Ok(res) => {
            if is_json {
                println!("{}", serde_json::to_string_pretty(&res).unwrap_or_default());
            } else {
                println!(
                    "    [✓] Conversations preserved: {} | pruned: {}",
                    res.preserved_count, res.pruned_count
                );
                println!(
                    "    [✓] Total disk space freed: {:.2} MB (Staged at: {})",
                    res.total_freed_bytes as f64 / 1024.0 / 1024.0,
                    res.staging_dir
                );
                crate::misc_cmds::cmd_clean(&[]);
            }
        }
        Err(e) => {
            if is_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&serde_json::json!({ "error": e }))
                        .unwrap_or_default()
                );
            } else {
                eprintln!("[WARN] Conversation prune warning: {}", e);
                crate::misc_cmds::cmd_clean(&[]);
            }
        }
    }
}

pub(crate) fn cmd_logs(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("AGM System Logs Viewer:");
        println!("  agm logs [-n <lines>] [-f <filter>]");
        println!("\nDescription:");
        println!("  Streams and filters runtime diagnostic logs from the AGM background service.");
        println!("\nAliases: agm logs, agm log");
        println!("\nOptions:");
        println!("    -n, --tail <N>      Number of log lines to show (default: 25)");
        println!("    -f, --filter <str>  Filter log output by substring");
        println!("\nExamples:");
        println!("  agm logs                            # Display the last 25 log lines");
        println!("  agm logs -n 50                      # View last 50 lines");
        println!("  agm logs -f \"AutoSwitcher\"          # Filter logs matching 'AutoSwitcher'");
        return;
    }

    let mut tail = 25;
    let mut filter: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        if args[i] == "--tail" || args[i] == "-n" {
            if i + 1 < args.len() {
                tail = args[i + 1].parse().unwrap_or(25);
                i += 2;
                continue;
            }
        } else if (args[i] == "--filter" || args[i] == "-f") && i + 1 < args.len() {
            filter = Some(args[i + 1].clone());
            i += 2;
            continue;
        }
        i += 1;
    }

    let data_dir = match account::get_data_dir() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("[ERROR] Could not resolve data dir: {}", e);
            return;
        }
    };

    let log_file = data_dir.join("logs").join("antigravity.log");
    let fallback_log = PathBuf::from("antigravity.log");

    let target_log = if log_file.exists() {
        log_file
    } else if fallback_log.exists() {
        fallback_log
    } else {
        println!("No log file found at {:?}.", log_file);
        return;
    };

    println!("[*] Reading logs from {:?} (tail: {})...", target_log, tail);
    let file = match fs::File::open(&target_log) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("[ERROR] Failed to open log file: {}", e);
            return;
        }
    };

    let reader = io::BufReader::new(file);
    let mut matched_lines = Vec::new();

    for line in reader.lines().map_while(Result::ok) {
        if let Some(ref kw) = filter {
            if !line.to_lowercase().contains(&kw.to_lowercase()) {
                continue;
            }
        }
        matched_lines.push(line);
    }

    let start_idx = if matched_lines.len() > tail {
        matched_lines.len() - tail
    } else {
        0
    };

    for line in &matched_lines[start_idx..] {
        println!("{}", line);
    }
}
