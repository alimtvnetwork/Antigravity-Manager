use super::utils::derive_session_uuid;
use super::*;
use crate::proxy::monitor::proxyrequestlog::CURRENT_UPSTREAM_CAPTURE;

impl UpstreamClient {
    /// Build v1internal URL
    pub(crate) fn build_url(base_url: &str, method: &str, query_string: Option<&str>) -> String {
        if let Some(qs) = query_string {
            format!("{}:{}?{}", base_url, method, qs)
        } else {
            format!("{}:{}", base_url, method)
        }
    }

    /// Determine if we should try next endpoint (fallback logic)
    pub(crate) fn should_try_next_endpoint(status: StatusCode) -> bool {
        status == StatusCode::REQUEST_TIMEOUT
            || status == StatusCode::NOT_FOUND
            || status.is_server_error()
    }

    /// Call v1internal API (Basic Method)
    ///
    /// Initiates a basic network request, supporting multi-endpoint auto-fallback.
    /// [UPDATED] Takes optional account_id for per-account proxy selection.
    pub async fn call_v1_internal(
        &self,
        method: &str,
        access_token: &str,
        body: Value,
        query_string: Option<&str>,
        account_id: Option<&str>, // [NEW] Account ID for proxy selection
    ) -> Result<UpstreamCallResult, String> {
        self.call_v1_internal_with_headers(
            method,
            access_token,
            body,
            query_string,
            std::collections::HashMap::new(),
            account_id,
        )
        .await
    }

    /// [FIX #765] Call v1internal API to support transparent transmission of additional headers
    /// [ENHANCED] Returns an UpstreamCallResult containing downgrade attempt records for debug logging
    pub async fn call_v1_internal_with_headers(
        &self,
        method: &str,
        access_token: &str,
        mut body: Value,
        query_string: Option<&str>,
        extra_headers: std::collections::HashMap<String, String>,
        account_id: Option<&str>, // [NEW] Account ID
    ) -> Result<UpstreamCallResult, String> {
        // [DEFENSE] Global ultimate defense interception: purify damaged/empty inlineData in all upstream packets and illegal accounting metadata that triggers Google WAF interception, and finally align the prefix topology in a unified manner
        if let Some(inner) = body.get_mut("request") {
            crate::proxy::mappers::common_utils::sanitize_gemini_payload_inline_data(inner);
            crate::proxy::mappers::prompt_sanitizer::PromptSanitizer::sanitize_gemini_payload(
                inner,
            );
            crate::proxy::mappers::common_utils::ensure_gemini_payload_ends_with_user(inner);
            crate::proxy::pipeline::InboundThinkingPipeline::align_google_request_prefix_topology(
                inner,
            );
        } else {
            crate::proxy::mappers::common_utils::sanitize_gemini_payload_inline_data(&mut body);
            crate::proxy::mappers::prompt_sanitizer::PromptSanitizer::sanitize_gemini_payload(
                &mut body,
            );
            crate::proxy::mappers::common_utils::ensure_gemini_payload_ends_with_user(&mut body);
            crate::proxy::pipeline::InboundThinkingPipeline::align_google_request_prefix_topology(
                &mut body,
            );
        }

        // [NEW] Get client based on account (cached in proxy pool manager)
        let client = self.get_client(account_id).await;

        // Build Headers (reused by all endpoints)
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );
        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", access_token))
                .map_err(|e| e.to_string())?,
        );

        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_str(&self.get_user_agent().await).unwrap_or_else(|e| {
                tracing::warn!("Invalid User-Agent header value, using fallback: {}", e);
                header::HeaderValue::from_static("antigravity")
            }),
        );

        // [ENHANCED] Inject Antigravity official client key feature headers
        // 1. Client Identity
        headers.insert(
            "x-client-name",
            header::HeaderValue::from_static("antigravity"),
        );
        if let Ok(ver) = header::HeaderValue::from_str(&crate::constants::CURRENT_VERSION) {
            headers.insert("x-client-version", ver);
        }

        // 2. Device & Session Identity
        // Machine ID (Persistent)
        if let Ok(mid) = machine_uid::get() {
            if let Ok(mid_val) = header::HeaderValue::from_str(&mid) {
                headers.insert("x-machine-id", mid_val);
            }
        }
        // Session ID (Per Conversation Isolation)
        let sess_uuid = if let Some(sid) = extra_headers.get("x-session-id") {
            derive_session_uuid(sid)
        } else {
            crate::constants::SESSION_ID.clone()
        };
        if let Ok(sess_val) = header::HeaderValue::from_str(&sess_uuid) {
            headers.insert("x-vscode-sessionid", sess_val);
        }

        // [REMOVED v4.1.24] x-goog-api-client (gl-node/fire/grpc) header has been removed.
        // This header belongs to the IDE's JS layer, not the official client's egress.
        // Sending it creates a contradictory "Electron + Node.js" fingerprint.

        // Keep body.project for content requests, but omit the quota-project header.
        let is_content_request = matches!(method, "generateContent" | "streamGenerateContent");
        if !is_content_request {
            if let Some(proj) = body.get("project").and_then(|v| v.as_str()) {
                if !proj.is_empty() && proj != "test-project" && proj != "project-id" {
                    if let Ok(hv) = header::HeaderValue::from_str(proj) {
                        headers.insert("x-goog-user-project", hv);
                    }
                }
            }
        }

        // Inject additional headers (such as anthropoc-beta)
        // Strictly prohibit transparent transmission of the client's inbound user-agent to ensure that the outbound fingerprint is always a supported Antigravity version
        for (k, v) in extra_headers {
            if k.eq_ignore_ascii_case("user-agent") {
                continue;
            }
            if let Ok(hk) = header::HeaderName::from_bytes(k.as_bytes()) {
                if let Ok(hv) = header::HeaderValue::from_str(&v) {
                    headers.insert(hk, hv);
                }
            }
        }
        if is_content_request {
            headers.remove("x-goog-user-project");
        }

        // [DEBUG] Log headers for verification
        tracing::debug!(?headers, "Final Upstream Request Headers");

        // Justification: best-effort task-local access; absence outside a request task is expected
        crate::error::record_ignored(
            crate::proxy::monitor::CURRENT_UPSTREAM_CAPTURE.try_with(|holder| {
                let pairs: Vec<(&str, &str)> = headers
                    .iter()
                    .filter_map(|(k, v)| v.to_str().ok().map(|s| (k.as_str(), s)))
                    .collect();
                holder.set_headers_json(
                    crate::proxy::payload_audit::header_pairs_to_redacted_json(pairs),
                );
            }),
            "task-local capture",
        );

        let mut has_triggered_downgrade = false;

        // [TEMPORARY FIX #3074] Auto-downgrade retry logic for 403 SERVICE_DISABLED
        // We wrap a loop so that when a specific error is detected we remove the header and try again
        loop {
            let mut last_err: Option<String> = None;
            let mut fallback_attempts: Vec<FallbackAttemptLog> = Vec::new();
            let mut should_retry_without_header = false;

            // Traverse all endpoints and automatically switch on failure
            for (idx, base_url) in V1_INTERNAL_BASE_URL_FALLBACKS.iter().enumerate() {
                let url = Self::build_url(base_url, method, query_string);
                let has_next = idx + 1 < V1_INTERNAL_BASE_URL_FALLBACKS.len();

                let body_bytes = serde_json::to_vec(&body).map_err(|e| e.to_string())?;

                let mut req_builder = client.post(&url).headers(headers.clone());

                // [FIX] Use chunked transfer emulation only for streaming interface (streamGenerateContent)
                // Send normal fixed length Body to other interfaces (such as generateContent, loadCodeAssist)
                // Otherwise, image generation will be rejected or limited by the Google server due to lack of Content-Length (429)
                if method == "streamGenerateContent" {
                    let stream_bytes = body_bytes.clone();
                    req_builder = req_builder.body(rquest::Body::wrap_stream(
                        futures::stream::once(async move { Ok::<_, std::io::Error>(stream_bytes) }),
                    ));
                } else {
                    req_builder = req_builder.body(body_bytes.clone());
                }

                let response = req_builder.send().await;

                match response {
                    Ok(resp) => {
                        let status = resp.status();
                        if status.is_success() {
                            if idx > 0 {
                                tracing::info!(
                                    "✓ Upstream fallback succeeded | Endpoint: {} | Status: {} | Next endpoints available: {}",
                                    base_url,
                                    status,
                                    V1_INTERNAL_BASE_URL_FALLBACKS.len() - idx - 1
                                );
                            } else {
                                tracing::debug!(
                                    "✓ Upstream request succeeded | Endpoint: {} | Status: {}",
                                    base_url,
                                    status
                                );
                            }
                            return Ok(UpstreamCallResult {
                                response: resp,
                                fallback_attempts,
                            });
                        }

                        // [NEW] Detecting 403 errors (Issue #3074)
                        // As long as it has a project header and returns 403, we will try to downgrade and try again.
                        if status == StatusCode::FORBIDDEN
                            && !has_triggered_downgrade
                            && headers.contains_key("x-goog-user-project")
                        {
                            tracing::warn!(
                                "Detected 403 Forbidden with project header, retrying WITHOUT x-goog-user-project header (Account: {:?})",
                                account_id
                            );
                            should_retry_without_header = true;
                            break;
                        }

                        // Switch if there is a next endpoint and the current error can be retried
                        if has_next && Self::should_try_next_endpoint(status) {
                            let err_msg = format!("Upstream {} returned {}", base_url, status);
                            tracing::warn!(
                                "Upstream endpoint returned {} at {} (method={}), trying next endpoint",
                                status,
                                base_url,
                                method
                            );
                            // [NEW] Logging downgrade attempts
                            fallback_attempts.push(FallbackAttemptLog {
                                endpoint_url: url.clone(),
                                status: Some(status.as_u16()),
                                error: err_msg.clone(),
                            });
                            last_err = Some(err_msg);
                            continue;
                        }

                        // Non-retryable error or it is the last endpoint, return directly
                        return Ok(UpstreamCallResult {
                            response: resp,
                            fallback_attempts,
                        });
                    }
                    Err(e) => {
                        let msg = format!("HTTP request failed at {}: {}", base_url, e);
                        tracing::debug!("{}", msg);
                        // [NEW] Log downgrade attempts with network errors
                        fallback_attempts.push(FallbackAttemptLog {
                            endpoint_url: url.clone(),
                            status: None,
                            error: msg.clone(),
                        });
                        last_err = Some(msg);

                        // If it is the last endpoint, exit the loop
                        if !has_next {
                            break;
                        }
                        continue;
                    }
                }
            }

            // Handle downgrade logic
            if should_retry_without_header {
                headers.remove("x-goog-user-project");
                has_triggered_downgrade = true;
                // Restart the outer loop and try again from the first endpoint
                continue;
            }

            // If no downgrade was triggered and all endpoints were tried, return the last error
            return Err(last_err.unwrap_or_else(|| "All endpoints failed".to_string()));
        }
    }

    /// Get a list of available models
    ///
    /// Get the remote model list and support automatic fallback of multiple endpoints
    #[allow(dead_code)] // API ready for future model discovery feature
    pub async fn fetch_available_models(
        &self,
        access_token: &str,
        account_id: Option<&str>,
    ) -> Result<Value, String> {
        // Reuse call_v1_internal and then parse the JSON
        let result = self
            .call_v1_internal(
                "fetchAvailableModels",
                access_token,
                serde_json::json!({}),
                None,
                account_id,
            )
            .await?;
        let json: Value = result
            .response
            .json()
            .await
            .map_err(|e| format!("Parse json failed: {}", e))?;
        Ok(json)
    }

    /// Auxiliary v1internal calls (non-user request paths, such as background context digests).
    ///
    /// Why it is needed: This type of call point is located in the mapper / auxiliary function, and `AppState.upstream` cannot be obtained historically.
    /// Just handwrite the URL - the result shape and host both drift
    /// （`{host}/v1internal/projects/{p}/locations/global/models/{m}:generateContent`
    /// HTML 404 is always returned on the three hosts daily / sandbox / prod, causing this function to never succeed).
    ///
    /// Here we reuse four sources that are exactly the same as the main request path to avoid drifting again:
    /// - Client `self.get_client(account_id)` - the same set of proxy pool selection by account and `client_cache`,
    ///   It will take effect with the hot update of the upstream proxy (see `rebuild_default_client` / `clear_client_cache`).
    ///   When the account is bound to a dedicated proxy, auxiliary requests will also go through it and will not leak out from the real IP.
    /// - Endpoint order `V1_INTERNAL_BASE_URL_FALLBACKS` (Daily → Sandbox → Prod)
    /// - URL shape `Self::build_url` (` {base} : {method} `, the model name is placed in the body)
    /// - Fallback determination `Self::should_try_next_endpoint` (change the endpoint only when 408 / 404 / 5xx)
    ///
    /// Difference from `call_v1_internal` (intentional): no request-level headers are injected
    /// (`x-vscode-sessionid` etc.). Timeout is specified by the caller using `timeout_secs` -
    /// The client defaults to 600s, which is too long for the summary, so it is explicitly tightened.
    pub async fn call_v1_internal_auxiliary(
        &self,
        method: &str,
        access_token: &str,
        body: Value,
        account_id: Option<&str>,
        timeout_secs: u64,
    ) -> Result<Value, String> {
        let client = self.get_client(account_id).await;
        let mut last_error = String::new();

        for base_url in V1_INTERNAL_BASE_URL_FALLBACKS.iter() {
            let url = Self::build_url(base_url, method, None);

            let response = match client
                .post(&url)
                .header("Authorization", format!("Bearer {}", access_token))
                .header("Content-Type", "application/json")
                .timeout(Duration::from_secs(timeout_secs))
                .json(&body)
                .send()
                .await
            {
                Ok(response) => response,
                Err(e) => {
                    tracing::warn!(
                        endpoint = %url,
                        error = %e,
                        "Auxiliary v1internal request failed, trying next endpoint"
                    );
                    last_error = format!("request to {} failed: {}", url, e);
                    continue;
                }
            };

            let status = response.status();
            if !status.is_success() {
                let text = response.text().await.unwrap_or_default();
                last_error = format!("{} returned {}: {}", url, status, text);

                // Same determination as the main request path: only 408 / 404 / 5xx endpoints are changed;
                // The remaining status (such as 400) indicates that there is a problem with the request itself and is terminated directly without triple retry.
                if !Self::should_try_next_endpoint(status) {
                    return Err(last_error);
                }
                tracing::warn!(
                    endpoint = %url,
                    status = %status,
                    "Auxiliary v1internal request returned retryable status, trying next endpoint"
                );
                continue;
            }

            return response
                .json()
                .await
                .map_err(|e| format!("failed to parse response from {}: {}", url, e));
        }

        Err(if last_error.is_empty() {
            "no v1internal endpoint available".to_string()
        } else {
            last_error
        })
    }
}
