use super::*;
use crate::proxy::mappers::gemini::wrapper::CONTEXT_SUMMARY_PROMPT;
use crate::proxy::handlers::claude::consts::INTERNAL_BACKGROUND_TASK;

// ===== [Helper] Synchronous Upstream Call =====
// Reusable function for making non-streaming calls to Gemini API
// Used by Layer 3 and potentially other internal operations

/// Call Gemini API synchronously and return the response text
///
/// This is used for internal operations that need to wait for a complete response,
/// such as generating summaries or other background tasks.
async fn call_gemini_sync(
    model: &str,
    request: &ClaudeRequest,
    token_manager: &Arc<crate::proxy::TokenManager>,
    upstream: &Arc<crate::proxy::upstream::client::UpstreamClient>,
    trace_id: &str,
) -> Result<String, String> {
    // Get token and transform request
    let (access_token, project_id, _, account_id, _wait_ms) = token_manager
        .get_token("gemini", false, None, model)
        .await
        .map_err(|e| format!("Failed to get account: {}", e))?;

    let token_obj = token_manager.get_token_by_id(&account_id);
    let gemini_body = crate::proxy::mappers::claude::transform_claude_request_in(
        request,
        &project_id,
        false,
        Some(account_id.as_str()),
        trace_id,
        token_obj.as_ref(),
    )
    .map_err(|e| format!("Failed to transform request: {}", e))?;

    // 走共享的辅助调用通道：复用主请求路径的客户端（含按账号代理池）、
    // 端点顺序（Daily → Sandbox → Prod）、URL 形状与回退判定。
    //
    // 历史实现把 `transform_claude_request_in` 产出的**已包装 cloudcode 信封**
    // （内含 project / request / model / userAgent / requestId / requestType）
    // POST 到 `https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent`
    // —— host 与形状双双不符。实测该 host 对 Antigravity 账号恒返回
    // 403 `ACCESS_TOKEN_SCOPE_INSUFFICIENT`（token 不具备公共 Generative Language API 权限），
    // 因此该后台摘要从未成功过。
    debug!(
        "[{}] Calling {} via cloudcode v1internal for summary",
        trace_id, model
    );

    let gemini_response = upstream
        .call_v1_internal_auxiliary(
            "generateContent",
            &access_token,
            gemini_body,
            Some(account_id.as_str()),
            SUMMARY_REQUEST_TIMEOUT_SECS,
        )
        .await?;

    // 上游返回 `{"response": {...}}` 包装体，必须先解包。
    let unwrapped = crate::proxy::mappers::gemini::unwrap_response(&gemini_response);

    // 只拼接「非思考 part」的文本：前面可能存在 `{"thought": true, "text": ""}` 的空块，
    // 直接取 `parts[0].text` 会拿到空串并把空摘要当成功。
    unwrapped
        .get("candidates")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("content"))
        .and_then(|c| c.get("parts"))
        .and_then(|parts| parts.as_array())
        .map(|parts| {
            parts
                .iter()
                .filter(|part| !crate::proxy::thinking_store::is_thought_part(part))
                .filter_map(|part| part.get("text").and_then(|t| t.as_str()))
                .collect::<String>()
        })
        .filter(|text| !text.trim().is_empty())
        .ok_or_else(|| "Failed to extract text from response".to_string())
}

// ===== [Layer 3] Fork Conversation + XML Summary =====
// This is the ultimate context compression strategy
// Borrowed from Practical-Guide-to-Context-Engineering + Claude Code official practice

/// Try to compress context by generating an XML summary and forking the conversation
///
/// This function:
/// 1. Extracts the last valid thinking signature
/// 2. Calls a cheap model (gemini-2.5-flash-lite) to generate XML summary
/// 3. Creates a new message sequence with summary as prefix
/// 4. Preserves the signature in the summary
/// 5. Returns the forked request
///
/// Returns Ok(forked_request) on success, Err(error_message) on failure
async fn try_compress_with_summary(
    original_request: &ClaudeRequest,
    trace_id: &str,
    token_manager: &Arc<crate::proxy::TokenManager>,
    upstream: &Arc<crate::proxy::upstream::client::UpstreamClient>,
) -> Result<ClaudeRequest, String> {
    info!(
        "[{}] [Layer-3] Starting context compression with XML summary",
        trace_id
    );

    // 1. Extract last valid signature
    let last_signature = ContextManager::extract_last_valid_signature(&original_request.messages);

    if let Some(ref sig) = last_signature {
        debug!(
            "[{}] [Layer-3] Extracted signature (len: {})",
            trace_id,
            sig.len()
        );
    }

    // 2. Build summary request
    let mut summary_messages = original_request.messages.clone();

    // Add instruction to include signature in summary
    let signature_instruction = if let Some(ref sig) = last_signature {
        format!("\n\n**CRITICAL**: The last thinking signature is:\n```\n{}\n```\nYou MUST include this EXACTLY in the <latest_thinking_signature> section.", sig)
    } else {
        "\n\n**Note**: No thinking signature found in history. Leave <latest_thinking_signature> empty.".to_string()
    };

    // Append summary request as the last user message
    summary_messages.push(Message {
        role: "user".to_string(),
        content: MessageContent::String(format!(
            "{}{}",
            CONTEXT_SUMMARY_PROMPT, signature_instruction
        )),
    });

    let summary_request = ClaudeRequest {
        model: INTERNAL_BACKGROUND_TASK.to_string(),
        messages: summary_messages,
        system: None,
        stream: false,
        max_tokens: Some(8000),
        temperature: Some(0.3),
        tools: None,
        thinking: None,
        metadata: None,
        top_p: None,
        top_k: None,
        output_config: None,
        size: None,
        quality: None,
    };

    debug!(
        "[{}] [Layer-3] Calling {} for summary generation",
        trace_id, INTERNAL_BACKGROUND_TASK
    );

    // 3. Call upstream using helper function (reuse existing infrastructure)
    let xml_summary = call_gemini_sync(
        INTERNAL_BACKGROUND_TASK,
        &summary_request,
        token_manager,
        upstream,
        trace_id,
    )
    .await?;

    info!(
        "[{}] [Layer-3] Generated XML summary (len: {} chars)",
        trace_id,
        xml_summary.len()
    );

    // 4. Create forked conversation with summary as prefix
    // Wrap text inside a ContentBlock::Text and attach cache_control to freeze it in upstream's Prompt Cache
    let mut forked_messages = vec![
        Message {
            role: "user".to_string(),
            content: MessageContent::Array(vec![
                crate::proxy::mappers::claude::models::ContentBlock::Text {
                    text: format!(
                        "Context has been compressed. Here is the structured summary of our conversation history:\n\n{}",
                        xml_summary
                    ),
                }
            ]),
        },
        Message {
            role: "assistant".to_string(),
            content: MessageContent::String(
                "I have reviewed the compressed context summary. I understand the current state and will continue from here.".to_string()
            ),
        },
    ];

    // 5. Append the user's latest message (if exists and is not the summary request)
    if let Some(last_msg) = original_request.messages.last() {
        if last_msg.role == "user" {
            // Check if it's not the summary instruction we just added
            if !matches!(&last_msg.content, MessageContent::String(s) if s.contains(CONTEXT_SUMMARY_PROMPT))
            {
                forked_messages.push(last_msg.clone());
            }
        }
    }

    info!(
        "[{}] [Layer-3] Fork successful: {} messages → {} messages",
        trace_id,
        original_request.messages.len(),
        forked_messages.len()
    );

    // 6. Return forked request
    Ok(ClaudeRequest {
        model: original_request.model.clone(),
        messages: forked_messages,
        system: original_request.system.clone(),
        stream: original_request.stream,
        max_tokens: original_request.max_tokens,
        temperature: original_request.temperature,
        tools: original_request.tools.clone(),
        thinking: original_request.thinking.clone(),
        metadata: original_request.metadata.clone(),
        top_p: original_request.top_p,
        top_k: original_request.top_k,
        output_config: original_request.output_config.clone(),
        size: original_request.size.clone(),
        quality: original_request.quality.clone(),
    })
}
