//! prompt_resend_queue_cmds — CLI command handlers for the durable queued-prompt
//! backup + FIFO re-send pipeline (Task-03).
//!
//! Distinct from `prompt_resend::cmd_resend_running_commands` (which re-sends
//! in-flight RUNNING commands via .antigravity_resume_task.json): these commands
//! operate on ENQUEUED prompts through the durable `prompt_backups` SQLite
//! store, in strict FIFO order, with RCA-15 dedup (never re-dispatch an already
//! dispatched prompt).

use antigravity_tools_lib::modules::{backup_prompts_db, instance, repo_db};

fn print_resend_queued_help() {
    println!("AGM Resend Queued Prompts:");
    println!("  agm prompts resend-queued [--all] [-i <instance>] [N] [--json]");
    println!("\nDescription:");
    println!("  Takes a durable FIFO snapshot of the live queue into the prompt-backup");
    println!("  SQLite store, then re-sends every enqueued prompt in FIFO order with a");
    println!("  SINGLE dispatch pass. Already-dispatched prompts are skipped (RCA-15");
    println!("  dedup); counts are reported, never silently swallowed.");
    println!("\nAliases: agm prompts resend-queued, agm resend-queued, agm resend-queued-prompts, agm rqp");
    println!("\nOptions:");
    println!("    [N]                 Maximum prompts the dispatch pass may send (default: 20)");
    println!("    --all               Re-send queued prompts across ALL instances (default: active instance)");
    println!("    -i, --instance <id> Target instance id (default: active instance)");
    println!("    --json              Output pure JSON outcome payload");
    println!("\nExamples:");
    println!("  agm prompts resend-queued            # Snapshot + re-send active instance queue");
    println!("  agm resend-queued --all              # Re-send queued prompts on every instance");
    println!("  agm rqp 5 --json                     # Re-send up to 5, JSON outcome");
}

fn print_backup_queued_help() {
    println!("AGM Backup Queued Prompts:");
    println!("  agm prompts backup-queued [--all] [-i <instance>] [-f <file.db>] [--json]");
    println!("\nDescription:");
    println!("  Durably backs up ONLY the currently queued prompts into the split");
    println!("  SQLite store, preserving strict FIFO enqueue order (sequence_id =");
    println!("  queue position). Captures prompt text, destination");
    println!("  (project/instance/conversation), and image/attachment references.");
    println!("\nAliases: agm prompts backup-queued, agm backup-queued, agm backup-queued-prompts, agm bqp");
    println!("\nOptions:");
    println!("    --all               Back up queued prompts across ALL instances (default: active instance)");
    println!("    -i, --instance <id> Target instance id (default: active instance)");
    println!("    -f, --file <path>   Target custom SQLite database file");
    println!("    --json              Output pure JSON backup payload");
    println!("\nExamples:");
    println!("  agm prompts backup-queued            # Snapshot active instance queue");
    println!("  agm backup-queued --all --json       # Snapshot every instance, JSON output");
}

struct QueueScope {
    /// None = all instances; Some(id) = scoped.
    scope: Option<String>,
}

fn parse_queue_scope(args: &[String]) -> QueueScope {
    let mut explicit_all = false;
    let mut explicit_instance: Option<String> = None;
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--all" {
            explicit_all = true;
            explicit_instance = None;
            i += 1;
            continue;
        } else if (arg == "-i" || arg == "--instance" || arg == "-instance") && i + 1 < args.len() {
            let spec = &args[i + 1];
            explicit_instance =
                Some(instance::resolve_instance_id(spec).unwrap_or_else(|_| spec.clone()));
            explicit_all = false;
            i += 2;
            continue;
        } else if arg.starts_with("--instance=") || arg.starts_with("-i=") {
            if let Some(spec) = arg.split('=').nth(1) {
                explicit_instance =
                    Some(instance::resolve_instance_id(spec).unwrap_or_else(|_| spec.to_string()));
                explicit_all = false;
            }
        }
        i += 1;
    }
    let scope = if explicit_all {
        // Explicit --all: unscoped (every instance).
        None
    } else {
        // Explicit -i wins; otherwise default to the active instance.
        Some(explicit_instance.unwrap_or_else(|| {
            instance::get_active_instance_id().unwrap_or_else(|_| "default".to_string())
        }))
    };
    QueueScope { scope }
}

fn parse_limit(args: &[String], default: usize) -> usize {
    for arg in args {
        if !arg.starts_with('-') {
            if let Ok(n) = arg.parse::<usize>() {
                return n.max(1);
            }
        }
    }
    default
}

fn print_outcome_table(outcome: &repo_db::ResendQueuedOutcome) {
    println!("\n================================================================================");
    println!("  AGM RESEND QUEUED PROMPTS (FIFO)");
    println!("================================================================================");
    if let Some(ref batch) = outcome.backup_batch_id {
        println!("  Durable snapshot batch: {}", batch);
    }
    println!("  Instance scope:             {}", outcome.instance_id);
    println!("  Queued prompts snapshotted: {}", outcome.backed_up_queued);
    println!("  Re-injected from backup:     {}", outcome.reinjected);
    println!(
        "  Dispatched:                 {} ({} from backup)",
        outcome.dispatched, outcome.dispatched_from_backup
    );
    println!(
        "  Skipped (already sent/queued): {}",
        outcome.skipped_duplicates
    );
    println!("  Failed:                     {}", outcome.failed);
    if !outcome.failures.is_empty() {
        println!("\n  Failures:");
        for f in &outcome.failures {
            println!("    - {}", f);
        }
    }
    if outcome.dispatched == 0 && outcome.failed == 0 {
        println!("\n[INFO] No queued prompts to re-send.");
    } else if outcome.failed == 0 {
        println!(
            "\n[SUCCESS] Re-sent {} queued prompt(s); {} skipped as duplicates.",
            outcome.dispatched, outcome.skipped_duplicates
        );
    } else {
        println!(
            "\n[WARN] Completed with {} failure(s); see above.",
            outcome.failed
        );
    }
}

pub(crate) fn cmd_resend_queued_prompts(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        print_resend_queued_help();
        return;
    }

    let is_json = args.iter().any(|a| a == "--json");
    let limit = parse_limit(args, 20);
    let scope = parse_queue_scope(args);
    let scope_ref = scope.scope.as_deref();

    match repo_db::resend_queued_prompts_for_instance(scope_ref, limit) {
        Ok(outcome) => {
            if is_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&outcome).unwrap_or_else(|_| "{}".to_string())
                );
                return;
            }
            print_outcome_table(&outcome);
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to re-send queued prompts: {}", e);
            std::process::exit(1);
        }
    }
}

pub(crate) fn cmd_backup_queued_prompts(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        print_backup_queued_help();
        return;
    }

    let is_json = args.iter().any(|a| a == "--json");
    let scope = parse_queue_scope(args);
    let scope_ref = scope.scope.as_deref();
    let mut custom_file: Option<&str> = None;
    let mut i = 0;
    while i < args.len() {
        if (args[i] == "-f" || args[i] == "--file" || args[i] == "-file") && i + 1 < args.len() {
            custom_file = Some(&args[i + 1]);
            i += 2;
            continue;
        }
        i += 1;
    }

    match backup_prompts_db::backup_queued_prompts_for_instance(scope_ref, custom_file) {
        Ok((batch, records)) => {
            if is_json {
                let payload = serde_json::json!({
                    "batch_id": batch.id,
                    "file_path": batch.file_path,
                    "instance_scope": scope_ref,
                    "prompts_count": records.len(),
                    "created_at": batch.created_at,
                    "records": records,
                });
                println!(
                    "{}",
                    serde_json::to_string_pretty(&payload).unwrap_or_else(|_| "{}".to_string())
                );
                return;
            }
            println!("\n================================================================================");
            println!("  AGM BACKUP QUEUED PROMPTS (FIFO)");
            println!(
                "================================================================================"
            );
            println!(
                "[Snapshotted {} queued prompt(s) in FIFO order into {}]\n",
                records.len(),
                batch.file_path
            );
            if records.is_empty() {
                println!("No queued prompts found to back up.");
                return;
            }
            println!(
                "{:<5} {:<10} {:<24} {:<10} {:<16} PROMPT SNIPPET",
                "SEQ", "ID", "PROJECT", "STATUS", "IMAGES"
            );
            println!("{}", "-".repeat(110));
            for r in &records {
                let short_id: String = r.prompt_id.chars().take(8).collect();
                let (snippet, _) = crate::common::truncate_words(&r.prompt_text, 15);
                let img_label = if r.has_images { "Yes" } else { "None" };
                println!(
                    "#{:<4} {:<10} {:<24} {:<10} {:<16} {}",
                    r.sequence_id, short_id, r.project_name, r.status, img_label, snippet
                );
            }
            println!("\n[SUCCESS] Backup batch '{}' recorded.", batch.id);
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to backup queued prompts: {}", e);
            std::process::exit(1);
        }
    }
}
