use super::*;

async fn refresh_access_token_once(
    refresh_token: &str,
    account_id: Option<&str>,
    client_cfg: &OAuthClientConfig,
) -> Result<TokenResponse, (Option<reqwest::StatusCode>, String)> {
    // [PHASE 2] 根据 account_id 使用对应的代理
    let client = if let Some(pool) = crate::proxy::proxy_pool::get_global_proxy_pool() {
        pool.get_effective_standard_client(account_id, 60).await
    } else {
        crate::utils::http::get_long_standard_client()
    };

    let params = [
        ("client_id", client_cfg.client_id.as_str()),
        ("client_secret", client_cfg.client_secret.as_str()),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
    ];

    // [FIX #1583] 提供更详细的日志，帮助诊断 Docker 环境下的代理问题
    if let Some(id) = account_id {
        crate::modules::logger::log_info(&format!("Refreshing Token for account: {}...", id));
    } else {
        crate::modules::logger::log_info("Refreshing Token for generic request (no account_id)...");
    }

    tracing::debug!(
        "[OAuth] Sending refresh_access_token request with User-Agent: {}",
        crate::constants::NATIVE_OAUTH_USER_AGENT.as_str()
    );

    let response = client
        .post(TOKEN_URL)
        .header(
            rquest::header::USER_AGENT,
            crate::constants::NATIVE_OAUTH_USER_AGENT.as_str(),
        )
        .form(&params)
        .send()
        .await
        .map_err(|e| {
            if e.is_connect() || e.is_timeout() {
                (
                    None,
                    format!(
                        "Refresh request failed: {}. 无法连接 Google 授权服务器，请检查代理设置。",
                        e
                    ),
                )
            } else {
                (None, format!("Refresh request failed: {}", e))
            }
        })?;

    if response.status().is_success() {
        let mut token_data = response
            .json::<TokenResponse>()
            .await
            .map_err(|e| (None, format!("Refresh data parsing failed: {}", e)))?;
        token_data.oauth_client_key = Some(client_cfg.key.clone());

        crate::modules::logger::log_info(&format!(
            "Token refreshed successfully via [{}]! Expires in: {} seconds",
            client_cfg.key, token_data.expires_in
        ));
        Ok(token_data)
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        Err((Some(status), format!("Refresh failed: {}", error_text)))
    }
}

/// Refresh access_token using refresh_token with optional preferred OAuth client key.
/// If client mismatch occurs, it retries with other configured clients.
pub async fn refresh_access_token_with_client(
    refresh_token: &str,
    account_id: Option<&str>,
    preferred_client_key: Option<&str>,
) -> Result<TokenResponse, String> {
    let candidates = get_candidate_clients(preferred_client_key);
    if candidates.is_empty() {
        return Err("No OAuth clients configured".to_string());
    }

    let mut attempt_errors: Vec<String> = Vec::new();

    for (idx, client_cfg) in candidates.iter().enumerate() {
        let mut last_attempt_err = None;
        let mut recovered_token = None;

        // 对同一个 OAuth Client 最多执行 2 次尝试（首次遇 invalid_grant 进行 500ms 短退避重试确认）
        for retry_count in 0..2 {
            if retry_count > 0 {
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            }
            match refresh_access_token_once(refresh_token, account_id, client_cfg).await {
                Ok(token_res) => {
                    recovered_token = Some(token_res);
                    break;
                }
                Err((status_opt, err_msg)) => {
                    let is_grant_error = err_msg.contains("invalid_grant");
                    if is_grant_error && retry_count == 0 {
                        crate::modules::logger::log_warn(&format!(
                            "[OAuth] Client [{}] 收到疑似 invalid_grant，将在 500ms 后进行二次退避确认...",
                            client_cfg.key
                        ));
                        last_attempt_err = Some((status_opt, err_msg));
                        continue;
                    }
                    last_attempt_err = Some((status_opt, err_msg));
                    break;
                }
            }
        }

        if let Some(token_res) = recovered_token {
            if idx > 0 {
                crate::modules::logger::log_info(&format!(
                    "Refresh recovered via fallback OAuth client [{}]",
                    client_cfg.key
                ));
            }
            return Ok(token_res);
        }

        if let Some((status_opt, err_msg)) = last_attempt_err {
            let should_fallback = status_opt
                .map(|status| is_client_mismatch_error(status, &err_msg))
                .unwrap_or(false);

            attempt_errors.push(format!("{} => {}", client_cfg.key, err_msg));

            if should_fallback {
                crate::modules::logger::log_warn(&format!(
                    "Refresh failed for client [{}], trying next client: {}",
                    client_cfg.key, err_msg
                ));
                continue;
            }

            return Err(format!(
                "Refresh failed for client [{}]: {}",
                client_cfg.key, err_msg
            ));
        }
    }

    Err(format!(
        "Refresh failed for all OAuth clients: {}",
        attempt_errors.join(" | ")
    ))
}

/// Refresh access_token using refresh_token
pub async fn refresh_access_token(
    refresh_token: &str,
    account_id: Option<&str>,
) -> Result<TokenResponse, String> {
    refresh_access_token_with_client(refresh_token, account_id, None).await
}

/// Get user info
pub async fn get_user_info(
    access_token: &str,
    account_id: Option<&str>,
) -> Result<UserInfo, String> {
    let client = if let Some(pool) = crate::proxy::proxy_pool::get_global_proxy_pool() {
        pool.get_effective_client(account_id, 15).await
    } else {
        crate::utils::http::get_client()
    };

    let response = client
        .get(USERINFO_URL)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| format!("User info request failed: {}", e))?;

    if response.status().is_success() {
        response
            .json::<UserInfo>()
            .await
            .map_err(|e| format!("User info parsing failed: {}", e))
    } else {
        let error_text = response.text().await.unwrap_or_default();
        Err(format!("Failed to get user info: {}", error_text))
    }
}

/// Check and refresh Token if needed
/// Returns the latest access_token
pub async fn ensure_fresh_token(
    current_token: &crate::models::TokenData,
    account_id: Option<&str>,
) -> Result<crate::models::TokenData, String> {
    let now = chrono::Local::now().timestamp();

    // Keep enough validity to avoid immediate post-switch refresh failure.
    if current_token.expiry_timestamp > now + TOKEN_REFRESH_SKEW_SECONDS {
        return Ok(current_token.clone());
    }

    // Need to refresh
    crate::modules::logger::log_info(&format!(
        "Token expiring soon for account {:?}, refreshing...",
        account_id
    ));
    let response = refresh_access_token_with_client(
        &current_token.refresh_token,
        account_id,
        current_token.oauth_client_key.as_deref(),
    )
    .await?;

    let oauth_client_key =
        normalize_refreshed_oauth_client_key(current_token, response.oauth_client_key.clone());

    // Construct new TokenData
    Ok(crate::models::TokenData::new(
        response.access_token,
        current_token.refresh_token.clone(), // refresh_token may not be returned on refresh
        response.expires_in,
        current_token.email.clone(),
        current_token.project_id.clone(), // Keep original project_id
        None,                             // session_id will be generated in token_manager
        current_token.is_gcp_tos,
        response.id_token.or(current_token.id_token.clone()), // Use new id_token or keep old one
    )
    .with_oauth_client_key(oauth_client_key))
}
