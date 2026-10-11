use crate::modules::account;
use crate::modules::email_vault_db;
use crate::modules::email_watcher;
use crate::modules::git_info;
use crate::modules::repo_db;
use crate::modules::supabase_sync;
use crate::modules::*;

use super::*;

pub fn format_ping_report() -> String {
    let local_config = supabase_sync::load_config().unwrap_or_default();
    let machine_name = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| email_watcher::detect_machine_name());
    let raw_alias = if !local_config.node_alias.trim().is_empty() {
        local_config.node_alias.trim().to_string()
    } else if let Ok(settings) = email_vault_db::get_notification_settings() {
        settings.local_machine_name.trim().to_string()
    } else {
        String::new()
    };
    let display_alias = if raw_alias.starts_with("Node-") || raw_alias.is_empty() {
        "None".to_string()
    } else {
        raw_alias
    };
    let local_ip = email_watcher::detect_local_ip();
    let uptime_min = supabase_sync::get_uptime_seconds() / 60;
    let ver = git_info::get_app_version();
    let hash = git_info::get_git_hash();
    let branch = git_info::get_git_branch();
    let last_rel = git_info::get_last_release();

    let active_acc = account::load_account_index()
        .ok()
        .and_then(|idx| {
            let cid = idx.current_account_id?;
            idx.accounts.into_iter().find(|a| a.id == cid)
        })
        .map(|a| a.email)
        .unwrap_or_else(|| "None".to_string());

    format!(
        "🏓 <b>PONG — AGM v{} Online</b>\n\n\
        • <b>Machine:</b> <code>{}</code>\n\
        • <b>Alias:</b> <code>{}</code>\n\
        • <b>IP:</b> <code>{}</code>\n\
        • <b>Build:</b> <code>v{}</code> (commit <code>{}</code>)\n\
        • <b>Branch:</b> <code>{}</code> | <b>Release:</b> <code>{}</code>\n\
        • <b>Uptime:</b> <code>{}m</code>\n\
        • <b>Active Account:</b> <code>{}</code>\n\n\
        💡 Send <code>/observe</code> for live workspaces &amp; prompts or <code>/help</code> for all commands.",
        clean_for_telegram_html(&ver, 24),
        clean_for_telegram_html(&machine_name, 48),
        clean_for_telegram_html(&display_alias, 48),
        clean_for_telegram_html(&local_ip, 48),
        clean_for_telegram_html(&ver, 24),
        clean_for_telegram_html(&hash, 24),
        clean_for_telegram_html(&branch, 24),
        clean_for_telegram_html(&last_rel, 24),
        uptime_min,
        clean_for_telegram_html(&active_acc, 64)
    )
}

/// Format live observation & status report (Accounts, Workspaces, Running Prompts)
pub fn format_observe_report() -> String {
    let local_config = supabase_sync::load_config().unwrap_or_default();
    let machine_name = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| email_watcher::detect_machine_name());
    let raw_alias = if !local_config.node_alias.trim().is_empty() {
        local_config.node_alias.trim().to_string()
    } else if let Ok(settings) = email_vault_db::get_notification_settings() {
        settings.local_machine_name.trim().to_string()
    } else {
        String::new()
    };
    let display_alias = if raw_alias.starts_with("Node-") || raw_alias.is_empty() {
        "None".to_string()
    } else {
        raw_alias
    };
    let local_ip = email_watcher::detect_local_ip();
    let ver = git_info::get_app_version();
    let hash = git_info::get_git_hash();

    let switch_threshold = crate::modules::config::load_app_config()
        .map(|c| c.quota_protection.threshold_percentage)
        .unwrap_or(15);

    let (active_email, total_accounts, quota_summary) = match account::load_account_index() {
        Ok(idx) => {
            let total = idx.accounts.len();
            let cid = idx.current_account_id.unwrap_or_default();
            let email = idx
                .accounts
                .iter()
                .find(|a| a.id == cid)
                .map(|a| a.email.clone())
                .unwrap_or_else(|| "None".to_string());
            let quota_str = account::load_account(&cid)
                .ok()
                .and_then(|acc| acc.quota)
                .map(|q| {
                    let tier = q.subscription_tier.unwrap_or_else(|| "PRO".to_string());
                    let min_pct = q.models.iter().map(|m| m.percentage).min().unwrap_or(100);
                    format!(
                        "{} ({}% min model quota | switch threshold: {}%)",
                        tier, min_pct, switch_threshold
                    )
                })
                .unwrap_or_else(|| format!("N/A (switch threshold: {}%)", switch_threshold));
            (email, total, quota_str)
        }
        Err(_) => (
            "None".to_string(),
            0,
            format!("N/A (switch threshold: {}%)", switch_threshold),
        ),
    };

    let projects = repo_db::get_live_project_execution_info();
    let all_prompts = repo_db::list_all_prompts().unwrap_or_default();

    // Deduplicate running projects using a distinct dictionary/map
    let mut project_map: std::collections::BTreeMap<
        String,
        (String, Vec<(String, usize, String)>),
    > = std::collections::BTreeMap::new();
    let mut seen_prompt_ids = std::collections::HashSet::new();
    let mut idle_names = Vec::new();
    let mut running_items = Vec::new();

    for p in &projects {
        let short_name = shorten_project_name(&p.repo_name);
        if p.is_running {
            let matched_prompt = all_prompts.iter().find(|ap| {
                (ap.status == "running" || ap.status == "dispatched" || ap.status == "executing")
                    && !seen_prompt_ids.contains(&ap.id)
                    && (ap.project_id == p.project_id
                        || ap.repo_path == p.repo_path
                        || (!p.repo_name.is_empty() && ap.project_id.contains(&p.repo_name)))
            });

            if let Some(ap) = matched_prompt {
                seen_prompt_ids.insert(ap.id.clone());
                let duration_str = format_running_duration(ap.created_at);
                let (preview_txt, wc) =
                    repo_db::extract_prompt_words_preview(&ap.prompt_content, 200);
                let prompt_id_short = if ap.id.len() > 8 {
                    ap.id[..8].to_string()
                } else {
                    ap.id.clone()
                };
                let entry = project_map
                    .entry(short_name.clone())
                    .or_insert_with(|| (duration_str.clone(), Vec::new()));
                if !preview_txt.is_empty() {
                    entry.1.push((preview_txt, wc, prompt_id_short));
                }
            } else if let Some(ref txt) = p.active_prompt {
                let duration_str = if p.last_detected_at > 0 {
                    format_running_duration(p.last_detected_at)
                } else {
                    String::new()
                };
                let (preview_txt, wc) = repo_db::extract_prompt_words_preview(txt, 200);
                let entry = project_map
                    .entry(short_name.clone())
                    .or_insert_with(|| (duration_str.clone(), Vec::new()));
                if !preview_txt.is_empty() {
                    entry.1.push((preview_txt, wc, String::new()));
                }
            } else {
                project_map
                    .entry(short_name.clone())
                    .or_insert_with(|| (String::new(), Vec::new()));
            }
        } else {
            idle_names.push(short_name);
        }
    }

    for ap in all_prompts.iter().filter(|ap| {
        ap.status == "running" || ap.status == "dispatched" || ap.status == "executing"
    }) {
        if seen_prompt_ids.contains(&ap.id) {
            continue;
        }
        let friendly_ws =
            repo_db::format_friendly_workspace_label(&ap.project_id, "", &ap.repo_path);
        let short_name = shorten_project_name(&friendly_ws);
        seen_prompt_ids.insert(ap.id.clone());

        let duration_str = format_running_duration(ap.created_at);
        let (preview_txt, wc) = repo_db::extract_prompt_words_preview(&ap.prompt_content, 200);
        let prompt_id_short = if ap.id.len() > 8 {
            ap.id[..8].to_string()
        } else {
            ap.id.clone()
        };
        let entry = project_map
            .entry(short_name.clone())
            .or_insert_with(|| (duration_str.clone(), Vec::new()));
        if !preview_txt.is_empty() {
            entry.1.push((preview_txt, wc, prompt_id_short));
        }
    }

    for (name, (dur, prompts)) in &project_map {
        let dur_display = if !dur.is_empty() {
            format!(" {}", dur)
        } else {
            String::new()
        };
        let mut block = format!("• <b>{}</b> 🟢{}\n", name, dur_display);
        for (preview_txt, wc, prompt_id) in prompts {
            block.push_str(&format!(
                "   {} <i>({} words)</i>\n",
                clean_for_telegram_html(preview_txt, 2500),
                wc
            ));
            if !prompt_id.is_empty() {
                block.push_str(&format!(
                    "   <i>Expand: <code>/expand {}</code></i>\n",
                    prompt_id
                ));
            }
        }
        running_items.push(block);
    }

    idle_names.retain(|name| !project_map.contains_key(name));
    idle_names.sort();
    idle_names.dedup();

    let mut body_sections = String::new();
    body_sections.push_str("🟢 <b>Running:</b>\n");
    if running_items.is_empty() {
        body_sections.push_str("• None (All workspaces idle)\n");
    } else {
        for item in running_items {
            body_sections.push_str(&item);
        }
    }

    if !idle_names.is_empty() {
        body_sections.push_str("\n⚪ <b>Idle:</b>\n");
        for name in idle_names {
            body_sections.push_str(&format!("• {} ⚪\n", name));
        }
    }

    let last_rel = git_info::get_last_release();
    let current_ver = format!("v{}", ver);
    let settings = crate::modules::update_checker::load_update_settings().ok();
    let has_remote_newer = settings
        .as_ref()
        .map(|s| {
            !s.last_known_version.is_empty()
                && crate::modules::update_checker::compare_versions(&s.last_known_version, &ver)
        })
        .unwrap_or(false);

    let (update_badge, update_notice) = if has_remote_newer {
        let latest_v = settings
            .as_ref()
            .map(|s| s.last_known_version.as_str())
            .unwrap_or("latest");
        (
            format!("🚀 <b>Update Available:</b> <code>v{}</code> (current: <code>v{}</code> — Send <code>/update</code>)", latest_v, ver),
            format!("\n🚀 <b>Update Available:</b> <code>v{}</code> (current: <code>v{}</code>) — Run <code>/update</code> to install\n", latest_v, ver),
        )
    } else if !last_rel.is_empty()
        && last_rel != current_ver
        && last_rel != ver
        && last_rel != "unknown"
    {
        (
            format!("🚀 <b>Update Available:</b> <code>{}</code> (current: <code>v{}</code> — Send <code>/update</code>)", last_rel, ver),
            format!("\n🚀 <b>Update Available:</b> <code>{}</code> (current: <code>v{}</code>) — Run <code>/update</code> to install\n", last_rel, ver),
        )
    } else {
        (
            "<code>Up to date</code> (Send <code>/update</code> or <code>/upgrade</code>)"
                .to_string(),
            String::new(),
        )
    };

    format!(
        "🤖 <b>AGM v{} Status</b>{}\n\n\
        • <b>Machine:</b> {}\n\
        • <b>Alias:</b> {}\n\
        • <b>IP:</b> {}\n\
        • <b>Build:</b> v{} (commit {})\n\
        • <b>Active Account:</b> {} ({} total)\n\
        • <b>Quota / Tier:</b> {}\n\
        • <b>Update Status:</b> {}\n\n\
        {}\n\
        💡 Send <code>/expand &lt;id&gt;</code> to view full prompt text, or <code>/active</code> for live table.",
        clean_for_telegram_html(&ver, 24),
        update_notice,
        clean_for_telegram_html(&machine_name, 64),
        clean_for_telegram_html(&display_alias, 64),
        clean_for_telegram_html(&local_ip, 48),
        clean_for_telegram_html(&ver, 24),
        clean_for_telegram_html(&hash, 24),
        clean_for_telegram_html(&active_email, 96),
        total_accounts,
        clean_for_telegram_html(&quota_summary, 128),
        update_badge,
        body_sections
    )
}

/// Format standalone Fleet & Machine Identity report without requiring Supabase
pub fn format_fleet_identity_report() -> String {
    let machine_name = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| email_watcher::detect_machine_name());
    let local_ip = email_watcher::detect_local_ip();
    let ver = git_info::get_app_version();

    let reg = crate::modules::instance::load_registry().unwrap_or_default();
    let active_inst = reg
        .instances
        .iter()
        .find(|i| i.id == reg.active_instance_id || i.is_default);
    let active_inst_name = active_inst.map(|i| i.name.as_str()).unwrap_or("default");
    let active_inst_id = active_inst.map(|i| i.id.as_str()).unwrap_or("default");
    let active_email = active_inst
        .and_then(|i| i.bound_email.as_deref())
        .unwrap_or("none");

    let instances_count = reg.instances.len();
    let accounts_count = account::load_account_index()
        .map(|idx| idx.accounts.len())
        .unwrap_or(0);

    let (live_pids, is_running) = active_inst
        .map(|i| {
            let pids = crate::modules::instance::find_pids_for_data_dir(&i.data_dir, i.is_default);
            let running = !pids.is_empty();
            (pids, running)
        })
        .unwrap_or_default();

    let running_tag = if is_running {
        "🟢 RUNNING"
    } else {
        "⚪ IDLE"
    };
    let pid_str = if live_pids.is_empty() {
        "-".to_string()
    } else {
        live_pids
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    };

    format!(
        "🌐 <b>Standalone Fleet Identity Card</b>\n\
        ━━━━━━━━━━━━━━━━━━━━━━━━\n\
        🖥️ <b>Machine:</b> <code>{}</code>\n\
        📡 <b>LAN IP:</b> <code>{}</code>\n\
        📦 <b>AGM Version:</b> <code>v{}</code>\n\
        ⚙️ <b>Active Instance:</b> <code>{}</code> (<code>{}</code>)\n\
        🚦 <b>Instance State:</b> {} (PIDs: <code>{}</code>)\n\
        👤 <b>Bound Account:</b> <code>{}</code>\n\
        📊 <b>Fleet Inventory:</b> {} instances | {} accounts\n\n\
        💡 <b>Available Commands:</b>\n\
        • <code>/instances</code> — List all instance profiles on this machine\n\
        • <code>/accounts</code> — List all registered accounts and quota levels\n\
        • <code>/observe</code> — Live running prompts and status\n\
        • <code>/whoami</code> — Quick node identity",
        clean_for_telegram_html(&machine_name, 64),
        clean_for_telegram_html(&local_ip, 48),
        clean_for_telegram_html(&ver, 24),
        clean_for_telegram_html(active_inst_name, 48),
        clean_for_telegram_html(active_inst_id, 48),
        running_tag,
        clean_for_telegram_html(&pid_str, 48),
        clean_for_telegram_html(active_email, 64),
        instances_count,
        accounts_count
    )
}

/// Format instances report listing all local instance profiles and their bound accounts
pub fn format_instances_report() -> String {
    let reg = crate::modules::instance::load_registry().unwrap_or_default();
    if reg.instances.is_empty() {
        return "📁 <b>Instances:</b> No instances registered.".to_string();
    }

    let mut out = format!(
        "💻 <b>Registered Instance Profiles ({} total)</b>\n\
        ━━━━━━━━━━━━━━━━━━━━━━━━\n",
        reg.instances.len()
    );

    for (idx, inst) in reg.instances.iter().enumerate() {
        let is_active = reg.active_instance_id == inst.id;
        let is_running =
            crate::modules::instance::is_instance_running(&inst.id, &inst.data_dir, inst.pid);
        let status_emoji = if is_running { "🟢" } else { "⚪" };
        let active_badge = if is_active { " <i>[ACTIVE]</i>" } else { "" };
        let bound = inst.bound_email.as_deref().unwrap_or("(none)");

        out.push_str(&format!(
            "{}. {} <b>{}</b> (<code>{}</code>){}\n   👤 Bound: <code>{}</code>\n   📁 Data: <code>{}</code>\n",
            idx + 1,
            status_emoji,
            clean_for_telegram_html(&inst.name, 48),
            clean_for_telegram_html(&inst.id, 48),
            active_badge,
            clean_for_telegram_html(bound, 64),
            clean_for_telegram_html(&inst.data_dir, 64)
        ));
    }

    out
}

/// Format accounts report listing all accounts and their quota balances
pub fn format_accounts_report() -> String {
    let accounts = match crate::modules::account::list_accounts() {
        Ok(accs) => accs,
        Err(e) => {
            return format!(
                "❌ <b>Failed to load accounts:</b> {}",
                escape_telegram_html(&e)
            )
        }
    };

    if accounts.is_empty() {
        return "👥 <b>Accounts:</b> No accounts registered.".to_string();
    }

    let curr_id = crate::modules::account::get_current_account_id().unwrap_or_default();

    let mut out = format!(
        "👥 <b>Registered Accounts ({} total)</b>\n\
        ━━━━━━━━━━━━━━━━━━━━━━━━\n",
        accounts.len()
    );

    for (idx, acc) in accounts.iter().enumerate() {
        let is_current = curr_id.as_deref() == Some(&acc.id);
        let curr_badge = if is_current { " <i>[ACTIVE]</i>" } else { "" };
        let tier = acc.token.subscription_tier.as_deref().unwrap_or("Standard");
        let (q_4h, q_weekly) =
            crate::modules::auto_switcher::extract_dual_window_quotas(acc, "gemini-2.5-pro");
        let q_str = match (q_4h, q_weekly) {
            (Some(h), Some(w)) => format!("4h: {:.1}% | Weekly: {:.1}%", h, w),
            (Some(h), None) => format!("4h: {:.1}%", h),
            _ => "Quota: active".to_string(),
        };

        out.push_str(&format!(
            "{}. <b>{}</b>{}\n   🏷️ Tier: <code>{}</code> | 📊 <code>{}</code>\n",
            idx + 1,
            clean_for_telegram_html(&acc.email, 64),
            curr_badge,
            clean_for_telegram_html(tier, 32),
            clean_for_telegram_html(&q_str, 48)
        ));
    }

    out
}
