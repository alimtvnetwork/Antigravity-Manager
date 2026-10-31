// Tools phase (split from request.rs).
// OpenAI → Gemini request conversion
use super::super::models::*;
use crate::proxy::model_specs;
use crate::proxy::token_manager::ProxyToken;

use serde_json::{json, Value};

use super::helpers::flatten_tools;
use super::session_setup::SetupState;
use crate::proxy::mappers::openai::request::transform::enforce_uppercase_types;

pub(crate) fn phase_tools(
    request: &super::super::models::OpenAIRequest,
    mapped_model: &str,
    token: Option<&crate::proxy::token_manager::ProxyToken>,
    setup: &super::session_setup::SetupState,
    system_instructions: &[String],
    inner_request: &mut serde_json::Value,
) {
    // 4. Handle Tools (Merged Cleaning)
    let _is_codex_style = request.model.contains("codex")
        || request.model.contains("realtime")
        || request.instructions.is_some()
        || request.input.is_some();

    let mut function_declarations: Vec<Value> = Vec::new();

    // [CACHE:L2] Calculate the hash of the original tools and check the Layer 2 cache
    // If hit, all tools processing logic will be skipped and processed tools will be reused across sessions.
    let mut tools_layer_hit = false;
    let tools_raw_hash = if let Some(ref original_tools) = request.tools {
        let raw_json = serde_json::to_string(original_tools).unwrap_or_default();
        if !raw_json.is_empty() {
            let key = crate::proxy::cache_manager::CacheManager::compute_tools_key(&format!(
                "pure_tools_v3:{raw_json}"
            ));
            let cm = crate::proxy::cache_manager::global_cache_manager();
            if let Some(cached_json) = cm.lookup_tools(&key) {
                if let Ok(parsed) = serde_json::from_str::<Vec<Value>>(&cached_json) {
                    function_declarations = parsed;
                    tools_layer_hit = true;
                    tracing::debug!(
                        "[Cache-Opt:L2-Tools] HIT hash={} declarations={}",
                        &key[..key.len().min(16)],
                        function_declarations.len()
                    );
                }
            }
            Some(key)
        } else {
            None
        }
    } else {
        None
    };

    if !tools_layer_hit {
        if let Some(original_tools) = &request.tools {
            let tools = flatten_tools(original_tools);
            for tool in tools.iter() {
                let mut gemini_func = if let Some(func) = tool.get("function") {
                    func.clone()
                } else {
                    let mut func = tool.clone();
                    // [FIX] If "name" does not exist before removing "type", extract "type" as the name
                    if func.get("name").is_none() {
                        let tool_type_opt = func
                            .get("type")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string());
                        if let Some(tool_type) = tool_type_opt {
                            if let Some(obj) = func.as_object_mut() {
                                obj.insert("name".to_string(), json!(tool_type));
                            }
                        }
                    }
                    if let Some(obj) = func.as_object_mut() {
                        obj.remove("type");
                        obj.remove("strict");
                        obj.remove("additionalProperties");
                    }
                    func
                };

                let name_opt = gemini_func
                    .get("name")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                if name_opt.is_none() {
                    // [FIX] If the tool has no name, it is treated as an invalid tool and skipped directly (prevent REQUIRED_FIELD_MISSING)
                    tracing::warn!(
                        "[OpenAI-Request] Skipping tool without name: {:?}",
                        gemini_func
                    );
                    continue;
                }

                // [NEW CRITICAL FIX] Keep legal fields at the root level of function definition and remove all illegal fields (such as type, execution, format, etc.)
                if let Some(obj) = gemini_func.as_object_mut() {
                    let mut clean_obj = serde_json::Map::new();
                    if let Some(name) = obj.get("name") {
                        clean_obj.insert("name".to_string(), name.clone());
                    }
                    if let Some(desc) = obj.get("description") {
                        clean_obj.insert("description".to_string(), desc.clone());
                    }
                    if let Some(params) = obj.get("parameters") {
                        clean_obj.insert("parameters".to_string(), params.clone());
                    }
                    *obj = clean_obj;
                }

                if let Some(params) = gemini_func.get_mut("parameters") {
                    // [DEEP FIX] Unified call for public library cleaning: expand $ref and remove format/definitions at all levels
                    crate::proxy::common::json_schema::clean_json_schema(params);

                    // Gemini v1internal requirements:
                    // 1. type must be uppercase (OBJECT, STRING, etc.)
                    // 2. The root object must have "type": "OBJECT", and "properties" must be declared (even if it is empty), and MALFORMED_FUNCTION_CALL must be eliminated
                    if let Some(params_obj) = params.as_object_mut() {
                        if !params_obj.contains_key("type") {
                            params_obj.insert("type".to_string(), json!("OBJECT"));
                        }
                        if !params_obj.contains_key("properties") {
                            params_obj.insert("properties".to_string(), json!({}));
                        }
                    }

                    // Recursively convert type to uppercase (in line with Protobuf definition)
                    enforce_uppercase_types(params);
                } else {
                    gemini_func.as_object_mut().unwrap().insert(
                        "parameters".to_string(),
                        json!({
                            "type": "OBJECT",
                            "properties": {}
                        }),
                    );
                }
                function_declarations.push(gemini_func);
            }
        }

        // [CACHE:L2] Cache the tools after processing, and the same schema can be directly hit next time
        if let Some(ref key) = tools_raw_hash {
            if !tools_layer_hit {
                if let Ok(cached_json) = serde_json::to_string(&function_declarations) {
                    let cm = crate::proxy::cache_manager::global_cache_manager();
                    cm.cache_tools(key.clone(), cached_json);
                    tracing::debug!(
                        "[Cache-Opt:L2-Tools] INSERT hash={} declarations={}",
                        &key[..key.len().min(16)],
                        function_declarations.len()
                    );
                }
            }
        }
    } // end if !tools_layer_hit (includes the sort and insert below)

    // [CACHE] Stable sorting by function name to ensure tool schema byte consistency across requests
    function_declarations.sort_by(|a, b| {
        let name_a = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let name_b = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
        name_a.cmp(name_b)
    });

    // Removed auto-inject since we handle it above now if Codex passes it.

    if !function_declarations.is_empty() {
        inner_request["tools"] = json!([{ "functionDeclarations": function_declarations }]);

        // [REMOVED v4.8.2] toolConfig / tool_config double writing has been removed: the official Antigravity message does not carry this field,
        // And the double copy of camelCase and snake_case will write a pair of contradictory configurations (AUTO vs VALIDATED).
        // Tool calling behavior is handled uniformly in the protocol-agnostic node (align_google_request_prefix_topology).
        let _mode = None as Option<&str>;
    }

    let global_prompt_config = crate::proxy::config::get_global_system_prompt();
    let global_prompt =
        if global_prompt_config.enabled && !global_prompt_config.content.trim().is_empty() {
            Some(global_prompt_config.content.as_str())
        } else {
            None
        };
    let system_parts = super::super::context_blocks::build_system_instruction_parts(
        &system_instructions,
        global_prompt,
    );
    if !system_parts.is_empty() {
        inner_request["systemInstruction"] = json!({
            "role": "user",
            "parts": system_parts
        });
    }

    if setup.config.inject_google_search {
        crate::proxy::mappers::common_utils::inject_google_search_tool(
            inner_request,
            Some(mapped_model),
        );
        // [REMOVED v4.8.2] toolConfig / tool_config injection has been removed (officially this field is not included),
        // The googleSearch tool declaration itself has been written to tools by inject_google_search_tool.
    }

    if let Some(ref image_config) = setup.config.image_config {
        if let Some(obj) = inner_request.as_object_mut() {
            obj.remove("tools");
            obj.remove("systemInstruction");
            let gen_config = obj.entry("generationConfig").or_insert_with(|| json!({}));
            if let Some(gen_obj) = gen_config.as_object_mut() {
                // [REMOVED] thinkingConfig interception has been removed, allowing the output of thinking chains when images are generated
                // gen_obj.remove("thinkingConfig");
                gen_obj.remove("responseMimeType");
                gen_obj.remove("responseModalities");
                gen_obj.insert("imageConfig".to_string(), image_config.clone());
            }
        }
    }

    // [ADDED v4.1.24] Inject stable sessionId to align with official specifications
    // [FIX session-1M] sessionId mixed with session fingerprint and algebra:
    //   - Remain stable within the same session (retain the upstream server session cache revenue)
    //   - Use different sessionIds for different conversations to avoid sharing cumulative sessions on the same server
    //   - After detecting 1M cumulative error reports from the upstream, bump generation, new sessionId = new upstream session, and the conversation is restored without any sense.
    if let Some(t) = token {
        let generation =
            crate::proxy::common::session::current_bump(&t.account_id, &setup.session_id);
        inner_request["sessionId"] = json!(crate::proxy::common::session::derive_session_scoped(
            &t.account_id,
            &setup.session_id,
            generation
        ));
    }

    // [CACHE] Rebuild inner_request field order - stable prefix first, dynamic content last
    // [CACHE] Unified entrust the inbound pipeline to perform prefix topology standardization and alignment (Pipeline First core normalization)
    crate::proxy::pipeline::InboundThinkingPipeline::align_google_request_prefix_topology(
        inner_request,
    );
}
