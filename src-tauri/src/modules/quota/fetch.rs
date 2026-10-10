use crate::models::QuotaData;
use serde_json::json;

use super::*;
use crate::utils::http::create_standard_client;

/// Fetch project ID and subscription tier
async fn fetch_project_id(
    access_token: &str,
    email: &str,
    account_id: Option<&str>,
) -> (Option<String>, Option<String>) {
    let client = create_standard_client(account_id).await;
    let meta = json!({"metadata": {"ideType": "ANTIGRAVITY"}});

    for (ep_idx, ep_url) in CLOUD_CODE_LOAD_PROJECT_ENDPOINTS.iter().enumerate() {
        let res = client
            .post(*ep_url)
            .header(
                rquest::header::AUTHORIZATION,
                format!("Bearer {}", access_token),
            )
            .header(rquest::header::CONTENT_TYPE, "application/json")
            .header(
                rquest::header::USER_AGENT,
                crate::constants::NATIVE_OAUTH_USER_AGENT.as_str(),
            )
            .json(&meta)
            .send()
            .await;

        match res {
            Ok(res) => {
                if res.status().is_success() {
                    if let Ok(data) = res.json::<LoadProjectResponse>().await {
                        let project_id = data.project_id.clone();

                        // Core logic: Multi-level fallback for tier extraction
                        // 1. Paid Tier (Google One AI Premium etc.)
                        // 2. Current Tier (If not ineligible)
                        // 3. Allowed Tiers (Restricted/Default proxy access)
                        let mut subscription_tier = data.paid_tier.as_ref().and_then(tier_text);

                        let is_ineligible = data.ineligible_tiers.is_some()
                            && !data.ineligible_tiers.as_ref().unwrap().is_empty();

                        if subscription_tier.is_none() && !is_ineligible {
                            subscription_tier = data.current_tier.as_ref().and_then(tier_text);
                        }
                        if subscription_tier.is_none() {
                            if let Some(allowed) = data.allowed_tiers.as_ref() {
                                let chosen = allowed
                                    .iter()
                                    .find(|tier| tier.is_default == Some(true))
                                    .or_else(|| allowed.first());
                                if let Some(label) = chosen.and_then(tier_text) {
                                    subscription_tier = Some(if is_ineligible {
                                        format!("{} (Restricted)", label)
                                    } else {
                                        label
                                    });
                                }
                            }
                        }
                        subscription_tier = finalize_subscription_tier(subscription_tier, email);

                        if ep_idx > 0 {
                            crate::modules::logger::log_info(&format!(
                                "loadCodeAssist fallback succeeded at endpoint #{}",
                                ep_idx + 1
                            ));
                        }

                        return (project_id, subscription_tier);
                    }
                } else {
                    crate::modules::logger::log_warn(&format!(
                        "⚠️  [{}] loadCodeAssist failed at {}: Status: {}",
                        email,
                        ep_url,
                        res.status()
                    ));
                    continue;
                }
            }
            Err(e) => {
                crate::modules::logger::log_error(&format!(
                    "❌ [{}] loadCodeAssist network error at {}: {}",
                    email, ep_url, e
                ));
                continue;
            }
        }
    }

    (None, None)
}

/// Unified entry point for fetching account quota
pub async fn fetch_quota(
    access_token: &str,
    email: &str,
    account_id: Option<&str>,
) -> crate::error::AppResult<(QuotaData, Option<String>)> {
    fetch_quota_with_cache(access_token, email, None, account_id, None).await
}
