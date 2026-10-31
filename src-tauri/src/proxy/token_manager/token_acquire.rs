//! Token acquisition main entrance: get_token / get_image_token and core scheduling logic.

use super::helpers::wait_for_image_account_change;
use super::helpers::wait_for_image_token_selection;
use super::types::OnDiskAccountState;
use super::ProxyToken;
use super::TokenManager;
use crate::proxy::server::ImagePermit;
use crate::proxy::server::ImageScheduler;
use crate::proxy::sticky_config::SchedulingMode;
use axum::http::StatusCode;
use std::collections::HashSet;
use std::sync::Arc;

impl TokenManager {
    /// Get currently available Tokens (supports sticky sessions and intelligent scheduling)
    /// Parameter `quota_group` is used to distinguish "claude" vs "gemini" groups
    /// When the parameter `force_rotate` is true, the lock will be ignored and the account will be forced to switch.
    /// Parameter `session_id` is used to maintain session stickiness across requests
    /// Parameter `target_model` is used to check quota protection (Issue #621)
    pub async fn get_token(
        &self,
        quota_group: &str,
        force_rotate: bool,
        session_id: Option<&str>,
        target_model: &str,
    ) -> Result<(String, String, String, String, u64), String> {
        let excluded_accounts = HashSet::new();
        self.get_token_filtered(
            quota_group,
            force_rotate,
            session_id,
            target_model,
            &excluded_accounts,
        )
        .await
    }

    pub async fn get_image_token(
        &self,
        force_rotate: bool,
        session_id: Option<&str>,
        target_model: &str,
        scheduler: &Arc<ImageScheduler>,
        request_timeout: u64,
    ) -> Result<(String, String, String, String, u64, ImagePermit), (StatusCode, String)> {
        let deadline =
            tokio::time::Instant::now() + std::time::Duration::from_secs(request_timeout);
        let mut scheduler_changes = scheduler.subscribe_changes();

        loop {
            scheduler_changes.borrow_and_update();
            let mut busy_accounts = HashSet::new();

            loop {
                let selection = wait_for_image_token_selection(
                    deadline,
                    self.get_token_filtered(
                        "image_gen",
                        force_rotate,
                        session_id,
                        target_model,
                        &busy_accounts,
                    ),
                )
                .await;
                match selection {
                    None => {
                        return Err((
                            StatusCode::TOO_MANY_REQUESTS,
                            "Image queue wait timeout".to_string(),
                        ));
                    }
                    Some(Ok((access_token, project_id, email, account_id, wait_ms))) => {
                        if let Some(permit) = scheduler.try_acquire(&account_id) {
                            return Ok((
                                access_token,
                                project_id,
                                email,
                                account_id,
                                wait_ms,
                                permit,
                            ));
                        }
                        busy_accounts.insert(account_id);
                    }
                    Some(Err(selection_error)) => {
                        if busy_accounts.is_empty() {
                            return Err((
                                StatusCode::SERVICE_UNAVAILABLE,
                                format!("Token error: {}", selection_error),
                            ));
                        }
                        break;
                    }
                }
            }

            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if !wait_for_image_account_change(&mut scheduler_changes, remaining).await {
                return Err((
                    StatusCode::TOO_MANY_REQUESTS,
                    "Image queue wait timeout".to_string(),
                ));
            }
        }
    }

    async fn get_token_filtered(
        &self,
        quota_group: &str,
        force_rotate: bool,
        session_id: Option<&str>,
        target_model: &str,
        excluded_accounts: &HashSet<String>,
    ) -> Result<(String, String, String, String, u64), String> {
        // [FIX] Check and process accounts to be reloaded (quota protection synchronization)
        let pending_reload = crate::proxy::server::take_pending_reload_accounts();
        for account_id in pending_reload {
            if let Err(e) = self.reload_account(&account_id).await {
                tracing::warn!("[Quota] Failed to reload account {}: {}", account_id, e);
            } else {
                tracing::info!(
                    "[Quota] Reloaded account {} (protected_models synced)",
                    account_id
                );
            }
        }

        // [FIX #1477] Check and process accounts to be deleted (clean cache completely)
        let pending_delete = crate::proxy::server::take_pending_delete_accounts();
        for account_id in pending_delete {
            self.remove_account(&account_id);
            tracing::info!(
                "[Proxy] Purged deleted account {} from all caches",
                account_id
            );
        }

        // [Optimization Issue #284] Add 5 seconds timeout to prevent deadlock
        let timeout_duration = std::time::Duration::from_secs(5);
        match tokio::time::timeout(
            timeout_duration,
            self.get_token_internal(
                quota_group,
                force_rotate,
                session_id,
                target_model,
                excluded_accounts,
            ),
        )
        .await
        {
            Ok(result) => result,
            Err(_) => Err(
                "Token acquisition timeout (5s) - system too busy or deadlock detected".to_string(),
            ),
        }
    }

    /// Internal implementation: core logic of obtaining Token
    async fn get_token_internal(
        &self,
        quota_group: &str,
        force_rotate: bool,
        session_id: Option<&str>,
        target_model: &str,
        excluded_accounts: &HashSet<String>,
    ) -> Result<(String, String, String, String, u64), String> {
        let mut tokens_snapshot: Vec<ProxyToken> =
            self.tokens.iter().map(|e| e.value().clone()).collect();
        tokens_snapshot.retain(|token| !excluded_accounts.contains(&token.account_id));
        let mut total = tokens_snapshot.len();
        if total == 0 {
            return Err("Token pool is empty".to_string());
        }

        tokens_snapshot = Self::sort_candidates_by_capability(tokens_snapshot, target_model)?;
        total = tokens_snapshot.len();

        // 0. Read the current scheduling configuration
        let scheduling = self.sticky_config.read().await.clone();
        use crate::proxy::sticky_config::SchedulingMode;

        // [New] Check whether quota protection is enabled (if it is off, the protected_models check is ignored)
        let quota_protection_enabled = crate::modules::config::load_app_config()
            .map(|cfg| cfg.quota_protection.enabled)
            .unwrap_or(false);

        // ===== [FIX #820] Fixed account mode: Prioritize the use of designated accounts =====
        let preferred_id = self.preferred_account_id.read().await.clone();
        if let Some(ref pref_id) = preferred_id {
            if let Some(result) = self
                .try_preferred_account(
                    &mut tokens_snapshot,
                    &mut total,
                    pref_id,
                    target_model,
                    quota_protection_enabled,
                )
                .await
            {
                return result;
            }
        }
        // ===== [END FIX #820] =====

        // [Optimization Issue #284] Move the lock operation outside the loop to avoid repeated lock acquisition
        // Get a snapshot of last_used_account in advance to avoid multiple locks in the loop
        let last_used_account_id = if quota_group != "image_gen" {
            let last_used = self.last_used_account.lock().await;
            last_used.clone()
        } else {
            None
        };

        let mut attempted: HashSet<String> = HashSet::new();
        let mut last_error: Option<String> = None;
        let mut need_update_last_used: Option<(String, std::time::Instant)> = None;

        for attempt in 0..total {
            let rotate = force_rotate || attempt > 0;
            // Normalized target model named Standard ID, used for quota protection checks
            let normalized_target =
                crate::proxy::common::model_mapping::normalize_to_standard_id(target_model)
                    .unwrap_or_else(|| target_model.to_string());

            let target_token = self
                .select_target_token(
                    &tokens_snapshot,
                    &attempted,
                    session_id,
                    scheduling.mode,
                    quota_group,
                    quota_protection_enabled,
                    target_model,
                    rotate,
                    total,
                    &last_used_account_id,
                    &mut need_update_last_used,
                )
                .await;

            let mut token = self
                .resolve_token_with_optimistic_reset(
                    target_token,
                    &tokens_snapshot,
                    &attempted,
                    &normalized_target,
                    quota_protection_enabled,
                )
                .await?;

            // Safety net: avoid selecting an account that has been disabled on disk but still
            // exists in the in-memory snapshot (e.g. stale cache + sticky session binding).
            match Self::get_account_state_on_disk(&token.account_path).await {
                OnDiskAccountState::Disabled => {
                    tracing::warn!(
                        "Selected account {} is disabled on disk, purging and retrying",
                        token.email
                    );
                    attempted.insert(token.account_id.clone());
                    self.remove_account(&token.account_id);
                    continue;
                }
                OnDiskAccountState::Unknown => {
                    tracing::warn!(
                        "Selected account {} state on disk is unavailable, skipping",
                        token.email
                    );
                    attempted.insert(token.account_id.clone());
                    continue;
                }
                OnDiskAccountState::Enabled => {}
            }

            if !self
                .refresh_token_if_expired(
                    &mut token,
                    quota_group,
                    &last_used_account_id,
                    &mut last_error,
                    &mut attempted,
                    &mut need_update_last_used,
                )
                .await
            {
                continue;
            }

            let project_id = self.resolve_project_id(&token).await;

            // [Optimization] Update last_used_account uniformly before returning successfully (if necessary)
            if let Some((new_account_id, new_time)) = need_update_last_used {
                if quota_group != "image_gen" {
                    let mut last_used = self.last_used_account.lock().await;
                    if new_account_id.is_empty() {
                        // An empty string indicates that the lock needs to be cleared
                        *last_used = None;
                    } else {
                        *last_used = Some((new_account_id, new_time));
                    }
                }
            }

            return Ok((
                token.access_token,
                project_id,
                token.email,
                token.account_id,
                0,
            ));
        }

        Err(last_error.unwrap_or_else(|| "All accounts failed".to_string()))
    }
}
