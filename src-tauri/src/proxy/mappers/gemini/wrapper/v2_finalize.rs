// V2 finalize phase (split from wrapper.rs).
use serde_json::{json, Value};

pub(crate) fn phase_finalize(
    mut inner_request: serde_json::Value,
    final_model_name: &str,
    original_model: &str,
    body: &serde_json::Value,
    token: Option<&crate::proxy::token_manager::ProxyToken>,
    account_id: Option<&str>,
    session_id: Option<&str>,
    project_id: &str,
    is_under_v3: bool,
) -> serde_json::Value {
    // [NEW] Three-tier limit on maxOutputTokens by model (Dynamic > Static Default > 65535)
    // Fix: 131072 sent by clients such as gemini-cli exceeds the upper limit supported by some models, causing v1internal to return 400 INVALID_ARGUMENT
    {
        let final_cap = crate::proxy::model_specs::get_max_output_tokens(final_model_name, token);
        let gen_config = inner_request
            .as_object_mut()
            .unwrap()
            .entry("generationConfig")
            .or_insert(serde_json::json!({}))
            .as_object_mut()
            .unwrap();
        if let Some(current) = gen_config.get("maxOutputTokens").and_then(|v| v.as_u64()) {
            if current > final_cap {
                tracing::debug!(
                    "[Gemini-Wrap] Capped maxOutputTokens from {} to {} for model {}",
                    current,
                    final_cap,
                    final_model_name
                );
                gen_config.insert("maxOutputTokens".to_string(), serde_json::json!(final_cap));
            }
        }
        if is_under_v3 {
            gen_config.remove("thinkingConfig");
        }
    }

    // This caused upstream to return empty/invalid responses, leading to 'NoneType' object has no attribute 'strip' in Python clients.
    // relying on upstream defaults or user provided values is safer.

    // Extract tools list for network probing (Gemini style may be nested)
    let tools_val: Option<Vec<Value>> = inner_request
        .get("tools")
        .and_then(|t| t.as_array())
        .map(|arr| arr.clone());

    // [FIX] Extract OpenAI-compatible image parameters from root (for gemini-3-pro-image)
    let size = body.get("size").and_then(|v| v.as_str());
    let quality = body.get("quality").and_then(|v| v.as_str());
    let image_size = body.get("imageSize").and_then(|v| v.as_str()); // [NEW] Direct imageSize support

    // Use shared grounding/config logic
    let config = crate::proxy::mappers::common_utils::resolve_request_config(
        original_model,
        final_model_name,
        &tools_val,
        size,       // [FIX] Pass size parameter
        quality,    // [FIX] Pass quality parameter
        image_size, // [NEW] Pass direct imageSize parameter
        Some(body), // [NEW] Pass request body for imageConfig parsing
    );

    // Clean tool declarations (remove forbidden Schema fields like multipleOf, and remove redundant search decls)
    if let Some(tools) = inner_request.get_mut("tools") {
        if let Some(tools_arr) = tools.as_array_mut() {
            for tool in tools_arr {
                if let Some(decls) = tool.get_mut("functionDeclarations") {
                    if let Some(decls_arr) = decls.as_array_mut() {
                        // Clean Schema: If parametersJsonSchema exists, normalize it to parameters
                        for decl in decls_arr.iter_mut() {
                            // Detect and convert field names
                            if let Some(decl_obj) = decl.as_object_mut() {
                                // If parametersJsonSchema exists, rename it to parameters
                                if let Some(params_json_schema) =
                                    decl_obj.remove("parametersJsonSchema")
                                {
                                    let mut params = params_json_schema;
                                    crate::proxy::common::json_schema::clean_json_schema(
                                        &mut params,
                                    );
                                    crate::proxy::mappers::openai::request::enforce_uppercase_types(
                                        &mut params,
                                    );
                                    decl_obj.insert("parameters".to_string(), params);
                                } else if let Some(params) = decl_obj.get_mut("parameters") {
                                    // Standard parameters field
                                    crate::proxy::common::json_schema::clean_json_schema(params);
                                    crate::proxy::mappers::openai::request::enforce_uppercase_types(
                                        params,
                                    );
                                }
                            }
                        }
                        // [CACHE] Stable lexicographic sorting by function name to ensure complete protocol tool schema byte consistency
                        decls_arr.sort_by(|a, b| {
                            let name_a = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
                            let name_b = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
                            name_a.cmp(name_b)
                        });
                    }
                }
            }
        }
    }

    tracing::debug!(
        "[Debug] Gemini Wrap: original='{}', mapped='{}', final='{}', type='{}'",
        original_model,
        final_model_name,
        config.final_model,
        config.request_type
    );

    // Inject googleSearch tool if needed (stacking alongside existing tools)
    if config.inject_google_search {
        if config.request_type == "web_search" {
            if let Some(obj) = inner_request.as_object_mut() {
                let tools_entry = obj.entry("tools").or_insert_with(|| json!([]));
                if let Some(tools_arr) = tools_entry.as_array_mut() {
                    let has_functions = tools_arr.iter().any(|t| {
                        t.as_object().map_or(false, |o| {
                            o.contains_key("functionDeclarations")
                                || o.contains_key("function_declarations")
                        })
                    });
                    if !has_functions {
                        // Clean up existing googleSearch
                        tools_arr.retain(|t| {
                            if let Some(o) = t.as_object() {
                                !(o.contains_key("googleSearch")
                                    || o.contains_key("google_search")
                                    || o.contains_key("googleSearchRetrieval"))
                            } else {
                                true
                            }
                        });
                        tools_arr.push(json!({
                            "googleSearch": {
                                "enhancedContent": {
                                    "imageSearch": {
                                        "maxResultCount": 5
                                    }
                                }
                            }
                        }));
                    }
                }
            }
        } else {
            crate::proxy::mappers::common_utils::inject_google_search_tool(
                &mut inner_request,
                Some(&config.final_model),
            );
        }
    }

    // Inject imageConfig if present (for image generation models)
    if let Some(image_config) = config.image_config {
        if let Some(obj) = inner_request.as_object_mut() {
            // 1. Filter tools: remove tools for image gen
            obj.remove("tools");

            // 2. Remove systemInstruction (image generation does not support system prompts)
            obj.remove("systemInstruction");

            // [FIX] Ensure 'role' field exists for all contents (Native clients might omit it)
            if let Some(contents) = obj.get_mut("contents").and_then(|c| c.as_array_mut()) {
                for content in contents {
                    if let Some(c_obj) = content.as_object_mut() {
                        if !c_obj.contains_key("role") {
                            c_obj.insert("role".to_string(), json!("user"));
                        }
                    }
                }
            }

            // 3. Clean generationConfig (remove responseMimeType, responseModalities etc.)
            let gen_config = obj.entry("generationConfig").or_insert_with(|| json!({}));
            if let Some(gen_obj) = gen_config.as_object_mut() {
                // [NEW] Determine whether to retain thinkingConfig based on global configuration
                let image_thinking_mode = crate::proxy::config::get_image_thinking_mode();
                tracing::debug!("[Gemini-Wrap] Image thinking mode: {}", image_thinking_mode);

                if image_thinking_mode == "disabled" {
                    // [FIX] Explicitly disable thinking instead of just removing the config
                    // Removing it might cause the model to fallback to default (which might be ON)
                    gen_obj.insert(
                        "thinkingConfig".to_string(),
                        json!({
                            "includeThoughts": false
                        }),
                    );
                    tracing::debug!(
                        "[Gemini-Wrap] Image thinking mode disabled: set includeThoughts=false"
                    );
                }

                gen_obj.remove("responseMimeType");
                gen_obj.remove("responseModalities"); // Cherry Studio sends this, might conflict
                gen_obj.insert("imageConfig".to_string(), image_config);
            }
        }
    } else {
        // [FIX] Completely remove any preset search Bot prompt word injection such as web_search, and keep the client's original prompt completely pure and transparent.
        // Additional injection is only performed when a user-defined global system prompt word is configured.
        let global_prompt_config = crate::proxy::config::get_global_system_prompt();

        // Check if systemInstruction already exists
        if let Some(system_instruction) = inner_request.get_mut("systemInstruction") {
            // Completion role: user
            if let Some(obj) = system_instruction.as_object_mut() {
                if !obj.contains_key("role") {
                    obj.insert("role".to_string(), json!("user"));
                }
            }

            if let Some(parts) = system_instruction.get_mut("parts") {
                if let Some(parts_array) = parts.as_array_mut() {
                    // Inject global system prompt words (remove duplication + line break isolation, do not include official identity)
                    if global_prompt_config.enabled
                        && !global_prompt_config.content.trim().is_empty()
                    {
                        let prompt_content = global_prompt_config.content.trim();
                        let already_has_global = parts_array.iter().any(|p| {
                            p.get("text")
                                .and_then(|t| t.as_str())
                                .map(|s| s.contains(prompt_content))
                                .unwrap_or(false)
                        });

                        if !already_has_global {
                            let formatted = format!("{}\n\n", prompt_content);
                            parts_array.push(json!({"text": formatted}));
                        }
                    }
                }
            }
        } else {
            // No systemInstruction, only created when global prompt words are enabled
            if global_prompt_config.enabled && !global_prompt_config.content.trim().is_empty() {
                inner_request["systemInstruction"] = json!({
                    "role": "user",
                    "parts": [{"text": format!("{}\n\n", global_prompt_config.content.trim())}]
                });
            }
        }
    }

    // [REMOVED v4.8.2] toolConfig / tool_config double writing has been removed: the official Antigravity message does not carry this field,
    // And the double copy of camelCase and snake_case will write a pair of contradictory configurations (VALIDATED). Has been uniformly removed from protocol-independent nodes.

    // [ADDED v4.1.24] Inject stable sessionId based on account
    // [FIX session-1M] Mix conversation fingerprints and algebra, isolate server sessions from different conversations, bump self-healing after 1M accumulated errors
    if let Some(account_id_str) = account_id {
        let fingerprint = session_id.unwrap_or("default");
        let generation = crate::proxy::common::session::current_bump(account_id_str, fingerprint);
        inner_request["sessionId"] = json!(crate::proxy::common::session::derive_session_scoped(
            account_id_str,
            fingerprint,
            generation
        ));
    }

    let sid = session_id.unwrap_or("default");

    // [NEW] 1. requestId: Official 5-segment form, shared by three adapters.
    // Contains unixMs to ensure idempotent isolation (avoiding retry hitting the last 429 / old cache), and the shape is consistent with other entries.
    let step = inner_request
        .get("contents")
        .and_then(|c| c.as_array())
        .map(|a| a.len() as u64)
        .unwrap_or(0);
    let official_request_id =
        crate::proxy::mappers::common_utils::build_official_request_id(sid, step);

    // [NEW] 2. Dynamic userAgent simulation (supports jetski)
    // Enterprise/GCP accounts (non-gmail emails) use jetski fingerprints in the official Go Worker.
    // The judgment is provided by common_utils - the three adapters must be shared, otherwise the fingerprint of the same account will drift.
    let (official_user_agent, official_ide_type) =
        crate::proxy::mappers::common_utils::resolve_official_fingerprint(token);

    // [NEW] If it is a loadCodeAssist request, inject the metadata field to align with the official
    if final_model_name == "loadCodeAssist" || inner_request.get("metadata").is_some() {
        let metadata = inner_request
            .as_object_mut()
            .unwrap()
            .entry("metadata")
            .or_insert(json!({}));
        if let Some(m_obj) = metadata.as_object_mut() {
            if m_obj.get("ideType").is_none() {
                m_obj.insert("ideType".to_string(), json!(official_ide_type));
            }
        }
    }

    // [NEW] 3. Dynamically determine whether agent requestType and enabledCreditTypes are needed
    // Align with the native design of the official language service: enter agent mode only when tool definitions (tools) exist or contain tool calling context.
    // Ordinary Q&A and plain text completion do not inject requestType: "agent" to avoid the overload current limit of the Agent resource pool in Google's backend.
    let has_tools = inner_request
        .get("tools")
        .and_then(|t| t.as_array())
        .map(|arr| !arr.is_empty())
        .unwrap_or(false);
    let has_tool_interactions = inner_request
        .get("contents")
        .map(crate::proxy::mappers::common_utils::contents_has_tool_interactions)
        .unwrap_or(false);

    let is_agent_request =
        config.request_type != "image_gen" && (has_tools || has_tool_interactions);

    // [CACHE] Unified entrust the inbound pipeline to perform prefix topology standardization and alignment (Pipeline First core normalization)
    crate::proxy::pipeline::InboundThinkingPipeline::align_google_request_prefix_topology(
        &mut inner_request,
    );
    let reordered_inner = inner_request;

    let mut final_request_obj = json!({
        "project": project_id,
        "request": reordered_inner,
        "model": config.final_model,
        "userAgent": official_user_agent,
        // [CACHE] requestId is moved to the end to avoid dynamic values breaking prefix byte consistency
        "requestId": official_request_id,
    });

    if config.request_type == "image_gen" {
        final_request_obj["requestType"] = json!("image_gen");
    } else if is_agent_request {
        final_request_obj["requestType"] = json!("agent");
        if let Some(obj) = final_request_obj.as_object_mut() {
            // Force injection of Google One AI credit limit support label
            obj.insert("enabledCreditTypes".to_string(), json!(["GOOGLE_ONE_AI"]));
        }
    }

    final_request_obj
}
