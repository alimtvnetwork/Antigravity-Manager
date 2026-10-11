use super::*;
use crate::proxy::monitor::proxyrequestlog::CURRENT_UPSTREAM_CAPTURE;
use super::utils::derive_session_uuid;

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

    /// [FIX #765] 调用 v1internal API，支持透传额外的 Headers
    /// [ENHANCED] 返回 UpstreamCallResult，包含降级尝试记录，用于 debug 日志
    pub async fn call_v1_internal_with_headers(
        &self,
        method: &str,
        access_token: &str,
        mut body: Value,
        query_string: Option<&str>,
        extra_headers: std::collections::HashMap<String, String>,
        account_id: Option<&str>, // [NEW] Account ID
    ) -> Result<UpstreamCallResult, String> {
        // [DEFENSE] 全局终极防御拦截：净化所有发往上游报文中的损坏/空 inlineData 以及触发 Google WAF 拦截的违规计费元数据，并最终统一对齐前缀拓扑
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

        // 构建 Headers (所有端点复用)
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

        // [ENHANCED] 注入 Antigravity 官方客户端关键特征 Headers
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

        // 注入额外的 Headers (如 anthropic-beta)
        // 严格禁止透传客户端入站的 user-agent，确保出站指纹始终为受支持的 Antigravity 版本
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

        // [TEMPORARY FIX #3074] 针对 403 SERVICE_DISABLED 的自动降级重试逻辑
        // 我们包装一层循环，以便在检测到特定错误时移除 Header 并重试
        loop {
            let mut last_err: Option<String> = None;
            let mut fallback_attempts: Vec<FallbackAttemptLog> = Vec::new();
            let mut should_retry_without_header = false;

            // 遍历所有端点，失败时自动切换
            for (idx, base_url) in V1_INTERNAL_BASE_URL_FALLBACKS.iter().enumerate() {
                let url = Self::build_url(base_url, method, query_string);
                let has_next = idx + 1 < V1_INTERNAL_BASE_URL_FALLBACKS.len();

                let body_bytes = serde_json::to_vec(&body).map_err(|e| e.to_string())?;

                let mut req_builder = client.post(&url).headers(headers.clone());

                // [FIX] 仅对流式接口 (streamGenerateContent) 使用分块传输仿真
                // 对其他接口 (如 generateContent, loadCodeAssist) 发送正常的固定长度 Body
                // 否则图像生成会因为缺少 Content-Length 而被 Google 服务端拒绝或限流 (429)
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

                        // [NEW] 检测 403 错误 (Issue #3074)
                        // 只要带有项目 Header 且返回 403，我们就尝试降级重试一次
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

                        // 如果有下一个端点且当前错误可重试，则切换
                        if has_next && Self::should_try_next_endpoint(status) {
                            let err_msg = format!("Upstream {} returned {}", base_url, status);
                            tracing::warn!(
                                "Upstream endpoint returned {} at {} (method={}), trying next endpoint",
                                status,
                                base_url,
                                method
                            );
                            // [NEW] 记录降级尝试
                            fallback_attempts.push(FallbackAttemptLog {
                                endpoint_url: url.clone(),
                                status: Some(status.as_u16()),
                                error: err_msg.clone(),
                            });
                            last_err = Some(err_msg);
                            continue;
                        }

                        // 不可重试的错误或已是最后一个端点，直接返回
                        return Ok(UpstreamCallResult {
                            response: resp,
                            fallback_attempts,
                        });
                    }
                    Err(e) => {
                        let msg = format!("HTTP request failed at {}: {}", base_url, e);
                        tracing::debug!("{}", msg);
                        // [NEW] 记录网络错误的降级尝试
                        fallback_attempts.push(FallbackAttemptLog {
                            endpoint_url: url.clone(),
                            status: None,
                            error: msg.clone(),
                        });
                        last_err = Some(msg);

                        // 如果是最后一个端点，退出循环
                        if !has_next {
                            break;
                        }
                        continue;
                    }
                }
            }

            // 处理降级逻辑
            if should_retry_without_header {
                headers.remove("x-goog-user-project");
                has_triggered_downgrade = true;
                // 重启外层 loop，从第一个端点再次尝试
                continue;
            }

            // 如果没有触发降级且所有端点都尝试过，返回最后的错误
            return Err(last_err.unwrap_or_else(|| "All endpoints failed".to_string()));
        }
    }

    /// 获取可用模型列表
    ///
    /// 获取远端模型列表，支持多端点自动 Fallback
    #[allow(dead_code)] // API ready for future model discovery feature
    pub async fn fetch_available_models(
        &self,
        access_token: &str,
        account_id: Option<&str>,
    ) -> Result<Value, String> {
        // 复用 call_v1_internal，然后解析 JSON
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

    /// 辅助型 v1internal 调用（非用户请求路径，例如后台上下文摘要）。
    ///
    /// 为什么需要它：这类调用点位于 mapper / 辅助函数里，历史上拿不到 `AppState.upstream`
    /// 便手写 URL —— 结果形状与 host 双双漂移
    /// （`{host}/v1internal/projects/{p}/locations/global/models/{m}:generateContent`
    /// 在 daily / sandbox / prod 三个 host 上一律返回 HTML 404，导致该功能从未成功过）。
    ///
    /// 这里复用与主请求路径**完全相同**的四个来源，避免再次漂移：
    /// - 客户端 `self.get_client(account_id)` —— 同一套按账号代理池选择与 `client_cache`，
    ///   并随上游代理热更新一并生效（见 `rebuild_default_client` / `clear_client_cache`）。
    ///   账号绑定专属代理时辅助请求同样走它，不会从真实 IP 泄漏出去。
    /// - 端点顺序 `V1_INTERNAL_BASE_URL_FALLBACKS`（Daily → Sandbox → Prod）
    /// - URL 形状 `Self::build_url`（`{base}:{method}`，模型名放在 body 里）
    /// - 回退判定 `Self::should_try_next_endpoint`（408 / 404 / 5xx 才换端点）
    ///
    /// 与 `call_v1_internal` 的差异（有意为之）：不注入请求级 Header
    /// （`x-vscode-sessionid` 等）。超时由调用方用 `timeout_secs` 指定 ——
    /// 客户端默认 600s，对摘要过长，故显式收紧。
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

                // 与主请求路径同一判定：仅 408 / 404 / 5xx 换端点；
                // 其余状态（如 400）说明请求本身有问题，直接终止，不做三倍重试。
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
