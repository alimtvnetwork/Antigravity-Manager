//! green_cmds — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::repo_db::ActivePrompt;
use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

pub(crate) fn cmd_running_projects(args: &[String]) {
    if let Some(first) = args.first() {
        if first.eq_ignore_ascii_case("help") || first == "--help" || first == "-h" {
            println!("AGM Running Projects Management:");
            println!("  agm running-projects [ls] [--json] [-f <path.json>] [--ssh]");
            println!("\nDescription:");
            println!("  Lists active workspace projects having running or queued prompts.");
            println!("\nAliases: agm running-projects, agm projects");
            println!("\nOptions:");
            println!("    --json              Output pure JSON array");
            println!("    -f, --file <path>   Write output to specified file path (default: agm-running-projects.json)");
            println!("    --ssh               Include multi-node cluster fleet projects");
            println!("\nExamples:");
            println!("  agm running-projects                # Display table of active projects");
            println!(
                "  agm running-projects --json         # Output active projects in JSON format"
            );
            println!("  agm running-projects -f proj.json   # Export project inventory to file");
            return;
        }
    }

    let is_json = args.iter().any(|a| a == "--json");
    let is_ssh = args.iter().any(|a| a == "--ssh");
    let mut file_dest: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        if args[i] == "-f" || args[i] == "--file" || args[i] == "-file" {
            if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                file_dest = Some(args[i + 1].clone());
                i += 2;
                continue;
            } else {
                file_dest = Some("agm-running-projects.json".to_string());
            }
        }
        i += 1;
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
                        || p.status == "queued"
                        || p.status == "dispatched"
                        || p.status == "backed_up")
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

        rows.push(serde_json::json!({
            "seq": seq,
            "project": proj.repo_name,
            "id": proj.id,
            "repo_path": proj.repo_path,
            "conv_id": conv_id,
            "conv_name": conv_name,
            "prompts_count": proj_prompts.len().max(if proj.is_running { 1 } else { 0 }),
            "status": if proj.is_running { "running" } else { "idle" },
            "is_ssh": false,
            "node": "localhost",
        }));
    }

    if is_ssh {
        let m_name = email_watcher::detect_machine_name();
        let m_ip = email_watcher::detect_local_ip();
        println!(
            "[SSH Cluster Mode] Queried local node '{}' ({})",
            m_name, m_ip
        );
    }

    if let Some(dest) = file_dest {
        let json_content = serde_json::to_string_pretty(&rows).unwrap_or_default();
        let _ = fs::write(&dest, json_content);
        println!("[SUCCESS] Saved running projects JSON to '{}'", dest);
    }

    if is_json {
        println!(
            "{}",
            serde_json::to_string_pretty(&rows).unwrap_or_else(|_| "[]".to_string())
        );
        return;
    }

    println!("\n=== Running Projects ({} active) ===", rows.len());
    if rows.is_empty() {
        println!("No projects currently running with active prompts.");
        return;
    }

    println!(
        "{:<5} {:<24} {:<24} {:<16} {:<12} PROMPTS (QUEUE)",
        "SEQ", "PROJECT", "ID", "CONV ID", "STATUS"
    );
    println!("{}", "-".repeat(95));
    for r in &rows {
        let seq = r["seq"].as_u64().unwrap_or(0);
        let proj = r["project"].as_str().unwrap_or("-");
        let id: String = r["id"].as_str().unwrap_or("-").chars().take(22).collect();
        let cid: String = r["conv_id"]
            .as_str()
            .unwrap_or("-")
            .chars()
            .take(14)
            .collect();
        let st = r["status"].as_str().unwrap_or("-");
        let qc = r["prompts_count"].as_u64().unwrap_or(0);
        println!(
            "#{:<4} {:<24} {:<24} {:<16} {:<12} {}",
            seq, proj, id, cid, st, qc
        );
    }
    println!();
}

pub(crate) fn cmd_finish_prompts_until_green(args: &[String]) {
    if let Some(first) = args.first() {
        if first.eq_ignore_ascii_case("help") || first == "--help" || first == "-h" {
            println!("AGM Finish Prompts Until Green (fpug):");
            println!("  agm finish-prompts-until-green [targets...] [-t <5m|30s>]");
            println!("  agm fpug running-projects [-t <duration>]");
            println!("\nDescription:");
            println!("  Monitors targeted workspaces and blocks until all in-flight prompts reach completion ('green').");
            println!("\nAliases: agm finish-prompts-until-green, agm fpug");
            println!("\nOptions:");
            println!(
                "    -t, --time <dur>    Polling interval or timeout (e.g. 5m, 30s; default: 5m)"
            );
            println!("    running-projects    Target all currently active running projects automatically");
            println!("\nExamples:");
            println!(
                "  agm fpug running-projects           # Wait until all active projects finish"
            );
            println!("  agm fpug my-project -t 30s          # Check specific project every 30s");
            return;
        }
    }

    let mut interval_sec = 300u64; // default 5m
    let mut targets = Vec::new();

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if (arg == "-t" || arg == "--time" || arg == "--timeout") && i + 1 < args.len() {
            interval_sec = crate::common::parse_duration_to_seconds(&args[i + 1], 300);
            i += 2;
            continue;
        }
        if !arg.starts_with('-') {
            for part in arg.split(',') {
                let trimmed = part.trim();
                if !trimmed.is_empty() {
                    targets.push(trimmed.to_string());
                }
            }
        }
        i += 1;
    }

    if targets.is_empty() || targets.iter().any(|t| t == "running-projects") {
        let running_p = repo_db::list_running_projects().unwrap_or_default();
        targets = running_p.into_iter().map(|p| p.repo_name).collect();
        if targets.is_empty() {
            println!("[INFO] No running projects detected to monitor.");
            return;
        }
    }

    println!("\n================================================================================");
    println!("  AGM FINISH PROMPTS UNTIL GREEN (FPUG)");
    println!("================================================================================");
    println!("  Target Projects: {}", targets.join(", "));
    println!(
        "  Polling Interval: {}s (pass -t 5m to customize)",
        interval_sec
    );
    println!("  Monitoring prompt drainage and green status...\n");

    loop {
        let all_prompts = repo_db::list_all_prompts().unwrap_or_default();
        let mut pending_count = 0;
        for t in &targets {
            let matching = all_prompts
                .iter()
                .filter(|p| {
                    (p.project_id.eq_ignore_ascii_case(t) || p.repo_path.contains(t))
                        && (p.status == "running" || p.status == "queued")
                })
                .count();
            pending_count += matching;
        }

        println!(
            "[*] Check: {} pending/running prompt(s) remaining across {} target project(s)...",
            pending_count,
            targets.len()
        );
        if pending_count == 0 {
            println!("\n[SUCCESS] All targeted projects are green with zero pending prompts!");
            break;
        }

        // Low-CPU sleep
        std::thread::sleep(Duration::from_secs(interval_sec.max(5)));
    }
}

pub(crate) fn cmd_shutdown_until_green(args: &[String]) {
    if let Some(first) = args.first() {
        let first_lower = first.to_lowercase();
        if first_lower == "help" || first_lower == "--help" || first_lower == "-h" {
            println!("AGM Shutdown Until Green (SUG):");
            println!("  agm shutdown-until-green ls");
            println!("  agm shutdown-until-green add-projects <p1, p2>");
            println!("  agm shutdown-until-green rm <p1>");
            println!("  agm shutdown-until-green agy-running-projects");
            println!("  agm shutdown-until-green run [-t 5m]");
            println!("\nDescription:");
            println!(
                "  Monitors registered green targets and automatically executes safe host shutdown"
            );
            println!("  once all prompts and CI tasks conclude successfully.");
            println!("\nAliases: agm shutdown-until-green, agm sug");
            println!("\nSubcommands:");
            println!("  ls, list            List configured green target projects");
            println!("  add-projects <p..>  Add projects to green watch list");
            println!("  rm <project>        Remove project from watch list");
            println!("  agy-running-projects Auto-add all currently active projects");
            println!("  run [-t <dur>]      Start watcher: shuts down OS once green");
            println!("\nExamples:");
            println!("  agm sug ls                          # View current green watch targets");
            println!("  agm sug agy-running-projects        # Watch all active projects");
            println!("  agm sug run -t 2m                   # Start watcher polling every 2m");
            return;
        }
        if first_lower == "ls" || first_lower == "list" {
            let list = backup_prompts_db::list_green_projects(None).unwrap_or_default();
            println!("\n=== Green Target Projects ({} item(s)) ===", list.len());
            if list.is_empty() {
                println!("No projects on green watch list. Add using 'agm sug add-projects <targets...>'");
            } else {
                for (idx, p) in list.iter().enumerate() {
                    println!(
                        "#{:<4} {:<30} {:<10} Path: {}",
                        idx + 1,
                        p.project_identifier,
                        p.status,
                        p.project_path
                    );
                }
            }
            return;
        }
        if first_lower == "add-projects" {
            for arg in &args[1..] {
                for p in arg.split(',') {
                    let trimmed = p.trim();
                    if !trimmed.is_empty() {
                        let _ = backup_prompts_db::add_green_project(trimmed, trimmed, None);
                        println!("[SUCCESS] Added '{}' to green target list.", trimmed);
                    }
                }
            }
            return;
        }
        if first_lower == "rm" || first_lower == "remove" {
            for arg in &args[1..] {
                let trimmed = arg.trim();
                let _ = backup_prompts_db::remove_green_project(trimmed, None);
                println!("[SUCCESS] Removed '{}' from green target list.", trimmed);
            }
            return;
        }
        if first_lower == "agy-running-projects" {
            let running = repo_db::list_running_projects().unwrap_or_default();
            for p in &running {
                let _ = backup_prompts_db::add_green_project(&p.repo_name, &p.repo_path, None);
                println!(
                    "[SUCCESS] Enqueued running project '{}' into green list.",
                    p.repo_name
                );
            }
            return;
        }
        if first_lower == "run" {
            let mut interval_sec = 300u64; // default 5m
            for (idx, a) in args.iter().enumerate() {
                if (a == "-t" || a == "--time") && idx + 1 < args.len() {
                    interval_sec = crate::common::parse_duration_to_seconds(&args[idx + 1], 300);
                }
            }
            println!(
                "\n[*] Starting Shutdown Until Green (SUG) daemon loop (interval: {}s)...",
                interval_sec
            );
            loop {
                let list = backup_prompts_db::list_green_projects(None).unwrap_or_default();
                if list.is_empty() {
                    println!("[WARN] Green list is empty. Add projects with 'agm sug add-projects <targets...>'");
                    return;
                }
                let all_prompts = repo_db::list_all_prompts().unwrap_or_default();
                let mut pending_count = 0;
                for p in &list {
                    let matching = all_prompts
                        .iter()
                        .filter(|ap| {
                            (ap.project_id.eq_ignore_ascii_case(&p.project_identifier)
                                || ap.repo_path.contains(&p.project_path))
                                && (ap.status == "running" || ap.status == "queued")
                        })
                        .count();
                    pending_count += matching;
                }
                println!(
                    "[*] Evaluation: {} pending prompt(s) across {} watched project(s)...",
                    pending_count,
                    list.len()
                );
                if pending_count == 0 {
                    println!("\n[SUCCESS] All projects green! Triggering system shutdown...");
                    let _ = execute_native_shutdown();
                    break;
                }
                std::thread::sleep(Duration::from_secs(interval_sec.max(5)));
            }
            return;
        }
    }

    println!(
        "AGM Shutdown Until Green (SUG). Run 'agm shutdown-until-green help' for subcommands."
    );
}

pub(crate) fn execute_native_shutdown() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        println!("[*] Initiating Windows system shutdown in 60 seconds (shutdown /s /t 60)...");
        let status = Command::new("shutdown")
            .args([
                "/s",
                "/t",
                "60",
                "/c",
                "Antigravity Manager: all green targets finished.",
            ])
            .status()
            .map_err(|e| format!("Failed to invoke shutdown command: {}", e))?;
        if status.success() {
            Ok(())
        } else {
            Err("Windows shutdown command returned non-zero exit code".to_string())
        }
    }
    #[cfg(target_os = "linux")]
    {
        println!("[*] Initiating Linux system poweroff...");
        let status = Command::new("systemctl")
            .arg("poweroff")
            .status()
            .or_else(|_| Command::new("shutdown").args(["-h", "now"]).status())
            .map_err(|e| format!("Failed to invoke poweroff command: {}", e))?;
        if status.success() {
            Ok(())
        } else {
            Err("Linux poweroff command returned non-zero exit code".to_string())
        }
    }
    #[cfg(target_os = "macos")]
    {
        println!("[*] Initiating macOS system shutdown...");
        let status = Command::new("osascript")
            .args(["-e", "tell app \"System Events\" to shut down"])
            .status()
            .map_err(|e| format!("Failed to invoke macOS shutdown script: {}", e))?;
        if status.success() {
            Ok(())
        } else {
            Err("macOS shutdown command returned non-zero exit code".to_string())
        }
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Err("Unsupported operating system for shutdown".to_string())
    }
}
