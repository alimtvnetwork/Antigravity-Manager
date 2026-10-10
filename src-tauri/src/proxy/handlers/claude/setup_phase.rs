use super::*;
use crate::proxy::common::client_adapter::ClientAdapter;
use crate::proxy::config::DebugLoggingConfig;
use std::sync::Arc;

/// Output of the request preprocessing phase, extracted from `handle_messages`.
/// Pure code move: no logic changes.
pub(crate) struct SetupOutput {
    pub trace_id: String,
    pub debug_cfg: DebugLoggingConfig,
    pub client_adapter: Option<Arc<dyn ClientAdapter>>,
    pub request: crate::proxy::mappers::claude::models::ClaudeRequest,
    pub original_body: Value,
    pub compression_level: String,
    pub scaling_enabled: bool,
    pub threshold_l1: f32,
    pub threshold_l3: f32,
    pub clean_ms: f64,
}

/// Preprocess the inbound Claude request: trace setup, client-adapter detection,
/// z.ai dispatch decision, request parsing, thinking/variant normalization,
/// message cleanup, compression, warmup interception and z.ai forwarding.
///
/// Returns `Err(Response)` for the early-exit paths (bad request / warmup /
/// z.ai passthrough) that previously did `return` directly from `handle_messages`.
/// Pure code move: no logic changes.
pub(crate) async fn preprocess_request(
    state: &AppState,
    headers: &HeaderMap,
    body: Value,
    upstream_recorder: &Option<
        axum::extract::Extension<crate::proxy::monitor::UpstreamRequestBodyHolder>,
    >,
) -> Result<SetupOutput, Response> {
    // [FIX] 保存原始请求体的完整副本，用于日志记录
    // 这确保了即使结构体定义遗漏字段，日志也能完整记录所有参数
    let original_body = body.clone();

    tracing::debug!(
        "handle_messages called. Body JSON len: {}",
        body.to_string().len()
    );

    // 生成随机 Trace ID 用户追踪
    let trace_id: String =
        rand::Rng::sample_iter(rand::thread_rng(), &rand::distributions::Alphanumeric)
            .take(6)
            .map(char::from)
            .collect::<String>()
            .to_lowercase();
    let debug_cfg = state.debug_logging.read().await.clone();

    // [NEW] Detect Client Adapter
    // 检查是否有匹配的客户端适配器（如 opencode）
    let client_adapter = CLIENT_ADAPTERS
        .iter()
        .find(|a| a.matches(&headers))
        .cloned();
    if let Some(_adapter) = &client_adapter {
        tracing::debug!(
            "[{}] Client Adapter detected: Applying custom strategies",
            trace_id
        );
    }

    // Decide whether this request should be handled by z.ai (Anthropic passthrough) or the existing Google flow.
    let zai = state.zai.read().await.clone();
    let zai_enabled =
        zai.enabled && !matches!(zai.dispatch_mode, crate::proxy::ZaiDispatchMode::Off);
    let google_accounts = state.token_manager.len();

    // [CRITICAL REFACTOR] 优先解析请求以获取模型信息(用于智能兜底判断)
    let mut request: crate::proxy::mappers::claude::models::ClaudeRequest =
        match serde_json::from_value(body.clone()) {
            Ok(r) => r,
            Err(e) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    Json(json!({
                        "type": "error",
                        "error": {
                            "type": "invalid_request_error",
                            "message": format!("Invalid request body: {}", e)
                        }
                    })),
                )
                    .into_response());
            }
        };

    // [Variant] Resolve canonical model + variant → real model + real params.
    let model_lower = request.model.to_lowercase();
    let is_v3_or_above = model_specs::is_gemini_v3_or_above(&request.model);
    let is_explicit_tier_model = model_lower.ends_with("-high")
        || model_lower.ends_with("-medium")
        || model_lower.ends_with("-low")
        || model_lower.ends_with("-extra-low");

    let thinking_hint = extract_thinking_hint(&original_body);
    let tb_config = crate::proxy::config::get_thinking_budget_config();
    let is_client_control =
        tb_config.control_source == crate::proxy::config::ThinkingControlSource::Client;

    let client_switch = crate::proxy::pipeline::extract_client_thinking_switch(
        request.thinking.as_ref().map(|t| t.type_.as_str()),
        request
            .thinking
            .as_ref()
            .and_then(|t| t.budget_tokens.map(|b| b as u64)),
        request
            .output_config
            .as_ref()
            .and_then(|c| c.effort.as_deref())
            .or_else(|| request.thinking.as_ref().and_then(|t| t.effort.as_deref())),
    );
    let client_disabled = client_switch.is_disabled();

    let raw_client_budget = request.thinking.as_ref().and_then(|t| t.budget_tokens);

    // [USER RULE] 对于 Gemini >= 3 或显式指定档位的模型，进站阶段彻底忽略客户端思考与预算参数，绝不被客户端 1024 或 low 污染
    if is_client_control {
        if client_disabled {
            request.thinking = Some(crate::proxy::mappers::claude::models::ThinkingConfig {
                type_: "disabled".to_string(),
                budget_tokens: Some(0),
                effort: None,
            });
        }
    } else if is_v3_or_above || is_explicit_tier_model {
        // 无论客户端未提供 thinking，或者传了 disabled，只要是 3+ 或显式模型，强制矫正为 enabled，清理客户端 budget_tokens
        let effort_in_thinking = request.thinking.as_ref().and_then(|t| t.effort.clone());
        request.thinking = Some(crate::proxy::mappers::claude::models::ThinkingConfig {
            type_: "enabled".to_string(),
            budget_tokens: None,
            effort: effort_in_thinking,
        });
    } else {
        // Fall back to model default budget before account resolution
        let temp_cap = model_specs::get_thinking_budget(&request.model, None);
        apply_thinking_hints(&mut request, &thinking_hint, &trace_id, temp_cap);
    }

    // [USER RULE] 对显式指定档位或 Gemini >= 3 的思考模型，进站阶段彻底忽略客户端思考预算，绝不参与档位推断
    let effective_budget_hint = if !is_client_control && (is_explicit_tier_model || is_v3_or_above)
    {
        None
    } else {
        original_body
            .get("thinking")
            .and_then(|t| t.get("budget_tokens"))
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
    };

    let effort_hint = request
        .output_config
        .as_ref()
        .and_then(|config| config.effort.clone())
        .or_else(|| request.thinking.as_ref().and_then(|t| t.effort.clone()))
        .or_else(|| thinking_hint.level.clone());
    let effort_tier =
        crate::proxy::common::variant_mapping::tier_from_effort(effort_hint.as_deref());
    let canonical_model = request.model.clone();
    if let Some(spec) = apply_variant(&mut request, effort_tier, effective_budget_hint) {
        if is_client_control && client_disabled {
            request.thinking = Some(crate::proxy::mappers::claude::models::ThinkingConfig {
                type_: "disabled".to_string(),
                budget_tokens: Some(0),
                effort: None,
            });
        } else if is_client_control && raw_client_budget.is_some() {
            request.thinking = Some(crate::proxy::mappers::claude::models::ThinkingConfig {
                type_: "enabled".to_string(),
                budget_tokens: raw_client_budget,
                effort: effort_hint.clone(),
            });
        } else if is_client_control {
            // [CRITICAL FIX] 客户端控制模式下，客户端未传数字预算（全缺省或仅传等级）
            // 严禁保留 apply_variant 内部赋予的 spec.thinking_budget (4000)！保持真实客户端状态
            if let Some(ref mut t) = request.thinking {
                t.budget_tokens = None;
            }
        }
        tracing::info!(
            "[{}] [Variant] canonical='{}' effort_hint={:?} budget_hint={:?} -> real_model='{}' budget={} maxOut={}",
            trace_id, canonical_model, effort_hint, effective_budget_hint, spec.id, spec.thinking_budget, spec.max_output_tokens
        );
    }

    if debug_logger::is_enabled(&debug_cfg) {
        // [FIX] 使用原始 body 副本记录日志，确保不丢失任何字段
        let original_payload = json!({
            "kind": "original_request",
            "protocol": "anthropic",
            "trace_id": trace_id,
            "original_model": request.model,
            "request": crate::proxy::payload_audit::reorder_payload_fields(&original_body),  // 原始请求体（字段按关注度重排），不是结构体序列化
        });
        debug_logger::write_debug_payload(
            &debug_cfg,
            Some(&trace_id),
            "original_request",
            &original_payload,
        )
        .await;
    }

    // [Issue #703 Fix] 智能兜底判断:需要归一化模型名用于配额保护检查
    let normalized_model =
        crate::proxy::common::model_mapping::normalize_to_standard_id(&request.model)
            .unwrap_or_else(|| request.model.clone());

    let use_zai = if !zai_enabled {
        false
    } else {
        match zai.dispatch_mode {
            crate::proxy::ZaiDispatchMode::Off => false,
            crate::proxy::ZaiDispatchMode::Exclusive => true,
            crate::proxy::ZaiDispatchMode::Fallback => {
                if google_accounts == 0 {
                    // 没有 Google 账号,使用兜底
                    tracing::info!(
                        "[{}] No Google accounts available, using fallback provider",
                        trace_id
                    );
                    true
                } else {
                    // [Issue #703 Fix] 智能判断:检查是否有可用的 Google 账号
                    let has_available = state
                        .token_manager
                        .has_available_account("claude", &normalized_model)
                        .await;
                    if !has_available {
                        tracing::info!(
                            "[{}] All Google accounts unavailable (rate-limited or quota-protected for {}), using fallback provider",
                            trace_id,
                            request.model
                        );
                    }
                    !has_available
                }
            }
            crate::proxy::ZaiDispatchMode::Pooled => {
                // Treat z.ai as exactly one extra slot in the pool.
                // No strict guarantees: it may get 0 requests if selection never hits.
                let total = google_accounts.saturating_add(1).max(1);
                let slot = state.provider_rr.fetch_add(1, Ordering::Relaxed) % total;
                slot == 0
            }
        }
    };

    // [Stage 1 Timing] Initial session clean timing
    let clean_start = std::time::Instant::now();

    // [CRITICAL FIX] 预先清理所有消息中的 cache_control 字段 (Issue #744)
    // 必须在序列化之前处理，以确保 z.ai 和 Google Flow 都不受历史消息缓存标记干扰
    clean_cache_control_from_messages(&mut request.messages);

    // [FIX #813] 合并连续的同角色消息 (Consecutive User Messages)
    // 这对于 z.ai (Anthropic 直接转发) 路径至关重要，因为原始结构必须符合协议
    merge_consecutive_messages(&mut request.messages);

    // Get model family for signature validation
    let target_family = if use_zai {
        Some("claude")
    } else {
        let mapped_model =
            crate::proxy::common::model_mapping::map_claude_model_to_gemini(&request.model);
        if mapped_model.contains("gemini") {
            Some("gemini")
        } else {
            Some("claude")
        }
    };

    // [CRITICAL FIX] 过滤并修复 Thinking 块签名 (Enhanced with family check)
    filter_invalid_thinking_blocks_with_family(&mut request.messages, target_family);

    // [FIX Prompt-Cache] Do NOT inject synthetic messages (close_tool_loop_for_thinking).
    // InboundThinkingPipeline and ThinkingStore automatically restore real thought blocks
    // and sentinel signatures without mutating prompt history or breaking prompt caching.

    let experimental_cfg = state.experimental.read().await;
    let compression_level = if experimental_cfg.compression_level == "disabled" {
        if experimental_cfg.enable_usage_scaling {
            "high".to_string()
        } else {
            "disabled".to_string()
        }
    } else {
        experimental_cfg.compression_level.clone()
    };

    if compression_level != "disabled" {
        // [ACC-P RTK] Low, Medium, High 等级均对传入的工具返回日志执行静态 RTK 去噪折叠
        for msg in &mut request.messages {
            crate::proxy::mappers::context_manager::ContextManager::clean_tool_message(msg);
        }

        // [ACC-P Caveman] Medium, High 等级对除最近 4 条（~2轮）以外的旧对话常驻执行 Caveman 提纯
        if compression_level == "medium" || compression_level == "high" {
            let total_msgs = request.messages.len();
            let start_protection_idx = total_msgs.saturating_sub(4);
            for (i, msg) in request.messages.iter_mut().enumerate() {
                if i >= start_protection_idx {
                    continue;
                }
                if msg.role == "user" || msg.role == "assistant" {
                    match &mut msg.content {
                        crate::proxy::mappers::claude::models::MessageContent::String(s) => {
                            let cleaned =
                                crate::proxy::mappers::caveman_cleaner::CavemanCleaner::clean(s);
                            if cleaned != *s {
                                *s = cleaned;
                            }
                        }
                        crate::proxy::mappers::claude::models::MessageContent::Array(blocks) => {
                            for block in blocks {
                                if let crate::proxy::mappers::claude::models::ContentBlock::Text {
                                    text,
                                } = block
                                {
                                    let cleaned = crate::proxy::mappers::caveman_cleaner::CavemanCleaner::clean(text);
                                    if cleaned != *text {
                                        *text = cleaned;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // ===== [Issue #467 Fix] 拦截 Claude Code Warmup 请求 =====
    // Claude Code 会每 10 秒发送一次 warmup 请求来保持连接热身，
    // 这些请求会消耗大量配额。检测到 warmup 请求后直接返回模拟响应。
    if is_warmup_request(&request) {
        tracing::info!(
            "[{}] 🔥 拦截 Warmup 请求，返回模拟响应（节省配额）",
            trace_id
        );
        return Err(create_warmup_response(&request, request.stream));
    }

    if use_zai {
        // 重新序列化修复后的请求体
        let mut new_body = match serde_json::to_value(&request) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!("Failed to serialize fixed request for z.ai: {}", e);
                return Err(StatusCode::INTERNAL_SERVER_ERROR.into_response());
            }
        };

        // Inject cache_control into the XML summary message if it is a Forked session
        inject_cache_control_to_forked_summary(&mut new_body);

        if let Some(ref recorder) = upstream_recorder {
            recorder.set_value(&new_body);
        }

        return Err(
            crate::proxy::providers::zai_anthropic::forward_anthropic_json(
                &state,
                axum::http::Method::POST,
                "/v1/messages",
                &headers,
                new_body,
                request.messages.len(), // [NEW v4.0.0] Pass message count
            )
            .await,
        );
    }

    // Google Flow 继续使用 request 对象
    // (后续代码不需要再次 filter_invalid_thinking_blocks)

    // [NEW] 获取上下文控制配置
    let experimental = state.experimental.read().await;
    let scaling_enabled = experimental.enable_usage_scaling;
    let threshold_l1 = experimental.context_compression_threshold_l1;
    let threshold_l3 = experimental.context_compression_threshold_l3;

    let clean_ms = clean_start.elapsed().as_micros() as f64 / 1000.0;

    Ok(SetupOutput {
        trace_id,
        debug_cfg,
        client_adapter,
        request,
        original_body,
        compression_level,
        scaling_enabled,
        threshold_l1,
        threshold_l3,
        clean_ms,
    })
}
