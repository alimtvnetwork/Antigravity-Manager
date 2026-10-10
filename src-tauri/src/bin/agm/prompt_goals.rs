//! prompt_goals — CLI command handlers, split from agm.rs.

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

pub(crate) fn cmd_prompt_goal_worker(args: &[String]) {
    let mut instance_id = "default".to_string();
    let mut data_dir = String::new();
    let mut repo_path = ".".to_string();
    let mut heartbeat_file = ".antigravity_goal_prompt.log".to_string();
    let mut interval = 5u64;
    let mut prompt = "Continuous 5-second prompt goal heartbeat verification".to_string();

    let mut idx = 0;
    while idx < args.len() {
        let arg = &args[idx];
        if arg == "--instance" && idx + 1 < args.len() {
            instance_id = args[idx + 1].clone();
            idx += 2;
        } else if (arg == "--user-data-dir" || arg == "--data-dir") && idx + 1 < args.len() {
            data_dir = args[idx + 1].clone();
            idx += 2;
        } else if arg.starts_with("--user-data-dir=") {
            data_dir = arg.trim_start_matches("--user-data-dir=").to_string();
            idx += 1;
        } else if (arg == "--project" || arg == "--workspace" || arg == "--repo")
            && idx + 1 < args.len()
        {
            repo_path = args[idx + 1].clone();
            idx += 2;
        } else if arg == "--heartbeat-file" && idx + 1 < args.len() {
            heartbeat_file = args[idx + 1].clone();
            idx += 2;
        } else if arg == "--interval" && idx + 1 < args.len() {
            interval = args[idx + 1].parse().unwrap_or(5);
            idx += 2;
        } else if arg == "--prompt" && idx + 1 < args.len() {
            prompt = args[idx + 1].clone();
            idx += 2;
        } else {
            idx += 1;
        }
    }

    if data_dir.is_empty() {
        if let Ok(registry) = instance::load_registry() {
            if let Some(inst) = registry.instances.iter().find(|i| i.id == instance_id) {
                data_dir = inst.data_dir.clone();
            }
        }
    }

    repo_db::run_prompt_goal_worker_loop(
        &instance_id,
        &data_dir,
        &repo_path,
        &heartbeat_file,
        interval,
        &prompt,
    );
}

pub(crate) fn cmd_prompt_start_goal(args: &[String]) {
    let mut instance_id = "default".to_string();
    let mut data_dir = String::new();
    let mut repo_path = ".".to_string();
    let mut heartbeat_file = ".antigravity_goal_prompt.log".to_string();
    let mut interval = 5u64;
    let mut prompt = "Continuous 5-second prompt goal heartbeat verification".to_string();

    let mut idx = 0;
    while idx < args.len() {
        let arg = &args[idx];
        if arg == "--instance" && idx + 1 < args.len() {
            instance_id = args[idx + 1].clone();
            idx += 2;
        } else if (arg == "--user-data-dir" || arg == "--data-dir") && idx + 1 < args.len() {
            data_dir = args[idx + 1].clone();
            idx += 2;
        } else if (arg == "--project" || arg == "--workspace" || arg == "--repo")
            && idx + 1 < args.len()
        {
            repo_path = args[idx + 1].clone();
            idx += 2;
        } else if arg == "--heartbeat-file" && idx + 1 < args.len() {
            heartbeat_file = args[idx + 1].clone();
            idx += 2;
        } else if arg == "--interval" && idx + 1 < args.len() {
            interval = args[idx + 1].parse().unwrap_or(5);
            idx += 2;
        } else if arg == "--prompt" && idx + 1 < args.len() {
            prompt = args[idx + 1].clone();
            idx += 2;
        } else {
            idx += 1;
        }
    }

    if data_dir.is_empty() {
        if let Ok(registry) = instance::load_registry() {
            if let Some(inst) = registry.instances.iter().find(|i| i.id == instance_id) {
                data_dir = inst.data_dir.clone();
            }
        }
    }

    match repo_db::start_prompt_goal_heartbeat(
        &instance_id,
        &data_dir,
        &repo_path,
        &heartbeat_file,
        &prompt,
        interval,
    ) {
        Ok(cfg) => {
            println!(
                "[SUCCESS] Prompt goal heartbeat started for instance '{}':",
                instance_id
            );
            println!("  ● Heartbeat File: {}", cfg.heartbeat_file);
            println!("  ● Interval:       {} seconds", cfg.interval_secs);
            println!("  ● Prompt Goal:    {}", cfg.prompt_content);
            println!("  ● Config:         .antigravity_goal_heartbeat.json");
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to start prompt goal heartbeat: {}", e);
            std::process::exit(1);
        }
    }
}

pub(crate) fn cmd_prompt_check_goal(args: &[String]) {
    let mut instance_id = "default".to_string();
    let mut repo_path = ".".to_string();

    let mut idx = 0;
    while idx < args.len() {
        let arg = &args[idx];
        if arg == "--instance" && idx + 1 < args.len() {
            instance_id = args[idx + 1].clone();
            idx += 2;
        } else if (arg == "--project" || arg == "--workspace" || arg == "--repo")
            && idx + 1 < args.len()
        {
            repo_path = args[idx + 1].clone();
            idx += 2;
        } else if !arg.starts_with('-') {
            instance_id = arg.clone();
            idx += 1;
        } else {
            idx += 1;
        }
    }

    let (is_alive, hb_file, last_line) =
        repo_db::inspect_prompt_goal_status(&instance_id, &[repo_path.clone()]);
    println!(
        "Prompt Goal Status for Instance '{}' (workspace: '{}'):",
        instance_id, repo_path
    );
    println!(
        "  ● Active/Fresh: {}",
        if is_alive {
            "YES (RUNNING)"
        } else {
            "NO (STOPPED or Stale)"
        }
    );
    if let Some(f) = hb_file {
        println!("  ● Heartbeat File: {}", f);
    }
    if let Some(l) = last_line {
        println!("  ● Latest Record:  {}", l);
    }
}

pub(crate) fn cmd_observe(args: &[String]) {
    let is_json = args.iter().any(|a| a == "--json" || a == "-j");
    let target = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .map(|s| s.as_str())
        .unwrap_or("active");

    let resolved = match instance::resolve_instance_id(target) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("[ERROR] Failed to resolve instance '{}': {}", target, e);
            std::process::exit(1);
        }
    };

    match instance::observe_instance(&resolved) {
        Ok(obs) => {
            if is_json {
                println!("{}", serde_json::to_string_pretty(&obs).unwrap_or_default());
            } else {
                println!("================================================================================");
                println!(
                    "  AGM Instance Live Observation: '{}' ({})",
                    obs.name, obs.instance_id
                );
                println!("================================================================================");
                println!(
                    "  ● Status:                 {}",
                    if obs.is_running {
                        format!("RUNNING (PIDs: {:?})", obs.pids)
                    } else {
                        "STOPPED".to_string()
                    }
                );
                println!("  ● Data Directory:         {}", obs.data_dir);
                println!(
                    "  ● Bound Account Email:    {}",
                    obs.bound_account_email.as_deref().unwrap_or("None")
                );
                println!(
                    "  ● Injected state.vscdb:   {}",
                    obs.injected_email_in_db.as_deref().unwrap_or("None")
                );
                println!("  ● Active Prompts Queue:   {}", obs.active_prompts_count);
                println!(
                    "  ● Prompt Goal Running:    {}",
                    if obs.prompt_goal_running {
                        "YES (5s Heartbeat Active)"
                    } else {
                        "NO / IDLE"
                    }
                );
                if let Some(ref last_ts) = obs.last_heartbeat_timestamp {
                    println!("  ● Last Verified At:       {}", last_ts);
                }
                if let Some(ref last_line) = obs.last_heartbeat_line {
                    println!("  ● Heartbeat Telemetry:    {}", last_line);
                }
                if !obs.workspace_folders.is_empty() {
                    println!(
                        "  ● Bound Workspaces:       {}",
                        obs.workspace_folders.join(", ")
                    );
                }
                println!("================================================================================");
            }
        }
        Err(e) => {
            eprintln!("[ERROR] Failed to observe instance '{}': {}", resolved, e);
            std::process::exit(1);
        }
    }
}

pub(crate) fn cmd_rerun(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("AGM Rerun Prompts:");
        println!("  agm rerun [N] [--prefix <category>] [--suffix <category>]");
        println!("\nDescription:");
        println!("  Re-queues and dispatches the last N executed prompts from the database into");
        println!("  the active workspace with optional template prefix/suffix wrappers.");
        println!("\nExamples:");
        println!("  agm rerun                           # Rerun the most recent prompt");
        println!("  agm rerun 3                         # Rerun the last 3 prompts in sequence");
        println!("  agm rerun 1 --prefix coding-standards # Wrap prompt with prefix template");
        return;
    }

    let mut count_n: usize = 1;
    let mut prefix_cat: Option<String> = None;
    let mut suffix_cat: Option<String> = None;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg.eq_ignore_ascii_case("prompts") || arg.eq_ignore_ascii_case("prompt") {
            i += 1;
            continue;
        } else if arg == "-prefix" || arg == "--prefix" {
            if i + 1 < args.len() {
                prefix_cat = Some(args[i + 1].clone());
                i += 2;
                continue;
            }
        } else if arg == "-suffix" || arg == "--suffix" {
            if i + 1 < args.len() {
                suffix_cat = Some(args[i + 1].clone());
                i += 2;
                continue;
            }
        } else if let Ok(n) = arg.parse::<usize>() {
            count_n = n.max(1);
        }
        i += 1;
    }

    if Path::new(".git").exists() {
        println!("[*] Running git pull before rerunning prompt(s)...");
        let _ = Command::new("git").args(["pull"]).status();
    }

    let mut prompts = repo_db::list_all_prompts().unwrap_or_default();
    if let Ok(cwd) = env::current_dir() {
        let cwd_norm = cwd.to_string_lossy().to_lowercase().replace('\\', "/");
        let repo_matched: Vec<repo_db::ActivePrompt> = prompts
            .iter()
            .filter(|p| {
                let rp = p.repo_path.to_lowercase().replace('\\', "/");
                !rp.is_empty() && (cwd_norm.starts_with(&rp) || rp.starts_with(&cwd_norm))
            })
            .cloned()
            .collect();
        if !repo_matched.is_empty() {
            prompts = repo_matched;
        }
    }

    if prompts.is_empty() {
        eprintln!("[ERROR] No historical prompts found in repo_prompts.db to rerun.");
        std::process::exit(1);
    }

    prompts.truncate(count_n);
    prompts.reverse(); // Rerun in chronological ASC order

    let now = chrono::Utc::now().timestamp();
    let conn_opt = repo_db::connect_db().ok();

    for (idx, p) in prompts.iter().enumerate() {
        let wrapped = crate::prompt_dispatch::wrap_prompt_with_templates(
            &p.prompt_content,
            prefix_cat.as_deref(),
            suffix_cat.as_deref(),
        );
        if let Some(ref conn) = conn_opt {
            let _ = conn.execute(
                "UPDATE active_prompts SET prompt_content = ?1, status = 'dispatched', updated_at = ?2 WHERE id = ?3",
                rusqlite::params![&wrapped, now, &p.id],
            );
        }

        let task_file = PathBuf::from(&p.repo_path).join(".antigravity_resume_task.json");
        let payload = serde_json::json!({
            "prompt_id": p.id,
            "project_id": p.project_id,
            "instance_id": p.instance_id,
            "repo_path": p.repo_path,
            "prompt_content": wrapped,
            "model": p.model,
            "image_payload": p.image_payload,
            "rerun_seq": idx + 1,
            "resumed_at": now,
        });
        if let Ok(js) = serde_json::to_string_pretty(&payload) {
            let _ = fs::write(&task_file, js);
        }

        let (preview, _) = crate::common::truncate_words(&wrapped, 20);
        println!(
            "  [Rerun #{}] Project '{}' -> {}",
            idx + 1,
            p.project_id,
            preview
        );
    }

    println!(
        "[SUCCESS] Queued and dispatched {} prompt(s) for rerun.",
        prompts.len()
    );
}
