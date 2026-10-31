//! Token pre-filtering: ability filtering and sorting, fixed account (FIX #820) priority.

use super::types::OnDiskAccountState;
use super::ProxyToken;
use super::TokenManager;
use std::sync::Arc;

impl TokenManager {
    pub(crate) fn sort_candidates_by_capability(
        mut tokens_snapshot: Vec<ProxyToken>,
        target_model: &str,
    ) -> Result<Vec<ProxyToken>, String> {
        // [NEW] 1. Dynamic capability filter (Capability Filter)

        // Define constants
        const RESET_TIME_THRESHOLD_SECS: i64 = 600; // 10 minute threshold

        // The normalized target model is named Standard ID
        let normalized_target =
            crate::proxy::common::model_mapping::normalize_to_standard_id(target_model)
                .unwrap_or_else(|| target_model.to_string());

        // Only keep accounts that explicitly have a quota for that model
        // This step ensures that "there is a model before you can enter polling", especially for high-end models such as Opus 4.6
        let candidate_count_before = tokens_snapshot.len();

        // It is assumed here that all supported models will be present in model_quotas
        // If the quota information returned by the API is incomplete, it may lead to accidental killing, but for the sake of strictness, we perform this filtering
        tokens_snapshot.retain(|t| t.model_quotas.contains_key(&normalized_target));

        if tokens_snapshot.is_empty() {
            if candidate_count_before > 0 {
                // If there were accounts before filtering but disappeared after filtering, it means that all accounts do not have quotas for this model.
                tracing::warn!(
                    "No accounts have satisfied quota for model: {}",
                    normalized_target
                );
                return Err(format!(
                    "No accounts available with quota for model: {}",
                    normalized_target
                ));
            }
            return Err("Token pool is empty".to_string());
        }

        tokens_snapshot.sort_by(|a, b| {
            let priority_cmp = a.priority.cmp(&b.priority);
            if priority_cmp != std::cmp::Ordering::Equal {
                return priority_cmp;
            }

            // Priority 0: Strict subscription ranking (ULTRA > PRO > FREE)
            // User requirement: Polling should follow Ultra -> Pro -> Free
            // Now that the accounts that do not support the model have been filtered out, the rest are supported.
            // At this time we prioritize premium subscriptions
            // Use models::quota::tier_priority in a unified manner to ensure that the same set of keyword tables is used for UI/quota resolution.
            // Unknown levels will always be treated as FREE (no return to 3), otherwise "UI displays FREE,
            // The scheduler ranks it in the invisible position after FREE.
            let tier_priority =
                |tier: &Option<String>| crate::models::quota::tier_priority(tier.as_deref());

            let tier_cmp =
                tier_priority(&a.subscription_tier).cmp(&tier_priority(&b.subscription_tier));
            if tier_cmp != std::cmp::Ordering::Equal {
                return tier_cmp;
            }

            // Priority 1: Target model's quota (higher is better) -> Protect low-quota accounts
            // After filtering, the key must exist
            let quota_a = a.model_quotas.get(&normalized_target).copied().unwrap_or(0);
            let quota_b = b.model_quotas.get(&normalized_target).copied().unwrap_or(0);

            let quota_cmp = quota_b.cmp(&quota_a);
            if quota_cmp != std::cmp::Ordering::Equal {
                return quota_cmp;
            }

            // Priority 2: Health score (higher is better)
            let health_cmp = b
                .health_score
                .partial_cmp(&a.health_score)
                .unwrap_or(std::cmp::Ordering::Equal);
            if health_cmp != std::cmp::Ordering::Equal {
                return health_cmp;
            }

            // Priority 3: Reset time (earlier is better, but only if diff > 10 min)
            let reset_a = a.reset_time.unwrap_or(i64::MAX);
            let reset_b = b.reset_time.unwrap_or(i64::MAX);
            if (reset_a - reset_b).abs() >= RESET_TIME_THRESHOLD_SECS {
                reset_a.cmp(&reset_b)
            } else {
                std::cmp::Ordering::Equal
            }
        });

        // [Debug Log] Print the sorted account order (display the quota of the target model)
        tracing::debug!(
            "🔄 [Token Rotation] target={} Accounts: {:?}",
            normalized_target,
            tokens_snapshot
                .iter()
                .map(|t| format!(
                    "{}(quota={}%, reset={:?}, health={:.2})",
                    t.email,
                    t.model_quotas.get(&normalized_target).copied().unwrap_or(0),
                    t.reset_time.map(|ts| {
                        let now = chrono::Utc::now().timestamp();
                        let diff_secs = ts - now;
                        if diff_secs > 0 {
                            format!("{}m", diff_secs / 60)
                        } else {
                            "now".to_string()
                        }
                    }),
                    t.health_score
                ))
                .collect::<Vec<_>>()
        );
        Ok(tokens_snapshot)
    }

    /// [FIX #820] Fixed account mode: Prioritize the use of designated accounts.
    /// Returning `Some` means the caller can return the result directly; returning `None` means falling back to regular polling.
    pub(crate) async fn try_preferred_account(
        &self,
        tokens_snapshot: &mut Vec<ProxyToken>,
        total: &mut usize,
        pref_id: &str,
        target_model: &str,
        quota_protection_enabled: bool,
    ) -> Option<Result<(String, String, String, String, u64), String>> {
        // Find priority account
        if let Some(preferred_token) = tokens_snapshot
            .iter()
            .find(|t| &t.account_id == pref_id)
            .cloned()
        {
            // Check whether the account is available (not limited, not protected by quota)
            match Self::get_account_state_on_disk(&preferred_token.account_path).await {
                OnDiskAccountState::Disabled => {
                    tracing::warn!(
                            "🔒 [FIX #820] Preferred account {} is disabled on disk, purging and falling back",
                            preferred_token.email
                        );
                    self.remove_account(&preferred_token.account_id);
                    tokens_snapshot.retain(|t| t.account_id != preferred_token.account_id);
                    *total = tokens_snapshot.len();

                    {
                        let mut preferred = self.preferred_account_id.write().await;
                        if preferred.as_deref() == Some(pref_id) {
                            *preferred = None;
                        }
                    }

                    if *total == 0 {
                        return Some(Err("Token pool is empty".to_string()));
                    }
                }
                OnDiskAccountState::Unknown => {
                    tracing::warn!(
                            "🔒 [FIX #820] Preferred account {} state on disk is unavailable, falling back",
                            preferred_token.email
                        );
                    // Don't purge on transient read/parse failures; just skip this token for this request.
                    tokens_snapshot.retain(|t| t.account_id != preferred_token.account_id);
                    *total = tokens_snapshot.len();
                    if *total == 0 {
                        return Some(Err("Token pool is empty".to_string()));
                    }
                }
                OnDiskAccountState::Enabled => {
                    let normalized_target =
                        crate::proxy::common::model_mapping::normalize_to_standard_id(target_model)
                            .unwrap_or_else(|| target_model.to_string());

                    let is_rate_limited = self
                        .is_rate_limited(&preferred_token.account_id, Some(&normalized_target))
                        .await;
                    let is_quota_protected = quota_protection_enabled
                        && preferred_token
                            .protected_models
                            .contains(&normalized_target);

                    if !is_rate_limited && !is_quota_protected {
                        tracing::info!(
                            "🔒 [FIX #820] Using preferred account: {} (fixed mode)",
                            preferred_token.email
                        );

                        // Use the priority account directly and skip the polling logic
                        let mut token = preferred_token.clone();

                        // [NEW] Check whether the token has expired (adjust the refresh timing to align with the official: 90s grace period)
                        let now = chrono::Utc::now().timestamp();
                        if now >= token.timestamp - 90 {
                            // [NEW] Double-Checked Locking
                            // 1. Obtain (or create) a refresh lock exclusive to the account
                            let refresh_mu = self
                                .refresh_locks
                                .entry(token.account_id.clone())
                                .or_insert_with(|| Arc::new(tokio::sync::Mutex::new(())))
                                .clone();

                            // 2. Try to acquire the lock
                            let _guard = refresh_mu.lock().await;

                            // 3. Check the latest status of this account again (it may have been refreshed by other concurrent requests)
                            let latest_token_opt =
                                self.tokens.get(&token.account_id).map(|r| r.clone());
                            if let Some(latest) = latest_token_opt {
                                if now < latest.timestamp - 90 {
                                    // It has been refreshed by others, synchronize the latest data and skip the refresh action
                                    token = latest.clone();
                                    tracing::debug!(
                                        "Account {} has been refreshed by a concurrent thread, skipping repeated refreshes",
                                        token.email
                                    );
                                } else {
                                    // Really needs to be refreshed
                                    tracing::debug!(
                                        "The token of account {} is about to expire ( {} s) and is being refreshed...",
                                        token.email,
                                        token.timestamp - now
                                    );
                                    match crate::modules::oauth::refresh_access_token(
                                        &token.refresh_token,
                                        Some(&token.account_id),
                                    )
                                    .await
                                    {
                                        Ok(token_response) => {
                                            token.access_token =
                                                token_response.access_token.clone();
                                            token.expires_in = token_response.expires_in;
                                            token.timestamp = now + token_response.expires_in;

                                            if let Some(mut entry) =
                                                self.tokens.get_mut(&token.account_id)
                                            {
                                                entry.access_token = token.access_token.clone();
                                                entry.expires_in = token.expires_in;
                                                entry.timestamp = token.timestamp;
                                            }
                                            // [FIX] Backend disk writing operations: avoid blocking the 5s timeout window of get_token
                                            // The memory has been updated, and the disk persistence is spawned to the blocking thread pool.
                                            {
                                                let write_path = token.account_path.clone();
                                                let access_token =
                                                    token_response.access_token.clone();
                                                let expires_in = token_response.expires_in;
                                                let id_token = token_response.id_token.clone();
                                                let new_rt = token_response.refresh_token.clone();
                                                let write_ts = now + token_response.expires_in;
                                                tokio::task::spawn_blocking(move || {
                                                    let Ok(_lk) = crate::modules::account::lock_account_file_updates() else { return; };
                                                    let Ok(raw) =
                                                        std::fs::read_to_string(&write_path)
                                                    else {
                                                        return;
                                                    };
                                                    let Ok(mut val) =
                                                        serde_json::from_str::<serde_json::Value>(
                                                            &raw,
                                                        )
                                                    else {
                                                        return;
                                                    };
                                                    val["token"]["access_token"] =
                                                        access_token.into();
                                                    val["token"]["expires_in"] = expires_in.into();
                                                    val["token"]["expiry_timestamp"] =
                                                        write_ts.into();
                                                    if let Some(it) = id_token {
                                                        val["token"]["id_token"] = it.into();
                                                    }
                                                    if let Some(rt) = new_rt {
                                                        val["token"]["refresh_token"] = rt.into();
                                                    }
                                                    if let Ok(s) =
                                                        serde_json::to_string_pretty(&val)
                                                    {
                                                        // Justification: best-effort file write; failure is logged and surfaces on the next read
                                                        crate::error::record_ignored(
                                                            std::fs::write(&write_path, s),
                                                            "fs::write",
                                                        );
                                                    }
                                                });
                                            }
                                        }
                                        Err(e) => {
                                            tracing::warn!(
                                                "Preferred account token refresh failed: {}",
                                                e
                                            );
                                            // Continue to use the old token, causing subsequent logic processing to fail.
                                        }
                                    }
                                }
                            }
                        }

                        // Make sure there is project_id (filter empty strings to trigger re-fetch)
                        let project_id = if let Some(pid) = &token.project_id {
                            if pid.is_empty() {
                                None
                            } else {
                                Some(pid.clone())
                            }
                        } else {
                            None
                        };
                        let project_id = if let Some(pid) = project_id {
                            pid
                        } else {
                            match crate::proxy::project_resolver::fetch_project_id(
                                &token.access_token,
                            )
                            .await
                            {
                                Ok(pid) => {
                                    if let Some(mut entry) = self.tokens.get_mut(&token.account_id)
                                    {
                                        entry.project_id = Some(pid.clone());
                                    }
                                    // [FIX] Disk writing background: project_id has been written to memory, disk persistence does not block the hot path
                                    {
                                        let write_path = token.account_path.clone();
                                        let pid_clone = pid.clone();
                                        tokio::task::spawn_blocking(move || {
                                            let Ok(_lk) =
                                                crate::modules::account::lock_account_file_updates(
                                                )
                                            else {
                                                return;
                                            };
                                            let Ok(raw) = std::fs::read_to_string(&write_path)
                                            else {
                                                return;
                                            };
                                            let Ok(mut val) =
                                                serde_json::from_str::<serde_json::Value>(&raw)
                                            else {
                                                return;
                                            };
                                            val["token"]["project_id"] = pid_clone.into();
                                            if let Ok(s) = serde_json::to_string_pretty(&val) {
                                                // Justification: best-effort file write; failure is logged and surfaces on the next read
                                                crate::error::record_ignored(
                                                    std::fs::write(&write_path, s),
                                                    "fs::write",
                                                );
                                            }
                                        });
                                    }
                                    pid
                                }
                                Err(_) => "bamboo-precept-lgxtn".to_string(), // fallback
                            }
                        };

                        return Some(Ok((
                            token.access_token,
                            project_id,
                            token.email,
                            token.account_id,
                            0,
                        )));
                    } else {
                        if is_rate_limited {
                            tracing::warn!("🔒 [FIX #820] Preferred account {} is rate-limited, falling back to round-robin", preferred_token.email);
                        } else {
                            tracing::warn!("🔒 [FIX #820] Preferred account {} is quota-protected for {}, falling back to round-robin", preferred_token.email, target_model);
                        }
                    }
                }
            }
        } else {
            tracing::warn!(
                "🔒 [FIX #820] Preferred account {} not found in pool, falling back to round-robin",
                pref_id
            );
        }
        None
    }
}
