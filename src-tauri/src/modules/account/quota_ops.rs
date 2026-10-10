pub use crate::models::{
    Account, AccountIndex, AccountSummary, DeviceProfile, DeviceProfileVersion, QuotaData,
    TokenData,
};
use crate::modules;
use std::collections::HashMap;

use super::*;

/// Update account quota
pub fn update_account_quota(account_id: &str, quota: QuotaData) -> Result<(), String> {
    let _account_write = lock_account_file_updates()?;
    let mut account = load_account(account_id)?;
    account.update_quota(quota);

    // --- Quota protection logic start ---
    if let Ok(config) = crate::modules::config::load_app_config() {
        if config.quota_protection.enabled {
            if let Some(ref q) = account.quota {
                let threshold = config.quota_protection.threshold_percentage as i32;

                let mut group_max_percentage: HashMap<String, i32> = HashMap::new();

                for model in &q.models {
                    if let Some(std_id) =
                        crate::proxy::common::model_mapping::normalize_to_standard_id(&model.name)
                    {
                        let entry = group_max_percentage.entry(std_id).or_insert(-1);
                        if model.percentage > *entry {
                            *entry = model.percentage;
                        }
                    }
                }

                for std_id in &config.quota_protection.monitored_models {
                    let lookup_key =
                        crate::proxy::common::model_mapping::normalize_to_standard_id(std_id)
                            .unwrap_or_else(|| std_id.clone());
                    let max_pct = group_max_percentage
                        .get(&lookup_key)
                        .cloned()
                        .unwrap_or(100);

                    if max_pct < threshold {
                        if !account.protected_models.contains(&lookup_key) {
                            crate::modules::logger::log_info(&format!(
                                "[Quota] Triggering model protection: {} (Group: {} Max: {}% < Thres: {}%)",
                                account.email, lookup_key, max_pct, threshold
                            ));
                            account.protected_models.insert(lookup_key.clone());
                        }
                    } else {
                        if account.protected_models.contains(&lookup_key) {
                            crate::modules::logger::log_info(&format!(
                                "[Quota] Model protection recovered: {} (Group: {} Max: {}% >= Thres: {}%)",
                                account.email, lookup_key, max_pct, threshold
                            ));
                            account.protected_models.remove(&lookup_key);
                        }
                    }
                }

                // [Compatibility] Migrate from account-level to model-level protection if previously disabled for quota
                if account.proxy_disabled
                    && account
                        .proxy_disabled_reason
                        .as_ref()
                        .map_or(false, |r| r == "quota_protection")
                {
                    crate::modules::logger::log_info(&format!(
                        "[Quota] Migrating account {} from account-level to model-level protection",
                        account.email
                    ));
                    account.proxy_disabled = false;
                    account.proxy_disabled_reason = None;
                    account.proxy_disabled_at = None;
                }
            }
        } else {
            // [FIX] 当配额保护在全局关闭时，清空受保护模型列表，避免遗留历史锁
            if !account.protected_models.is_empty() {
                crate::modules::logger::log_info(&format!(
                    "[Quota] Quota protection disabled globally, clearing protected models for {}",
                    account.email
                ));
                account.protected_models.clear();
            }
        }
    }
    // --- Quota protection logic end ---

    // Quota snapshots may recover before an explicit long image lock expires. Other live
    // records retain the baseline percentage-based cleanup behavior.
    if let Some(ref q) = account.quota {
        let now = chrono::Utc::now().timestamp();
        account.live_limited_models.retain(|model_key, status| {
            if crate::proxy::rate_limit::is_active_persisted_long_image_limit(
                model_key, status, now,
            ) {
                return true;
            }
            let recovered = q.models.iter().any(|model| {
                let is_matching = model.name == *model_key
                    || crate::proxy::common::model_mapping::normalize_to_standard_id(&model.name)
                        .is_some_and(|standard| standard == *model_key);
                is_matching && model.percentage > 0
            });
            !recovered
        });
    }

    // Save account first
    save_account(&account)?;

    // [FIX] 同时更新索引文件中的摘要信息，确保列表页图标即时刷新
    {
        if let Ok(mut index) = load_account_index() {
            if let Some(summary) = index.accounts.iter_mut().find(|a| a.id == account_id) {
                summary.protected_models = account.protected_models.clone();
                // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                crate::error::record_ignored(save_account_index(&index), "save_account_index");
            }
        }
    }

    // [FIX] Trigger TokenManager account reload signal
    // This ensures in-memory protected_models are updated
    crate::proxy::server::trigger_account_reload(account_id);

    Ok(())
}

/// Toggle proxy disabled status for an account
pub fn toggle_proxy_status(
    account_id: &str,
    enable: bool,
    reason: Option<&str>,
) -> Result<(), String> {
    let _lock = ACCOUNT_INDEX_LOCK
        .lock()
        .map_err(|e| format!("failed_to_acquire_lock: {}", e))?;

    let mut account = load_account(account_id)?;

    account.proxy_disabled = !enable;
    account.proxy_disabled_reason = if !enable {
        reason.map(|s| s.to_string())
    } else {
        None
    };
    account.proxy_disabled_at = if !enable {
        Some(chrono::Utc::now().timestamp())
    } else {
        None
    };

    save_account(&account)?;

    // Also update index summary
    let mut index = load_account_index()?;
    if let Some(summary) = index.accounts.iter_mut().find(|a| a.id == account_id) {
        summary.proxy_disabled = !enable;
        save_account_index(&index)?;
    }

    Ok(())
}

/// Find account ID by email (from index)
pub fn find_account_id_by_email(email: &str) -> Option<String> {
    load_account_index()
        .ok()?
        .accounts
        .into_iter()
        .find(|a| a.email == email)
        .map(|a| a.id)
}

pub fn mark_account_forbidden(account_id: &str, reason: &str) -> Result<(), String> {
    let _lock = ACCOUNT_INDEX_LOCK
        .lock()
        .map_err(|e| format!("failed_to_acquire_lock: {}", e))?;

    let mut account = load_account(account_id)?;

    // 1. Update quota status
    if let Some(ref mut q) = account.quota {
        q.is_forbidden = true;
        q.forbidden_reason = Some(reason.to_string());
    } else {
        let mut q = crate::models::QuotaData::new();
        q.is_forbidden = true;
        q.forbidden_reason = Some(reason.to_string());
        account.quota = Some(q);
    }

    // 2. Disable proxy for this account
    account.proxy_disabled = true;
    account.proxy_disabled_reason = Some(format!("Forbidden (403): {}", reason));
    account.proxy_disabled_at = Some(chrono::Utc::now().timestamp());

    save_account(&account)?;

    // 3. Update index summary
    let mut index = load_account_index()?;
    if let Some(summary) = index.accounts.iter_mut().find(|a| a.id == account_id) {
        summary.proxy_disabled = true;
        save_account_index(&index)?;
    }

    // 4. Notify frontend to refresh account list
    crate::modules::log_bridge::emit_accounts_refreshed();

    Ok(())
}

/// Export accounts by IDs (for backup/migration)
pub fn export_accounts_by_ids(
    account_ids: &[String],
) -> Result<crate::models::AccountExportResponse, String> {
    use crate::models::{AccountExportItem, AccountExportResponse};

    let accounts = list_accounts()?;

    let export_items: Vec<AccountExportItem> = accounts
        .into_iter()
        .filter(|acc| account_ids.contains(&acc.id))
        .map(|acc| AccountExportItem {
            email: acc.email,
            refresh_token: acc.token.refresh_token,
        })
        .collect();

    Ok(AccountExportResponse {
        accounts: export_items,
    })
}

/// Export all accounts' refresh_tokens (legacy, kept for compatibility)
#[allow(dead_code)]
pub fn export_accounts() -> Result<Vec<(String, String)>, String> {
    let accounts = list_accounts()?;
    let mut exports = Vec::new();

    for account in accounts {
        exports.push((account.email, account.token.refresh_token));
    }

    Ok(exports)
}
