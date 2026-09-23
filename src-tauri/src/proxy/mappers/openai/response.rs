// OpenAI protocol response conversion module
use super::models::*;
use serde_json::Value;

/// Pure passthrough of tool arguments without truncating, synthesizing, or rewriting fields
pub fn normalize_and_sanitize_tool_args(tool_name: &str, args: &mut Value) {
    if let Some(obj) = args.as_object() {
        tracing::debug!(
            "[OpenAI] Tool Call (Passthrough): '{}' Args: {:?}",
            tool_name,
            obj
        );
    }
}

pub fn resolve_shell_tool_name(
    model_tool_name: &str,
    _client_tool_names: &std::collections::HashSet<String>,
) -> String {
    // Pure passthrough of tool name without rewriting
    model_tool_name.to_string()
}

fn extract_apply_patch_input(args: &Value) -> String {
    if let Some(obj) = args.as_object() {
        if let Some(input) = obj.get("input").and_then(|v| v.as_str()) {
            return input.to_string();
        }
        if let Some(arr) = obj.get("command").and_then(|v| v.as_array()) {
            if arr.len() > 1 {
                if let Some(patch) = arr[1].as_str() {
                    return patch.to_string();
                }
            }
        }
        if let Some(cmd_str) = obj.get("command").and_then(|v| v.as_str()) {
            if let Some(patch) = cmd_str.strip_prefix(
                "apply_patch
",
            ) {
                return patch.to_string();
            }
            if let Some(patch) = cmd_str.strip_prefix("apply_patch ") {
                return patch.to_string();
            }
            return cmd_str.to_string();
        }
        for key in ["patch_text", "patch", "diff", "content"] {
            if let Some(patch) = obj.get(key).and_then(|v| v.as_str()) {
                return patch.to_string();
            }
        }
    }
    args.as_str()
        .map(str::to_string)
        .unwrap_or_else(|| serde_json::to_string(args).unwrap_or_default())
}

pub fn transform_openai_response(
    gemini_response: &Value,
    session_id: Option<&str>,
    message_count: usize,
    client_tool_names: Option<&std::collections::HashSet<String>>,
) -> OpenAIResponse {
    let empty_set = std::collections::HashSet::new();
    let client_tool_names = client_tool_names.unwrap_or(&empty_set);

    // Unwrap response field
    let raw = gemini_response.get("response").unwrap_or(gemini_response);

    let mut choices = Vec::new();

    // Support multiple candidate results (n > 1)
    if let Some(candidates) = raw.get("candidates").and_then(|c| c.as_array()) {
        for (idx, candidate) in candidates.iter().enumerate() {
            let mut content_out = String::new();
            let mut thought_out = String::new();
            let mut tool_calls = Vec::new();

            // Extract content and tool_calls
            if let Some(parts) = candidate
                .get("content")
                .and_then(|c| c.get("parts"))
                .and_then(|p| p.as_array())
            {
                for part in parts {
                    // Capture thoughtSignature (required for Gemini 3 tool calls)
                    if let Some(sig) = part
                        .get("thoughtSignature")
                        .or(part.get("thought_signature"))
                        .and_then(|s| s.as_str())
                    {
                        if let Some(sid) = session_id {
                            super::streaming::store_thought_signature(sig, sid, message_count);
                        }
                    }

                    // Check if part is thought content (thought: true)
                    let is_thought_part = part
                        .get("thought")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    // Text part
                    if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                        if is_thought_part {
                            // When thought: true, text is thinking content
                            thought_out.push_str(text);
                        } else {
                            // Regular content
                            content_out.push_str(text);
                        }
                    }

                    // Tool call part
                    if let Some(fc) = part.get("functionCall") {
                        let name = fc.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
                        let mut args_json =
                            fc.get("args").unwrap_or(&serde_json::json!({})).clone();

                        // [FIX #1575 & #3430] Normalize and sanitize tool parameter names and required fields for shell/PowerShell
                        normalize_and_sanitize_tool_args(name, &mut args_json);

                        let mut arguments_str = args_json.to_string();

                        // [FIX] Codex CLI apply_patch freeform raw string
                        if name == "apply_patch" || name == "apply_patch_v2" {
                            let extracted_patch = extract_apply_patch_input(&args_json);
                            let (optimized_patch, _) =
                                crate::proxy::adapters::apply_patch_preflight::optimize_patch(
                                    &extracted_patch,
                                    None,
                                    true,
                                );
                            arguments_str = optimized_patch;
                            if let Some((line, message)) =
                                crate::proxy::adapters::apply_patch_preflight::validate_v4a_for_codex(
                                    &arguments_str,
                                )
                            {
                                if !content_out.is_empty() {
                                    content_out.push('\n');
                                }
                                content_out.push_str(&format!(
                                    "apply_patch format invalid, execution stopped to prevent repeated failures. Line {line}: {message}"
                                ));
                                continue;
                            }
                        }
                        let final_name = resolve_shell_tool_name(name, client_tool_names);

                        let id = fc
                            .get("id")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| format!("{}-{}", final_name, uuid::Uuid::new_v4()));

                        if let Some(sig) = part
                            .get("thoughtSignature")
                            .or(part.get("thought_signature"))
                            .and_then(|s| s.as_str())
                        {
                            crate::proxy::SignatureCache::global()
                                .cache_tool_signature(&id, sig.to_string());
                        }

                        tool_calls.push(ToolCall {
                            id,
                            r#type: "function".to_string(),
                            function: Some(ToolFunction {
                                name: final_name.to_string(),
                                arguments: arguments_str,
                            }),
                            signature: None,
                            status: None,
                            call_id: None,
                            operation: None,
                        });
                    }

                    // Image handling (direct image returns in response)
                    if let Some(img) = part.get("inlineData") {
                        let mime_type = img
                            .get("mimeType")
                            .and_then(|v| v.as_str())
                            .unwrap_or("image/png");
                        let data = img.get("data").and_then(|v| v.as_str()).unwrap_or("");
                        if !data.is_empty() {
                            content_out
                                .push_str(&format!("![image](data:{};base64,{})", mime_type, data));
                        }
                    }

                    // Handle native code execution (executableCode)
                    if let Some(exec_code) = part.get("executableCode") {
                        let lang = exec_code
                            .get("language")
                            .and_then(|v| v.as_str())
                            .unwrap_or("python");
                        let code = exec_code.get("code").and_then(|v| v.as_str()).unwrap_or("");
                        if !code.is_empty() {
                            content_out.push_str(&format!(
                                "\n\n```{}\n{}\n```\n",
                                lang.to_lowercase(),
                                code
                            ));
                        }
                    }

                    // Handle code execution results (codeExecutionResult)
                    if let Some(exec_result) = part.get("codeExecutionResult") {
                        let output = exec_result
                            .get("output")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        if !output.is_empty() {
                            content_out.push_str(&format!(
                                "\n**Execution Output:**\n```text\n{}\n```\n",
                                output
                            ));
                        }
                    }
                }
                if let Some(sid) = session_id {
                    crate::proxy::thinking_store::capture_gemini_parts(sid, parts);
                }
            }

            // Extract and handle search citations (Grounding Metadata)
            if let Some(grounding) = candidate.get("groundingMetadata") {
                let mut grounding_text = String::new();

                // 1. Process search query
                if let Some(queries) = grounding.get("webSearchQueries").and_then(|q| q.as_array())
                {
                    let query_list: Vec<&str> = queries.iter().filter_map(|v| v.as_str()).collect();
                    if !query_list.is_empty() {
                        grounding_text.push_str("\n\n---\n**🔍 Searched for:** ");
                        grounding_text.push_str(&query_list.join(", "));
                    }
                }

                // 2. Process source links (Chunks)
                if let Some(chunks) = grounding.get("groundingChunks").and_then(|c| c.as_array()) {
                    let mut links = Vec::new();
                    for (i, chunk) in chunks.iter().enumerate() {
                        if let Some(web) = chunk.get("web") {
                            let title = web
                                .get("title")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Web source");
                            let uri = web.get("uri").and_then(|v| v.as_str()).unwrap_or("#");
                            links.push(format!("[{}] [{}]({})", i + 1, title, uri));
                        }
                    }

                    if !links.is_empty() {
                        grounding_text.push_str("\n\n**🌐 Sources:**\n");
                        grounding_text.push_str(&links.join("\n"));
                    }
                }

                if !grounding_text.is_empty() {
                    content_out.push_str(&grounding_text);
                }
            }

            // Extract legacy citationMetadata
            if let Some(citation) = candidate.get("citationMetadata") {
                if let Some(sources) = citation.get("citationSources").and_then(|s| s.as_array()) {
                    let mut links = Vec::new();
                    for (i, source) in sources.iter().enumerate() {
                        if let Some(uri) = source.get("uri").and_then(|v| v.as_str()) {
                            // If title is missing, use URI as title
                            links.push(format!("[{}] [{}]({})", i + 1, uri, uri));
                        }
                    }
                    if !links.is_empty() {
                        content_out.push_str("\n\n**📚 Citations:**\n");
                        content_out.push_str(&links.join("\n"));
                    }
                }
            }

            let raw_finish_reason = candidate.get("finishReason").and_then(|f| f.as_str());
            let is_malformed_function_call = raw_finish_reason == Some("MALFORMED_FUNCTION_CALL");

            let finish_reason = raw_finish_reason
                .map(|f| match f {
                    "STOP" => "stop",
                    "MAX_TOKENS" => "length",
                    "SAFETY" => "content_filter",
                    "RECITATION" => "content_filter",
                    "MALFORMED_FUNCTION_CALL" => "stop",
                    _ => "stop",
                })
                .unwrap_or("stop");

            let refusal_val = if finish_reason == "content_filter" {
                Some("Generation stopped due to safety policy or recitation checks.".to_string())
            } else {
                None
            };

            // [FIX MALFORMED_FUNCTION_CALL] Avoid blank response
            if is_malformed_function_call && content_out.is_empty() {
                content_out.push_str("We apologize, but the model encountered a format anomaly while attempting to retrieve real-time information. To query real-time weather or news, please use online mode (-online suffix) or configure a search plugin.");
            }

            choices.push(Choice {
                index: idx as u32,
                message: OpenAIMessage {
                    role: "assistant".to_string(),
                    content: if content_out.is_empty() {
                        None
                    } else {
                        Some(OpenAIContent::String(content_out))
                    },
                    reasoning_content: if thought_out.is_empty() {
                        None
                    } else {
                        Some(thought_out)
                    },
                    signature: None,
                    tool_calls: if tool_calls.is_empty() {
                        None
                    } else {
                        Some(tool_calls)
                    },
                    tool_call_id: None,
                    name: None,
                    refusal: refusal_val,
                },
                finish_reason: Some(finish_reason.to_string()),
            });
        }
    }

    // If candidates is empty but promptFeedback exists (blocked by safety), forge a refused choice
    if choices.is_empty() {
        if let Some(feedback) = raw.get("promptFeedback") {
            let reason = feedback
                .get("blockReason")
                .and_then(|v| v.as_str())
                .unwrap_or("UNKNOWN");
            let refusal_msg = format!("Request blocked by safety policy (blockReason: {})", reason);
            choices.push(Choice {
                index: 0,
                message: OpenAIMessage {
                    role: "assistant".to_string(),
                    content: None,
                    reasoning_content: None,
                    signature: None,
                    tool_calls: None,
                    tool_call_id: None,
                    name: None,
                    refusal: Some(refusal_msg),
                },
                finish_reason: Some("content_filter".to_string()),
            });
        }
    }

    // Extract and map usage metadata from Gemini to OpenAI format
    // Supports both legacy v1internal format (promptTokenCount/candidatesTokenCount/totalTokenCount/cachedContentTokenCount)
    // and new Interactions API format (total_input_tokens/total_output_tokens/total_thought_tokens/total_cached_tokens)
    let usage = raw.get("usageMetadata").map(|u| {
        let canonical = crate::proxy::pipeline::CanonicalUsage::from_gemini(u);
        let mut usage = super::models::OpenAIUsage::from(&canonical);
        usage.input_tokens_by_modality = u.get("input_tokens_by_modality").cloned();
        usage.total_tool_use_tokens = u
            .get("total_tool_use_tokens")
            .and_then(|v| v.as_u64())
            .map(|v| v as u32);
        usage
    });

    OpenAIResponse {
        id: raw
            .get("responseId")
            .and_then(|v| v.as_str())
            .unwrap_or("resp_unknown")
            .to_string(),
        object: "chat.completion".to_string(),
        created: chrono::Utc::now().timestamp() as u64,
        model: raw
            .get("modelVersion")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string(),
        choices,
        usage,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_transform_openai_response() {
        let gemini_resp = json!({
            "candidates": [{
                "content": {
                    "parts": [{"text": "Hello!"}]
                },
                "finishReason": "STOP"
            }],
            "modelVersion": "gemini-2.5-flash",
            "responseId": "resp_123"
        });

        let result = transform_openai_response(&gemini_resp, Some("session-123"), 1, None);
        assert_eq!(result.object, "chat.completion");
        let content = match result.choices[0].message.content.as_ref().unwrap() {
            OpenAIContent::String(s) => s,
            _ => panic!("Expected string content"),
        };
        assert_eq!(content, "Hello!");
        assert_eq!(result.choices[0].finish_reason, Some("stop".to_string()));
    }

    #[test]
    fn test_usage_metadata_mapping() {
        let gemini_resp = json!({
            "candidates": [{
                "content": {"parts": [{"text": "Hello!"}]},
                "finishReason": "STOP"
            }],
            "usageMetadata": {
                "promptTokenCount": 100,
                "candidatesTokenCount": 50,
                "totalTokenCount": 150,
                "cachedContentTokenCount": 25
            },
            "modelVersion": "gemini-2.5-flash",
            "responseId": "resp_123"
        });

        let result = transform_openai_response(&gemini_resp, Some("session-123"), 1, None);

        assert!(result.usage.is_some());
        let usage = result.usage.unwrap();
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
        assert!(usage.prompt_tokens_details.is_some());
        assert_eq!(usage.prompt_tokens_details.unwrap().cached_tokens, Some(25));
    }

    #[test]
    fn test_interactions_usage_metadata_mapping() {
        let gemini_resp = json!({
            "candidates": [{
                "content": {"parts": [{"text": "Hello!"}]},
                "finishReason": "STOP"
            }],
            "usageMetadata": {
                "input_tokens_by_modality": [
                    {
                        "modality": "text",
                        "tokens": 7
                    }
                ],
                "total_cached_tokens": 0,
                "total_input_tokens": 7,
                "total_output_tokens": 20,
                "total_thought_tokens": 22,
                "total_tokens": 49,
                "total_tool_use_tokens": 0
            },
            "modelVersion": "gemini-3-flash-preview",
            "responseId": "resp_123"
        });

        let result = transform_openai_response(&gemini_resp, Some("session-123"), 1, None);
        let usage = result.usage.unwrap();

        assert_eq!(usage.prompt_tokens, 7);
        assert_eq!(usage.completion_tokens, 42);
        assert_eq!(usage.total_tokens, 49);
        assert_eq!(
            usage
                .completion_tokens_details
                .as_ref()
                .unwrap()
                .reasoning_tokens,
            Some(22)
        );

        let responses_usage = usage.to_responses_usage_value();
        assert_eq!(responses_usage["input_tokens"], 7);
        assert_eq!(responses_usage["input_tokens_details"]["cached_tokens"], 0);
        assert_eq!(responses_usage["output_tokens"], 42);
        assert_eq!(
            responses_usage["output_tokens_details"]["reasoning_tokens"],
            22
        );
        assert_eq!(responses_usage["total_tokens"], 49);
    }

    #[test]
    fn test_response_without_usage_metadata() {
        let gemini_resp = json!({
            "candidates": [{
                "content": {"parts": [{"text": "Hello!"}]},
                "finishReason": "STOP"
            }],
            "modelVersion": "gemini-2.5-flash",
            "responseId": "resp_123"
        });

        let result = transform_openai_response(&gemini_resp, Some("session-123"), 1, None);
        assert!(result.usage.is_none());
    }

    #[test]
    fn test_normalize_and_sanitize_tool_args_passthrough() {
        let mut args = json!({
            "cmd": "ls -la /tmp",
            "description": "Custom description",
            "arbitrary_field": 123
        });
        normalize_and_sanitize_tool_args("shell", &mut args);
        // Verify pure passthrough: arguments remain intact, no fields renamed, deleted, or injected
        assert_eq!(args["cmd"], "ls -la /tmp");
        assert_eq!(args["description"], "Custom description");
        assert_eq!(args["arbitrary_field"], 123);
        assert!(!args.as_object().unwrap().contains_key("command"));
    }
}
