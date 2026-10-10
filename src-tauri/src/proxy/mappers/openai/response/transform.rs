// Gemini -> OpenAI non-streaming response transform (split from response.rs)
use super::sanitize::{normalize_and_sanitize_tool_args, resolve_shell_tool_name};
use crate::proxy::mappers::openai::models::*;
use serde_json::Value;

pub fn transform_openai_response(
    gemini_response: &Value,
    session_id: Option<&str>,
    message_count: usize,
    client_tool_names: Option<&std::collections::HashSet<String>>,
) -> OpenAIResponse {
    let empty_set = std::collections::HashSet::new();
    let client_tool_names = client_tool_names.unwrap_or(&empty_set);

    // 解包 response 字段
    let raw = gemini_response.get("response").unwrap_or(gemini_response);

    let mut choices = Vec::new();

    // 支持多候选结果 (n > 1)
    if let Some(candidates) = raw.get("candidates").and_then(|c| c.as_array()) {
        for (idx, candidate) in candidates.iter().enumerate() {
            let mut content_out = String::new();
            let mut thought_out = String::new();
            let mut tool_calls = Vec::new();

            // 提取 content 和 tool_calls
            if let Some(parts) = candidate
                .get("content")
                .and_then(|c| c.get("parts"))
                .and_then(|p| p.as_array())
            {
                for part in parts {
                    // 捕获 thoughtSignature (Gemini 3 工具调用必需)
                    if let Some(sig) = part
                        .get("thoughtSignature")
                        .or(part.get("thought_signature"))
                        .and_then(|s| s.as_str())
                    {
                        if let Some(sid) = session_id {
                            super::streaming::store_thought_signature(sig, sid, message_count);
                        }
                    }

                    // 检查该 part 是否是思考内容 (thought: true)
                    let is_thought_part = part
                        .get("thought")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);

                    // 文本部分
                    if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                        if is_thought_part {
                            // thought: true 时，text 是思考内容
                            thought_out.push_str(text);
                        } else {
                            // 正常内容
                            content_out.push_str(text);
                        }
                    }

                    // 工具调用部分
                    if let Some(fc) = part.get("functionCall") {
                        let name = fc.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
                        let mut args_json =
                            fc.get("args").unwrap_or(&serde_json::json!({})).clone();

                        // [FIX #1575 & #3430] 标准化并清洗 shell / PowerShell 等工具参数名称与必填字段
                        normalize_and_sanitize_tool_args(name, &mut args_json);

                        let arguments_str = args_json.to_string();
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

                    // 图片处理 (响应中直接返回图片的情况)
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

                    // 处理原生代码执行 (executableCode)
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

                    // 处理代码执行结果 (codeExecutionResult)
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

            // 提取并处理该候选结果的联网搜索引文 (Grounding Metadata)
            if let Some(grounding) = candidate.get("groundingMetadata") {
                let mut grounding_text = String::new();

                // 1. 处理搜索词
                if let Some(queries) = grounding.get("webSearchQueries").and_then(|q| q.as_array())
                {
                    let query_list: Vec<&str> = queries.iter().filter_map(|v| v.as_str()).collect();
                    if !query_list.is_empty() {
                        grounding_text.push_str("\n\n---\n**🔍 已为您搜索：** ");
                        grounding_text.push_str(&query_list.join(", "));
                    }
                }

                // 2. 处理来源链接 (Chunks)
                if let Some(chunks) = grounding.get("groundingChunks").and_then(|c| c.as_array()) {
                    let mut links = Vec::new();
                    for (i, chunk) in chunks.iter().enumerate() {
                        if let Some(web) = chunk.get("web") {
                            let title = web
                                .get("title")
                                .and_then(|v| v.as_str())
                                .unwrap_or("网页来源");
                            let uri = web.get("uri").and_then(|v| v.as_str()).unwrap_or("#");
                            links.push(format!("[{}] [{}]({})", i + 1, title, uri));
                        }
                    }

                    if !links.is_empty() {
                        grounding_text.push_str("\n\n**🌐 来源引文：**\n");
                        grounding_text.push_str(&links.join("\n"));
                    }
                }

                if !grounding_text.is_empty() {
                    content_out.push_str(&grounding_text);
                }
            }

            // 提取传统的 citationMetadata
            if let Some(citation) = candidate.get("citationMetadata") {
                if let Some(sources) = citation.get("citationSources").and_then(|s| s.as_array()) {
                    let mut links = Vec::new();
                    for (i, source) in sources.iter().enumerate() {
                        if let Some(uri) = source.get("uri").and_then(|v| v.as_str()) {
                            // 由于有时没有 title，直接用 URI 当标题
                            links.push(format!("[{}] [{}]({})", i + 1, uri, uri));
                        }
                    }
                    if !links.is_empty() {
                        content_out.push_str("\n\n**📚 引用来源：**\n");
                        content_out.push_str(&links.join("\n"));
                    }
                }
            }

            let raw_finish_reason = candidate.get("finishReason").and_then(|f| f.as_str());

            // 规范化 finish_reason：若包含工具调用，强制遵循 OpenAI 规范映射为 tool_calls
            let finish_reason = if !tool_calls.is_empty() {
                "tool_calls"
            } else {
                raw_finish_reason
                    .map(|f| match f {
                        "STOP" => "stop",
                        "MAX_TOKENS" => "length",
                        "SAFETY" | "RECITATION" => "content_filter",
                        "MALFORMED_FUNCTION_CALL" => "stop",
                        _ => "stop",
                    })
                    .unwrap_or("stop")
            };

            let refusal_val = if finish_reason == "content_filter" {
                Some(
                    "Generation was terminated due to safety policy or recitation checks."
                        .to_string(),
                )
            } else {
                None
            };

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

    // 如果 candidates 为空，但存在 promptFeedback（被安全拦截），伪造一个被拒绝的 choice
    if choices.is_empty() {
        if let Some(feedback) = raw.get("promptFeedback") {
            let reason = feedback
                .get("blockReason")
                .and_then(|v| v.as_str())
                .unwrap_or("UNKNOWN");
            let refusal_msg = format!(
                "Request was blocked due to safety policy (blockReason: {}).",
                reason
            );
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
