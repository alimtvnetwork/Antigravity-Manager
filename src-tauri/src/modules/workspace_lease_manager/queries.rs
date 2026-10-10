use chrono::Utc;

use super::*;

/// Synchronously check if an account is currently leased by another active node
pub fn is_account_leased_by_other(account_id: &str) -> bool {
    is_account_or_email_leased_by_other(account_id, "")
}

/// Synchronously check if an account (by ID or profile email) is currently leased by another active node.
/// Automatically marks remote leases as inactive if more than `stale_binding_timeout_hours` (default 6h, configurable 6h-10h)
/// have elapsed since `leased_at` with no ping or lease refresh.
pub fn is_account_or_email_leased_by_other(account_id: &str, email: &str) -> bool {
    let local_node = supabase_sync::get_local_node_id();
    let now = Utc::now().timestamp();
    let app_config = crate::modules::config::load_app_config();
    let stale_hours = app_config
        .as_ref()
        .map(|c| {
            c.auto_profile_switcher
                .stale_binding_timeout_hours
                .clamp(1, 24)
        })
        .unwrap_or(6);
    let cooldown_minutes = app_config
        .as_ref()
        .map(|c| {
            c.auto_profile_switcher
                .account_cooldown_minutes
                .max(c.auto_profile_switcher.account_lockout_window_minutes)
        })
        .unwrap_or(60);
    let stale_timeout_secs = (stale_hours as i64) * 3600;
    let lockout_window_secs = (cooldown_minutes as i64) * 60;
    let acc_id_clean = account_id.trim();
    let resolved_email = if !email.trim().is_empty() {
        email.trim().to_lowercase()
    } else if !acc_id_clean.is_empty() {
        crate::modules::account::load_account(acc_id_clean)
            .map(|a| a.email.trim().to_lowercase())
            .unwrap_or_default()
    } else {
        String::new()
    };
    let email_clean = resolved_email.as_str();

    if let Ok(cache) = ACTIVE_REMOTE_LEASES.read() {
        for (k, lease) in cache.iter() {
            let is_match_id = !acc_id_clean.is_empty()
                && (k.eq_ignore_ascii_case(acc_id_clean)
                    || lease.account_id.trim().eq_ignore_ascii_case(acc_id_clean)
                    || lease
                        .account_email
                        .trim()
                        .eq_ignore_ascii_case(acc_id_clean));
            let is_match_email = !email_clean.is_empty()
                && (lease.account_email.trim().to_lowercase() == email_clean
                    || lease.profile_name.trim().to_lowercase() == email_clean
                    || lease.account_id.trim().to_lowercase() == email_clean);
            let is_match = is_match_id || is_match_email;
            let is_stale_without_ping =
                lease.leased_at > 0 && (now - lease.leased_at) > stale_timeout_secs;
            let is_locked_or_unexpired = (lease.leased_at > 0
                && (now - lease.leased_at) < lockout_window_secs)
                || lease.expires_at > now;
            if is_match
                && is_locked_or_unexpired
                && !is_stale_without_ping
                && !lease.node_id.trim().eq_ignore_ascii_case(local_node.trim())
            {
                return true;
            }
        }
    }
    false
}

/// If an account or email is leased by another active node, return (node_alias, profile_name, remaining_seconds)
pub fn get_remote_lease_holder_info(
    account_id: &str,
    email: &str,
) -> Option<(String, String, i64)> {
    let local_node = supabase_sync::get_local_node_id();
    let now = Utc::now().timestamp();
    let app_config = crate::modules::config::load_app_config();
    let stale_hours = app_config
        .as_ref()
        .map(|c| {
            c.auto_profile_switcher
                .stale_binding_timeout_hours
                .clamp(1, 24)
        })
        .unwrap_or(6);
    let cooldown_minutes = app_config
        .as_ref()
        .map(|c| {
            c.auto_profile_switcher
                .account_cooldown_minutes
                .max(c.auto_profile_switcher.account_lockout_window_minutes)
        })
        .unwrap_or(60);
    let stale_timeout_secs = (stale_hours as i64) * 3600;
    let lockout_window_secs = (cooldown_minutes as i64) * 60;
    let acc_id_clean = account_id.trim();
    let resolved_email = if !email.trim().is_empty() {
        email.trim().to_lowercase()
    } else if !acc_id_clean.is_empty() {
        crate::modules::account::load_account(acc_id_clean)
            .map(|a| a.email.trim().to_lowercase())
            .unwrap_or_default()
    } else {
        String::new()
    };
    let email_clean = resolved_email.as_str();

    if let Ok(cache) = ACTIVE_REMOTE_LEASES.read() {
        for (k, lease) in cache.iter() {
            let is_match_id = !acc_id_clean.is_empty()
                && (k.eq_ignore_ascii_case(acc_id_clean)
                    || lease.account_id.trim().eq_ignore_ascii_case(acc_id_clean)
                    || lease
                        .account_email
                        .trim()
                        .eq_ignore_ascii_case(acc_id_clean));
            let is_match_email = !email_clean.is_empty()
                && (lease.account_email.trim().to_lowercase() == email_clean
                    || lease.profile_name.trim().to_lowercase() == email_clean
                    || lease.account_id.trim().to_lowercase() == email_clean);
            let is_match = is_match_id || is_match_email;
            let is_stale_without_ping =
                lease.leased_at > 0 && (now - lease.leased_at) > stale_timeout_secs;
            let is_locked_or_unexpired = (lease.leased_at > 0
                && (now - lease.leased_at) < lockout_window_secs)
                || lease.expires_at > now;
            if is_match
                && is_locked_or_unexpired
                && !is_stale_without_ping
                && !lease.node_id.trim().eq_ignore_ascii_case(local_node.trim())
            {
                let remaining = (lease.expires_at - now).max(0);
                return Some((
                    lease.node_alias.clone(),
                    lease.profile_name.clone(),
                    remaining,
                ));
            }
        }
    }
    None
}

/// Find any cached workspace lease matching account ID or email (case-insensitive)
pub fn find_cached_lease(account_id: &str, email: &str) -> Option<WorkspaceLease> {
    let acc_id_clean = account_id.trim();
    let resolved_email = if !email.trim().is_empty() {
        email.trim().to_lowercase()
    } else if !acc_id_clean.is_empty() {
        crate::modules::account::load_account(acc_id_clean)
            .map(|a| a.email.trim().to_lowercase())
            .unwrap_or_default()
    } else {
        String::new()
    };
    let email_clean = resolved_email.as_str();

    if let Ok(cache) = ACTIVE_REMOTE_LEASES.read() {
        for (k, lease) in cache.iter() {
            let is_match_id = !acc_id_clean.is_empty()
                && (k.eq_ignore_ascii_case(acc_id_clean)
                    || lease.account_id.trim().eq_ignore_ascii_case(acc_id_clean)
                    || lease
                        .account_email
                        .trim()
                        .eq_ignore_ascii_case(acc_id_clean));
            let is_match_email = !email_clean.is_empty()
                && (lease.account_email.trim().to_lowercase() == email_clean
                    || lease.profile_name.trim().to_lowercase() == email_clean
                    || lease.account_id.trim().to_lowercase() == email_clean);
            if is_match_id || is_match_email {
                return Some(lease.clone());
            }
        }
    }
    None
}

/// Check if an account or email has an active or recent lease in the remote cache within the given cooldown window.
pub fn has_active_lease_in_cooldown(account_id: &str, email: &str, cooldown_secs: i64) -> bool {
    let now = Utc::now().timestamp();
    if let Some(lease) = find_cached_lease(account_id, email) {
        if (lease.leased_at > 0 && (now - lease.leased_at) < cooldown_secs)
            || lease.expires_at > now
        {
            return true;
        }
    }
    false
}
