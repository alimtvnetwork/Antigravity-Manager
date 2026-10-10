use super::*;

impl InboundThinkingPipeline {
    /// 统一规范化与对齐四大协议转译后的 Google Request 前缀拓扑（Pipeline First 核心归一节点）
    /// 确保 OpenAI Chat, OpenAI Responses, Claude 与 Gemini 在上游呈现 100% 字节级同构的前缀：
    /// 1. systemInstruction: 统一规整内部 role -> parts 键序
    /// 2. tools: 清理 Schema，递归转大写 type，绝不拦截任何客户端工具，统一按 name 字母序稳定排序
    /// 3. toolConfig & tool_config: 存在工具时统一补齐并规范模式为 VALIDATED，并开启 includeServerSideToolInvocations
    /// 4. generationConfig: 稳定键序与标准 topK/topP
    /// 5. safetySettings: 缺省统一补齐 4 项 OFF 安全等级，彻底避免跨协议缺失漂移
    /// 6. sessionId: 会话标识
    /// 7. contents: 动态上下文历史
    /// 8. 严格前缀顺序重组: systemInstruction -> tools -> toolConfig -> tool_config -> generationConfig -> safetySettings -> sessionId -> contents
    pub fn align_google_request_prefix_topology(inner_request: &mut Value) {
        if !inner_request.is_object() {
            return;
        }

        // 1. systemInstruction (规范化统一键序: role -> parts -> 其余)
        let canonical_si = if let Some(si) = inner_request.get("systemInstruction") {
            if let Some(si_obj) = si.as_object() {
                let mut c = json!({});
                c["role"] = si_obj.get("role").cloned().unwrap_or(json!("user"));
                if let Some(parts) = si_obj.get("parts") {
                    c["parts"] = parts.clone();
                }
                for (k, v) in si_obj {
                    if k != "role" && k != "parts" {
                        c[k] = v.clone();
                    }
                }
                Some(c)
            } else {
                Some(si.clone())
            }
        } else {
            None
        };

        // 2. tools: 规范化 parameters 并按 name 严格字典序排序，杜绝任何工具拦截过滤
        let canonical_tools = if let Some(tools) = inner_request.get_mut("tools") {
            if let Some(tools_arr) = tools.as_array_mut() {
                for tool in tools_arr.iter_mut() {
                    let decls_opt = if tool.get("functionDeclarations").is_some() {
                        tool.get_mut("functionDeclarations")
                    } else {
                        tool.get_mut("function_declarations")
                    };
                    if let Some(decls) = decls_opt {
                        if let Some(decls_arr) = decls.as_array_mut() {
                            for decl in decls_arr.iter_mut() {
                                if let Some(decl_obj) = decl.as_object_mut() {
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
                                        crate::proxy::common::json_schema::clean_json_schema(
                                            params,
                                        );
                                        crate::proxy::mappers::openai::request::enforce_uppercase_types(
                                            params,
                                        );
                                    }
                                }
                            }
                            decls_arr.sort_by(|a, b| {
                                let name_a = a.get("name").and_then(|v| v.as_str()).unwrap_or("");
                                let name_b = b.get("name").and_then(|v| v.as_str()).unwrap_or("");
                                name_a.cmp(name_b)
                            });
                        }
                    }
                }
            }
            Some(tools.clone())
        } else {
            None
        };

        // [REMOVED v4.8.2] 不再注入 [CRITICAL DISPATCH DISCIPLINE] / 异步派发工具 NOTE：
        // 网关不改动客户端提供的工具描述与系统提示词，提示词保持客户端原样。

        // 3. [REMOVED v4.8.2] toolConfig / tool_config: 官方 Antigravity 报文不携带该字段，
        //    且历史实现同时写出 camelCase 与 snake_case 双份、mode 值自相矛盾 (AUTO vs VALIDATED)。
        //    统一在协议无关节点移除，对齐官方信封形状。
        inner_request.as_object_mut().map(|m| {
            m.remove("toolConfig");
            m.remove("tool_config");
        });

        // 4. generationConfig (对齐默认 topK/topP)
        let canonical_gc = inner_request.get_mut("generationConfig").map(|gc| {
            if let Some(gc_obj) = gc.as_object_mut() {
                if !gc_obj.contains_key("topK") {
                    gc_obj.insert("topK".to_string(), json!(40));
                }
                if !gc_obj.contains_key("topP") {
                    gc_obj.insert("topP".to_string(), json!(1.0));
                }
            }
            gc.clone()
        });

        // 5. safetySettings (统一补齐 4 项 OFF 安全等级，彻底避免跨协议缺失漂移)
        let canonical_safety = if let Some(ss) = inner_request
            .get("safetySettings")
            .and_then(|v| v.as_array())
            .filter(|a| !a.is_empty())
        {
            Some(Value::Array(ss.clone()))
        } else {
            Some(json!([
                { "category": "HARM_CATEGORY_HARASSMENT", "threshold": "OFF" },
                { "category": "HARM_CATEGORY_HATE_SPEECH", "threshold": "OFF" },
                { "category": "HARM_CATEGORY_SEXUALLY_EXPLICIT", "threshold": "OFF" },
                { "category": "HARM_CATEGORY_DANGEROUS_CONTENT", "threshold": "OFF" },
            ]))
        };

        // 6. sessionId
        let canonical_sid = inner_request.get("sessionId").cloned();

        // 7. contents
        let canonical_contents = inner_request.get("contents").cloned().unwrap_or(json!([]));

        // 8. 严格前缀顺序重组:
        // systemInstruction -> tools -> toolConfig -> tool_config -> generationConfig -> safetySettings -> sessionId -> contents
        let mut reordered = json!({});
        if let Some(si) = canonical_si {
            reordered["systemInstruction"] = si;
        }
        if let Some(tools) = canonical_tools {
            reordered["tools"] = tools;
        }
        if let Some(gc) = canonical_gc {
            reordered["generationConfig"] = gc;
        }
        if let Some(ss) = canonical_safety {
            reordered["safetySettings"] = ss;
        }
        if let Some(sid) = canonical_sid {
            reordered["sessionId"] = sid;
        }
        reordered["contents"] = canonical_contents;

        // 保留其余未知/特定扩展字段在末尾
        if let Some(obj) = inner_request.as_object() {
            for (k, v) in obj {
                if !reordered.as_object().map_or(false, |o| o.contains_key(k)) {
                    reordered[k] = v.clone();
                }
            }
        }
        *inner_request = reordered;
    }

    /// 剥离遗留思考块的前缀标记 (**Thinking**)
    fn strip_thinking_prefix(text: &str) -> String {
        let trimmed = text.trim_start();
        if let Some(rest) = trimmed.strip_prefix("**Thinking**") {
            let rest = rest.trim_start_matches(':');
            rest.trim_start_matches(|c| c == '\r' || c == '\n' || c == ' ' || c == '\t')
                .to_string()
        } else {
            text.to_string()
        }
    }
}
