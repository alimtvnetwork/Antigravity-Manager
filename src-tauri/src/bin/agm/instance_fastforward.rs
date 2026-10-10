//! instance_fastforward — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};
use chrono::Utc;

pub(crate) fn cmd_fast_forward(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("AGM Fast-Forward Smart Switch:");
        println!("  agm ff [target_instance] [--json]");
        println!("\nDescription:");
        println!(
            "  Instantly selects and rotates to the freshest account in the pool with 100% quota."
        );
        println!(
            "  Pre-verifies quota with Google API, checks Supabase and Email collision locks,"
        );
        println!("  snapshots running prompts across active workspaces, executes rotation via the Fast-Forward");
        println!("  bridge, restores and re-injects prompts, and broadcasts telemetry to Telegram & Email.");
        println!("\nAliases: agm ff, agm fast-forward, agm smart-switch");
        println!("\nArguments:");
        println!(
            "  [target_instance]   Optional target sandbox instance ID (defaults to 'default')"
        );
        println!("\nOptions:");
        println!("  --json, -j          Output switch results in structured JSON format");
        println!("\nExamples:");
        println!("  agm ff                              # Fast-forward switch default instance to highest quota");
        println!("  agm ff test-sandbox                 # Fast-forward switch a specific sandbox instance");
        println!("  agm ff --json                       # Fast-forward with JSON response");
        return;
    }

    let is_json = args.iter().any(|a| a == "--json");
    let non_flag_args: Vec<&String> = args.iter().filter(|a| !a.starts_with('-')).collect();
    let target_opt = non_flag_args.first().map(|s| s.as_str());

    let machine_name = email_watcher::detect_machine_name();
    let node_alias = supabase_sync::load_config()
        .map(|c| c.node_alias)
        .unwrap_or_else(|_| machine_name.clone());
    let local_ip = email_watcher::detect_local_ip();
    let status_before = auto_switcher::get_status();

    let all_prompts = repo_db::list_all_prompts().unwrap_or_default();
    let running_prompts_count = all_prompts
        .iter()
        .filter(|p| p.status == "running" || p.status == "dispatched" || p.status == "backed_up")
        .count();
    let has_images = all_prompts.iter().any(|p| {
        (p.status == "running" || p.status == "dispatched" || p.status == "backed_up")
            && p.image_payload.is_some()
    });

    let rt = match tokio::runtime::Runtime::new() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to initialize async runtime: {}", e);
            std::process::exit(1);
        }
    };

    let target_instance = match target_opt {
        Some(target) => {
            instance::resolve_instance_id(target).unwrap_or_else(|_| target.to_string())
        }
        None => instance::get_active_instance_id().unwrap_or_else(|_| "default".to_string()),
    };

    // Snapshot & backup running prompts before rotation
    let _ = repo_db::backup_running_prompts(&target_instance);

    if !is_json {
        println!(
            "[*] Triggering fast-forward account rotation for instance '{}'...",
            target_instance
        );
    }
    let result = rt.block_on(auto_switcher::trigger_manual_rotation_for_instance(Some(
        &target_instance,
    )));

    if result.is_ok() {
        // Immediately restore and dispatch running prompts for target instance
        let _ = repo_db::resend_running_commands_for_instance(Some(&target_instance), 20);
        let _ = repo_db::dispatch_running_prompts(&target_instance);
    }

    let status_after = auto_switcher::get_status();

    match result {
        Ok(res_msg) => {
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
            let inst_ref = target_opt.unwrap_or("default");
            let predicted_candidate = auto_switcher::select_candidate_profiles(
                inst_ref,
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

            if is_json {
                let out = serde_json::json!({
                    "success": true,
                    "result": res_msg,
                    "machine_name": machine_name,
                    "node_alias": node_alias,
                    "vm_alias": node_alias,
                    "local_ip": local_ip,
                    "tool_version": format!("v{}", crate::common::VERSION),
                    "previous_account": prev_email,
                    "previous_email": prev_email,
                    "predicted_next_account": predicted_email,
                    "selected_account": selected_email,
                    "selected_email": selected_email,
                    "active_account": status_after.active_account_email,
                    "credit_before_switch": status_before.current_quota_percent,
                    "current_quota_percent": status_after.current_quota_percent,
                    "prompts_running": running_prompts_count,
                    "prompts_resent": prompts_resent,
                    "has_images": has_images,
                    "timestamp": chrono::Utc::now().timestamp(),
                });
                println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
            } else {
                println!("[OK] Fast-forward completed: {}", res_msg);
                if let Some(ref p) = prev_email {
                    println!("     Previous Profile:    {}", p);
                }
                if let Some(ref s) = selected_email {
                    println!("     New Active Profile:  {}", s);
                }
                if let Some(ref pr) = predicted_email {
                    println!("     Predicted Next:      {}", pr);
                }
            }
        }
        Err(e) => {
            if is_json {
                let out = serde_json::json!({
                    "success": false,
                    "error": e,
                    "machine_name": machine_name,
                    "node_alias": node_alias,
                    "vm_alias": node_alias,
                    "local_ip": local_ip,
                    "tool_version": format!("v{}", crate::common::VERSION),
                    "timestamp": chrono::Utc::now().timestamp(),
                });
                println!("{}", serde_json::to_string_pretty(&out).unwrap_or_default());
            } else {
                eprintln!("[ERROR] Fast-forward failed: {}", e);
            }
            std::process::exit(1);
        }
    }
}
