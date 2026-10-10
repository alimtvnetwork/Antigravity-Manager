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
    // [NEW] 按模型对 maxOutputTokens 进行三层限额 (Dynamic > Static Default > 65535)
    // 修复: gemini-cli 等客户端发送的 131072 超过部分模型支持的上限，导致 v1internal 返回 400 INVALID_ARGUMENT
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

    // 提取 tools 列表以进行联网探测 (Gemini 风格可能是嵌套的)
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
                        // 清洗 Schema: 如果存在 parametersJsonSchema，将其标准化为 parameters
                        for decl in decls_arr.iter_mut() {
                            // 检测并转换字段名
                            if let Some(decl_obj) = decl.as_object_mut() {
                                // 如果存在 parametersJsonSchema，将其重命名为 parameters
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
                                    // 标准 parameters 字段
                                    crate::proxy::common::json_schema::clean_json_schema(params);
                                    crate::proxy::mappers::openai::request::enforce_uppercase_types(
                                        params,
                                    );
                                }
                            }
                        }
                        // [CACHE] 按 function name 稳定字典序排序，确保全协议 tool schema 字节完全一致
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
                        // 清理已存在的 googleSearch
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
                // [NEW] 根据全局配置决定是否保留 thinkingConfig
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
        // [FIX] 彻底移除 web_search 等任何预置搜索 Bot 提示词注入，保持客户端原始 prompt 完全纯净透传。
        // 仅在配置了用户自定义全局系统提示词时进行追加注入。
        let global_prompt_config = crate::proxy::config::get_global_system_prompt();

        // 检查是否已有 systemInstruction
        if let Some(system_instruction) = inner_request.get_mut("systemInstruction") {
            // 补全 role: user
            if let Some(obj) = system_instruction.as_object_mut() {
                if !obj.contains_key("role") {
                    obj.insert("role".to_string(), json!("user"));
                }
            }

            if let Some(parts) = system_instruction.get_mut("parts") {
                if let Some(parts_array) = parts.as_array_mut() {
                    // 注入全局系统提示词（去重 + 换行隔离，不夹官方身份）
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
            // 没有 systemInstruction，仅在启用全局提示词时创建
            if global_prompt_config.enabled && !global_prompt_config.content.trim().is_empty() {
                inner_request["systemInstruction"] = json!({
                    "role": "user",
                    "parts": [{"text": format!("{}\n\n", global_prompt_config.content.trim())}]
                });
            }
        }
    }

    // [REMOVED v4.8.2] toolConfig / tool_config 双写已移除：官方 Antigravity 报文不带该字段，
    // 且 camelCase 与 snake_case 双份会写出一对矛盾配置 (VALIDATED)。已在协议无关节点统一移除。

    // [ADDED v4.1.24] 注入基于账号的稳定 sessionId
    // [FIX session-1M] 混入对话指纹与代数,不同对话隔离服务端会话,1M 累计报错后 bump 自愈
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

    // [NEW] 1. requestId：官方 5 段形态，三适配器共用。
    // 含 unixMs 保证幂等隔离（避免重试命中上一次的 429 / 旧缓存），形态也与其他入口一致。
    let step = inner_request
        .get("contents")
        .and_then(|c| c.as_array())
        .map(|a| a.len() as u64)
        .unwrap_or(0);
    let official_request_id =
        crate::proxy::mappers::common_utils::build_official_request_id(sid, step);

    // [NEW] 2. 动态 userAgent 仿真 (支持 jetski)
    // 企业 / GCP 账号（非 gmail 邮箱）在官方 Go Worker 中使用 jetski 指纹。
    // 判定由 common_utils 统一提供 —— 三适配器必须共用，否则同一账号指纹漂移。
    let (official_user_agent, official_ide_type) =
        crate::proxy::mappers::common_utils::resolve_official_fingerprint(token);

    // [NEW] 如果是 loadCodeAssist 请求，注入 metadata 字段对齐官方
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

    // [NEW] 3. 动态判断是否需要 agent requestType 与 enabledCreditTypes
    // 对齐官方语言服务原生设计：只有存在工具定义 (tools) 或包含工具调用上下文时才进入 agent 模式。
    // 普通问答、纯文本补全不注入 requestType: "agent"，避开 Google 后端针对 Agent 资源池的过载限流。
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

    // [CACHE] 统一委托进站流水线进行前缀拓扑规范化与对齐（Pipeline First 核心归一）
    crate::proxy::pipeline::InboundThinkingPipeline::align_google_request_prefix_topology(
        &mut inner_request,
    );
    let reordered_inner = inner_request;

    let mut final_request_obj = json!({
        "project": project_id,
        "request": reordered_inner,
        "model": config.final_model,
        "userAgent": official_user_agent,
        // [CACHE] requestId 移到末尾避免动态值破坏前缀字节一致性
        "requestId": official_request_id,
    });

    if config.request_type == "image_gen" {
        final_request_obj["requestType"] = json!("image_gen");
    } else if is_agent_request {
        final_request_obj["requestType"] = json!("agent");
        if let Some(obj) = final_request_obj.as_object_mut() {
            // 强制注入 Google One AI 信用额度支持标号
            obj.insert("enabledCreditTypes".to_string(), json!(["GOOGLE_ONE_AI"]));
        }
    }

    final_request_obj
}
