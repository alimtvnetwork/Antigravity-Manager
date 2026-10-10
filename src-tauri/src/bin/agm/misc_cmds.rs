//! misc_cmds — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub(crate) fn cmd_tree(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("AGM Project → Conversation → 200-Word Prompt Tree View:");
        println!("  agm tree [all] [--words <N>] [--json]");
        println!("\nDescription:");
        println!("  Renders a hierarchical tree of Projects ([P001]), Conversations ([C001]),");
        println!("  and their latest user prompt (up to 200 words by default), persisting");
        println!("  independent AGM Sequence IDs in repo_prompts.db.");
        println!("\nOptions:");
        println!(
            "  all, --all          Include idle projects and conversations (default: running only)"
        );
        println!(
            "  --words, -w <N>     Maximum words to preview per conversation prompt (default: 200)"
        );
        println!("  --json, -j          Output full tree structure as JSON");
        println!("\nExamples:");
        println!("  agm tree                            # Show running projects, conversations & 200w prompts");
        println!("  agm tree all                        # Show all workspaces & conversations");
        println!(
            "  agm prompt C001 \"Is it done?\"       # Target conversation C001 directly from tree"
        );
        return;
    }

    let only_running = !args
        .iter()
        .any(|a| a.eq_ignore_ascii_case("all") || a == "--all" || a == "-a");
    let is_json = args.iter().any(|a| a == "--json" || a == "-j");
    let mut max_words = 200usize;
    let mut i = 0;
    while i < args.len() {
        if (args[i] == "--words" || args[i] == "-w") && i + 1 < args.len() {
            if let Ok(w) = args[i + 1].parse::<usize>() {
                max_words = w.max(1);
            }
            i += 2;
            continue;
        }
        i += 1;
    }

    if is_json {
        let tree = repo_db::get_project_conversation_tree(max_words, only_running);
        println!(
            "{}",
            serde_json::to_string_pretty(&tree).unwrap_or_else(|_| "[]".to_string())
        );
    } else {
        println!("{}", repo_db::format_tree_view_cli(max_words, only_running));
    }
}

pub(crate) fn cmd_agy(args: &[String]) {
    let sub = args
        .first()
        .map(|s| s.trim_start_matches('/').to_lowercase())
        .unwrap_or_else(|| "help".to_string());
    let rt = tokio::runtime::Runtime::new().unwrap();
    match sub.as_str() {
        "tree" => {
            let rest = if args.len() > 1 { &args[1..] } else { &[] };
            cmd_tree(rest);
        }
        "active" | "running" => {
            println!("{}", repo_db::format_tree_view_cli(200, true));
            if let Ok(out) = std::process::Command::new("gitmap")
                .args(["agy", "active"])
                .output()
            {
                let stdout = String::from_utf8_lossy(&out.stdout);
                if !stdout.trim().is_empty() {
                    println!("\n[GitMap AGY Active Output]\n{}", stdout.trim());
                }
            }
        }
        "backup" | "backup-running-prompts" => {
            let rest = if args.len() > 1 { &args[1..] } else { &[] };
            crate::running_backup_cmds::cmd_backup_running_prompts(rest);
            let _ = std::process::Command::new("gitmap")
                .arg("backup-running-prompts")
                .status();
        }
        "restore" | "restore-running-prompts" => {
            let rest = if args.len() > 1 { &args[1..] } else { &[] };
            crate::running_backup_cmds::cmd_restore_running_prompts(rest);
        }
        "running-prompts" => {
            let rest = if args.len() > 1 { &args[1..] } else { &[] };
            crate::running_prompts_cmds::cmd_running_prompts(rest);
        }
        "fpug" | "finish-prompts-until-green" => {
            let rest = if args.len() > 1 { &args[1..] } else { &[] };
            crate::green_cmds::cmd_finish_prompts_until_green(rest);
        }
        "sug" | "shutdown-until-green" => {
            let rest = if args.len() > 1 { &args[1..] } else { &[] };
            crate::green_cmds::cmd_shutdown_until_green(rest);
        }
        "rerun" => {
            let rest = if args.len() > 1 { &args[1..] } else { &[] };
            crate::prompt_goals::cmd_rerun(rest);
        }
        "queues" | "queue" => {
            println!(
                "{}",
                rt.block_on(telegram_inbound::format_prompt_queues_report())
            );
        }
        "projects" | "workspaces" | "ls" => {
            println!("{}", telegram_inbound::format_projects_list());
        }
        "prompts" | "prompt-ls" | "templates" => {
            println!("{}", telegram_inbound::format_prompts_templates_report());
        }
        "prompt" | "p" | "prompt-project" => {
            let rest = if args.len() > 1 { &args[1..] } else { &[] };
            crate::prompt_dispatch::cmd_prompt_dispatch(rest);
        }
        "nodes" => {
            println!(
                "{}",
                rt.block_on(telegram_inbound::format_cluster_nodes_report())
            );
        }
        _ => {
            let gm_res = std::process::Command::new("gitmap")
                .arg("agy")
                .args(args)
                .status();
            if let Ok(st) = gm_res {
                if st.success() {
                    return;
                }
            }
            println!("AGM Antigravity (AGY) Management & GitMap Parity:");
            println!(
                "  agm agy tree [all]              Project → Conversation → 200w Prompt Tree ([AGM:P001 | GM:#1])"
            );
            println!(
                "  agm agy active                  List active running prompts & AGM tree view"
            );
            println!(
                "  agm agy running-prompts [ls|backup|restore] Manage running storage prompts"
            );
            println!(
                "  agm agy backup                  Snapshot active running storage prompts (AGM + GitMap)"
            );
            println!("  agm agy restore                 Restore backed-up running prompts");
            println!("  agm agy fpug [ls|add-projects|run] Finish Prompts Until Green loop");
            println!("  agm agy sug [ls|add-projects|run]  Shutdown Until Green loop");
            println!("  agm agy rerun [N]               Rerun last N prompts from repo_prompts.db");
            println!("  agm agy queues                  List workspace prompt queues");
            println!("  agm agy ls                      List registered projects and workspaces");
            println!("  agm agy prompts                 List reusable prompt templates");
            println!(
                "  agm agy prompt <args>           Inject prompt by Seq ID (C001/P001/GM:#1), instance, or node"
            );
            println!(
                "  agm agy nodes                   List cluster VM nodes & connectivity status"
            );
            println!("\nGitMap AGY Direct Equivalents:");
            println!("  gitmap agy active");
            println!("  gitmap agy running-prompts ls | backup | restore");
            println!("  gitmap backup-running-prompts && gitmap restore-running-prompts");
            println!("  gitmap agy prompt -n read-all -t \"Read memory and continue\"");
            println!("  gitmap agy prompt -n is-done -t \"Verify if all tasks are complete\"");
            println!("  gitmap agy prompt-project P001 -n is-done -t \"Check build\"");
            println!("  gitmap agy fpug ls && gitmap agy sug ls");
        }
    }
}

pub(crate) fn cmd_clean(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("AGM Storage & Build Artifact Hygiene:");
        println!("  agm clean [--help]");
        println!("\nDescription:");
        println!("  Performs safe cleanup of temporary test directories, build artifacts,");
        println!("  and stale lock files while strictly protecting all database vaults.");
        println!("\nAliases: agm clean, agm purge");
        println!("\nExamples:");
        println!("  agm clean                           # Run safe artifact cleanup");
        return;
    }

    println!("[*] Performing safe AGM storage and build cache hygiene...");

    let mut removed_dirs = 0;
    let mut reclaimed_bytes: u64 = 0;

    // 1. Clean temporary test directories in OS temp
    let temp_dir = env::temp_dir();
    if let Ok(entries) = fs::read_dir(&temp_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with("antigravity_test_") {
                let p = entry.path();
                // Safety invariant: NEVER delete vault files
                if crate::common::is_safe_to_delete(&p) {
                    if let Ok(meta) = fs::metadata(&p) {
                        reclaimed_bytes += meta.len();
                    }
                    if fs::remove_dir_all(&p).is_ok() {
                        removed_dirs += 1;
                    }
                }
            }
        }
    }

    // 2. Clean build-demo, target-demo, and Cargo incremental compiler caches
    for target_name in &[
        "build-demo",
        "target-demo",
        "src-tauri/build-demo",
        "src-tauri/target-demo",
        "src-tauri/target/debug/incremental",
        "src-tauri/target/release/incremental",
        "src-tauri/target/debug/.fingerprint",
        "src-tauri/target/release/.fingerprint",
        "target/debug/incremental",
        "target/release/incremental",
        "target/debug/.fingerprint",
        "target/release/.fingerprint",
    ] {
        let target_p = PathBuf::from(target_name);
        if target_p.is_dir() {
            if let Ok(meta) = fs::metadata(&target_p) {
                reclaimed_bytes += meta.len();
            }
            if fs::remove_dir_all(&target_p).is_ok() {
                removed_dirs += 1;
            }
        }
    }

    // 2.5. Clean stale cargo locks or temporary debug outputs
    for lock_path in &["src-tauri/target/.cargo-lock", "target/.cargo-lock"] {
        let p = PathBuf::from(lock_path);
        if p.is_file() {
            let _ = fs::remove_file(p);
        }
    }

    println!(
        "    [✓] Temporary test, build-demo, and Cargo caches removed: {} folder(s)",
        removed_dirs
    );
    println!(
        "[SUCCESS] Cleanup finished. Space reclaimed: {} KB.",
        reclaimed_bytes / 1024
    );
    println!("\n  💡 Optimization & Next Steps Suggestions:");
    println!("    • Clear Terminal Session:    agm clear-terminal");
    println!("    • Clear AGM Cache Only:      agm clear-cache");
    println!("    • Verify SSH Fleet Health:   agm ssh nodes");
    println!("    • Deploy Public Keys:        agm ssh deploy-keys");
    println!("    • Inspect Failed Commands:   agm failed-commands\n");
}

pub(crate) fn cmd_history(args: &[String]) {
    if args
        .iter()
        .any(|arg| arg == "--help" || arg == "-h" || arg == "help")
    {
        println!("agm history [--page <n>] [--json]");
        println!("  Page size is 100. The root index is task_index.db beside the split files.");
        return;
    }
    let mut page: u32 = 1;
    let mut index = 0;
    while index < args.len() {
        if args[index] == "--page" || args[index] == "-p" {
            if let Some(next) = args.get(index + 1) {
                page = next.parse::<u32>().unwrap_or(1).max(1);
                index += 2;
                continue;
            }
        }
        index += 1;
    }
    let offset = (page - 1) * 100;
    match antigravity_tools_lib::modules::task_history_db::list_page(offset, 100) {
        Ok(result) => {
            if args.iter().any(|arg| arg == "--json") {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&result).unwrap_or_default()
                );
                return;
            }
            println!("Task history root: data/task-history or the app data task-history folder");
            for split in &result.splits {
                let state = if split.is_current {
                    "current"
                } else {
                    "closed"
                };
                println!(
                    "  split {} {} rows {} {}",
                    state, split.row_count, split.file_path, split.id
                );
            }
            println!(
                "Page {} of rows {}-{} ({} total)",
                page,
                offset + 1,
                offset + result.items.len() as u32,
                result.total
            );
            for item in &result.items {
                println!(
                    "  {} {} {} {} {}",
                    item.created_at, item.status, item.action_label, item.subject, item.detail
                );
            }
        }
        Err(err) => {
            eprintln!("[FAIL] history: {}", err);
            std::process::exit(1);
        }
    }
}

pub(crate) fn cmd_failed_commands(args: &[String]) {
    let main_cmd = std::env::args().nth(1).unwrap_or_default().to_lowercase();
    let is_count_sub = args
        .iter()
        .any(|a| a == "count" || a == "-c" || a == "--count" || a == "stats");
    let is_count_main = main_cmd == "fcc" || main_cmd == "failed-commands-count";
    let is_count = is_count_main || is_count_sub;
    let is_json = args.iter().any(|a| a == "--json" || a == "-j");

    if is_count {
        match repo_db::count_failed_commands() {
            Ok((distinct, total)) => {
                if is_json {
                    println!(
                        r#"{{"distinctCommands":{},"totalHits":{}}}"#,
                        distinct, total
                    );
                } else {
                    println!("\n  📊 Failed / Undetected Commands Count:");
                    println!("    • Distinct failed commands: {}", distinct);
                    println!("    • Total failed attempts:   {}", total);
                    println!();
                }
            }
            Err(e) => eprintln!("[ERROR] Failed to count failed commands: {}", e),
        }
        return;
    }

    let is_clear = args
        .iter()
        .any(|a| a == "clear" || a == "-y" || a == "--clear");
    if is_clear {
        match repo_db::clear_failed_commands() {
            Ok(cleared) => {
                println!(
                    "✅ Cleared {} recorded failed commands from database.",
                    cleared
                );
            }
            Err(e) => eprintln!("[ERROR] Failed to clear failed commands: {}", e),
        }
        return;
    }

    let limit = args
        .iter()
        .find_map(|a| a.parse::<usize>().ok())
        .unwrap_or(20);
    match repo_db::list_failed_commands(limit) {
        Ok(records) => {
            if is_json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&records).unwrap_or_default()
                );
            } else if records.is_empty() {
                println!(
                    "\n  ✓ No failed commands recorded. All entered commands were successfully recognized!\n"
                );
            } else {
                let (distinct, total) = repo_db::count_failed_commands()
                    .unwrap_or((records.len() as i64, records.len() as i64));
                println!("================================================================================");
                println!(
                    "  AGM Failed / Undetected Commands Inspector (Distinct: {}, Total Hits: {})",
                    distinct, total
                );
                println!("================================================================================");
                println!(
                    "  {:<4} {:<24} {:<8} {:<10} {}",
                    "#", "COMMAND", "HITS", "DOMAIN", "SUGGESTION"
                );
                println!("  ------------------------------------------------------------------------------");
                for (i, r) in records.iter().enumerate() {
                    println!(
                        "  {:<4} {:<24} {:<8} {:<10} {}",
                        i + 1,
                        r.command,
                        r.hit_count,
                        r.domain,
                        r.suggestions
                    );
                }
                println!("================================================================================");
                println!("  • Check count only: agm failed-commands count");
                println!("  • Clear history:    agm failed-commands clear\n");
            }
        }
        Err(e) => eprintln!("[ERROR] Failed to list failed commands: {}", e),
    }
}

pub(crate) fn cmd_gitignore(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("AGM Gitignore & Resume Task Hygiene:");
        println!("  agm gitignore [agm|agy] [path] [flags]");
        println!("\nDescription:");
        println!("  Untrack, delete, and ignore antigravity-resume_task.json across");
        println!("  repositories, committing deletion and .gitignore updates.");
        println!("\nExamples:");
        println!(
            "  agm gitignore agm                   # Untrack, delete, and ignore in current repo"
        );
        println!("  agm gitignore agm D:\\work           # Remediate repos in target directory\n");
        return;
    }

    if let Some(gitmap_bin) = crate::update_helpers::resolve_gitmap_bin() {
        let mut cmd = Command::new(gitmap_bin);
        cmd.arg("gitignore");
        cmd.arg("agm");
        for a in args {
            if a != "agm" && a != "agy" {
                cmd.arg(a);
            }
        }
        cmd.stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        if let Ok(status) = cmd.status() {
            if !status.success() {
                let code = status.code().unwrap_or(1);
                std::process::exit(code);
            }
            return;
        }
    }

    crate::remediate_cmds::remediate_repo_gitignore_native(args);
}
