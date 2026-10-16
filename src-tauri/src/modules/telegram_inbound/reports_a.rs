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

    let palette = active_card_palette();
    let body = section(
        palette,
        "Node",
        &[
            kv(
                palette,
                "Active account",
                &clean_for_telegram_html(&active_acc, 64),
            ),
            kv(
                palette,
                "App build",
                &format!(
                    "v{} · {}",
                    clean_for_telegram_html(&ver, 24),
                    clean_for_telegram_html(&hash, 24)
                ),
            ),
            kv(
                palette,
                "Branch",
                &format!(
                    "{} · {}",
                    clean_for_telegram_html(&branch, 24),
                    clean_for_telegram_html(&last_rel, 24)
                ),
            ),
            kv(palette, "Uptime", &format!("{}m", uptime_min)),
        ],
    );
    format!(
        "{} <b>Pong · AGM v{}</b>\n\
        {}\n\
        🖥️ <b>Machine:</b> <code>{}</code> (<code>{}</code>) · <b>Alias:</b> <code>{}</code>\n\
        {}\n\
        {}\
        {} <b>Next</b>  <code>/observe</code> · <code>/fleet</code>\n\
        {}",
        palette.mark,
        clean_for_telegram_html(&ver, 24),
        palette.rule,
        clean_for_telegram_html(&machine_name, 48),
        clean_for_telegram_html(&local_ip, 48),
        clean_for_telegram_html(&display_alias, 48),
        palette.rule,
        body,
        palette.bullet,
        theme_footer(palette)
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
    let accounts = crate::modules::account::list_accounts().unwrap_or_default();
    let accounts_count = accounts.len();

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

    let proxy_port = crate::modules::config::load_app_config()
        .map(|c| c.proxy.port)
        .unwrap_or(8045);

    // Extract bound account details: tier and dual quotas (4h rolling & weekly)
    let bound_acc = accounts
        .iter()
        .find(|a| a.email.eq_ignore_ascii_case(active_email))
        .or_else(|| {
            let curr_id = account::get_current_account_id().unwrap_or_default();
            accounts.iter().find(|a| Some(&a.id) == curr_id.as_ref())
        });

    let full_acc = bound_acc.and_then(|a| account::load_account(&a.id).ok());
    let (tier_str, quota_str) = if let Some(acc) = full_acc.as_ref() {
        let t = acc
            .quota
            .as_ref()
            .and_then(|q| q.subscription_tier.as_deref())
            .unwrap_or("PRO");
        let (q_4h, q_weekly) =
            crate::modules::auto_switcher::extract_dual_window_quotas(acc, "gemini-2.5-pro");
        let q_formatted = match (q_4h, q_weekly) {
            (Some(h), Some(w)) => format!("4h: {:.1}% | Weekly: {:.1}%", h, w),
            (Some(h), None) => format!("4h: {:.1}%", h),
            (None, Some(w)) => format!("Weekly: {:.1}%", w),
            _ => "Active".to_string(),
        };
        (t.to_string(), q_formatted)
    } else {
        ("Standard".to_string(), "Active".to_string())
    };

    let mut inst_summary = String::new();
    for (_idx, inst) in reg.instances.iter().take(6).enumerate() {
        let is_act = reg.active_instance_id == inst.id;
        let pids =
            crate::modules::instance::find_pids_for_data_dir(&inst.data_dir, inst.is_default);
        let is_run = !pids.is_empty()
            || crate::modules::instance::is_instance_running(&inst.id, &inst.data_dir, inst.pid);
        let tag = if is_run { "🟢" } else { "⚪" };
        let act_mark = if is_act { " <i>[ACTIVE]</i>" } else { "" };
        let bound = inst.bound_email.as_deref().unwrap_or("(none)");
        let pid_display = if pids.is_empty() {
            String::new()
        } else {
            format!(
                " (PIDs: {})",
                pids.iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };
        inst_summary.push_str(&format!(
            "  • {} <b>{}</b>{}{} → <code>{}</code>\n",
            tag,
            clean_for_telegram_html(&inst.name, 32),
            act_mark,
            clean_for_telegram_html(&pid_display, 32),
            clean_for_telegram_html(bound, 48)
        ));
    }

    format!(
        "🌐 <b>Standalone Fleet Identity Card</b>\n\
        ━━━━━━━━━━━━━━━━━━━━━━━━\n\
        🖥️ <b>Machine:</b> <code>{}</code> (IP: <code>{}</code>)\n\
        📦 <b>Version:</b> <code>v{}</code> | <b>Platform:</b> <code>{}/{}</code>\n\
        ⚙️ <b>Active Instance:</b> <code>{}</code> (ID: <code>{}</code>)\n\
        🚦 <b>State:</b> {} (PIDs: <code>{}</code>) | 🔌 <b>Proxy Port:</b> <code>{}</code>\n\
        👤 <b>Bound Account:</b> <code>{}</code>\n\
        🏷️ <b>Tier:</b> <code>{}</code> | 📊 <b>Quota:</b> <code>{}</code>\n\n\
        📁 <b>Local Instances ({}):</b>\n{}\n\
        👥 <b>Total Accounts Registered:</b> {}\n\n\
        💡 <b>Instant Commands:</b>\n\
        • <code>/instances</code> — Full instance profiles\n\
        • <code>/accounts</code> — Account quota levels\n\
        • <code>/observe</code> — Live in-flight prompts\n\
        • <code>/whoami</code> — Fast identity ping",
        clean_for_telegram_html(&machine_name, 64),
        clean_for_telegram_html(&local_ip, 48),
        clean_for_telegram_html(&ver, 24),
        std::env::consts::OS,
        std::env::consts::ARCH,
        clean_for_telegram_html(active_inst_name, 48),
        clean_for_telegram_html(active_inst_id, 48),
        running_tag,
        clean_for_telegram_html(&pid_str, 48),
        proxy_port,
        clean_for_telegram_html(active_email, 64),
        clean_for_telegram_html(&tier_str, 32),
        clean_for_telegram_html(&quota_str, 48),
        instances_count,
        if inst_summary.is_empty() {
            "  • (none registered)\n".to_string()
        } else {
            inst_summary
        },
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
        let pids =
            crate::modules::instance::find_pids_for_data_dir(&inst.data_dir, inst.is_default);
        let is_running = !pids.is_empty()
            || crate::modules::instance::is_instance_running(&inst.id, &inst.data_dir, inst.pid);
        let status_emoji = if is_running { "🟢" } else { "⚪" };
        let active_badge = if is_active { " <i>[ACTIVE]</i>" } else { "" };
        let bound = inst.bound_email.as_deref().unwrap_or("(none)");
        let pid_str = if pids.is_empty() {
            "-".to_string()
        } else {
            pids.iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        };

        out.push_str(&format!(
            "{}. {} <b>{}</b> (<code>{}</code>){}\n   👤 Bound: <code>{}</code>\n   🚦 PIDs: <code>{}</code>\n   📁 Data: <code>{}</code>\n",
            idx + 1,
            status_emoji,
            clean_for_telegram_html(&inst.name, 48),
            clean_for_telegram_html(&inst.id, 48),
            active_badge,
            clean_for_telegram_html(bound, 64),
            clean_for_telegram_html(&pid_str, 48),
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
                clean_for_telegram_html(&e.to_string(), 200)
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
        let tier = acc
            .quota
            .as_ref()
            .and_then(|q| q.subscription_tier.as_deref())
            .unwrap_or("Standard");
        let (q_4h, q_weekly) =
            crate::modules::auto_switcher::extract_dual_window_quotas(acc, "gemini-2.5-pro");
        let q_str = match (q_4h, q_weekly) {
            (Some(h), Some(w)) => format!("4h: {:.1}% | Weekly: {:.1}%", h, w),
            (Some(h), None) => format!("4h: {:.1}%", h),
            (None, Some(w)) => format!("Weekly: {:.1}%", w),
            _ => "Active".to_string(),
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

    out.push_str(
        "\n💡 Export CSV: <code>/accounts --csv</code> or <code>/export accounts csv</code>",
    );
    out
}

/// Format dedicated Account & Instance Credit Balance dashboard (/balance, /credits, /quota)
pub fn format_balance_report(filter_arg: &str) -> String {
    let local_config = supabase_sync::load_config().unwrap_or_default();
    let machine_name = std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| email_watcher::detect_machine_name());
    let local_ip = email_watcher::detect_local_ip();
    let raw_alias = if !local_config.node_alias.trim().is_empty() {
        local_config.node_alias.trim().to_string()
    } else if let Ok(settings) = email_vault_db::get_notification_settings() {
        settings.local_machine_name.trim().to_string()
    } else {
        machine_name.clone()
    };
    let display_alias = if raw_alias.is_empty() {
        machine_name.clone()
    } else {
        raw_alias
    };

    let app_cfg = crate::modules::config::load_app_config().unwrap_or_default();
    let switch_threshold = app_cfg.quota_protection.threshold_percentage;
    let target_model = app_cfg.auto_profile_switcher.target_model.clone();

    let reg = crate::modules::instance::load_registry().unwrap_or_default();
    let accounts = crate::modules::account::list_accounts().unwrap_or_default();
    let curr_acc_id = crate::modules::account::get_current_account_id().unwrap_or_default();

    let filter_clean = filter_arg
        .trim()
        .replace("--csv", "")
        .replace("--json", "")
        .trim()
        .to_lowercase();

    let mut out = format!(
        "💰 <b>Account &amp; Instance Credit Balance Dashboard</b>\n\
        ━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
        🖥️ <b>Machine:</b> <code>{}</code> (<code>{}</code>) | 🏷️ <b>Node:</b> <code>{}</code>\n\
        🛡️ <b>Switch Threshold:</b> <code>{}%</code> | 🎯 <b>Model:</b> <code>{}</code>\n\
        👥 <b>Total Accounts:</b> <code>{}</code> | 💻 <b>Instances:</b> <code>{}</code>\n\
        ━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n\
        <b>🏷️ INSTANCE BOUND ACCOUNTS &amp; BALANCES</b>\n",
        clean_for_telegram_html(&machine_name, 48),
        clean_for_telegram_html(&local_ip, 48),
        clean_for_telegram_html(&display_alias, 48),
        switch_threshold,
        clean_for_telegram_html(&target_model, 40),
        accounts.len(),
        reg.instances.len()
    );

    let mut bound_emails_set = std::collections::HashSet::new();

    if reg.instances.is_empty() {
        let default_acc = accounts
            .iter()
            .find(|a| curr_acc_id.as_deref() == Some(&a.id))
            .or_else(|| accounts.first());
        if let Some(acc) = default_acc {
            bound_emails_set.insert(acc.email.to_lowercase());
            let tier = acc
                .quota
                .as_ref()
                .and_then(|q| q.subscription_tier.as_deref())
                .unwrap_or("PRO");
            let (q_4h, q_weekly) =
                crate::modules::auto_switcher::extract_dual_window_quotas(acc, &target_model);
            let h_str = q_4h
                .map(|v| format!("{:.1}%", v))
                .unwrap_or_else(|| "100.0%".to_string());
            let w_str = q_weekly
                .map(|v| format!("{:.1}%", v))
                .unwrap_or_else(|| "100.0%".to_string());
            out.push_str(&format!(
                "• 🟢 <b>default</b> (<code>{}</code>)\n  \u{2007}\u{2007}└ 👤 <code>{}</code> [<code>{}</code>]\n  \u{2007}\u{2007}└ 📊 <b>4h:</b> <code>{}</code> | <b>Weekly:</b> <code>{}</code>\n",
                clean_for_telegram_html(&display_alias, 24),
                clean_for_telegram_html(&acc.email, 48),
                clean_for_telegram_html(tier, 16),
                h_str,
                w_str
            ));
        } else {
            out.push_str("• ⚪ <b>default</b> — <i>No active account bound</i>\n");
        }
    } else {
        for inst in &reg.instances {
            if !filter_clean.is_empty()
                && filter_clean != "all"
                && !inst.name.to_lowercase().contains(&filter_clean)
                && !inst.id.to_lowercase().contains(&filter_clean)
            {
                continue;
            }
            let pids =
                crate::modules::instance::find_pids_for_data_dir(&inst.data_dir, inst.is_default);
            let is_running = !pids.is_empty()
                || crate::modules::instance::is_instance_running(
                    &inst.id,
                    &inst.data_dir,
                    inst.pid,
                );
            let state_icon = if is_running { "🟢" } else { "⚪" };
            let bound_email = inst
                .bound_email
                .clone()
                .or_else(|| {
                    if inst.is_default || inst.id == "default" {
                        accounts
                            .iter()
                            .find(|a| curr_acc_id.as_deref() == Some(&a.id))
                            .map(|a| a.email.clone())
                    } else {
                        None
                    }
                })
                .unwrap_or_else(|| "(unbound)".to_string());

            bound_emails_set.insert(bound_email.to_lowercase());

            let matched_acc = accounts
                .iter()
                .find(|a| a.email.eq_ignore_ascii_case(&bound_email))
                .or_else(|| {
                    inst.bound_account_id
                        .as_ref()
                        .and_then(|bid| accounts.iter().find(|a| &a.id == bid))
                });

            let (tier_str, h_str, w_str) = if let Some(acc) = matched_acc {
                let t = acc
                    .quota
                    .as_ref()
                    .and_then(|q| q.subscription_tier.as_deref())
                    .unwrap_or("PRO");
                let (q_4h, q_weekly) =
                    crate::modules::auto_switcher::extract_dual_window_quotas(acc, &target_model);
                (
                    t.to_string(),
                    q_4h.map(|v| format!("{:.1}%", v))
                        .unwrap_or_else(|| "100.0%".to_string()),
                    q_weekly
                        .map(|v| format!("{:.1}%", v))
                        .unwrap_or_else(|| "100.0%".to_string()),
                )
            } else {
                ("N/A".to_string(), "-".to_string(), "-".to_string())
            };

            out.push_str(&format!(
                "• {} <b>{}</b> (<code>{}</code> | <code>{}</code>)\n  \u{2007}\u{2007}└ 👤 <code>{}</code> [<code>{}</code>]\n  \u{2007}\u{2007}└ 📊 <b>4h:</b> <code>{}</code> | <b>Weekly:</b> <code>{}</code>\n",
                state_icon,
                clean_for_telegram_html(&inst.name, 32),
                clean_for_telegram_html(&inst.id, 24),
                clean_for_telegram_html(&display_alias, 24),
                clean_for_telegram_html(&bound_email, 48),
                clean_for_telegram_html(&tier_str, 16),
                h_str,
                w_str
            ));
        }
    }

    // Top Standby Accounts Pool
    let mut standby: Vec<(String, String, f64, f64)> = accounts
        .iter()
        .filter(|a| !bound_emails_set.contains(&a.email.to_lowercase()))
        .map(|a| {
            let tier = a
                .quota
                .as_ref()
                .and_then(|q| q.subscription_tier.as_deref())
                .unwrap_or("PRO")
                .to_string();
            let (q_4h, q_weekly) =
                crate::modules::auto_switcher::extract_dual_window_quotas(a, &target_model);
            (
                a.email.clone(),
                tier,
                q_4h.unwrap_or(100.0),
                q_weekly.unwrap_or(100.0),
            )
        })
        .collect();

    standby.sort_by(|a, b| {
        b.2.partial_cmp(&a.2)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| b.3.partial_cmp(&a.3).unwrap_or(std::cmp::Ordering::Equal))
    });

    out.push_str("\n<b>🔋 STANDBY ACCOUNT POOL (TOP READY)</b>\n");
    if standby.is_empty() {
        out.push_str("• <i>All registered accounts are currently bound to instances.</i>\n");
    } else {
        for (idx, (email, tier, h, w)) in standby.iter().take(6).enumerate() {
            out.push_str(&format!(
                "{}. <code>{}</code> [<code>{}</code>] — <b>4h:</b> <code>{:.1}%</code> | <b>Wk:</b> <code>{:.1}%</code>\n",
                idx + 1,
                clean_for_telegram_html(email, 48),
                clean_for_telegram_html(tier, 16),
                h,
                w
            ));
        }
    }

    let ssh_nodes = crate::modules::ssh_manager::load_ssh_connections().unwrap_or_default();
    if !ssh_nodes.is_empty() {
        out.push_str(&format!(
            "\n🌐 <b>SSH Fleet Nodes Configured:</b> <code>{}</code> (Send <code>/ssh balance</code> or <code>/nodes</code>)\n",
            ssh_nodes.len()
        ));
    }

    out.push_str(
        "\n💡 <b>Quick Commands:</b> <code>/balance --csv</code> · <code>/export balance csv</code> · <code>/ff</code>",
    );
    out
}

/// Inline keyboard markup for `/balance` (`/credits`, `/quota`)
pub fn resolve_balance_markup() -> serde_json::Value {
    serde_json::json!({
        "inline_keyboard": [
            [
                { "text": "📥 Export Balance CSV", "callback_data": "cmd:/export balance csv" },
                { "text": "🔄 Refresh Balance", "callback_data": "cmd:/balance" }
            ],
            [
                { "text": "⚡ Fast-Forward (/ff)", "callback_data": "cmd:/ff" },
                { "text": "🚀 Send Prompt", "callback_data": "cmd:/send" }
            ],
            [
                { "text": "📥 Accounts CSV", "callback_data": "cmd:/export accounts csv" },
                { "text": "📥 Instances CSV", "callback_data": "cmd:/export instances csv" }
            ]
        ]
    })
}

/// RFC 4180 compliant CSV field escaping
pub fn escape_csv_field(val: &str) -> String {
    if val.contains(',') || val.contains('"') || val.contains('\n') || val.contains('\r') {
        format!("\"{}\"", val.replace('"', "\"\""))
    } else {
        val.to_string()
    }
}

/// Export document container for Telegram sendDocument upload
#[derive(Debug, Clone)]
pub struct TelegramExportDocument {
    pub file_name: String,
    pub mime_type: String,
    pub bytes: Vec<u8>,
    pub caption: String,
    pub markup: Option<serde_json::Value>,
}

/// Generate export dataset returning `(filename, mime_or_ext, file_content_string)`
/// Supports targets: `accounts`, `instances`, `balance` (`credits`/`quota`), `nodes` (`ssh`), `prompts`, `status`, `all`
/// Supports formats: `csv`, `json`
pub fn generate_export_dataset(
    target: &str,
    format: &str,
) -> Result<(String, String, String), String> {
    let clean_target = target.trim().to_lowercase();
    let clean_fmt = if format.trim().eq_ignore_ascii_case("json") {
        "json"
    } else {
        "csv"
    };
    let mime = if clean_fmt == "json" {
        "application/json".to_string()
    } else {
        "text/csv".to_string()
    };

    let machine_name = email_watcher::detect_machine_name();
    let local_ip = email_watcher::detect_local_ip();
    let app_cfg = crate::modules::config::load_app_config().unwrap_or_default();
    let threshold_pct = app_cfg.quota_protection.threshold_percentage;
    let target_model = app_cfg.auto_profile_switcher.target_model.clone();

    match clean_target.as_str() {
        "accounts" | "account" | "acc" => {
            let accounts = crate::modules::account::list_accounts().unwrap_or_default();
            let curr_id = crate::modules::account::get_current_account_id().unwrap_or_default();
            let reg = crate::modules::instance::load_registry().unwrap_or_default();

            if clean_fmt == "json" {
                let items: Vec<serde_json::Value> = accounts
                    .iter()
                    .map(|acc| {
                        let is_active = curr_id.as_deref() == Some(&acc.id);
                        let tier = acc
                            .quota
                            .as_ref()
                            .and_then(|q| q.subscription_tier.as_deref())
                            .unwrap_or("PRO");
                        let (q_4h, q_weekly) =
                            crate::modules::auto_switcher::extract_dual_window_quotas(
                                acc,
                                &target_model,
                            );
                        let min_pct = acc
                            .quota
                            .as_ref()
                            .and_then(|q| q.models.iter().map(|m| m.percentage).min())
                            .unwrap_or(100);
                        let bound_insts: Vec<String> = reg
                            .instances
                            .iter()
                            .filter(|i| {
                                i.bound_email
                                    .as_deref()
                                    .map(|e| e.eq_ignore_ascii_case(&acc.email))
                                    .unwrap_or(false)
                            })
                            .map(|i| i.name.clone())
                            .collect();
                        serde_json::json!({
                            "id": acc.id,
                            "email": acc.email,
                            "tier": tier,
                            "is_active": is_active,
                            "bound_instances": bound_insts,
                            "quota_4h_pct": q_4h.unwrap_or(100.0),
                            "quota_weekly_pct": q_weekly.unwrap_or(100.0),
                            "min_model_pct": min_pct,
                            "machine": machine_name,
                            "ip": local_ip,
                        })
                    })
                    .collect();
                let content = serde_json::to_string_pretty(&items)
                    .map_err(|e| format!("Failed to serialize accounts JSON: {}", e))?;
                Ok(("agm-accounts.json".to_string(), mime, content))
            } else {
                let mut csv = String::from(
                    "id,email,tier,is_active,bound_instances,quota_4h_pct,quota_weekly_pct,min_model_pct,machine,ip\n",
                );
                for acc in &accounts {
                    let is_active = curr_id.as_deref() == Some(&acc.id);
                    let tier = acc
                        .quota
                        .as_ref()
                        .and_then(|q| q.subscription_tier.as_deref())
                        .unwrap_or("PRO");
                    let (q_4h, q_weekly) =
                        crate::modules::auto_switcher::extract_dual_window_quotas(
                            acc,
                            &target_model,
                        );
                    let min_pct = acc
                        .quota
                        .as_ref()
                        .and_then(|q| q.models.iter().map(|m| m.percentage).min())
                        .unwrap_or(100);
                    let bound_insts: Vec<String> = reg
                        .instances
                        .iter()
                        .filter(|i| {
                            i.bound_email
                                .as_deref()
                                .map(|e| e.eq_ignore_ascii_case(&acc.email))
                                .unwrap_or(false)
                        })
                        .map(|i| i.name.clone())
                        .collect();
                    csv.push_str(&format!(
                        "{},{},{},{},{},{:.1},{:.1},{},{},{}\n",
                        escape_csv_field(&acc.id),
                        escape_csv_field(&acc.email),
                        escape_csv_field(tier),
                        is_active,
                        escape_csv_field(&bound_insts.join(";")),
                        q_4h.unwrap_or(100.0),
                        q_weekly.unwrap_or(100.0),
                        min_pct,
                        escape_csv_field(&machine_name),
                        escape_csv_field(&local_ip),
                    ));
                }
                Ok(("agm-accounts.csv".to_string(), mime, csv))
            }
        }
        "instances" | "instance" | "inst" => {
            let reg = crate::modules::instance::load_registry().unwrap_or_default();
            let accounts = crate::modules::account::list_accounts().unwrap_or_default();
            let curr_id = crate::modules::account::get_current_account_id().unwrap_or_default();

            if clean_fmt == "json" {
                let items: Vec<serde_json::Value> = reg
                    .instances
                    .iter()
                    .map(|inst| {
                        let pids = crate::modules::instance::find_pids_for_data_dir(
                            &inst.data_dir,
                            inst.is_default,
                        );
                        let running = !pids.is_empty()
                            || crate::modules::instance::is_instance_running(
                                &inst.id,
                                &inst.data_dir,
                                inst.pid,
                            );
                        let bound_email = inst
                            .bound_email
                            .clone()
                            .or_else(|| {
                                if inst.is_default || inst.id == "default" {
                                    accounts
                                        .iter()
                                        .find(|a| curr_id.as_deref() == Some(&a.id))
                                        .map(|a| a.email.clone())
                                } else {
                                    None
                                }
                            })
                            .unwrap_or_default();
                        serde_json::json!({
                            "id": inst.id,
                            "name": inst.name,
                            "is_default": inst.is_default,
                            "running": running,
                            "pid": pids.first().copied().or(inst.pid),
                            "bound_email": bound_email,
                            "data_dir": inst.data_dir,
                            "machine": machine_name,
                            "ip": local_ip,
                        })
                    })
                    .collect();
                let content = serde_json::to_string_pretty(&items)
                    .map_err(|e| format!("Failed to serialize instances JSON: {}", e))?;
                Ok(("agm-instances.json".to_string(), mime, content))
            } else {
                let mut csv = String::from(
                    "id,name,is_default,running,pid,bound_email,data_dir,machine,ip\n",
                );
                for inst in &reg.instances {
                    let pids = crate::modules::instance::find_pids_for_data_dir(
                        &inst.data_dir,
                        inst.is_default,
                    );
                    let running = !pids.is_empty()
                        || crate::modules::instance::is_instance_running(
                            &inst.id,
                            &inst.data_dir,
                            inst.pid,
                        );
                    let pid_str = pids
                        .first()
                        .copied()
                        .or(inst.pid)
                        .map(|p| p.to_string())
                        .unwrap_or_default();
                    let bound_email = inst
                        .bound_email
                        .clone()
                        .or_else(|| {
                            if inst.is_default || inst.id == "default" {
                                accounts
                                    .iter()
                                    .find(|a| curr_id.as_deref() == Some(&a.id))
                                    .map(|a| a.email.clone())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_default();
                    csv.push_str(&format!(
                        "{},{},{},{},{},{},{},{},{}\n",
                        escape_csv_field(&inst.id),
                        escape_csv_field(&inst.name),
                        inst.is_default,
                        running,
                        escape_csv_field(&pid_str),
                        escape_csv_field(&bound_email),
                        escape_csv_field(&inst.data_dir),
                        escape_csv_field(&machine_name),
                        escape_csv_field(&local_ip),
                    ));
                }
                Ok(("agm-instances.csv".to_string(), mime, csv))
            }
        }
        "balance" | "credits" | "quota" | "bal" | "all" | "cluster" => {
            let reg = crate::modules::instance::load_registry().unwrap_or_default();
            let accounts = crate::modules::account::list_accounts().unwrap_or_default();
            let curr_id = crate::modules::account::get_current_account_id().unwrap_or_default();

            let mut rows_json = Vec::new();
            let mut csv = String::from(
                "machine,ip,instance_id,instance_name,running,account_email,tier,quota_4h_pct,quota_weekly_pct,min_model_pct,threshold_pct\n",
            );

            if reg.instances.is_empty() {
                for acc in &accounts {
                    let is_curr = curr_id.as_deref() == Some(&acc.id);
                    let inst_id = if is_curr { "default" } else { "standby" };
                    let tier = acc
                        .quota
                        .as_ref()
                        .and_then(|q| q.subscription_tier.as_deref())
                        .unwrap_or("PRO");
                    let (q_4h, q_weekly) =
                        crate::modules::auto_switcher::extract_dual_window_quotas(
                            acc,
                            &target_model,
                        );
                    let min_pct = acc
                        .quota
                        .as_ref()
                        .and_then(|q| q.models.iter().map(|m| m.percentage).min())
                        .unwrap_or(100);
                    rows_json.push(serde_json::json!({
                        "machine": machine_name,
                        "ip": local_ip,
                        "instance_id": inst_id,
                        "instance_name": inst_id,
                        "running": is_curr,
                        "account_email": acc.email,
                        "tier": tier,
                        "quota_4h_pct": q_4h.unwrap_or(100.0),
                        "quota_weekly_pct": q_weekly.unwrap_or(100.0),
                        "min_model_pct": min_pct,
                        "threshold_pct": threshold_pct,
                    }));
                    csv.push_str(&format!(
                        "{},{},{},{},{},{},{},{:.1},{:.1},{},{}\n",
                        escape_csv_field(&machine_name),
                        escape_csv_field(&local_ip),
                        escape_csv_field(inst_id),
                        escape_csv_field(inst_id),
                        is_curr,
                        escape_csv_field(&acc.email),
                        escape_csv_field(tier),
                        q_4h.unwrap_or(100.0),
                        q_weekly.unwrap_or(100.0),
                        min_pct,
                        threshold_pct
                    ));
                }
            } else {
                for inst in &reg.instances {
                    let pids = crate::modules::instance::find_pids_for_data_dir(
                        &inst.data_dir,
                        inst.is_default,
                    );
                    let running = !pids.is_empty()
                        || crate::modules::instance::is_instance_running(
                            &inst.id,
                            &inst.data_dir,
                            inst.pid,
                        );
                    let bound_email = inst
                        .bound_email
                        .clone()
                        .or_else(|| {
                            if inst.is_default || inst.id == "default" {
                                accounts
                                    .iter()
                                    .find(|a| curr_id.as_deref() == Some(&a.id))
                                    .map(|a| a.email.clone())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_default();
                    let matched_acc = accounts
                        .iter()
                        .find(|a| a.email.eq_ignore_ascii_case(&bound_email));
                    let tier = matched_acc
                        .and_then(|a| a.quota.as_ref())
                        .and_then(|q| q.subscription_tier.as_deref())
                        .unwrap_or("PRO");
                    let (q_4h, q_weekly) = matched_acc
                        .map(|a| {
                            crate::modules::auto_switcher::extract_dual_window_quotas(
                                a,
                                &target_model,
                            )
                        })
                        .unwrap_or((Some(100.0), Some(100.0)));
                    let min_pct = matched_acc
                        .and_then(|a| a.quota.as_ref())
                        .and_then(|q| q.models.iter().map(|m| m.percentage).min())
                        .unwrap_or(100);

                    rows_json.push(serde_json::json!({
                        "machine": machine_name,
                        "ip": local_ip,
                        "instance_id": inst.id,
                        "instance_name": inst.name,
                        "running": running,
                        "account_email": bound_email,
                        "tier": tier,
                        "quota_4h_pct": q_4h.unwrap_or(100.0),
                        "quota_weekly_pct": q_weekly.unwrap_or(100.0),
                        "min_model_pct": min_pct,
                        "threshold_pct": threshold_pct,
                    }));
                    csv.push_str(&format!(
                        "{},{},{},{},{},{},{},{:.1},{:.1},{},{}\n",
                        escape_csv_field(&machine_name),
                        escape_csv_field(&local_ip),
                        escape_csv_field(&inst.id),
                        escape_csv_field(&inst.name),
                        running,
                        escape_csv_field(&bound_email),
                        escape_csv_field(tier),
                        q_4h.unwrap_or(100.0),
                        q_weekly.unwrap_or(100.0),
                        min_pct,
                        threshold_pct
                    ));
                }
            }

            if clean_fmt == "json" {
                let content = serde_json::to_string_pretty(&rows_json)
                    .map_err(|e| format!("Failed to serialize balance JSON: {}", e))?;
                Ok(("agm-balance.json".to_string(), mime, content))
            } else {
                Ok(("agm-balance.csv".to_string(), mime, csv))
            }
        }
        "nodes" | "node" | "ssh" => {
            let conns = crate::modules::ssh_manager::load_ssh_connections().unwrap_or_default();
            if clean_fmt == "json" {
                let items: Vec<serde_json::Value> = conns
                    .iter()
                    .map(|c| {
                        let port = if c.port == 0 { 22 } else { c.port };
                        serde_json::json!({
                            "alias": c.alias,
                            "ip_address": c.ip_address,
                            "username": c.username,
                            "port": port,
                            "os": c.os,
                            "auth_method": if !c.key_path.is_empty() { "key" } else { "rsa-password" },
                            "encryption_algo": if c.encryption_algo.is_empty() { "RSA-2048" } else { &c.encryption_algo },
                            "has_encrypted_password": !c.encrypted_password.is_empty(),
                        })
                    })
                    .collect();
                let content = serde_json::to_string_pretty(&items)
                    .map_err(|e| format!("Failed to serialize nodes JSON: {}", e))?;
                Ok(("agm-nodes.json".to_string(), mime, content))
            } else {
                let mut csv = String::from(
                    "alias,ip_address,username,port,os,auth_method,encryption_algo,has_encrypted_password\n",
                );
                for c in &conns {
                    let port = if c.port == 0 { 22 } else { c.port };
                    let auth = if !c.key_path.is_empty() {
                        "key"
                    } else {
                        "rsa-password"
                    };
                    let algo = if c.encryption_algo.is_empty() {
                        "RSA-2048"
                    } else {
                        &c.encryption_algo
                    };
                    csv.push_str(&format!(
                        "{},{},{},{},{},{},{},{}\n",
                        escape_csv_field(&c.alias),
                        escape_csv_field(&c.ip_address),
                        escape_csv_field(&c.username),
                        port,
                        escape_csv_field(&c.os),
                        auth,
                        escape_csv_field(algo),
                        !c.encrypted_password.is_empty()
                    ));
                }
                Ok(("agm-nodes.csv".to_string(), mime, csv))
            }
        }
        "prompts" | "prompt" => {
            let prompts = repo_db::list_all_prompts().unwrap_or_default();
            if clean_fmt == "json" {
                let items: Vec<serde_json::Value> = prompts
                    .iter()
                    .map(|p| {
                        let (preview, wc) =
                            repo_db::extract_prompt_words_preview(&p.prompt_content, 50);
                        serde_json::json!({
                            "id": p.id,
                            "instance_id": p.instance_id,
                            "project_id": p.project_id,
                            "status": p.status,
                            "word_count": wc,
                            "created_at": p.created_at,
                            "preview": preview,
                        })
                    })
                    .collect();
                let content = serde_json::to_string_pretty(&items)
                    .map_err(|e| format!("Failed to serialize prompts JSON: {}", e))?;
                Ok(("agm-prompts.json".to_string(), mime, content))
            } else {
                let mut csv =
                    String::from("id,instance_id,project_id,status,word_count,created_at,preview\n");
                for p in &prompts {
                    let (preview, wc) =
                        repo_db::extract_prompt_words_preview(&p.prompt_content, 40);
                    csv.push_str(&format!(
                        "{},{},{},{},{},{},{}\n",
                        escape_csv_field(&p.id),
                        escape_csv_field(&p.instance_id),
                        escape_csv_field(&p.project_id),
                        escape_csv_field(&p.status),
                        wc,
                        p.created_at,
                        escape_csv_field(&preview),
                    ));
                }
                Ok(("agm-prompts.csv".to_string(), mime, csv))
            }
        }
        "status" | "observe" => {
            let ver = git_info::get_app_version();
            let hash = git_info::get_git_hash();
            let reg = crate::modules::instance::load_registry().unwrap_or_default();
            let accounts = crate::modules::account::list_accounts().unwrap_or_default();
            let prompts = repo_db::list_all_prompts().unwrap_or_default();
            let running_prompts = prompts
                .iter()
                .filter(|p| p.status == "running" || p.status == "dispatched")
                .count();
            if clean_fmt == "json" {
                let obj = serde_json::json!({
                    "version": ver,
                    "git_hash": hash,
                    "machine": machine_name,
                    "ip": local_ip,
                    "accounts_total": accounts.len(),
                    "instances_total": reg.instances.len(),
                    "running_prompts": running_prompts,
                    "threshold_pct": threshold_pct,
                });
                let content = serde_json::to_string_pretty(&obj)
                    .map_err(|e| format!("Failed to serialize status JSON: {}", e))?;
                Ok(("agm-status.json".to_string(), mime, content))
            } else {
                let csv = format!(
                    "version,git_hash,machine,ip,accounts_total,instances_total,running_prompts,threshold_pct\n{},{},{},{},{},{},{},{}\n",
                    escape_csv_field(&ver),
                    escape_csv_field(&hash),
                    escape_csv_field(&machine_name),
                    escape_csv_field(&local_ip),
                    accounts.len(),
                    reg.instances.len(),
                    running_prompts,
                    threshold_pct
                );
                Ok(("agm-status.csv".to_string(), mime, csv))
            }
        }
        _ => Err(format!(
            "Unsupported export target '{}'. Valid targets: accounts, instances, balance, nodes, prompts, status",
            target
        )),
    }
}

/// Build full `TelegramExportDocument` with caption and inline keyboard for `/export`
pub fn generate_export_document(
    kind: &str,
    format: &str,
) -> Result<TelegramExportDocument, String> {
    let (file_name, mime_type, content) = generate_export_dataset(kind, format)?;
    let line_count = content.lines().count();
    let machine_name = email_watcher::detect_machine_name();
    let local_ip = email_watcher::detect_local_ip();
    let clean_kind = if kind.trim().is_empty() {
        "accounts"
    } else {
        kind.trim()
    };
    let clean_fmt = if format.trim().eq_ignore_ascii_case("json") {
        "JSON"
    } else {
        "CSV"
    };

    let caption = format!(
        "📥 <b>AGM {} {} Export</b>\n\
        🖥️ <b>Machine:</b> <code>{}</code> (<code>{}</code>)\n\
        📄 <b>File:</b> <code>{}</code> ({} lines / {} bytes)",
        clean_for_telegram_html(&clean_kind.to_uppercase(), 24),
        clean_fmt,
        clean_for_telegram_html(&machine_name, 48),
        clean_for_telegram_html(&local_ip, 48),
        clean_for_telegram_html(&file_name, 48),
        line_count,
        content.len()
    );

    let markup = serde_json::json!({
        "inline_keyboard": [
            [
                { "text": "💰 Check Balance", "callback_data": "cmd:/balance" },
                { "text": "📥 Accounts CSV", "callback_data": "cmd:/export accounts csv" }
            ],
            [
                { "text": "📥 Instances CSV", "callback_data": "cmd:/export instances csv" },
                { "text": "🔄 Refresh Status", "callback_data": "cmd:/status" }
            ]
        ]
    });

    Ok(TelegramExportDocument {
        file_name,
        mime_type,
        bytes: content.into_bytes(),
        caption,
        markup: Some(markup),
    })
}
