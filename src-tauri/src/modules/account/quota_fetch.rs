pub use crate::models::{
    Account, AccountIndex, AccountSummary, DeviceProfile, DeviceProfileVersion, QuotaData,
    TokenData,
};
use crate::modules;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::RwLock;

use super::*;

/// L1 in-memory quota cache entry (60s TTL)
#[derive(Debug, Clone)]
pub struct L1QuotaCacheEntry {
    pub quota_data: QuotaData,
    pub cached_at_sec: i64,
    pub ttl_seconds: i64,
    pub reset_time_unix: Option<i64>,
}

pub static L1_QUOTA_CACHE: Lazy<RwLock<HashMap<String, L1QuotaCacheEntry>>> =
    Lazy::new(|| RwLock::new(HashMap::new()));

/// Invalidate L1 quota cache entry for a specific account ID
pub fn invalidate_l1_quota_cache_for_account(account_id: &str) {
    if let Ok(mut cache) = L1_QUOTA_CACHE.write() {
        cache.remove(account_id);
    }
}

/// Invalidate all entries in the L1 quota cache
pub fn invalidate_all_l1_quota_cache() {
    if let Ok(mut cache) = L1_QUOTA_CACHE.write() {
        cache.clear();
    }
}

/// Helper to store quota data into L1 cache with proactive reset time extraction and adaptive TTL
pub(crate) fn store_l1_quota_cache(account_id: &str, quota_data: &QuotaData, now_sec: i64) {
    let earliest_reset_unix = quota_data
        .models
        .iter()
        .filter_map(|m| {
            if !m.reset_time.is_empty() {
                crate::modules::auto_switcher::parse_reset_time_to_unix(&m.reset_time)
            } else {
                None
            }
        })
        .chain(
            quota_data
                .quota_groups
                .as_ref()
                .into_iter()
                .flat_map(|groups| groups.iter())
                .flat_map(|g| g.buckets.iter())
                .filter_map(|b| {
                    if !b.reset_time.is_empty() {
                        crate::modules::auto_switcher::parse_reset_time_to_unix(&b.reset_time)
                    } else {
                        None
                    }
                }),
        )
        .min();

    let target_model = crate::modules::config::load_app_config()
        .map(|c| c.auto_profile_switcher.target_model)
        .unwrap_or_else(|_| "gemini-2.5-pro".to_string());

    let mut detected_percentage: Option<f64> = None;

    // 1. Direct target model match
    for m in &quota_data.models {
        if crate::modules::auto_switcher::is_model_matching_target(&m.name, &target_model) {
            detected_percentage = Some(m.percentage as f64);
            break;
        }
    }

    // 2. Primary / short-window quota bucket match
    if detected_percentage.is_none() {
        if let Some(ref groups) = quota_data.quota_groups {
            for g in groups {
                for b in &g.buckets {
                    let win = b.window.to_lowercase();
                    let bid = b.bucket_id.to_lowercase();
                    let is_short = win.contains("5h")
                        || win.contains("4h")
                        || bid.contains("5h")
                        || bid.contains("4h");
                    if is_short && (0.0..=1.0).contains(&b.remaining_fraction) {
                        detected_percentage = Some((b.remaining_fraction * 100.0).round());
                        break;
                    }
                }
                if detected_percentage.is_some() {
                    break;
                }
            }
        }
    }

    // 3. Fallback to first model percentage if available
    if detected_percentage.is_none() {
        if let Some(first) = quota_data.models.first() {
            detected_percentage = Some(first.percentage as f64);
        }
    }

    let percentage = detected_percentage.unwrap_or(100.0);
    let ttl_seconds = if percentage < 20.0 {
        120
    } else if percentage < 50.0 {
        180
    } else {
        300
    };

    if let Ok(mut cache) = L1_QUOTA_CACHE.write() {
        cache.insert(
            account_id.to_string(),
            L1QuotaCacheEntry {
                quota_data: quota_data.clone(),
                cached_at_sec: now_sec,
                ttl_seconds,
                reset_time_unix: earliest_reset_unix,
            },
        );
    }
}

/// Quota query with retry (moved from commands to modules for reuse)
pub async fn fetch_quota_with_retry(account: &mut Account) -> crate::error::AppResult<QuotaData> {
    use crate::error::AppError;
    use crate::modules::oauth;

    let now_sec = chrono::Utc::now().timestamp();

    // Check in-memory L1 cache (60s TTL)
    if let Ok(cache) = L1_QUOTA_CACHE.read() {
        if let Some(entry) = cache.get(&account.id) {
            let is_expired_by_ttl = (now_sec - entry.cached_at_sec) >= entry.ttl_seconds;
            let is_expired_by_reset = entry.reset_time_unix.map_or(false, |rts| now_sec >= rts);
            if !is_expired_by_ttl && !is_expired_by_reset {
                modules::logger::log_info(&format!(
                    "[L1QuotaCache] Serving cached quota for {} ({}) - age: {}s",
                    account.email,
                    account.id,
                    now_sec - entry.cached_at_sec
                ));
                return Ok(entry.quota_data.clone());
            }
        }
    }

    // 1. Time-based check - ensure Token is valid first
    let token = match oauth::ensure_fresh_token(&account.token, Some(&account.id)).await {
        Ok(t) => t,
        Err(e) => {
            if e.contains("invalid_grant") {
                modules::logger::log_error(&format!(
                    "Disabling account {} due to invalid_grant during token refresh (quota check)",
                    account.email
                ));
                account.disabled = true;
                account.disabled_at = Some(chrono::Utc::now().timestamp());
                account.disabled_reason = Some(format!("invalid_grant: {}", e));
                // Justification: best-effort persistence to account file and index; failure logged
                crate::error::record_ignored(
                    crate::modules::account::toggle_account_disabled(
                        &account.id,
                        true,
                        Some(&format!("invalid_grant: {}", e)),
                    )
                    .map(|_| ()),
                    "toggle_account_disabled on invalid_grant",
                );
                crate::proxy::server::trigger_account_reload(&account.id);
            }
            invalidate_l1_quota_cache_for_account(&account.id);
            return Err(AppError::OAuth(e));
        }
    };

    if token.access_token != account.token.access_token {
        modules::logger::log_info(&format!("Time-based Token refresh: {}", account.email));
        account.token = token.clone();

        // Get display name (incidental to Token refresh)
        let name = if account.name.is_none()
            || account.name.as_ref().map_or(false, |n| n.trim().is_empty())
        {
            match oauth::get_user_info(&token.access_token, Some(&account.id)).await {
                Ok(user_info) => user_info.get_display_name(),
                Err(_) => None,
            }
        } else {
            account.name.clone()
        };

        account.name = name.clone();
        upsert_account(account.email.clone(), name, token.clone()).map_err(AppError::Account)?;
    }

    // 0. Supplement display name (if missing or upper step failed)
    if account.name.is_none() || account.name.as_ref().map_or(false, |n| n.trim().is_empty()) {
        modules::logger::log_info(&format!(
            "Account {} missing display name, attempting to fetch...",
            account.email
        ));
        // Use updated token
        match oauth::get_user_info(&account.token.access_token, Some(&account.id)).await {
            Ok(user_info) => {
                let display_name = user_info.get_display_name();
                modules::logger::log_info(&format!(
                    "Successfully fetched display name: {:?}",
                    display_name
                ));
                account.name = display_name.clone();
                // Save immediately
                if let Err(e) =
                    upsert_account(account.email.clone(), display_name, account.token.clone())
                {
                    modules::logger::log_warn(&format!("Failed to save display name: {}", e));
                }
            }
            Err(e) => {
                modules::logger::log_warn(&format!("Failed to fetch display name: {}", e));
            }
        }
    }

    // 2. Attempt query (pass cached project_id if available to avoid unnecessary loadCodeAssist)
    let result: crate::error::AppResult<(QuotaData, Option<String>)> =
        modules::fetch_quota_with_cache(
            &account.token.access_token,
            &account.email,
            account.token.project_id.as_deref(),
            Some(&account.id),
            account.quota.as_ref(),
        )
        .await;

    // Capture potentially updated project_id and save
    if let Ok((ref _q, ref project_id)) = result {
        if project_id.is_some() && *project_id != account.token.project_id {
            modules::logger::log_info(&format!(
                "Detected project_id update ({}), saving...",
                account.email
            ));
            account.token.project_id = project_id.clone();
            if let Err(e) = upsert_account(
                account.email.clone(),
                account.name.clone(),
                account.token.clone(),
            ) {
                modules::logger::log_warn(&format!("Failed to sync project_id: {}", e));
            }
        }
    }

    // 3. Handle 401 error
    if let Err(AppError::Network(_, Some(401))) = result {
        modules::logger::log_warn(&format!(
            "401 Unauthorized for {}, forcing refresh...",
            account.email
        ));

        // Force refresh
        let token_res = match oauth::refresh_access_token_with_client(
            &account.token.refresh_token,
            Some(&account.id),
            account.token.oauth_client_key.as_deref(),
        )
        .await
        {
            Ok(t) => t,
            Err(e) => {
                if e.contains("invalid_grant") {
                    modules::logger::log_error(&format!(
                        "Disabling account {} due to invalid_grant during forced refresh (quota check)",
                        account.email
                    ));
                    account.disabled = true;
                    account.disabled_at = Some(chrono::Utc::now().timestamp());
                    account.disabled_reason = Some(format!("invalid_grant: {}", e));
                    // Justification: best-effort persistence to account file and index; failure logged
                    crate::error::record_ignored(
                        crate::modules::account::toggle_account_disabled(
                            &account.id,
                            true,
                            Some(&format!("invalid_grant: {}", e)),
                        )
                        .map(|_| ()),
                        "toggle_account_disabled on forced invalid_grant",
                    );
                    crate::proxy::server::trigger_account_reload(&account.id);
                }
                invalidate_l1_quota_cache_for_account(&account.id);
                return Err(AppError::OAuth(e));
            }
        };

        let new_token = TokenData::new(
            token_res.access_token.clone(),
            account.token.refresh_token.clone(),
            token_res.expires_in,
            account.token.email.clone(),
            account.token.project_id.clone(), // Keep original project_id
            None,                             // Add None as session_id
            account.token.is_gcp_tos,
            token_res.id_token.clone(),
        )
        .with_oauth_client_key(
            token_res
                .oauth_client_key
                .clone()
                .or_else(|| account.token.oauth_client_key.clone()),
        );

        // Re-fetch display name
        let name = if account.name.is_none()
            || account.name.as_ref().is_some_and(|n| n.trim().is_empty())
        {
            match oauth::get_user_info(&token_res.access_token, Some(&account.id)).await {
                Ok(user_info) => user_info.get_display_name(),
                Err(_) => None,
            }
        } else {
            account.name.clone()
        };

        account.token = new_token.clone();
        account.name = name.clone();
        upsert_account(account.email.clone(), name, new_token.clone())
            .map_err(AppError::Account)?;

        // Retry query (pass cached project_id if available)
        let retry_result: crate::error::AppResult<(QuotaData, Option<String>)> =
            modules::fetch_quota_with_cache(
                &new_token.access_token,
                &account.email,
                account.token.project_id.as_deref(),
                Some(&account.id),
                account.quota.as_ref(),
            )
            .await;

        // Also handle project_id saving during retry
        if let Ok((ref _q, ref project_id)) = retry_result {
            if project_id.is_some() && *project_id != account.token.project_id {
                modules::logger::log_info(&format!(
                    "Detected update of project_id after retry ({}), saving...",
                    account.email
                ));
                account.token.project_id = project_id.clone();
                // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                crate::error::record_ignored(
                    upsert_account(
                        account.email.clone(),
                        account.name.clone(),
                        account.token.clone(),
                    ),
                    "upsert_account",
                );
            }
        }

        if let Err(AppError::Network(_, Some(403))) = retry_result {
            invalidate_l1_quota_cache_for_account(&account.id);
            let mut q = QuotaData::new();
            q.is_forbidden = true;
            return Ok(q);
        }

        match retry_result {
            Ok((q, _)) => {
                if !q.is_forbidden {
                    clear_validation_blocked(account);
                }
                store_l1_quota_cache(&account.id, &q, now_sec);
                return Ok(q);
            }
            Err(e) => {
                invalidate_l1_quota_cache_for_account(&account.id);
                if is_validation_required_error(&e) {
                    mark_validation_blocked(account, &e.to_string());
                }
                if let Some(cached) = recover_cached_quota_on_rate_limit(account, &e) {
                    mark_validation_blocked(account, &format_rate_limit_block_reason(&e));
                    modules::logger::log_warn(&format!(
                        "Quota API rate-limited for {}, using cached model list as fallback",
                        account.email
                    ));
                    return Ok(cached);
                }
                return Err(e);
            }
        }
    }

    // fetch_quota already handles 403, with additional local fallback/validation handling.
    match result {
        Ok((q, _)) => {
            if !q.is_forbidden {
                clear_validation_blocked(account);
            }
            store_l1_quota_cache(&account.id, &q, now_sec);
            Ok(q)
        }
        Err(e) => {
            invalidate_l1_quota_cache_for_account(&account.id);
            if is_validation_required_error(&e) {
                mark_validation_blocked(account, &e.to_string());
            }
            if let Some(cached) = recover_cached_quota_on_rate_limit(account, &e) {
                mark_validation_blocked(account, &format_rate_limit_block_reason(&e));
                modules::logger::log_warn(&format!(
                    "Quota API rate-limited for {}, using cached model list as fallback",
                    account.email
                ));
                return Ok(cached);
            }
            Err(e)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::quota::ModelQuota;

    #[test]
    fn test_l1_quota_cache_roundtrip_and_invalidation() {
        let now_sec = 1790090000;
        let account_id = "test_l1_acc";
        let mut quota_data = QuotaData::new();
        quota_data.subscription_tier = Some("pro".to_string());

        // 1. Store into cache with no models (default 100% -> 300s TTL)
        store_l1_quota_cache(account_id, &quota_data, now_sec);

        // 2. Read from cache
        {
            let cache = L1_QUOTA_CACHE.read().unwrap();
            let entry = cache.get(account_id);
            assert!(entry.is_some());
            let entry = entry.unwrap();
            assert_eq!(entry.quota_data.subscription_tier.as_deref(), Some("pro"));
            assert_eq!(entry.ttl_seconds, 300);
        }

        // Test adaptive TTL with low quota (< 20% -> 120s TTL)
        let mut low_quota = QuotaData::new();
        low_quota.models.push(ModelQuota {
            name: "gemini-2.5-pro".to_string(),
            percentage: 15,
            reset_time: String::new(),
            display_name: None,
            supports_images: None,
            supports_thinking: None,
            thinking_budget: None,
            recommended: None,
            max_tokens: None,
            max_output_tokens: None,
            supported_mime_types: None,
        });
        store_l1_quota_cache("low_acc", &low_quota, now_sec);
        {
            let cache = L1_QUOTA_CACHE.read().unwrap();
            assert_eq!(cache.get("low_acc").unwrap().ttl_seconds, 120);
        }

        // Test adaptive TTL with medium quota (< 50% -> 180s TTL)
        let mut med_quota = QuotaData::new();
        med_quota.models.push(ModelQuota {
            name: "gemini-2.5-pro".to_string(),
            percentage: 45,
            reset_time: String::new(),
            display_name: None,
            supports_images: None,
            supports_thinking: None,
            thinking_budget: None,
            recommended: None,
            max_tokens: None,
            max_output_tokens: None,
            supported_mime_types: None,
        });
        store_l1_quota_cache("med_acc", &med_quota, now_sec);
        {
            let cache = L1_QUOTA_CACHE.read().unwrap();
            assert_eq!(cache.get("med_acc").unwrap().ttl_seconds, 180);
        }

        // 3. Invalidate
        invalidate_l1_quota_cache_for_account(account_id);
        {
            let cache = L1_QUOTA_CACHE.read().unwrap();
            assert!(cache.get(account_id).is_none());
        }
    }
}
