//! switch_check_cmds — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use chrono::Utc;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn cmd_is_low_credit_for_switch(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("AGM Is Low Credit Check:");
        println!("  agm is-low-credit-for-switch [-t <pct>] [--json] [-f [file]]");
        println!("\nDescription:");
        println!("  Evaluates active profile credits against a specified threshold.");
        println!("  Outputs true/false or JSON to indicate whether quota is below threshold.");
        println!("\nAliases: agm is-low-credit-for-switch, agm is-low-credit, agm ilc");
        println!("\nOptions:");
        println!("    -t, --threshold <N>   Threshold percentage to evaluate (default: 15.0%)");
        println!("    --json, -j            Output evaluation result in pure JSON format");
        println!("    -f, --file [path]     Export evaluation JSON payload to disk");
        println!("\nExamples:");
        println!("  agm ilc                             # Returns true/false based on 15% default");
        println!(
            "  agm ilc -t 98.0                     # Evaluate against 98% simulation threshold"
        );
        println!("  agm ilc -t 20.0 --json              # Output JSON evaluation");
        return;
    }

    let is_json = args.iter().any(|a| a == "--json");
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

    let app_config = config::load_app_config().unwrap_or_default();
    let switcher_cfg = app_config.auto_profile_switcher;
    let threshold_percent = custom_threshold.unwrap_or(switcher_cfg.low_quota_threshold_percent);
    let target_model = switcher_cfg.target_model.clone();

    let now_sec = chrono::Utc::now().timestamp();
    let active_acc = account::get_current_account().ok().flatten();

    let current_quota_percent = match active_acc.as_ref() {
        Some(acc) => {
            if let Some(ps) = auto_switcher::evaluate_account_period_status(
                acc,
                &target_model,
                threshold_percent,
                now_sec,
            ) {
                ps.quota_percent
            } else {
                auto_switcher::calculate_account_quota(acc, &target_model).unwrap_or(100.0)
            }
        }
        None => 0.0,
    };

    let is_low_credit = active_acc.is_none() || current_quota_percent <= threshold_percent;

    // Find next possible account
    let mut excluded = auto_switcher::get_active_in_use_account_ids();
    if let Some(ref acc) = active_acc {
        if !excluded.contains(&acc.id) {
            excluded.push(acc.id.clone());
        }
        if !excluded.contains(&acc.email) {
            excluded.push(acc.email.clone());
        }
    }
    let best_candidate = auto_switcher::select_next_best_profile(
        "default",
        &target_model,
        threshold_percent,
        &excluded,
    )
    .ok()
    .flatten();
    let next_possible_account = best_candidate
        .as_ref()
        .map(|a| a.email.clone())
        .filter(|em| {
            active_acc
                .as_ref()
                .map(|a| !em.trim().eq_ignore_ascii_case(a.email.trim()))
                .unwrap_or(true)
        });
    let next_possible_quota_percent = best_candidate.as_ref().map(|a| a.quota_percent);

    // Detect active running prompt and image payload
    let mut running_prompt: Option<String> = None;
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
            running_prompt = Some(snippet);
            if p.image_payload.is_some() {
                has_images = true;
            }
        }
    }
    if running_prompt.is_none() {
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
                                running_prompt = Some(snippet);
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

    let machine_name = email_watcher::detect_machine_name();
    let node_alias = supabase_sync::load_config()
        .map(|c| c.node_alias)
        .unwrap_or_else(|_| machine_name.clone());
    let local_ip = email_watcher::detect_local_ip();
    let tool_version = format!("v{}", crate::common::VERSION);
    let current_account = active_acc.map(|a| a.email);
    let running_prompts_count = if running_prompt.is_some() { 1 } else { 0 };

    let payload = serde_json::json!({
        "is_low_credit": is_low_credit,
        "machine_name": machine_name,
        "node_alias": node_alias,
        "vm_alias": node_alias,
        "local_ip": local_ip,
        "current_account": current_account,
        "previous_account": if is_low_credit && next_possible_account.is_some() {
            current_account.clone()
        } else {
            None
        },
        "predicted_next_account": next_possible_account,
        "selected_account": if is_low_credit { next_possible_account.clone() } else { None },
        "credit_before_switch": current_quota_percent,
        "current_quota_percent": current_quota_percent,
        "threshold_percent": threshold_percent,
        "threshold_activated": threshold_percent,
        "target_model": target_model,
        "tool_version": tool_version,
        "next_possible_account": next_possible_account,
        "next_possible_quota_percent": next_possible_quota_percent,
        "running_prompts_count": running_prompts_count,
        "prompts_resent": false,
        "is_reinjecting": false,
        "running_prompt": running_prompt,
        "has_images": has_images,
        "images_attached": has_images,
        "timestamp": now_sec,
    });

    let payload_str = serde_json::to_string_pretty(&payload).unwrap_or_default();

    if should_export_file {
        let file_path = crate::common::resolve_switch_filename(export_file.as_deref(), &node_alias);
        let _ = std::fs::write(&file_path, &payload_str);
    }

    if is_json {
        println!("{}", payload_str);
    } else {
        println!("{}", is_low_credit);
    }
}
