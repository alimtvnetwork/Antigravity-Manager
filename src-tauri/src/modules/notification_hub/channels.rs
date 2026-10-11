use crate::modules::logger;

use super::*;
use super::telegram_alerts::dispatch_telegram_switch_alert;

/// Dispatch rich notifications across Email and Telegram upon account/instance switch
pub async fn notify_account_switched_details(mut details: SwitchNotificationDetails) -> String {
    let now_ts = chrono::Utc::now().timestamp();
    let selected_clean = details.selected_email.trim().to_string();

    if let Ok(mut guard) = LAST_SWITCH_DISPATCH.lock() {
        if guard.0.eq_ignore_ascii_case(&selected_clean) && (now_ts - guard.1).abs() <= 5 {
            let line = format!(
                "[Notify] skipped duplicate switch to {} within 5s",
                selected_clean
            );
            logger::log_info(&line);
            println!("  [OK] {}", line);
            return line;
        }
        *guard = (selected_clean.clone(), now_ts);
    }

    let (context_old_email, inst_id, inst_name, instance_mode) =
        resolve_switch_context(&selected_clean, &details.instance_name);

    if details.instance_id.is_empty() {
        details.instance_id = inst_id;
    }
    if details.instance_name.is_empty() {
        details.instance_name = inst_name;
    }
    if details.instance_mode.is_empty() {
        details.instance_mode = instance_mode;
    }

    let resolved_prev = details
        .previous_email
        .as_deref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case("default"))
        .unwrap_or(context_old_email.trim());

    let final_prev = if resolved_prev.is_empty()
        || resolved_prev.eq_ignore_ascii_case("default")
        || resolved_prev.eq_ignore_ascii_case(&selected_clean)
    {
        "(none / standby)".to_string()
    } else {
        resolved_prev.to_string()
    };
    details.previous_email = Some(final_prev.clone());

    let need_predict = match &details.predicted_next_email {
        None => true,
        Some(em) => {
            let em_trimmed = em.trim();
            em_trimmed.is_empty()
                || em_trimmed.eq_ignore_ascii_case(&selected_clean)
                || em_trimmed.eq_ignore_ascii_case(&final_prev)
        }
    };

    if need_predict {
        let mut pred_exclusions = vec![selected_clean.clone()];
        if !final_prev.is_empty() && !final_prev.eq_ignore_ascii_case("(none / standby)") {
            pred_exclusions.push(final_prev.clone());
        }
        let target_inst = if details.instance_id.is_empty() {
            "default"
        } else {
            &details.instance_id
        };
        let predicted_candidate = crate::modules::auto_switcher::select_candidate_profiles(
            target_inst,
            "gemini-2.5-pro",
            15.0,
            &pred_exclusions,
        )
        .ok()
        .and_then(|v| v.into_iter().next())
        .filter(|c| {
            !c.email.trim().eq_ignore_ascii_case(&selected_clean)
                && (final_prev.eq_ignore_ascii_case("(none / standby)")
                    || !c.email.trim().eq_ignore_ascii_case(&final_prev))
        });
        details.predicted_next_email = predicted_candidate.map(|c| c.email);
    }

    // Resolve target dual-window quotas if absent
    if details.target_quota_4h.is_none() || details.target_quota_weekly.is_none() {
        if let Ok(accounts) = crate::modules::account::list_accounts() {
            if let Some(target_acc) = accounts
                .iter()
                .find(|a| a.email.eq_ignore_ascii_case(&selected_clean))
            {
                let (q4, qw) = crate::modules::auto_switcher::extract_dual_window_quotas(
                    target_acc,
                    "gemini-2.5-pro",
                );
                if details.target_quota_4h.is_none() {
                    details.target_quota_4h = q4;
                }
                if details.target_quota_weekly.is_none() {
                    details.target_quota_weekly = qw;
                }
            }
        }
    }

    // Resolve previous dual-window quotas if absent
    if (details.previous_quota_4h.is_none() || details.previous_quota_weekly.is_none())
        && !final_prev.eq_ignore_ascii_case("(none / standby)")
    {
        if let Ok(accounts) = crate::modules::account::list_accounts() {
            if let Some(prev_acc) = accounts
                .iter()
                .find(|a| a.email.eq_ignore_ascii_case(&final_prev))
            {
                let (q4, qw) = crate::modules::auto_switcher::extract_dual_window_quotas(
                    prev_acc,
                    "gemini-2.5-pro",
                );
                if details.previous_quota_4h.is_none() {
                    details.previous_quota_4h = q4;
                }
                if details.previous_quota_weekly.is_none() {
                    details.previous_quota_weekly = qw;
                }
            }
        }
    }

    deliver_switch_channels(&details).await
}

pub(crate) fn redact_secrets(text: &str) -> String {
    let mut out = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i..].starts_with(&['/', 'b', 'o', 't']) {
            out.push_str("/bot<redacted>");
            i += 4;
            while i < chars.len() && chars[i] != '/' && chars[i] != ' ' {
                i += 1;
            }
            continue;
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

pub(crate) fn record_channel(name: &str, result: Result<String, String>) -> String {
    match result {
        Ok(msg) => {
            let line = format!("[Notify] {}: OK {}", name, msg);
            logger::log_info(&line);
            println!("  [OK] {}", line);
            line
        }
        Err(err) => {
            let trace = std::backtrace::Backtrace::force_capture();
            let line = format!(
                "[Notify] {}: FAIL {}\n{}",
                name,
                redact_secrets(&err),
                trace
            );
            logger::log_error(&line);
            eprintln!("{}", line);
            format!("[Notify] {}: FAIL {}", name, redact_secrets(&err))
        }
    }
}

async fn deliver_switch_channels(details: &SwitchNotificationDetails) -> String {
    let email = record_channel("email", dispatch_email_switch_alert(details));
    dispatch_self_json_in_use_broadcast(details);
    let telegram = record_channel("telegram", dispatch_telegram_switch_alert(details).await);
    let supabase = record_channel(
        "supabase",
        crate::modules::supabase_sync::push_and_read_instance_email(&details.instance_id)
            .await
            .map_err(|err| redact_secrets(&err.to_string())),
    );
    format!("{}\n{}\n{}", email, telegram, supabase)
}

/// Dispatch notifications across Email and Telegram upon account/instance switch (backward-compatible)
pub fn notify_account_switched(
    account_email: &str,
    instance_name: &str,
    reason: &str,
    is_auto: bool,
) {
    let details = SwitchNotificationDetails {
        previous_email: None,
        previous_quota_4h: None,
        previous_quota_weekly: None,
        predicted_next_email: None,
        selected_email: account_email.to_string(),
        target_quota_4h: None,
        target_quota_weekly: None,
        credit_before_switch: None,
        threshold_activated: None,
        instance_id: instance_name.to_string(),
        instance_name: instance_name.to_string(),
        instance_mode: String::new(),
        reason: reason.to_string(),
        is_auto,
        backed_up_projects: Vec::new(),
        backed_up_prompts_count: None,
        restored_prompts_count: None,
    };
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => {
            handle.spawn(async move {
                // Justification: non-Result return value intentionally discarded — no error channel to track
                let _ = notify_account_switched_details(details).await;
            });
        }
        Err(_) => {
            if let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            {
                rt.block_on(notify_account_switched_details(details));
            }
        }
    }
}
