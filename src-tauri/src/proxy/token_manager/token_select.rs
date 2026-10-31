//! Token selection: P2C algorithm, sticky session scheduling, optimistic reset.

use super::ProxyToken;
use super::TokenManager;
use crate::proxy::sticky_config::SchedulingMode;
use std::collections::HashSet;

impl TokenManager {
    /// Power of 2 Choices (P2C) selection algorithm
    /// Randomly select 2 from the top 5 candidates, choose the one with higher quota -> avoid hotspots
    /// Return selected index
    ///
    /// # Parameters
    /// * `candidates` - sorted list of candidate tokens
    /// * `attempted` - a collection of account IDs that have failed to be attempted
    /// * `normalized_target` - normalized target model name
    /// * `quota_protection_enabled` - whether to enable quota protection
    pub(crate) fn select_with_p2c<'a>(
        &self,
        candidates: &'a [ProxyToken],
        attempted: &HashSet<String>,
        normalized_target: &str,
        quota_protection_enabled: bool,
    ) -> Option<&'a ProxyToken> {
        use rand::Rng;

        // Filter available tokens
        let mut available: Vec<&ProxyToken> = candidates
            .iter()
            .filter(|t| !attempted.contains(&t.account_id))
            .filter(|t| {
                !quota_protection_enabled || !t.protected_models.contains(normalized_target)
            })
            .collect();

        // Keep lower-priority groups for retries; only this draw is restricted.
        let priority = available.iter().map(|t| t.priority).min()?;
        available.retain(|t| t.priority == priority);
        if available.len() == 1 {
            return Some(available[0]);
        }

        // P2C: randomly select 2 from the previous min(P2C_POOL_SIZE, len)
        let pool_size = available.len().min(Self::P2C_POOL_SIZE);
        let mut rng = rand::thread_rng();

        let pick1 = rng.gen_range(0..pool_size);
        let pick2 = rng.gen_range(0..pool_size);
        // Make sure to select two different candidates
        let pick2 = if pick2 == pick1 {
            (pick1 + 1) % pool_size
        } else {
            pick2
        };

        let c1 = available[pick1];
        let c2 = available[pick2];

        // Choose one with a higher quota
        let selected = if c1.remaining_quota.unwrap_or(0) >= c2.remaining_quota.unwrap_or(0) {
            c1
        } else {
            c2
        };

        tracing::debug!(
            "🎲 [P2C] Selected {} ({}%) from [{}({}%), {}({}%)]",
            selected.email,
            selected.remaining_quota.unwrap_or(0),
            c1.email,
            c1.remaining_quota.unwrap_or(0),
            c2.email,
            c2.remaining_quota.unwrap_or(0)
        );

        Some(selected)
    }

    /// Sticky session and intelligent scheduling logic: select the target token from the candidate account and solidify the sticky session binding.
    pub(crate) async fn select_target_token(
        &self,
        tokens_snapshot: &[ProxyToken],
        attempted: &HashSet<String>,
        session_id: Option<&str>,
        scheduling_mode: SchedulingMode,
        quota_group: &str,
        quota_protection_enabled: bool,
        target_model: &str,
        rotate: bool,
        total: usize,
        last_used_account_id: &Option<(String, std::time::Instant)>,
        need_update_last_used: &mut Option<(String, std::time::Instant)>,
    ) -> Option<ProxyToken> {
        // ===== [Core] Sticky session and intelligent scheduling logic =====
        let mut target_token: Option<ProxyToken> = None;

        // Normalized target model named Standard ID, used for quota protection checks
        let normalized_target =
            crate::proxy::common::model_mapping::normalize_to_standard_id(target_model)
                .unwrap_or_else(|| target_model.to_string());

        // Mode A: Sticky session handling (CacheFirst or Balance with session_id)
        if !rotate && session_id.is_some() && scheduling_mode != SchedulingMode::PerformanceFirst {
            let sid = session_id.unwrap();

            // 1. Check whether the session has been bound to an account
            if let Some(bound_id) = self.session_accounts.get(sid).map(|v| v.clone()) {
                // [Fix] First find the corresponding account through account_id and get its email
                // 2. Convert email -> account_id to check whether the bound account is restricted.
                if let Some(bound_token) = tokens_snapshot.iter().find(|t| t.account_id == bound_id)
                {
                    let key = self
                        .email_to_account_id(&bound_token.email)
                        .unwrap_or_else(|| bound_token.account_id.clone());
                    // [FIX] Pass in the standardized ID of the target model and check whether the model has been accurately locked by the circuit breaker.
                    let reset_sec = self
                        .rate_limit_tracker
                        .get_remaining_wait(&key, Some(&normalized_target));
                    if reset_sec > 0 {
                        // [Fix Issue #284] Immediately unbind and switch accounts, no longer blocked and waiting
                        // Reason: Blocking and waiting will cause the client socket to time out (UND_ERR_SOCKET) during concurrent requests.
                        tracing::debug!(
                                "Sticky Session: Bound account {} is rate-limited for {} ({}s), unbinding and switching.",
                                bound_token.email, normalized_target, reset_sec
                            );
                        self.session_accounts.remove(sid);
                    } else if !attempted.contains(&bound_id)
                        && !(quota_protection_enabled
                            && bound_token.protected_models.contains(&normalized_target))
                    {
                        // 3. The account is available and has not been marked as a failed attempt, and will be reused first.
                        tracing::info!(
                            "Sticky Session: Successfully reusing bound account {} for session {}",
                            bound_token.email,
                            sid
                        );
                        target_token = Some(bound_token.clone());
                        *need_update_last_used =
                            Some((bound_token.account_id.clone(), std::time::Instant::now()));
                    } else if quota_protection_enabled
                        && bound_token.protected_models.contains(&normalized_target)
                    {
                        tracing::debug!("Sticky Session: Bound account {} is quota-protected for model {} [{}], unbinding and switching.", bound_token.email, normalized_target, target_model);
                        self.session_accounts.remove(sid);
                    } else if attempted.contains(&bound_id) {
                        // [FIX] The bound account has failed in the current round of requests. Unbind it immediately to avoid deadlock.
                        tracing::debug!("Sticky Session: Bound account {} already attempted in current request, unbinding", bound_token.email);
                        self.session_accounts.remove(sid);
                    }
                } else {
                    // The bound account no longer exists (may be deleted), unbind
                    tracing::debug!(
                        "Sticky Session: Bound account not found for session {}, unbinding",
                        sid
                    );
                    self.session_accounts.remove(sid);
                }
            }
        }

        // Mode B: Atomic 60s global lock (default protection for no session_id)
        // [Fix] Performance priority mode should skip 60s lock;
        if target_token.is_none()
            && !rotate
            && quota_group != "image_gen"
            && scheduling_mode != SchedulingMode::PerformanceFirst
        {
            // Only for stateless requests without session_id, use a 60s global lock guarantee to avoid rotation
            if session_id.is_none() {
                if let Some((account_id, last_time)) = &last_used_account_id {
                    // [FIX #3] 60s lock logic should check the `attempted` set to avoid repeated attempts on failed accounts
                    if last_time.elapsed().as_secs() < 60 && !attempted.contains(account_id) {
                        if let Some(found) =
                            tokens_snapshot.iter().find(|t| &t.account_id == account_id)
                        {
                            // [Fix] Check current limit status and quota protection to avoid reusing locked accounts
                            if !self
                                .is_rate_limited(&found.account_id, Some(&normalized_target))
                                .await
                                && !(quota_protection_enabled
                                    && found.protected_models.contains(&normalized_target))
                            {
                                tracing::debug!(
                                    "60s Window: Force reusing last account: {}",
                                    found.email
                                );
                                target_token = Some(found.clone());
                                *need_update_last_used =
                                    Some((found.account_id.clone(), std::time::Instant::now()));
                            } else {
                                if self
                                    .is_rate_limited(&found.account_id, Some(&normalized_target))
                                    .await
                                {
                                    tracing::debug!(
                                        "60s Window: Last account {} is rate-limited, skipping",
                                        found.email
                                    );
                                } else {
                                    tracing::debug!("60s Window: Last account {} is quota-protected for model {} [{}], skipping", found.email, normalized_target, target_model);
                                }
                            }
                        }
                    }
                }
            }

            // If there is no lock or with session_id (session first assigned), use P2C balancing to select the account
            if target_token.is_none() {
                // First filter out the accounts with no current limit
                let mut non_limited: Vec<ProxyToken> = Vec::new();
                for t in tokens_snapshot {
                    if !self
                        .is_rate_limited(&t.account_id, Some(&normalized_target))
                        .await
                    {
                        non_limited.push(t.clone());
                    }
                }

                if let Some(selected) = self.select_with_p2c(
                    &non_limited,
                    &attempted,
                    &normalized_target,
                    quota_protection_enabled,
                ) {
                    target_token = Some(selected.clone());
                    *need_update_last_used =
                        Some((selected.account_id.clone(), std::time::Instant::now()));
                }
            }
        } else if target_token.is_none() {
            // Mode C: P2C selection (alternative to pure polling)
            tracing::debug!("🔄 [Mode C] P2C selection from {} candidates", total);

            // First filter out the accounts with no current limit
            let mut non_limited: Vec<ProxyToken> = Vec::new();
            for t in tokens_snapshot {
                if !self
                    .is_rate_limited(&t.account_id, Some(&normalized_target))
                    .await
                {
                    non_limited.push(t.clone());
                }
            }

            if let Some(selected) = self.select_with_p2c(
                &non_limited,
                &attempted,
                &normalized_target,
                quota_protection_enabled,
            ) {
                tracing::debug!("  {} - SELECTED via P2C", selected.email);
                target_token = Some(selected.clone());

                if rotate {
                    tracing::debug!("Force Rotation: Switched to account: {}", selected.email);
                }
            }
        }

        // [Core Solidification] If available accounts are parsed and are currently scheduled for sticky sessions, ensure that the binding is immediately solidified to prevent rotation or session drift.
        if let Some(ref selected) = target_token {
            if let Some(sid) = session_id {
                if scheduling_mode != SchedulingMode::PerformanceFirst && !rotate {
                    self.session_accounts
                        .insert(sid.to_string(), selected.account_id.clone());
                    tracing::info!(
                        "Sticky Session: Ensured binding account {} to session {}",
                        selected.email,
                        sid
                    );
                }
            }
        }
        target_token
    }

    /// Optimistic reset strategy (double-layer protection): parse the final available Token from the candidate Token.
    pub(crate) async fn resolve_token_with_optimistic_reset(
        &self,
        target_token: Option<ProxyToken>,
        tokens_snapshot: &[ProxyToken],
        attempted: &HashSet<String>,
        normalized_target: &str,
        quota_protection_enabled: bool,
    ) -> Result<ProxyToken, String> {
        match target_token {
            Some(t) => Ok(t),
            None => {
                // Optimistic reset strategy: two-layer protection mechanism
                // Calculate the minimum waiting time
                let min_wait = tokens_snapshot
                    .iter()
                    .filter_map(|t| {
                        let wait = self
                            .rate_limit_tracker
                            .get_remaining_wait(&t.account_id, Some(&normalized_target));
                        if wait > 0 {
                            Some(wait)
                        } else {
                            None
                        }
                    })
                    .min();

                // Layer 1: If the minimum waiting time <= 2 seconds, execute buffer delay
                if let Some(wait_sec) = min_wait {
                    if wait_sec <= 2 {
                        let wait_ms = (wait_sec as f64 * 1000.0) as u64;
                        tracing::warn!(
                                "All accounts rate-limited but shortest wait is {}s. Applying {}ms buffer for state sync...",
                                wait_sec, wait_ms
                            );

                        // buffer delay
                        tokio::time::sleep(tokio::time::Duration::from_millis(wait_ms)).await;

                        // Try selecting an account again
                        let mut retry_token = None;
                        for token in tokens_snapshot {
                            if attempted.contains(&token.account_id)
                                || self
                                    .is_rate_limited(&token.account_id, Some(&normalized_target))
                                    .await
                                || (quota_protection_enabled
                                    && token.protected_models.contains(normalized_target))
                            {
                                continue;
                            }
                            retry_token = Some(token);
                            break;
                        }

                        if let Some(t) = retry_token {
                            tracing::info!(
                                "✅ Buffer delay successful! Found available account: {}",
                                t.email
                            );
                            Ok(t.clone())
                        } else {
                            // Layer 2: There is still no available account after buffering, perform optimistic reset
                            tracing::warn!(
                                    "Buffer delay failed. Executing optimistic reset for all {} accounts...",
                                    tokens_snapshot.len()
                                );

                            // Clear all throttling records
                            self.rate_limit_tracker.clear_for_optimistic_reset();

                            // Try to select the account again (the remaining current limit status must be re-verified, and it is strictly prohibited to lock the account for as long as the weekly quota is exhausted)
                            let final_token = tokens_snapshot.iter().find(|t| {
                                !attempted.contains(&t.account_id)
                                    && !self
                                        .rate_limit_tracker
                                        .is_rate_limited(&t.account_id, Some(&normalized_target))
                                    && !(quota_protection_enabled
                                        && t.protected_models.contains(normalized_target))
                                    && !self
                                        .rate_limit_tracker
                                        .is_rate_limited(&t.account_id, Some(&normalized_target))
                            });

                            if let Some(t) = final_token {
                                tracing::info!(
                                    "✅ Optimistic reset successful! Using account: {}",
                                    t.email
                                );
                                Ok(t.clone())
                            } else {
                                return Err(
                                    "All accounts failed after optimistic reset.".to_string()
                                );
                            }
                        }
                    } else {
                        return Err(format!("All accounts limited. Wait {}s.", wait_sec));
                    }
                } else {
                    return Err("All accounts failed or unhealthy.".to_string());
                }
            }
        }
    }
}
