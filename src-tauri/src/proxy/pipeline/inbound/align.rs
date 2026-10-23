use super::*;

impl InboundThinkingPipeline {
    /// Unified standardization and alignment of the Google Request prefix topology after translation of the four major protocols (Pipeline First core normalization node)
    /// Prefixes that ensure 100% byte-level isomorphism in OpenAI Chat, OpenAI Responses, Claude and Gemini upstream:
    /// 1. systemInstruction: unified and regular internal role -> parts key sequence
    /// 2. Tools: Clean Schema, recursively convert type to uppercase, never intercept any client tools, and sort stably in alphabetical order by name
    /// 3. toolConfig & tool_config: When tools exist, complete and standardize the mode to VALIDATED, and enable includeServerSideToolInvocations
    /// 4. generationConfig: stable bond order and standard topK/topP
    /// 5. safetySettings: 4 OFF safety levels are unified by default to completely avoid cross-protocol missing and drifting
    /// 6. sessionId: session ID
    /// 7. contents: dynamic context history
    /// 8. Strict prefix order reorganization: systemInstruction -> tools -> toolConfig -> tool_config -> generationConfig -> safetySettings -> sessionId -> contents
    pub fn align_google_request_prefix_topology(inner_request: &mut Value) {
        if !inner_request.is_object() {
            return;
        }

        // 1. systemInstruction (standardized unified key sequence: role -> parts -> rest)
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

        // 2. Tools: Standardize parameters and sort them in strict dictionary order by name to prevent any tool from intercepting and filtering
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

        // [REMOVED v4.8.2] No longer inject [CRITICAL DISPATCH DISCIPLINE] / asynchronous dispatch tool NOTE:
        // The gateway does not change the tool description and system prompt words provided by the client, and the prompt words remain the same as the client.

        // 3. [REMOVED v4.8.2] toolConfig / tool_config: Official Antigravity messages do not carry this field.
        //    And the historical implementation writes camelCase and snake_case in double copies at the same time, and the mode value is contradictory (AUTO vs VALIDATED).
        //    Unified across protocol-independent node removal, aligned to official envelope shape.
        inner_request.as_object_mut().map(|m| {
            m.remove("toolConfig");
            m.remove("tool_config");
        });

        // 4. generationConfig (alignment default topK/topP)
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

        // 5. safetySettings (uniformly complete the 4 OFF safety levels to completely avoid cross-protocol missing drift)
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

        // 8. Strict prefix order reorganization:
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

        // Keep remaining unknown/specific extension fields at the end
        if let Some(obj) = inner_request.as_object() {
            for (k, v) in obj {
                if !reordered.as_object().map_or(false, |o| o.contains_key(k)) {
                    reordered[k] = v.clone();
                }
            }
        }
        *inner_request = reordered;
    }

    /// Strip prefix tags of legacy thinking chunks (**Thinking**)
    pub(crate) fn strip_thinking_prefix(text: &str) -> String {
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
