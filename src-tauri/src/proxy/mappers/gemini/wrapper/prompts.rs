// Prompt constants (split from wrapper.rs).
use crate::proxy::mappers::gemini::wrapper::request::wrap_request;
use crate::proxy::mappers::gemini::wrapper::response::unwrap_response;
use bytes::Bytes;
use serde_json::{json, Value};
use tracing::{debug, info};

pub const INTERNAL_BACKGROUND_TASK: &str = "gemini-2.5-flash-lite";

/// Timeout in seconds for Layer-3 background digest requests.
///
/// The default timeout of the upstream client is 600s, which is too long for auxiliary tasks such as "summarizing the entire conversation" -
/// Once the upstream is stuck, a background task and connection will be occupied for a long time. This is explicitly tightened to bounded values.
pub const SUMMARY_REQUEST_TIMEOUT_SECS: u64 = 180;
pub const CONTEXT_SUMMARY_PROMPT: &str = r#"You are a context compression specialist. Your task is to create a structured XML snapshot of the conversation history.

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

pub(crate) async fn try_compress_gemini_with_summary(
    original_request: &Value,
    trace_id: &str,
    token_manager: &std::sync::Arc<crate::proxy::TokenManager>,
    upstream: &std::sync::Arc<crate::proxy::upstream::client::UpstreamClient>,
    session_id_str: &str,
    project_id: &str,
    account_id: &str,
) -> Result<Value, String> {
    info!(
        "[{}] [Layer-3] [Gemini] Starting context compression with XML summary",
        trace_id
    );

    let last_signature =
        crate::proxy::mappers::context_manager::ContextManager::extract_last_openai_valid_signature(
            session_id_str,
        );

    let signature_instruction = if let Some(ref sig) = last_signature {
        format!("\n\n**CRITICAL**: The last thinking signature is:\n```\n{}\n```\nYou MUST include this EXACTLY in the <latest_thinking_signature> section.", sig)
    } else {
        "\n\n**Note**: No thinking signature found in history. Leave <latest_thinking_signature> empty.".to_string()
    };

    let mut summary_messages = original_request
        .get("contents")
        .and_then(|c| c.as_array())
        .cloned()
        .unwrap_or_default();

    summary_messages.push(json!({
        "role": "user",
        "parts": [{
            "text": format!("{}{}", CONTEXT_SUMMARY_PROMPT, signature_instruction)
        }]
    }));

    let mut summary_request = original_request.clone();
    if let Some(obj) = summary_request.as_object_mut() {
        obj.insert("contents".to_string(), json!(summary_messages));
        obj.insert("model".to_string(), json!(INTERNAL_BACKGROUND_TASK));
        obj.remove("stream");
    }

    debug!(
        "[{}] [Layer-3] [Gemini] Calling {} for summary generation",
        trace_id, INTERNAL_BACKGROUND_TASK
    );

    let token_obj = token_manager.get_token_by_id(account_id);
    let access_token = token_obj
        .as_ref()
        .map(|t| t.access_token.clone())
        .ok_or_else(|| "No access token available".to_string())?;

    let wrapped_summary_body = wrap_request(
        &summary_request,
        project_id,
        INTERNAL_BACKGROUND_TASK,
        Some(account_id),
        Some(session_id_str),
        token_obj.as_ref(),
    );

    // Take the shared auxiliary call channel: the client that reuses the main request path (including the proxy pool by account),
    // Endpoint order (Daily → Sandbox → Prod), URL shape and fallback decision.
    //
    // History is written by hand
    // `{host}/v1internal/projects/{p}/locations/global/models/{m}:generateContent`
    // And hard-code the host as sandbox - this shape is on the three hosts daily / sandbox / prod
    // Always returns HTML 404, so this function has never succeeded (spawn in the background, only logs on failure, showing silent idling).
    let gemini_response = upstream
        .call_v1_internal_auxiliary(
            "generateContent",
            &access_token,
            wrapped_summary_body,
            Some(account_id),
            SUMMARY_REQUEST_TIMEOUT_SECS,
        )
        .await?;

    // The upstream returns the `{"response": {...} }` package, which must be unpacked before the candidates can be obtained.
    let unwrapped_summary = unwrap_response(&gemini_response);

    // Only the text of the "non-thinking part" is spliced.
    //
    // The actual measured (gemini-2.5-flash-lite, 3/3) response is as follows:
    //   parts[0] = { "thought": true, "text": "" } ← Empty thinking block, ranked first
    //   parts[1] = { "text": "```xml <summary>…" } ← real summary
    // Therefore, `parts[0].text` cannot be taken (an empty string will be obtained, and the empty summary will be silently written back as a success).
    // You can't just use `parts[0]` - you must filter by the `thought` flag and then splice all visible text.
    let xml_summary = unwrapped_summary
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
        .ok_or_else(|| "Failed to extract text from response".to_string())?;

    info!(
        "[{}] [Layer-3] [Gemini] Generated XML summary (len: {} chars)",
        trace_id,
        xml_summary.len()
    );

    let forked_messages = vec![
        json!({
            "role": "user",
            "parts": [{
                "text": format!("Context has been compressed. Here is the structured summary of our conversation history:\n\n{}", xml_summary)
            }]
        }),
        json!({
            "role": "model",
            "parts": [{
                "text": "I have reviewed the compressed context summary. I understand the current state and will continue from here."
            }]
        }),
    ];

    let mut forked_request = original_request.clone();
    if let Some(obj) = forked_request.as_object_mut() {
        let mut final_msgs = forked_messages;
        if let Some(last_msg) = original_request
            .get("contents")
            .and_then(|c| c.as_array())
            .and_then(|a| a.last())
        {
            if last_msg.get("role").and_then(|r| r.as_str()) == Some("user") {
                let has_summary_inst = last_msg
                    .get("parts")
                    .and_then(|p| p.as_array())
                    .map(|arr| {
                        arr.iter().any(|part| {
                            part.get("text")
                                .and_then(|t| t.as_str())
                                .map(|t| t.contains(CONTEXT_SUMMARY_PROMPT))
                                .unwrap_or(false)
                        })
                    })
                    .unwrap_or(false);
                if !has_summary_inst {
                    final_msgs.push(last_msg.clone());
                }
            }
        }
        obj.insert("contents".to_string(), json!(final_msgs));
    }

    Ok(forked_request)
}
