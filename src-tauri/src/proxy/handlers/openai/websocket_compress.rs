// Fork+Summary context compression for the completions handler.
use std::sync::Arc;

use serde_json::{json, Value};
use tracing::{debug, error, info, warn};

use crate::proxy::config::DebugLoggingConfig;
use crate::proxy::mappers::openai::OpenAIRequest;
use crate::proxy::server::UpstreamClient;
use crate::proxy::TokenManager;

const INTERNAL_BACKGROUND_TASK: &str = "gemini-2.5-flash-lite";
const CONTEXT_SUMMARY_PROMPT: &str = r#"You are a context compression specialist. Your task is to create a structured XML snapshot of the conversation history.

This snapshot will become the Agent's ONLY memory of the past. All key details, plans, errors, and user instructions MUST be preserved.

First, think through the entire history in a private <scratchpad>. Review the user's overall goal, the agent's actions, tool outputs, file modifications, and any unresolved issues. Identify every piece of information critical for future actions.

After reasoning, generate the final <state_snapshot> XML object. Information must be extremely dense. Omit any irrelevant conversational filler.

The structure MUST be as follows:

<state_snapshot>
  <overall_goal>
    <!-- Describe the user's high-level goal in one concise sentence -->
  </overall_goal>

  <technical_context>
    <!-- Tech stack: frameworks, languages, toolchain, dependency versions -->
  </technical_context>

  <file_system_state>
    <!-- List files that were created, read, modified, or deleted. Note their status -->
  </file_system_state>

  <code_changes>
    <!-- Key code snippets (preserve function signatures and important logic) -->
  </code_changes>

  <debugging_history>
    <!-- List all errors encountered, with stack traces, and how they were fixed -->
  </debugging_history>

  <current_plan>
    <!-- Step-by-step plan. Mark completed steps -->
  </current_plan>

  <user_preferences>
    <!-- User's work preferences for this project (test commands, code style, etc.) -->
  </user_preferences>

  <key_decisions>
    <!-- Critical architectural decisions and design choices -->
  </key_decisions>

  <latest_thinking_signature>
    <!-- [CRITICAL] Preserve the last valid thinking signature -->
    <!-- Format: base64-encoded signature string -->
    <!-- This MUST be copied exactly as-is, no modifications -->
  </latest_thinking_signature>
</state_snapshot>

**IMPORTANT**:
1. Code snippets must be complete, including function signatures and key logic
2. Error messages must be preserved verbatim, including line numbers and stacks
3. File paths must use absolute paths
4. The thinking signature must be copied exactly, no modifications
"#;

async fn call_openai_gemini_sync(
    model: &str,
    request: &OpenAIRequest,
    token_manager: &std::sync::Arc<crate::proxy::TokenManager>,
    upstream: &std::sync::Arc<crate::proxy::upstream::client::UpstreamClient>,
    trace_id: &str,
) -> Result<String, String> {
    let (access_token, project_id, _, account_id, _wait_ms) = token_manager
        .get_token("gemini", false, None, model)
        .await
        .map_err(|e| format!("Failed to get account: {}", e))?;

    let token_obj = token_manager.get_token_by_id(&account_id);
    let session_id = format!("bg_sid_{}", chrono::Utc::now().timestamp_subsec_millis());
    let (gemini_body, _, _, _) =
        transform_openai_request(request, &project_id, &session_id, token_obj.as_ref());

    debug!(
        "[{}] [OpenAI-BG] Calling {} via cloudcode v1internal for summary",
        trace_id, model
    );

    // 走共享的辅助调用通道。
    //
    // 历史实现把 `transform_openai_request` 产出的**已包装 cloudcode 信封**
    // （内含 project / request / model / userAgent / requestId / requestType）
    // POST 到 `https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent`
    // —— host 与形状双双不符。实测该 host 对 Antigravity 账号恒返回
    // 403 `ACCESS_TOKEN_SCOPE_INSUFFICIENT`（token 不具备公共 Generative Language API 权限），
    // 因此该后台摘要从未成功过。
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

async fn try_compress_openai_with_summary(
    original_request: &OpenAIRequest,
    trace_id: &str,
    token_manager: &std::sync::Arc<crate::proxy::TokenManager>,
    upstream: &std::sync::Arc<crate::proxy::upstream::client::UpstreamClient>,
    session_id_str: &str,
) -> Result<OpenAIRequest, String> {
    info!(
        "[{}] [Layer-3] [OpenAI] Starting context compression with XML summary",
        trace_id
    );

    let last_signature =
        crate::proxy::mappers::context_manager::ContextManager::extract_last_openai_valid_signature(
            session_id_str,
        );

    if let Some(ref sig) = last_signature {
        debug!(
            "[{}] [Layer-3] [OpenAI] Extracted signature (len: {})",
            trace_id,
            sig.len()
        );
    }

    let mut summary_messages = original_request.messages.clone();

    let signature_instruction = if let Some(ref sig) = last_signature {
        format!("\n\n**CRITICAL**: The last thinking signature is:\n```\n{}\n```\nYou MUST include this EXACTLY in the <latest_thinking_signature> section.", sig)
    } else {
        "\n\n**Note**: No thinking signature found in history. Leave <latest_thinking_signature> empty.".to_string()
    };

    summary_messages.push(OpenAIMessage {
        role: "user".to_string(),
        content: Some(
            crate::proxy::mappers::openai::models::OpenAIContent::String(format!(
                "{}{}",
                CONTEXT_SUMMARY_PROMPT, signature_instruction
            )),
        ),
        refusal: None,
        reasoning_content: None,
        signature: None,
        tool_calls: None,
        tool_call_id: None,
        name: None,
    });

    let mut summary_request = original_request.clone();
    summary_request.messages = summary_messages;
    summary_request.model = INTERNAL_BACKGROUND_TASK.to_string();
    summary_request.stream = false;
    summary_request.max_tokens = Some(8000);
    summary_request.temperature = Some(0.3);

    debug!(
        "[{}] [Layer-3] [OpenAI] Calling {} for summary generation",
        trace_id, INTERNAL_BACKGROUND_TASK
    );

    let xml_summary = call_openai_gemini_sync(
        INTERNAL_BACKGROUND_TASK,
        &summary_request,
        token_manager,
        upstream,
        trace_id,
    )
    .await?;

    info!(
        "[{}] [Layer-3] [OpenAI] Generated XML summary (len: {} chars)",
        trace_id,
        xml_summary.len()
    );

    let mut forked_messages = vec![
        OpenAIMessage {
            role: "user".to_string(),
            content: Some(crate::proxy::mappers::openai::models::OpenAIContent::String(format!(
                "Context has been compressed. Here is the structured summary of our conversation history:\n\n{}",
                xml_summary
            ))),
            refusal: None,
            reasoning_content: None,
            signature: None,
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
        OpenAIMessage {
            role: "assistant".to_string(),
            content: Some(crate::proxy::mappers::openai::models::OpenAIContent::String(
                "I have reviewed the compressed context summary. I understand the current state and will continue from here.".to_string()
            )),
            refusal: None,
            reasoning_content: None,
            signature: None,
            tool_calls: None,
            tool_call_id: None,
            name: None,
        },
    ];

    if let Some(last_msg) = original_request.messages.last() {
        if last_msg.role == "user" {
            if !matches!(&last_msg.content, Some(crate::proxy::mappers::openai::models::OpenAIContent::String(s)) if s.contains(CONTEXT_SUMMARY_PROMPT))
            {
                forked_messages.push(last_msg.clone());
            }
        }
    }

    let mut forked_request = original_request.clone();
    forked_request.messages = forked_messages;
    Ok(forked_request)
}
