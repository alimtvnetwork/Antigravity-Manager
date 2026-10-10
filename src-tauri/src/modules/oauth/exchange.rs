use super::*;

async fn exchange_code_once(
    code: &str,
    redirect_uri: &str,
    client_cfg: &OAuthClientConfig,
) -> Result<TokenResponse, (Option<reqwest::StatusCode>, String)> {
    // [PHASE 2] 对于登录行为，尚未有 account_id，使用全局池阶梯逻辑
    let client = if let Some(pool) = crate::proxy::proxy_pool::get_global_proxy_pool() {
        pool.get_effective_standard_client(None, 60).await
    } else {
        crate::utils::http::get_long_standard_client()
    };

    let params = [
        ("client_id", client_cfg.client_id.as_str()),
        ("client_secret", client_cfg.client_secret.as_str()),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("grant_type", "authorization_code"),
    ];

    tracing::debug!(
        "[OAuth] Sending exchange_code request with User-Agent: {}",
        crate::constants::NATIVE_OAUTH_USER_AGENT.as_str()
    );

    let response = client
        .post(TOKEN_URL)
        .header(rquest::header::USER_AGENT, crate::constants::NATIVE_OAUTH_USER_AGENT.as_str())
        .form(&params)
        .send()
        .await
        .map_err(|e| {
            if e.is_connect() || e.is_timeout() {
                (
                    None,
                    format!(
                        "Token exchange request failed: {}. 请检查你的网络代理设置，确保可以稳定连接 Google 服务。",
                        e
                    ),
                )
            } else {
                (None, format!("Token exchange request failed: {}", e))
            }
        })?;

    if response.status().is_success() {
        let mut token_res = response
            .json::<TokenResponse>()
            .await
            .map_err(|e| (None, format!("Token parsing failed: {}", e)))?;
        token_res.oauth_client_key = Some(client_cfg.key.clone());

        // Add detailed logs
        crate::modules::logger::log_info(&format!(
            "Token exchange successful via [{}]! access_token: {}..., refresh_token: {}",
            client_cfg.key,
            &token_res.access_token.chars().take(20).collect::<String>(),
            if token_res.refresh_token.is_some() {
                "✓"
            } else {
                "✗ Missing"
            }
        ));

        // Log warning if refresh_token is missing
        if token_res.refresh_token.is_none() {
            crate::modules::logger::log_warn(
                "Warning: Google did not return a refresh_token. Potential reasons:\n\
                 1. User has previously authorized this application\n\
                 2. Need to revoke access in Google Cloud Console and retry\n\
                 3. OAuth parameter configuration issue",
            );
        }

        Ok(token_res)
    } else {
        let status = response.status();
        let error_text = response.text().await.unwrap_or_default();
        Err((
            Some(status),
            format!("Token exchange failed: {}", error_text),
        ))
    }
}

/// Exchange authorization code for token using optional preferred client.
/// When preferred/active client mismatches, fallback to other configured clients.
pub async fn exchange_code_with_client(
    code: &str,
    redirect_uri: &str,
    preferred_client_key: Option<&str>,
) -> Result<TokenResponse, String> {
    let candidates = get_candidate_clients(preferred_client_key);
    if candidates.is_empty() {
        return Err("No OAuth clients configured".to_string());
    }

    let mut attempt_errors: Vec<String> = Vec::new();

    for (idx, client_cfg) in candidates.iter().enumerate() {
        match exchange_code_once(code, redirect_uri, client_cfg).await {
            Ok(token_res) => {
                if idx > 0 {
                    crate::modules::logger::log_info(&format!(
                        "OAuth code exchange recovered via fallback client [{}]",
                        client_cfg.key
                    ));
                }
                return Ok(token_res);
            }
            Err((status_opt, err_msg)) => {
                let should_fallback = status_opt
                    .map(|status| is_client_mismatch_error(status, &err_msg))
                    .unwrap_or(false);

                attempt_errors.push(format!("{} => {}", client_cfg.key, err_msg));

                if should_fallback {
                    crate::modules::logger::log_warn(&format!(
                        "OAuth code exchange failed for client [{}], trying next client: {}",
                        client_cfg.key, err_msg
                    ));
                    continue;
                }

                return Err(format!(
                    "Token exchange failed for client [{}]: {}",
                    client_cfg.key, err_msg
                ));
            }
        }
    }

    Err(format!(
        "Token exchange failed for all OAuth clients: {}",
        attempt_errors.join(" | ")
    ))
}

/// Exchange authorization code for token
pub async fn exchange_code(code: &str, redirect_uri: &str) -> Result<TokenResponse, String> {
    exchange_code_with_client(code, redirect_uri, None).await
}
