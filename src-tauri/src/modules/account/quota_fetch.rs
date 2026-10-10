pub use crate::models::{
    Account, AccountIndex, AccountSummary, DeviceProfile, DeviceProfileVersion, QuotaData,
    TokenData,
};
use crate::modules;

use super::*;

/// Quota query with retry (moved from commands to modules for reuse)
pub async fn fetch_quota_with_retry(account: &mut Account) -> crate::error::AppResult<QuotaData> {
    use crate::error::AppError;
    use crate::modules::oauth;

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
                // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                crate::error::record_ignored(save_account(account), "save_account");
                crate::proxy::server::trigger_account_reload(&account.id);
            }
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
    if let Err(AppError::Network(_, status)) = result {
        if let Some(code) = status {
            if code == 401 {
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
                            // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
                            crate::error::record_ignored(save_account(account), "save_account");
                            crate::proxy::server::trigger_account_reload(&account.id);
                        }
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
                    || account.name.as_ref().map_or(false, |n| n.trim().is_empty())
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

                if let Err(AppError::Network(_, status)) = retry_result {
                    if let Some(code) = status {
                        if code == 403 {
                            let mut q = QuotaData::new();
                            q.is_forbidden = true;
                            return Ok(q);
                        }
                    }
                }

                match retry_result {
                    Ok((q, _)) => {
                        clear_validation_blocked(account);
                        return Ok(q);
                    }
                    Err(e) => {
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
        }
    }

    // fetch_quota already handles 403, with additional local fallback/validation handling.
    match result {
        Ok((q, _)) => {
            clear_validation_blocked(account);
            Ok(q)
        }
        Err(e) => {
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
