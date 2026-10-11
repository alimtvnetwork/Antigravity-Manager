use crate::models::QuotaData;
use serde_json::json;

use super::*;
use crate::proxy::project_resolver::fetch_project_id;

/// Fetch quota with cache support
pub async fn fetch_quota_with_cache(
    access_token: &str,
    email: &str,
    cached_project_id: Option<&str>,
    account_id: Option<&str>,
    existing_quota: Option<&QuotaData>,
) -> crate::error::AppResult<(QuotaData, Option<String>)> {
    use crate::error::AppError;
use super::fetch_more::fetch_quota_summary;
    use crate::models::quota::{
        is_subscription_tier_fetch_needed, resolve_fetched_subscription_tier,
    };

    let now = chrono::Utc::now().timestamp();
    let needs_tier_fetch = is_subscription_tier_fetch_needed(existing_quota, now);
    let needs_project_fetch = cached_project_id.is_none();
    let load_assist_needed = needs_tier_fetch || needs_project_fetch;

    let fetched = if load_assist_needed {
        match fetch_project_id(access_token).await {
            Ok(pid) => (Some(pid), None),
            Err(_) => (None, None),
        }
    } else {
        (None, None)
    };

    let project_id = fetched
        .0
        .or_else(|| cached_project_id.map(|pid| pid.to_string()));
    let fetched_tier = fetched.1.clone();
    let subscription_tier =
        resolve_fetched_subscription_tier(existing_quota, fetched.1, load_assist_needed);
    let tier_from_network = load_assist_needed && fetched_tier.is_some();

    // We keep project_id to store in the DB, but we NO LONGER force inject it into payload if it's absent

    let client = create_standard_client(account_id).await;
    let payload = if let Some(ref pid) = project_id {
        json!({ "project": pid })
    } else {
        json!({}) // Empty payload fallback
    };

    let mut last_error: Option<AppError> = None;

    for (ep_idx, ep_url) in QUOTA_API_ENDPOINTS.iter().enumerate() {
        let has_next = ep_idx + 1 < QUOTA_API_ENDPOINTS.len();

        let mut current_payload = payload.clone();
        let mut retry_without_project = false;

        loop {
            match client
                .post(*ep_url)
                .bearer_auth(access_token)
                .header(
                    rquest::header::USER_AGENT,
                    crate::constants::NATIVE_OAUTH_USER_AGENT.as_str(),
                )
                .json(&current_payload)
                .send()
                .await
            {
                Ok(response) => {
                    // Convert HTTP error status to AppError
                    if let Err(_) = response.error_for_status_ref() {
                        let status = response.status();

                        // [FIX] 403 Forbidden handling: strip project_id if present and retry
                        if status == rquest::StatusCode::FORBIDDEN {
                            if current_payload.get("project").is_some() && !retry_without_project {
                                crate::modules::logger::log_warn(&format!(
                                    "Quota fetch got 403 with project ID, retrying without project ID..."
                                ));
                                current_payload = json!({});
                                retry_without_project = true;
                                continue;
                            }

                            crate::modules::logger::log_warn(&format!(
                                "Account unauthorized (403 Forbidden), marking as forbidden"
                            ));
                            let mut q = QuotaData::new();
                            q.is_forbidden = true;
                            q.subscription_tier = subscription_tier.clone();
                            if tier_from_network {
                                q.subscription_tier_fetched_at = Some(now);
                            } else if let Some(existing) = existing_quota {
                                q.subscription_tier_fetched_at =
                                    existing.subscription_tier_fetched_at;
                            }
                            return Ok((q, project_id.clone()));
                        }

                        let text = response.text().await.unwrap_or_default();

                        // 429/5xx: fallback to next endpoint
                        if has_next
                            && (status == rquest::StatusCode::TOO_MANY_REQUESTS
                                || status.is_server_error())
                        {
                            crate::modules::logger::log_warn(&format!(
                                "Quota API {} returned {}, falling back to next endpoint",
                                ep_url, status
                            ));
                            last_error =
                                Some(AppError::Unknown(format!("HTTP {} - {}", status, text)));
                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                            break; // Break the inner retry loop, continue to next endpoint
                        }

                        return Err(AppError::Unknown(format!(
                            "API Error: {} - {}",
                            status, text
                        )));
                    }

                    if ep_idx > 0 {
                        crate::modules::logger::log_info(&format!(
                            "Quota API fallback succeeded at endpoint #{}",
                            ep_idx + 1
                        ));
                    }

                    let quota_response: QuotaResponse =
                        response.json().await.map_err(AppError::from)?;

                    let mut quota_data = QuotaData::new();

                    // Use debug level for detailed info to avoid console noise
                    tracing::debug!("Quota API returned {} models", quota_response.models.len());

                    for (name, info) in quota_response.models {
                        if let Some(quota_info) = info.quota_info {
                            let percentage = quota_info
                                .remaining_fraction
                                .map(|f| (f * 100.0) as i32)
                                .unwrap_or(0);

                            let reset_time = quota_info.reset_time.clone().unwrap_or_default();

                            // Only keep models we care about (exclude internal chat models)
                            if name.starts_with("gemini")
                                || name.starts_with("claude")
                                || name.starts_with("gpt")
                                || name.starts_with("image")
                                || name.starts_with("imagen")
                            {
                                let model_quota = crate::models::quota::ModelQuota {
                                    name,
                                    percentage,
                                    reset_time,
                                    display_name: info.display_name,
                                    supports_images: info.supports_images,
                                    supports_thinking: info.supports_thinking,
                                    thinking_budget: info.thinking_budget,
                                    recommended: info.recommended,
                                    max_tokens: info.max_tokens,
                                    max_output_tokens: info.max_output_tokens,
                                    supported_mime_types: info.supported_mime_types,
                                };
                                quota_data.add_model(model_quota);
                            }
                        }
                    }

                    // Parse deprecated model routing rules
                    if let Some(deprecated) = quota_response.deprecated_model_ids {
                        for (old_id, info) in deprecated {
                            // Register forwarding rules (including those mapping to gemini-pro-agent)
                            quota_data
                                .model_forwarding_rules
                                .insert(old_id, info.new_model_id);
                        }
                    }

                    // Set subscription tier
                    quota_data.subscription_tier = subscription_tier.clone();
                    if tier_from_network {
                        quota_data.subscription_tier_fetched_at = Some(now);
                    } else if let Some(existing) = existing_quota {
                        quota_data.subscription_tier_fetched_at =
                            existing.subscription_tier_fetched_at;
                    }

                    // Best-effort: fetch grouped quota summary (weekly + 5h windows).
                    // Failure here must not block the primary quota result.
                    let quota_groups =
                        fetch_quota_summary(access_token, email, project_id.as_deref(), account_id)
                            .await;

                    // [FIX #3426] Fuse real bucket quotas into models so UI doesn't show fake 100%
                    if let Some(ref groups) = quota_groups {
                        for model in quota_data.models.iter_mut() {
                            let name_lower = model.name.to_lowercase();
                            let is_claude_or_gpt =
                                name_lower.starts_with("claude") || name_lower.starts_with("gpt");
                            let is_gemini = name_lower.starts_with("gemini");

                            for group in groups {
                                let gname = group.display_name.to_lowercase();
                                let matches_group = if is_claude_or_gpt {
                                    gname.contains("claude")
                                        || gname.contains("gpt")
                                        || gname.contains("3p")
                                } else if is_gemini {
                                    gname.contains("gemini")
                                        || (!gname.contains("claude")
                                            && !gname.contains("gpt")
                                            && !gname.contains("3p"))
                                } else {
                                    false
                                };

                                if matches_group {
                                    // Find 5h bucket and weekly bucket to compute effective available quota
                                    let bucket_5h = group.buckets.iter().find(|b| {
                                        let win = b.window.to_lowercase();
                                        let bid = b.bucket_id.to_lowercase();
                                        win.contains("5h")
                                            || bid.contains("5h")
                                            || win.contains("hour")
                                            || bid.contains("hour")
                                    });
                                    let bucket_weekly = group.buckets.iter().find(|b| {
                                        let win = b.window.to_lowercase();
                                        let bid = b.bucket_id.to_lowercase();
                                        win.contains("week")
                                            || bid.contains("week")
                                            || win.contains("7d")
                                            || bid.contains("7d")
                                    });

                                    let chosen_bucket = match (bucket_5h, bucket_weekly) {
                                        (Some(h), Some(w)) => {
                                            // If weekly quota is exhausted (<= 0.001), model is limited by weekly quota to 0%, use weekly reset
                                            if w.remaining_fraction <= 0.001 {
                                                Some(w)
                                            } else {
                                                // When weekly quota is not exhausted, always use 5h bucket to accurately reflect 5h rolling quota and reset
                                                Some(h)
                                            }
                                        }
                                        (Some(h), None) => Some(h),
                                        (None, Some(w)) => Some(w),
                                        _ => group.buckets.first(),
                                    };

                                    if let Some(b) = chosen_bucket {
                                        model.percentage =
                                            (b.remaining_fraction * 100.0).round() as i32;
                                        if !b.reset_time.is_empty() {
                                            model.reset_time = b.reset_time.clone();
                                        }
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    quota_data.quota_groups = quota_groups;

                    return Ok((quota_data, project_id.clone()));
                }
                Err(e) => {
                    crate::modules::logger::log_warn(&format!(
                        "Quota API request failed at {}: {}",
                        ep_url, e
                    ));
                    last_error = Some(AppError::from(e));
                    if has_next {
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    }
                    break; // Break the inner retry loop on network error, continue to next endpoint
                }
            }
        } // End of inner loop
    }

    Err(last_error.unwrap_or_else(|| {
        AppError::Unknown("Quota fetch failed: all endpoints exhausted".to_string())
    }))
}
