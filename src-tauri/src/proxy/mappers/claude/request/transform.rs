// Request transform entry points (split from request.rs).
// Claude requested conversion (Claude → Gemini v1internal)
// Corresponds to transformClaudeRequestIn

use super::super::models::*;
use crate::proxy::mappers::signature_store::get_thought_signature; // Deprecated, kept for fallback
use crate::proxy::session_manager::SessionManager;
use serde_json::{json, Value};
use std::collections::HashMap;

use super::build_config::{build_generation_config, build_tools};
use super::build_parts::{build_google_contents, build_system_instruction};
use super::messages::{
    clean_cache_control_from_messages, deep_clean_cache_control, merge_consecutive_messages,
    sort_thinking_blocks_first,
};
use super::safety::build_safety_settings;
use super::thinking::{model_supports_thinking, should_enable_thinking_by_default};

#[derive(Default)]
pub struct TransformTiming {
    pub think_fill_micros: u64,
}

pub fn transform_claude_request_in(
    claude_req: &ClaudeRequest,
    project_id: &str,
    is_retry: bool,
    account_id: Option<&str>,
    session_id: &str,
    token: Option<&crate::proxy::token_manager::ProxyToken>,
) -> Result<Value, String> {
    transform_claude_request_in_timed(
        claude_req, project_id, is_retry, account_id, session_id, token,
    )
    .map(|(body, _)| body)
}

pub fn transform_claude_request_in_timed(
    claude_req: &ClaudeRequest,
    project_id: &str,
    is_retry: bool,
    account_id: Option<&str>,
    _session_id: &str,
    token: Option<&crate::proxy::token_manager::ProxyToken>, // [NEW] Support dynamic specifications
) -> Result<(Value, TransformTiming), String> {
    let mut timing = TransformTiming::default();
    let message_count = claude_req.messages.len();

    // [CRITICAL FIX] Pre-clean cache_control field in all messages
    // This solves the problem that clients such as the VS Code plug-in will change the cache_control field of historical messages in multi-round conversations.
    // "Extra inputs are not permitted" error caused by sending back intact
    let mut cleaned_req = claude_req.clone();

    // [JEIKCODE FROZEN SYSTEM PRINCIPLE]
    // Only the first consecutive role == "system" messages are collected into extra_system_messages.
    // Once the conversation starts (encountering the first non-system turn), any subsequent system messages that appear in the middle are strictly prohibited from being promoted to systemInstruction.
    // Otherwise, the top-level system prefix sent to Google upstream will undergo byte-level mutation and cause KV Cache to crash!
    // System messages are converted to synthetic user messages on the spot and remain in the original timeline, and are naturally merged by merge_consecutive_messages later.
    let mut extra_system_messages = Vec::new();
    let mut filtered_messages = Vec::new();
    let mut in_leading_system = true;

    for msg in cleaned_req.messages {
        if msg.role == "system" {
            if in_leading_system {
                match &msg.content {
                    MessageContent::String(text) => {
                        extra_system_messages.push(text.clone());
                    }
                    MessageContent::Array(blocks) => {
                        for block in blocks {
                            if let ContentBlock::Text { text } = block {
                                extra_system_messages.push(text.clone());
                            }
                        }
                    }
                }
            } else {
                let text = match &msg.content {
                    MessageContent::String(t) => t.clone(),
                    MessageContent::Array(blocks) => {
                        let mut joined = String::new();
                        for b in blocks {
                            if let ContentBlock::Text { text } = b {
                                if !joined.is_empty() {
                                    joined.push('\n');
                                }
                                joined.push_str(text);
                            }
                        }
                        joined
                    }
                };
                let wrapped_reminder =
                    crate::proxy::mappers::common_utils::wrap_in_system_reminder(&text);
                if !wrapped_reminder.is_empty() {
                    filtered_messages.push(Message {
                        role: "user".to_string(),
                        content: MessageContent::String(wrapped_reminder),
                    });
                }
            }
        } else {
            in_leading_system = false;
            filtered_messages.push(msg);
        }
    }
    cleaned_req.messages = filtered_messages;

    // [FIX #813] Consecutive User Messages
    // Make sure the request complies with Anthropic and Gemini's role alternation protocol
    merge_consecutive_messages(&mut cleaned_req.messages);

    clean_cache_control_from_messages(&mut cleaned_req.messages);

    // [FIX #564] Pre-sort thinking blocks to be first in assistant messages
    // This handles cases where context compression (kilo) incorrectly reorders blocks
    sort_thinking_blocks_first(&mut cleaned_req.messages);

    // [FIX #1747] If thinking is auto-enabled by model default (e.g. Opus) but no
    // ThinkingConfig was provided by the client, inject a default config with a budget
    // to prevent 'thinking requires a budget' errors from upstream APIs.
    if cleaned_req.thinking.is_none() && should_enable_thinking_by_default(&cleaned_req.model) {
        let default_budget =
            crate::proxy::model_specs::get_thinking_budget(&cleaned_req.model, token);
        tracing::info!(
            "[Thinking-Mode] Injecting default ThinkingConfig (budget={}) for model: {}",
            default_budget,
            cleaned_req.model
        );
        cleaned_req.thinking = Some(ThinkingConfig {
            type_: "enabled".to_string(),
            budget_tokens: Some(default_budget as u32),
            effort: None,
        });
    }

    let claude_req = &cleaned_req; // Subsequent use of the cleaned request

    // Prefer the handler-resolved session id (tenant + X-Session-Id) when provided.
    let session_id = if !_session_id.is_empty() {
        _session_id.to_string()
    } else {
        SessionManager::extract_session_id(claude_req)
    };
    tracing::debug!("[Claude-Request] Session ID: {}", session_id);

    // Detect whether there is a networking tool (server tool or built-in tool)
    let has_web_search_tool = claude_req
        .tools
        .as_ref()
        .map(|tools| {
            tools.iter().any(|t| {
                t.is_web_search()
                    || t.name.as_deref() == Some("google_search")
                    || t.name.as_deref() == Some("builtin_web_search")
                    || t.type_.as_deref() == Some("web_search_20250305")
                    || t.type_.as_deref() == Some("builtin_web_search")
            })
        })
        .unwrap_or(false);

    // Used to store tool_use id -> name mapping
    let mut tool_id_to_name: HashMap<String, String> = HashMap::new();

    // [New] Pre-built mapping of tool names to original Schema for subsequent parameter type correction
    let mut tool_name_to_schema = HashMap::new();
    if let Some(tools) = &claude_req.tools {
        for tool in tools {
            if let (Some(name), Some(schema)) = (&tool.name, &tool.input_schema) {
                tool_name_to_schema.insert(name.clone(), schema.clone());
            }
        }
    }

    //  Map model name (Use standard mapping)
    // [IMPROVED] Extract web search model into constants for easy maintenance
    const WEB_SEARCH_FALLBACK_MODEL: &str = "gemini-2.5-flash";

    let mapped_model =
        crate::proxy::common::model_mapping::map_claude_model_to_gemini(&claude_req.model);

    // 1. System Instruction (Transparently transmit system prompt words into chunks, and automatically filter useless billing metadata if the target is Gemini #3452)
    let system_instruction =
        build_system_instruction(&claude_req.system, &mapped_model, &extra_system_messages);

    // Convert Claude tool to Value array for detecting networking
    let tools_val: Option<Vec<Value>> = claude_req.tools.as_ref().map(|list| {
        list.iter()
            .map(|t| serde_json::to_value(t).unwrap_or(json!({})))
            .collect()
    });

    // Resolve grounding config
    let config = crate::proxy::mappers::common_utils::resolve_request_config(
        &claude_req.model,
        &mapped_model,
        &tools_val,
        claude_req.size.as_deref(),    // [NEW] Pass size parameter
        claude_req.quality.as_deref(), // [NEW] Pass quality parameter
        None,                          // [NEW] image_size
        None,                          // body
    );

    // [CRITICAL FIX] Disable dummy thought injection for Vertex AI
    // [CRITICAL FIX] Disable dummy thought injection for Vertex AI
    // Vertex AI rejects thinking blocks without valid signatures
    // Even if thinking is enabled, we should NOT inject dummy blocks for historical messages
    let allow_dummy_thought = false;

    // Check if thinking is enabled in the request
    let client_switch = crate::proxy::pipeline::extract_client_thinking_switch(
        claude_req.thinking.as_ref().map(|t| t.type_.as_str()),
        claude_req
            .thinking
            .as_ref()
            .and_then(|t| t.budget_tokens.map(|b| b as u64)),
        claude_req
            .output_config
            .as_ref()
            .and_then(|c| c.effort.as_deref())
            .or_else(|| {
                claude_req
                    .thinking
                    .as_ref()
                    .and_then(|t| t.effort.as_deref())
            }),
    );

    let tb_config = crate::proxy::config::get_thinking_budget_config();
    let is_client_control =
        tb_config.control_source == crate::proxy::config::ThinkingControlSource::Client;
    let is_client_disabled = is_client_control && client_switch.is_disabled();

    let thinking_type = claude_req.thinking.as_ref().map(|t| t.type_.as_str());
    let force_server_thinking = crate::proxy::thinking_store::any_model_forces_server_thinking(&[
        claude_req.model.as_str(),
        mapped_model.as_str(),
    ]);
    let target_model_supports_thinking = model_supports_thinking(&mapped_model);
    let is_under_v3 = crate::proxy::model_specs::is_gemini_under_v3(&mapped_model)
        || crate::proxy::model_specs::is_gemini_under_v3(&claude_req.model);
    let mut is_thinking_enabled = !is_client_disabled
        && !is_under_v3
        && (target_model_supports_thinking
            || force_server_thinking
            || thinking_type == Some("enabled")
            || thinking_type == Some("adaptive")
            || (thinking_type.is_none() && should_enable_thinking_by_default(&claude_req.model)));

    if is_thinking_enabled && !target_model_supports_thinking && !force_server_thinking {
        tracing::warn!(
            "[Thinking-Mode] Target model '{}' does not support thinking. Force disabling thinking mode.",
            mapped_model
        );
        is_thinking_enabled = false;
    }

    // Check signature requirements for function calls
    if is_thinking_enabled {
        let _global_sig = get_thought_signature();

        // Check if there are function calls in the request
        let has_function_calls = claude_req.messages.iter().any(|m| {
            if let MessageContent::Array(blocks) = &m.content {
                blocks
                    .iter()
                    .any(|b| matches!(b, ContentBlock::ToolUse { .. }))
            } else {
                false
            }
        });

        if has_function_calls {
            // [FIX #2167] Rely on ThinkingStore / sentinel injection rather than disabling thinking
            tracing::info!(
                "[Thinking-Mode] Function calls present. Relying on InboundThinkingPipeline restoration and sentinel injection."
            );
        }
    }

    // 4. Generation Config & Thinking (Pass final is_thinking_enabled)
    let generation_config = build_generation_config(
        claude_req,
        &mapped_model,
        has_web_search_tool,
        is_thinking_enabled,
        token, // [NEW] Pass token for dynamic quota
    );

    // 2. Contents (Messages)
    let contents = build_google_contents(
        &claude_req.messages,
        claude_req,
        &mut tool_id_to_name,
        &tool_name_to_schema,
        is_thinking_enabled,
        allow_dummy_thought,
        &mapped_model,
        &session_id,
        is_retry,
        &mut timing,
    )?;

    // 3. Tools
    let tools = build_tools(&claude_req.tools, has_web_search_tool, &mapped_model)?;

    // 5. Safety Settings (configurable via GEMINI_SAFETY_THRESHOLD env var)
    let safety_settings = build_safety_settings();

    // Build inner request
    let mut inner_request = json!({
        "contents": contents,
        "safetySettings": safety_settings,
    });

    if let Some(sys_inst) = system_instruction {
        inner_request["systemInstruction"] = sys_inst;
    }

    if !generation_config.is_null() {
        inner_request["generationConfig"] = generation_config;
    }

    if let Some(tools_val) = tools {
        inner_request["tools"] = tools_val;
        // [REMOVED v4.8.2] toolConfig / tool_config double writing has been removed: the official Antigravity message does not carry this field,
        // And the double copy of camelCase and snake_case will write a pair of contradictory configurations (VALIDATED). Has been uniformly removed from protocol-independent nodes.
    }

    // [PIPELINE] Unified cleaning prompt words and risk control pseudo-Headers (including [undefined] deep cleaning, see PromptSanitizer)
    crate::proxy::mappers::prompt_sanitizer::PromptSanitizer::sanitize_gemini_payload(
        &mut inner_request,
    );

    if config.inject_google_search && !has_web_search_tool {
        crate::proxy::mappers::common_utils::inject_google_search_tool(
            &mut inner_request,
            Some(&mapped_model),
        );
    }

    // Inject imageConfig if present (for image generation models)
    if let Some(image_config) = config.image_config {
        if let Some(obj) = inner_request.as_object_mut() {
            // 1. Remove tools (image generation does not support tools)
            obj.remove("tools");

            // 2. Remove systemInstruction (image generation does not support system prompts)
            obj.remove("systemInstruction");

            // 3. Clean generationConfig (remove responseMimeType, responseModalities etc.)
            let gen_config = obj.entry("generationConfig").or_insert_with(|| json!({}));
            if let Some(gen_obj) = gen_config.as_object_mut() {
                // [RESOLVE #1694] Check image thinking mode
                let image_thinking_mode = crate::proxy::config::get_image_thinking_mode();
                if image_thinking_mode == "disabled" {
                    tracing::debug!(
                        "[Claude-Request] Image thinking mode disabled: enforcing includeThoughts=false for {}",
                        mapped_model
                    );
                    gen_obj.insert(
                        "thinkingConfig".to_string(),
                        json!({
                            "includeThoughts": false
                        }),
                    );
                }

                gen_obj.remove("responseMimeType");
                gen_obj.remove("responseModalities");
                gen_obj.insert("imageConfig".to_string(), image_config);
            }
        }
    }

    // [ADDED v4.1.24] Inject stable sessionId to align with official specifications
    // [FIX session-1M] Mix conversation fingerprints and algebra, isolate server sessions from different conversations, bump self-healing after 1M accumulated errors
    if let Some(account_id) = account_id {
        let generation = crate::proxy::common::session::current_bump(account_id, &session_id);
        inner_request["sessionId"] = json!(crate::proxy::common::session::derive_session_scoped(
            account_id,
            &session_id,
            generation
        ));
    }

    // Generate requestId - official 5-segment form, shared by three adapters.
    // Must contain unixMs: the historical implementation is `agent/antigravity/ {session[:8]} / {count} `, **excluding timestamp**,
    // Retrying the same session in the same round will get the exact same ID, thus pinning to the last 429/old cache.
    let request_id = super::super::super::common_utils::build_official_request_id(
        &session_id,
        message_count as u64,
    );

    // Official client fingerprint (enterprise/GCP account is jetski) - shared by three adapters to avoid fingerprint drift
    let (official_user_agent, _official_ide_type) =
        super::super::super::common_utils::resolve_official_fingerprint(token);

    // [CACHE] Unified entrust the inbound pipeline to perform prefix topology standardization and alignment (Pipeline First core normalization)
    crate::proxy::pipeline::InboundThinkingPipeline::align_google_request_prefix_topology(
        &mut inner_request,
    );
    let reordered_inner = inner_request;

    // [NEW] Dynamically detect whether requests need to be marked as agent
    let has_tools = reordered_inner
        .get("tools")
        .and_then(|t| t.as_array())
        .map(|arr| !arr.is_empty())
        .unwrap_or(false);
    let has_tool_interactions = reordered_inner
        .get("contents")
        .map(super::super::super::common_utils::contents_has_tool_interactions)
        .unwrap_or(false);
    let is_agent_request =
        config.request_type != "image_gen" && (has_tools || has_tool_interactions);

    // Construct the final request body (the top-level key sequence is stable: project -> request -> model -> userAgent -> requestId)
    let mut body = json!({
        "project": project_id,
        "request": reordered_inner,
        "model": config.final_model,
        "userAgent": official_user_agent,
        "requestId": request_id,
    });

    if config.request_type == "image_gen" {
        body["requestType"] = json!("image_gen");
    } else if is_agent_request {
        body["requestType"] = json!("agent");
        if let Some(obj) = body.as_object_mut() {
            obj.insert("enabledCreditTypes".to_string(), json!(["GOOGLE_ONE_AI"]));
        }
    }

    // [FIX #593] Last line of defense: recursively deep clean all cache_control fields
    // Make sure the request to Antigravity does not contain any cache_control
    deep_clean_cache_control(&mut body);
    tracing::debug!("[DEBUG-593] Final deep clean complete, request ready to send");

    // [DEFENSE] Sanitize inlineData in all contents, filter or degrade empty/corrupted data
    if let Some(inner) = body.get_mut("request") {
        crate::proxy::mappers::common_utils::sanitize_gemini_payload_inline_data(inner);
    }

    Ok((body, timing))
}
