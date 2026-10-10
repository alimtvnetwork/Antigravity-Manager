//! switch_credit_cmds — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn cmd_switch_if_low_credit(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("AGM Switch If Low Credit:");
        println!("  agm switch-if-low-credit [-t <pct>] [--json] [-f [path]] [--force]");
        println!("  Checks current active profile quota. If <= threshold, triggers rotation to highest quota account.");
        println!("\nAliases: agm switch-if-low-credit, agm swlc, agm sfc");
        println!("\nOptions:");
        println!("    -t, --threshold <N>   Threshold percentage to evaluate (default: 15.0%)");
        println!("    --json                Output evaluation result in JSON format");
        println!("    -f, --file [path]     Export JSON status to file");
        println!("    --force               Force rotation evaluation ignoring cooldown");
        println!("\nExamples:");
        println!("  agm switch-if-low-credit                  # Switch if active quota <= 15%");
        println!(
            "  agm switch-if-low-credit -t 98            # Simulation test: switch if quota <= 98%"
        );
        println!("  agm switch-if-low-credit -t 15 --json     # Query with JSON output");
        return;
    }

    let is_json = args.iter().any(|a| a == "--json");
    let force = args.iter().any(|a| a == "--force");
    let mut custom_threshold: Option<f64> = None;
    let mut export_file: Option<String> = None;
    let mut should_export_file = false;

    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if arg == "--threshold" || arg == "-t" {
            if i + 1 < args.len() {
                custom_threshold = args[i + 1].parse::<f64>().ok();
                i += 2;
                continue;
            }
        } else if arg == "-f" || arg == "--file" {
            should_export_file = true;
            if i + 1 < args.len() && !args[i + 1].starts_with('-') {
                export_file = Some(args[i + 1].clone());
                i += 2;
                continue;
            }
        } else if !arg.starts_with('-') {
            if let Ok(val) = arg.parse::<f64>() {
                custom_threshold = Some(val);
            }
        }
        i += 1;
    }

    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("[ERROR] Failed to initialize async runtime: {}", e);
            std::process::exit(1);
        }
    };

    let machine_name = email_watcher::detect_machine_name();
    let node_alias = supabase_sync::load_config()
        .map(|c| c.node_alias)
        .unwrap_or_else(|_| machine_name.clone());
    let local_ip = email_watcher::detect_local_ip();
    let tool_version = format!("v{}", crate::common::VERSION);

    // Capture running prompt snippet and image status
    let mut running_prompt_snippet: Option<String> = None;
    let mut has_images = false;
    if let Ok(prompts) = repo_db::list_all_prompts() {
        if let Some(p) = prompts
            .into_iter()
            .find(|p| p.status == "running" || p.status == "dispatched" || p.status == "backed_up")
        {
            let snippet = if p.prompt_content.len() > 120 {
                format!("{}...", &p.prompt_content[..120])
            } else {
                p.prompt_content
            };
            running_prompt_snippet = Some(snippet);
            if p.image_payload.is_some() {
                has_images = true;
            }
        }
    }
    if running_prompt_snippet.is_none() {
        if let Ok(projects) = repo_db::list_running_projects() {
            for proj in projects {
                let resume_file =
                    std::path::PathBuf::from(&proj.repo_path).join(".antigravity_resume_task.json");
                if resume_file.exists() {
                    if let Ok(content) = std::fs::read_to_string(&resume_file) {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                            if let Some(prompt_text) =
                                val.get("prompt_content").and_then(|v| v.as_str())
                            {
                                let snippet = if prompt_text.len() > 120 {
                                    format!("{}...", &prompt_text[..120])
                                } else {
                                    prompt_text.to_string()
                                };
                                running_prompt_snippet = Some(snippet);
                                if val.get("image_payload").and_then(|v| v.as_str()).is_some()
                                    || val
                                        .get("has_image")
                                        .and_then(|v| v.as_bool())
                                        .unwrap_or(false)
                                {
                                    has_images = true;
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    let status_before = auto_switcher::get_status();
    let app_cfg = config::load_app_config().unwrap_or_default();
    let effective_threshold =
        custom_threshold.unwrap_or(app_cfg.auto_profile_switcher.low_quota_threshold_percent);

    // Snapshot & backup all running prompts across Antigravity and workspaces before switch
    let _ = repo_db::backup_running_prompts("default");

    let res = rt.block_on(auto_switcher::check_and_rotate_for_threshold(
        custom_threshold,
        force,
    ));
    let status_after = auto_switcher::get_status();

    if res.as_ref().map(|o| o.is_some()).unwrap_or(false) {
        // Immediately restore and trigger execution of running prompts
        let _ = repo_db::resend_all_running_commands(20);
    }

    let running_prompts_count = repo_db::list_all_prompts()
        .unwrap_or_default()
        .iter()
        .filter(|p| p.status == "running" || p.status == "dispatched" || p.status == "backed_up")
        .count()
        .max(if running_prompt_snippet.is_some() {
            1
        } else {
            0
        });

    match res {
        Ok(Some(reason)) => {
            let prompts_resent = running_prompts_count > 0;
            let mut prev_email = status_before.active_account_email.clone();
            let selected_email = status_after.active_account_email.clone();
            if let (Some(ref p), Some(ref s)) = (&prev_email, &selected_email) {
                if p.trim().eq_ignore_ascii_case(s.trim()) {
                    prev_email = None;
                }
            }
            let mut pred_exclusions = Vec::new();
            if let Some(ref p) = prev_email {
                pred_exclusions.push(p.clone());
            }
            if let Some(ref s) = selected_email {
                pred_exclusions.push(s.clone());
            }
            let predicted_candidate = auto_switcher::select_candidate_profiles(
                "default",
                "gemini-2.5-pro",
                15.0,
                &pred_exclusions,
            )
            .ok()
            .and_then(|v| v.into_iter().next())
            .filter(|c| {
                selected_email
                    .as_deref()
                    .map(|s| !c.email.trim().eq_ignore_ascii_case(s.trim()))
                    .unwrap_or(true)
                    && prev_email
                        .as_deref()
                        .map(|p| !c.email.trim().eq_ignore_ascii_case(p.trim()))
                        .unwrap_or(true)
            });
            let predicted_email = predicted_candidate.map(|c| c.email);
            let credit_before = status_before.current_quota_percent;

            let out = serde_json::json!({
                "rotated": true,
                "reason": reason,
                "machine_name": machine_name,
                "node_alias": node_alias,
                "vm_alias": node_alias,
                "local_ip": local_ip,
                "tool_version": tool_version,
                "previous_account": prev_email,
                "predicted_next_account": predicted_email,
                "selected_account": selected_email,
                "active_account": status_after.active_account_email,
                "credit_before_switch": credit_before,
                "threshold_activated": effective_threshold,
                "quota_percent": status_after.current_quota_percent,
                "running_prompts_count": running_prompts_count,
                "prompts_resent": prompts_resent,
                "is_reinjecting": prompts_resent,
                "running_prompt": running_prompt_snippet,
                "has_images": has_images,
                "images_attached": has_images,
                "timestamp": chrono::Utc::now().timestamp(),
            });
            let out_str = serde_json::to_string_pretty(&out).unwrap_or_default();
            if should_export_file {
                let file_path =
                    crate::common::resolve_switch_filename(export_file.as_deref(), &node_alias);
                let _ = std::fs::write(&file_path, &out_str);
                if !is_json {
                    println!("[SUCCESS] Saved switch telemetry to {}", file_path);
                }
            }
            if is_json {
                println!("{}", out_str);
            } else {
                println!("[SUCCESS] Low-credit rotation triggered!");
                println!("          Reason:                 {}", reason);
                println!(
                    "          Previous Account:       {}",
                    prev_email.as_deref().unwrap_or("(none / standby)")
                );
                println!(
                    "          Predicted Next Account: {}",
                    predicted_email.as_deref().unwrap_or("(none)")
                );
                println!(
                    "          Selected Account:       {}",
                    selected_email.as_deref().unwrap_or("(none)")
                );
                println!(
                    "          Credit Before Switch:   {:.1}%",
                    credit_before.unwrap_or(0.0)
                );
                println!(
                    "          Threshold Activated:    {:.1}%",
                    effective_threshold
                );
                println!(
                    "          Running Prompts:        {} (Resent / Re-injected: {})",
                    running_prompts_count,
                    if prompts_resent {
                        "Yes (Auto-Resumed)"
                    } else {
                        "No"
                    }
                );
                println!(
                    "          Attached Images:        {}",
                    if has_images {
                        "Yes (Preserved)"
                    } else {
                        "None"
                    }
                );
            }
        }
        Ok(None) => {
            let current_email = status_after.active_account_email.clone();
            let credit_before = status_after.current_quota_percent;

            let mut pred_exclusions = Vec::new();
            if let Some(ref c) = current_email {
                pred_exclusions.push(c.clone());
            }
            let predicted_candidate = auto_switcher::select_candidate_profiles(
                "default",
                "gemini-2.5-pro",
                15.0,
                &pred_exclusions,
            )
            .ok()
            .and_then(|v| v.into_iter().next())
            .filter(|c| {
                current_email
                    .as_deref()
                    .map(|curr| !c.email.trim().eq_ignore_ascii_case(curr.trim()))
                    .unwrap_or(true)
            });
            let predicted_email = predicted_candidate.map(|c| c.email);

            let out = serde_json::json!({
                "rotated": false,
                "reason": "Quota is healthy (above threshold) or no alternative candidate needed",
                "machine_name": machine_name,
                "node_alias": node_alias,
                "vm_alias": node_alias,
                "local_ip": local_ip,
                "tool_version": tool_version,
                "previous_account": serde_json::Value::Null,
                "current_account": current_email,
                "predicted_next_account": predicted_email,
                "selected_account": serde_json::Value::Null,
                "active_account": status_after.active_account_email,
                "credit_before_switch": credit_before,
                "threshold_activated": effective_threshold,
                "quota_percent": status_after.current_quota_percent,
                "running_prompts_count": running_prompts_count,
                "prompts_resent": false,
                "is_reinjecting": false,
                "running_prompt": running_prompt_snippet,
                "has_images": has_images,
                "images_attached": has_images,
                "timestamp": chrono::Utc::now().timestamp(),
            });
            let out_str = serde_json::to_string_pretty(&out).unwrap_or_default();
            if should_export_file {
                let file_path =
                    crate::common::resolve_switch_filename(export_file.as_deref(), &node_alias);
                let _ = std::fs::write(&file_path, &out_str);
                if !is_json {
                    println!("[INFO] Saved switch evaluation to {}", file_path);
                }
            }
            if is_json {
                println!("{}", out_str);
            } else {
                println!(
                    "[OK] Credits are sufficient ({:.1}% remaining on {}). No switch needed.",
                    status_after.current_quota_percent.unwrap_or(100.0),
                    status_after
                        .active_account_email
                        .as_deref()
                        .unwrap_or("current profile")
                );
                println!(
                    "     Active Account:       {}",
                    status_after
                        .active_account_email
                        .as_deref()
                        .unwrap_or("none")
                );
                println!(
                    "     Credit Before Check:  {:.1}%",
                    credit_before.unwrap_or(100.0)
                );
                println!("     Configured Threshold: {:.1}%", effective_threshold);
                println!("     Running Prompts:      {}", running_prompts_count);
            }
        }
        Err(e) => {
            let out = serde_json::json!({
                "rotated": false,
                "error": e,
                "machine_name": machine_name,
                "node_alias": node_alias,
                "local_ip": local_ip,
                "tool_version": tool_version,
                "timestamp": chrono::Utc::now().timestamp(),
            });
            let out_str = serde_json::to_string_pretty(&out).unwrap_or_default();
            if should_export_file {
                let file_path =
                    crate::common::resolve_switch_filename(export_file.as_deref(), &node_alias);
                let _ = std::fs::write(&file_path, &out_str);
            }
            if is_json {
                println!("{}", out_str);
            } else {
                eprintln!("[ERROR] switch-if-low-credit failed: {}", e);
            }
            std::process::exit(1);
        }
    }
}
