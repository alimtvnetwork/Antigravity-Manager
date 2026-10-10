pub use crate::models::{
    Account, AccountIndex, AccountSummary, DeviceProfile, DeviceProfileVersion, QuotaData,
    TokenData,
};
use crate::modules;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::*;

#[derive(Serialize)]
pub struct RefreshStats {
    pub total: usize,
    pub success: usize,
    pub failed: usize,
    pub details: Vec<String>,
}

/// Core logic to batch refresh all account quotas (decoupled from Tauri status)
pub async fn refresh_all_quotas_logic() -> Result<RefreshStats, String> {
    use futures::future::join_all;
    use std::sync::Arc;
    use tokio::sync::Semaphore;

    const MAX_CONCURRENT: usize = 5;
    let start = std::time::Instant::now();

    crate::modules::logger::log_info(&format!(
        "Starting batch refresh of all account quotas (Concurrent mode, max: {})",
        MAX_CONCURRENT
    ));
    let accounts = list_accounts()?;

    let semaphore = Arc::new(Semaphore::new(MAX_CONCURRENT));

    let tasks: Vec<_> = accounts
        .into_iter()
        .filter(|account| {
            // [MOD] Now we allow refreshing disabled and proxy_disabled accounts
            // to support forced re-sync from UI.
            // Only strictly skip forbidden accounts if necessary, but even those
            // might want a retry to see if they are unbanned.
            if let Some(ref q) = account.quota {
                if q.is_forbidden {
                    crate::modules::logger::log_info(&format!(
                        "  - Skipping {} (Forbidden)",
                        account.email
                    ));
                    return false;
                }
            }
            true
        })
        .map(|mut account| {
            let email = account.email.clone();
            let account_id = account.id.clone();
            let permit = semaphore.clone();
            async move {
                let _guard = permit.acquire().await.unwrap();
                crate::modules::logger::log_info(&format!("  - Processing {}", email));
                match fetch_quota_with_retry(&mut account).await {
                    Ok(quota) => {
                        if let Err(e) = update_account_quota(&account_id, quota) {
                            let msg = format!("Account {}: Save quota failed - {}", email, e);
                            crate::modules::logger::log_error(&msg);
                            Err(msg)
                        } else {
                            crate::modules::logger::log_info(&format!("    Success {}", email));
                            Ok(())
                        }
                    }
                    Err(e) => {
                        let msg = format!("Account {}: Fetch quota failed - {}", email, e);
                        crate::modules::logger::log_error(&msg);
                        Err(msg)
                    }
                }
            }
        })
        .collect();

    let total = tasks.len();
    let results = join_all(tasks).await;

    let mut success = 0;
    let mut failed = 0;
    let mut details = Vec::new();

    for result in results {
        match result {
            Ok(()) => success += 1,
            Err(msg) => {
                failed += 1;
                details.push(msg);
            }
        }
    }

    let elapsed = start.elapsed();
    crate::modules::logger::log_info(&format!(
        "Batch refresh completed: {} success, {} failed, took: {}ms",
        success,
        failed,
        elapsed.as_millis()
    ));

    // After quota refresh, immediately check and trigger warmup for weekly recovered models
    tokio::spawn(async {
        check_and_trigger_warmup_for_recovered_models().await;
    });

    Ok(RefreshStats {
        total,
        success,
        failed,
        details,
    })
}

/// Check and trigger warmup for models that have recovered to 100%
/// Called automatically after quota refresh to enable immediate warmup
pub async fn check_and_trigger_warmup_for_recovered_models() {
    let accounts = match list_accounts() {
        Ok(acc) => acc,
        Err(_) => return,
    };

    // Load config to check if scheduled warmup is enabled
    let app_config = match crate::modules::config::load_app_config() {
        Ok(cfg) => cfg,
        Err(_) => return,
    };

    if !app_config.scheduled_warmup.enabled {
        return;
    }

    crate::modules::logger::log_info(&format!(
        "[Warmup] Checking {} accounts for recovered models after quota refresh...",
        accounts.len()
    ));

    for account in accounts {
        // Skip disabled accounts
        if account.disabled || account.proxy_disabled {
            continue;
        }

        // Trigger warmup check for this account
        crate::modules::scheduler::trigger_warmup_for_account(&account).await;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTierStats {
    pub total: usize,
    pub updated: usize,
    pub still_unknown: usize,
    pub failed: usize,
    pub details: Vec<String>,
}

/// Check if an account lacks an identified subscription tier (ULTRA, PRO, FREE)
pub fn is_tier_missing_or_unknown(account: &Account) -> bool {
    let Some(ref q) = account.quota else {
        return true;
    };
    let Some(ref raw) = q.subscription_tier else {
        return true;
    };
    let normalized = crate::models::quota::normalize_subscription_tier(raw);
    !crate::models::quota::is_known_tier(&normalized)
}

/// Refresh subscription tiers for accounts that currently lack an identified tier.
/// Uses bounded concurrency and returns counts (updated, still_unknown, failed).
pub async fn refresh_missing_tiers(
    limit_concurrency: Option<usize>,
) -> Result<RefreshTierStats, String> {
    refresh_missing_tiers_with_options(false, limit_concurrency).await
}

/// Refresh subscription tiers with option to refresh all accounts or only missing tiers.
pub async fn refresh_missing_tiers_with_options(
    refresh_all: bool,
    limit_concurrency: Option<usize>,
) -> Result<RefreshTierStats, String> {
    use futures::future::join_all;
    use std::sync::Arc;
    use tokio::sync::Semaphore;

    let concurrency = limit_concurrency.unwrap_or(3).clamp(1, 10);
    let accounts = list_accounts()?;

    let target_accounts: Vec<Account> = accounts
        .into_iter()
        .filter(|acc| {
            if let Some(ref q) = acc.quota {
                if q.is_forbidden {
                    return false;
                }
            }
            if refresh_all {
                true
            } else {
                is_tier_missing_or_unknown(acc)
            }
        })
        .collect();

    let total = target_accounts.len();
    if total == 0 {
        return Ok(RefreshTierStats {
            total: 0,
            updated: 0,
            still_unknown: 0,
            failed: 0,
            details: vec!["No accounts require tier refresh".to_string()],
        });
    }

    crate::modules::logger::log_info(&format!(
        "Starting tier refresh for {} accounts (concurrency: {}, refresh_all: {})",
        total, concurrency, refresh_all
    ));

    let semaphore = Arc::new(Semaphore::new(concurrency));
    let tasks: Vec<_> = target_accounts
        .into_iter()
        .map(|mut acc| {
            let email = acc.email.clone();
            let account_id = acc.id.clone();
            let permit = semaphore.clone();
            async move {
                let _guard = permit.acquire().await.unwrap();
                if refresh_all {
                    // Reset fetched_at timestamp to force cache invalidation in fetch_quota_with_cache
                    if let Some(ref mut q) = acc.quota {
                        q.subscription_tier_fetched_at = Some(0);
                    }
                }
                match fetch_quota_with_retry(&mut acc).await {
                    Ok(quota) => {
                        let tier_candidate = quota.subscription_tier.clone();
                        let has_known_tier = tier_candidate
                            .as_deref()
                            .map(|t| {
                                let norm = crate::models::quota::normalize_subscription_tier(t);
                                crate::models::quota::is_known_tier(&norm)
                            })
                            .unwrap_or(false);

                        if let Err(e) = update_account_quota(&account_id, quota) {
                            let msg = format!("{}: failed to save quota: {}", email, e);
                            crate::modules::logger::log_error(&msg);
                            Err(msg)
                        } else if has_known_tier {
                            let tier_name = tier_candidate.unwrap_or_default();
                            crate::modules::logger::log_info(&format!(
                                "Tier updated for {}: {}",
                                email, tier_name
                            ));
                            Ok((true, format!("{}: tier resolved as {}", email, tier_name)))
                        } else {
                            crate::modules::logger::log_info(&format!(
                                "Tier still unknown for {}",
                                email
                            ));
                            Ok((false, format!("{}: tier remains unknown", email)))
                        }
                    }
                    Err(e) => {
                        let msg = format!("{}: {}", email, e);
                        crate::modules::logger::log_error(&msg);
                        Err(msg)
                    }
                }
            }
        })
        .collect();

    let results = join_all(tasks).await;

    let mut updated = 0;
    let mut still_unknown = 0;
    let mut failed = 0;
    let mut details = Vec::new();

    for res in results {
        match res {
            Ok((is_updated, msg)) => {
                if is_updated {
                    updated += 1;
                } else {
                    still_unknown += 1;
                }
                details.push(msg);
            }
            Err(msg) => {
                failed += 1;
                details.push(msg);
            }
        }
    }

    crate::modules::log_bridge::emit_accounts_refreshed();

    Ok(RefreshTierStats {
        total,
        updated,
        still_unknown,
        failed,
        details,
    })
}
