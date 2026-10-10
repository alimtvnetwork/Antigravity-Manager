// Tools phase (split from request.rs).
// OpenAI → Gemini 请求转换
use super::super::models::*;
use crate::proxy::model_specs;
use crate::proxy::token_manager::ProxyToken;

use serde_json::{json, Value};

use super::session_setup::SetupState;
use super::helpers::flatten_tools;
use crate::proxy::mappers::openai::request::transform::enforce_uppercase_types;

pub(crate) fn phase_tools(
    request: &super::models::OpenAIRequest,
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

    // [CACHE:L2] 计算原始 tools 的 hash，查 Layer 2 缓存
    // 命中则跳过所有 tools 处理逻辑，跨 session 复用已处理的 tools
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
                    // [FIX] 剔除 "type" 前如果不存在 "name"，则提取 "type" 兜底作为名字
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
                    // [FIX] 如果工具没有名称，视为无效工具直接跳过 (防止 REQUIRED_FIELD_MISSING)
                    tracing::warn!(
                        "[OpenAI-Request] Skipping tool without name: {:?}",
                        gemini_func
                    );
                    continue;
                }

                // [NEW CRITICAL FIX] 保留函数定义根层级的合法字段，移除所有非法字段 (如 type, execution, format 等)
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
                    // [DEEP FIX] 统一调用公共库清洗：展开 $ref 并剔除所有层级的 format/definitions
                    crate::proxy::common::json_schema::clean_json_schema(params);

                    // Gemini v1internal 要求：
                    // 1. type 必须是大写 (OBJECT, STRING 等)
                    // 2. 根对象必须有 "type": "OBJECT"，且必须声明 "properties" (即使为空)，杜绝 MALFORMED_FUNCTION_CALL
                    if let Some(params_obj) = params.as_object_mut() {
                        if !params_obj.contains_key("type") {
                            params_obj.insert("type".to_string(), json!("OBJECT"));
                        }
                        if !params_obj.contains_key("properties") {
                            params_obj.insert("properties".to_string(), json!({}));
                        }
                    }

                    // 递归转换 type 为大写 (符合 Protobuf 定义)
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

        // [CACHE:L2] 缓存处理完成的 tools，下次相同 schema 可以直接命中
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

    // [CACHE] 按 function name 稳定排序，确保跨请求的 tool schema 字节一致
    function_declarations.sort_by(|a, b| {
        let name_a = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
        let name_b = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
        name_a.cmp(name_b)
    });

    // Removed auto-inject since we handle it above now if Codex passes it.

    if !function_declarations.is_empty() {
        inner_request["tools"] = json!([{ "functionDeclarations": function_declarations }]);

        // [REMOVED v4.8.2] toolConfig / tool_config 双写已移除：官方 Antigravity 报文不带该字段，
        // 且 camelCase 与 snake_case 双份会写出一对矛盾配置 (AUTO vs VALIDATED)。
        // 工具调用行为在协议无关节点 (align_google_request_prefix_topology) 统一处理。
        let _mode = None as Option<&str>;
    }

    let global_prompt_config = crate::proxy::config::get_global_system_prompt();
    let global_prompt =
        if global_prompt_config.enabled && !global_prompt_config.content.trim().is_empty() {
            Some(global_prompt_config.content.as_str())
        } else {
            None
        };
    let system_parts =
        super::context_blocks::build_system_instruction_parts(&system_instructions, global_prompt);
    if !system_parts.is_empty() {
        inner_request["systemInstruction"] = json!({
            "role": "user",
            "parts": system_parts
        });
    }

    if setup.config.inject_google_search {
        crate::proxy::mappers::common_utils::inject_google_search_tool(
            &mut inner_request,
            Some(mapped_model),
        );
        // [REMOVED v4.8.2] toolConfig / tool_config 注入已移除（官方不带该字段），
        // googleSearch 工具声明本身已由 inject_google_search_tool 写入 tools。
    }

    if let Some(image_config) = setup.config.image_config {
        if let Some(obj) = inner_request.as_object_mut() {
            obj.remove("tools");
            obj.remove("systemInstruction");
            let gen_config = obj.entry("generationConfig").or_insert_with(|| json!({}));
            if let Some(gen_obj) = gen_config.as_object_mut() {
                // [REMOVED] thinkingConfig 拦截已删除，允许图像生成时输出思维链
                // gen_obj.remove("thinkingConfig");
                gen_obj.remove("responseMimeType");
                gen_obj.remove("responseModalities");
                gen_obj.insert("imageConfig".to_string(), image_config);
            }
        }
    }

    // [ADDED v4.1.24] 注入稳定 sessionId 对齐官方规范
    // [FIX session-1M] sessionId 混入对话指纹与代数:
    //   - 同一对话内保持稳定(保留上游服务端会话缓存收益)
    //   - 不同对话使用不同 sessionId,避免共享同一服务端累计会话
    //   - 检测到上游 1M 累计报错后 bump 代数,新 sessionId = 全新上游会话,对话无感恢复
    if let Some(t) = token {
        let generation =
            crate::proxy::common::session::current_bump(&t.account_id, &setup.session_id);
        inner_request["sessionId"] = json!(crate::proxy::common::session::derive_session_scoped(
            &t.account_id,
            &setup.session_id,
            generation
        ));
    }

    // [CACHE] 重建 inner_request 字段顺序——稳定前缀在前，动态内容在后
    // [CACHE] 统一委托进站流水线进行前缀拓扑规范化与对齐（Pipeline First 核心归一）
    crate::proxy::pipeline::InboundThinkingPipeline::align_google_request_prefix_topology(
        &mut inner_request,
    );
    let reordered_request = inner_request;
}
