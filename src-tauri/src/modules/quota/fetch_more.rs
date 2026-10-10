use crate::models::QuotaData;
use serde_json::json;

use super::*;

/// Fetch grouped quota summary (weekly + 5h windows) via retrieveUserQuotaSummary.
///
/// Best-effort: returns `None` on any failure so that the primary 5h quota fetch
/// (fetchAvailableModels) is never blocked by this auxiliary endpoint.
async fn fetch_quota_summary(
    access_token: &str,
    email: &str,
    project_id: Option<&str>,
    account_id: Option<&str>,
) -> Option<Vec<crate::models::quota::QuotaGroup>> {
    let client = create_standard_client(account_id).await;
    let payload = if let Some(pid) = project_id {
        json!({ "project": pid })
    } else {
        json!({})
    };

    for ep_url in QUOTA_SUMMARY_ENDPOINTS.iter() {
        let res = client
            .post(*ep_url)
            .bearer_auth(access_token)
            .header(
                rquest::header::USER_AGENT,
                crate::constants::NATIVE_OAUTH_USER_AGENT.as_str(),
            )
            .json(&payload)
            .send()
            .await;

        match res {
            Ok(response) => {
                let status = response.status();
                if !status.is_success() {
                    crate::modules::logger::log_warn(&format!(
                        "QuotaSummary API {} returned {}, trying next endpoint",
                        ep_url, status
                    ));
                    continue;
                }

                let summary: QuotaSummaryResponse = match response.json().await {
                    Ok(s) => s,
                    Err(e) => {
                        crate::modules::logger::log_warn(&format!(
                            "QuotaSummary JSON parse failed for {}: {}",
                            email, e
                        ));
                        return None;
                    }
                };

                let groups: Vec<crate::models::quota::QuotaGroup> = summary
                    .groups
                    .into_iter()
                    .map(|g| crate::models::quota::QuotaGroup {
                        display_name: g.display_name.unwrap_or_default(),
                        description: g.description,
                        buckets: g
                            .buckets
                            .into_iter()
                            .filter_map(|b| {
                                Some(crate::models::quota::QuotaBucket {
                                    bucket_id: b.bucket_id.unwrap_or_default(),
                                    window: b.window.unwrap_or_default(),
                                    remaining_fraction: b.remaining_fraction?,
                                    reset_time: b.reset_time.unwrap_or_default(),
                                    observed_at: Some(chrono::Utc::now().timestamp_millis()),
                                    cycle_tokens: None,
                                    display_name: b.display_name,
                                    description: b.description,
                                })
                            })
                            .collect(),
                    })
                    .collect();

                tracing::debug!("[{}] QuotaSummary fetched {} groups", email, groups.len());
                return Some(groups);
            }
            Err(e) => {
                crate::modules::logger::log_warn(&format!(
                    "QuotaSummary API request failed at {}: {}",
                    ep_url, e
                ));
                continue;
            }
        }
    }

    None
}

/// Internal fetch quota logic
#[allow(dead_code)]
pub async fn fetch_quota_inner(
    access_token: &str,
    email: &str,
) -> crate::error::AppResult<(QuotaData, Option<String>)> {
    fetch_quota_with_cache(access_token, email, None, None, None).await
}

/// Batch fetch all account quotas (backup functionality)
#[allow(dead_code)]
pub async fn fetch_all_quotas(
    accounts: Vec<(String, String, String)>,
) -> Vec<(String, crate::error::AppResult<QuotaData>)> {
    let mut results = Vec::new();
    for (id, email, access_token) in accounts {
        let res = fetch_quota(&access_token, &email, Some(&id)).await;
        results.push((email, res.map(|(q, _)| q)));
    }
    results
}
