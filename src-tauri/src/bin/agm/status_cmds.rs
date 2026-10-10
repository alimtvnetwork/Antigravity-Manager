//! status_cmds — CLI command handlers, split from agm.rs.

use antigravity_tools_lib::modules::{
    account, agy_cleaner, auto_switcher, backup_prompts_db, config, db, email_inbound, email_io,
    email_sender, email_vault_db, email_watcher, instance, integration, json_envelope,
    notification_hub, oauth, proxy_db, repo_db, security_db, ssh_manager, supabase_client,
    supabase_schema, supabase_sync, telegram_inbound, training_api, workspace_lease_manager,
};

pub(crate) fn cmd_status(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
    {
        println!("AGM Node Status & Credits:");
        println!("  agm status [--json]");
        println!("  agm credits [--json]");
        println!("\nDescription:");
        println!(
            "  Displays current node runtime health, active profile details, immediate (4-hour)"
        );
        println!(
            "  and weekly credit percentages, active instance name, and threshold configuration."
        );
        println!("\nAliases: agm status, agm credits, agm credit");
        println!("\nOptions:");
        println!("    --json, -j          Output pure JSON payload for programmatic evaluation");
        println!("\nExamples:");
        println!("  agm status                          # Human-readable status card");
        println!(
            "  agm credits                         # View current active model quota balances"
        );
        println!("  agm status --json                   # Structured JSON output for scripts");
        return;
    }

    let is_json = args.iter().any(|a| a == "--json");
    let machine_name = email_watcher::detect_machine_name();
    let node_alias = supabase_sync::load_config()
        .map(|c| c.node_alias)
        .unwrap_or_else(|_| machine_name.clone());
    let local_ip = email_watcher::detect_local_ip();
    let active_acc = account::get_current_account().ok().flatten();

    let (immediate_quota, weekly_quota, tier) = match active_acc.as_ref() {
        Some(acc) => crate::common::extract_immediate_and_weekly_credits(acc),
        None => (0.0, 0.0, "NONE".to_string()),
    };

    let app_cfg = config::load_app_config().unwrap_or_default();
    let threshold_percent = app_cfg.auto_profile_switcher.low_quota_threshold_percent;
    let target_model = app_cfg.auto_profile_switcher.target_model.clone();

    let instances_list = instance::list_instances().unwrap_or_default();
    let running_instances = instances_list.iter().filter(|i| i.is_running).count();
    let all_prompts = repo_db::list_all_prompts().unwrap_or_default();
    let running_prompts = all_prompts
        .iter()
        .filter(|p| p.status == "running" || p.status == "dispatched" || p.status == "backed_up")
        .count();
    let has_images = all_prompts.iter().any(|p| {
        (p.status == "running" || p.status == "dispatched" || p.status == "backed_up")
            && p.image_payload.is_some()
    });
    let prompts_resent = running_prompts > 0;

    let mut excluded = auto_switcher::get_active_in_use_account_ids();
    if let Some(ref acc) = active_acc {
        if !excluded.contains(&acc.id) {
            excluded.push(acc.id.clone());
        }
        if !excluded.contains(&acc.email) {
            excluded.push(acc.email.clone());
        }
    }
    let predicted_candidate = auto_switcher::select_next_best_profile(
        "default",
        &target_model,
        threshold_percent,
        &excluded,
    )
    .ok()
    .flatten();
    let predicted_next_account = predicted_candidate
        .as_ref()
        .map(|c| c.email.clone())
        .filter(|em| {
            active_acc
                .as_ref()
                .map(|a| !em.trim().eq_ignore_ascii_case(a.email.trim()))
                .unwrap_or(true)
        });
    let predicted_next_quota = predicted_candidate.as_ref().map(|c| c.quota_percent);

    if is_json {
        let out = serde_json::json!({
            "version": crate::common::VERSION,
            "git_hash": antigravity_tools_lib::modules::git_info::get_git_hash(),
            "git_branch": antigravity_tools_lib::modules::git_info::get_git_branch(),
            "last_release": antigravity_tools_lib::modules::git_info::get_last_release(),
            "machine_name": machine_name,
            "node_alias": node_alias,
            "vm_alias": node_alias,
            "local_ip": local_ip,
            "active_account": active_acc.as_ref().map(|a| a.email.clone()),
            "active_account_id": active_acc.as_ref().map(|a| a.id.clone()),
            "previous_account": serde_json::Value::Null,
            "predicted_next_account": predicted_next_account,
            "predicted_next_quota_percent": predicted_next_quota,
            "selected_account": serde_json::Value::Null,
            "tier": tier,
            "credit_before_switch": immediate_quota,
            "immediate_quota_percent": immediate_quota,
            "weekly_quota_percent": weekly_quota,
            "threshold_percent": threshold_percent,
            "threshold_activated": threshold_percent,
            "target_model": target_model,
            "instances_total": instances_list.len(),
            "instances_running": running_instances,
            "prompts_running": running_prompts,
            "prompts_resent": prompts_resent,
            "has_images": has_images,
        });
        println!(
            "{}",
            serde_json::to_string_pretty(&out).unwrap_or_else(|_| "{}".to_string())
        );
        return;
    }

    let git_hash = antigravity_tools_lib::modules::git_info::get_git_hash();
    let git_branch = antigravity_tools_lib::modules::git_info::get_git_branch();
    let last_release = antigravity_tools_lib::modules::git_info::get_last_release();

    println!("[*] Antigravity-Manager Node & Credits Status:");
    println!("    Machine Name:      {}", machine_name);
    println!("    Node / VM Alias:   {}", node_alias);
    println!("    Local IP:          {}", local_ip);
    println!("    CLI Version:       v{}", crate::common::VERSION);
    println!("    Git Commit:        {}", git_hash);
    println!("    Git Branch:        {}", git_branch);
    println!("    Last Release:      {}", last_release);

    if let Some(acc) = active_acc {
        println!("    Active Account:    {} [{}]", acc.email, tier);
        println!(
            "    Account Name:      {}",
            acc.name.as_deref().unwrap_or("-")
        );
        println!("    Immediate Credits: {:.1}% remaining", immediate_quota);
        println!("    Weekly Credits:    {:.1}% remaining", weekly_quota);
        println!(
            "    Threshold Target:  {:.1}% ({})",
            threshold_percent, target_model
        );
        if let Some(ref pred) = predicted_next_account {
            println!(
                "    Predicted Next:    {} ({:.1}% quota)",
                pred,
                predicted_next_quota.unwrap_or(100.0)
            );
        } else {
            println!("    Predicted Next:    (No candidate available in pool)");
        }
    } else {
        println!("    Active Account:    (None / Default)");
    }

    println!(
        "    Sandbox Profiles:  {} configured ({} currently running)",
        instances_list.len(),
        running_instances
    );
    let resent_str = if prompts_resent {
        "Yes (Auto-Resumed via resume task file)"
    } else {
        "No (0 active in queue)"
    };
    println!(
        "    Queued/Running Prompts: {} [Resent: {} | Images: {}]",
        running_prompts,
        resent_str,
        if has_images { "Yes" } else { "None" }
    );
}
