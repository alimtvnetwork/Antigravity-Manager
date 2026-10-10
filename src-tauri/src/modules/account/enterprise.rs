pub use crate::models::{
    Account, AccountIndex, AccountSummary, DeviceProfile, DeviceProfileVersion, QuotaData,
    TokenData,
};
use crate::modules;

use super::*;

pub(crate) fn is_enterprise_client(client_key: Option<&str>) -> bool {
    client_key
        .map(str::trim)
        .filter(|key| !key.is_empty())
        .map(|key| key.eq_ignore_ascii_case("antigravity_enterprise"))
        .unwrap_or(false)
}

pub(crate) fn normalize_project_id(project_id: Option<&str>) -> Option<String> {
    project_id
        .map(str::trim)
        .filter(|pid| !pid.is_empty())
        .map(ToOwned::to_owned)
}

async fn ensure_enterprise_project_ready(account: &mut Account) -> Result<(), String> {
    if !is_enterprise_client(account.token.oauth_client_key.as_deref()) {
        return Ok(());
    }

    if normalize_project_id(account.token.project_id.as_deref()).is_some() {
        return Ok(());
    }

    crate::modules::logger::log_warn(&format!(
        "Account {} is using enterprise OAuth client but missing project_id. Trying to resolve before switch...",
        account.email
    ));

    match crate::proxy::project_resolver::fetch_project_id(&account.token.access_token).await {
        Ok(project_id) => {
            crate::modules::logger::log_info(&format!(
                "Resolved enterprise project_id for {}: {}",
                account.email, project_id
            ));
            account.token.project_id = Some(project_id);
            save_account(account)?;
            Ok(())
        }
        Err(e) => {
            crate::modules::logger::log_warn(&format!(
                "Account {} is currently missing enterprise project_id and auto-resolve failed ({}). Allowing switch to proceed, but certain enterprise features may be limited.",
                account.email, e
            ));
            Ok(())
        }
    }
}

pub(crate) fn is_rate_limit_error(err: &crate::error::AppError) -> bool {
    match err {
        crate::error::AppError::Network(_, Some(status)) => *status == 429,
        crate::error::AppError::Unknown(msg)
        | crate::error::AppError::OAuth(msg)
        | crate::error::AppError::Account(msg)
        | crate::error::AppError::Config(msg) => {
            let lower = msg.to_lowercase();
            lower.contains("429")
                || lower.contains("too many requests")
                || lower.contains("resource_exhausted")
                || lower.contains("resource has been exhausted")
        }
        _ => false,
    }
}

pub(crate) fn recover_cached_quota_on_rate_limit(
    account: &Account,
    err: &crate::error::AppError,
) -> Option<QuotaData> {
    if !is_rate_limit_error(err) {
        return None;
    }

    let cached = account.quota.clone()?;
    if cached.models.is_empty() {
        return None;
    }

    Some(cached)
}

pub(crate) fn is_validation_required_error(err: &crate::error::AppError) -> bool {
    let text = err.to_string().to_lowercase();
    text.contains("verify your account")
        || text.contains("further action is required")
        || text.contains("validation_url")
        || text.contains("appeal_url")
        || text.contains("validation required")
}

pub(crate) fn is_account_access_blocked_message(message: &str) -> bool {
    let text = message.to_lowercase();
    text.contains("verify your account")
        || text.contains("further action is required")
        || text.contains("validation_url")
        || text.contains("appeal_url")
        || text.contains("validation required")
        || text.contains("unauthorized_client")
        || text.contains("invalid_client")
        || text.contains("invalid_grant")
        || text.contains("resource_exhausted")
        || text.contains("resource has been exhausted")
}

pub(crate) fn format_switch_refresh_error(message: &str) -> String {
    let lower = message.to_lowercase();

    if lower.contains("unauthorized_client")
        || lower.contains("invalid_client")
        || lower.contains("invalid_grant")
    {
        return format!(
            "Token refresh failed: OAuth client is not authorized for this account. Please sign in again in Antigravity-Manager and complete authorization/verification. Raw error: {}",
            message
        );
    }

    if lower.contains("verify your account")
        || lower.contains("further action is required")
        || lower.contains("validation_url")
        || lower.contains("appeal_url")
        || lower.contains("validation required")
    {
        return format!(
            "Token refresh failed: account requires additional verification. Please finish verification in Antigravity, then retry account switch. Raw error: {}",
            message
        );
    }

    if lower.contains("resource_exhausted") || lower.contains("resource has been exhausted") {
        return format!(
            "Token refresh failed: account is rate-limited or temporarily restricted (RESOURCE_EXHAUSTED). Please retry later. Raw error: {}",
            message
        );
    }

    format!("Token refresh failed: {}", message)
}

pub(crate) fn format_rate_limit_block_reason(err: &crate::error::AppError) -> String {
    format!(
        "Account is temporarily rate-limited or risk-controlled (RESOURCE_EXHAUSTED). Please cool down and retry later. Raw error: {}",
        err
    )
}

pub(crate) fn mark_validation_blocked(account: &mut Account, reason: &str) {
    if account.validation_blocked && account.validation_blocked_reason.as_deref() == Some(reason) {
        return;
    }

    account.validation_blocked = true;
    account.validation_blocked_reason = Some(reason.to_string());
    if let Err(e) = save_account(account) {
        crate::modules::logger::log_warn(&format!(
            "Failed to persist validation_blocked state for {}: {}",
            account.email, e
        ));
    }
}

pub(crate) fn clear_validation_blocked(account: &mut Account) {
    if !account.validation_blocked {
        return;
    }

    account.validation_blocked = false;
    account.validation_blocked_until = None;
    account.validation_blocked_reason = None;
    account.validation_url = None;
    if let Err(e) = save_account(account) {
        crate::modules::logger::log_warn(&format!(
            "Failed to clear validation_blocked state for {}: {}",
            account.email, e
        ));
    }
}
