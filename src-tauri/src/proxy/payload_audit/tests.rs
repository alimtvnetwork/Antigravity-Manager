use super::*;

mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn redacts_api_keys_but_keeps_session_markers() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "authorization",
            HeaderValue::from_static("Bearer sk-secret-customer-key"),
        );
        headers.insert("x-api-key", HeaderValue::from_static("sk-another"));
        headers.insert(
            "x-session-id",
            HeaderValue::from_static("sess-ops-compare-001"),
        );
        headers.insert(
            "x-antigravity-session-id",
            HeaderValue::from_static("ag-think-42"),
        );
        headers.insert("user-agent", HeaderValue::from_static("claude-code/1.0"));

        let json = headers_to_redacted_json(&headers);
        assert!(json.contains("***REDACTED***"), "{json}");
        assert!(!json.contains("sk-secret-customer-key"), "{json}");
        assert!(!json.contains("sk-another"), "{json}");
        assert!(json.contains("sess-ops-compare-001"), "{json}");
        assert!(json.contains("ag-think-42"), "{json}");
        assert!(json.contains("claude-code/1.0"), "{json}");
    }

    #[test]
    fn test_simplify_tools_preserves_full_schema() {
        let tools = json!([
            {
                "functionDeclarations": [
                    {
                        "name": "edit_file",
                        "description": "Edit a file in codebase",
                        "parameters": {
                            "type": "OBJECT",
                            "properties": {
                                "file_path": { "type": "STRING", "description": "path to file" },
                                "content": { "type": "STRING" }
                            },
                            "required": ["file_path", "content"]
                        }
                    }
                ]
            },
            {
                "googleSearch": {}
            }
        ]);

        let req = json!({
            "model": "gemini-2.5-pro",
            "tools": tools,
            "contents": [
                {
                    "role": "user",
                    "parts": [{"text": "Please edit foo.rs"}]
                }
            ]
        });

        let simplified = simplify_payload_json(&req);
        assert_eq!(
            simplified["tools"], tools,
            "Tools schema should be completely preserved!"
        );
        assert_eq!(simplified["model"], "gemini-2.5-pro");
    }

    #[test]
    fn test_simplify_responses_payload_keeps_instructions_and_input() {
        // 回归：Responses / Codex 协议的系统提示词在 `instructions`、上下文在 `input`。
        // 这两个键一度不在白名单里 → simple 模式下整段消失（客户端原文看不到系统提示词与
        // 上下文，而中转报文是 Gemini 格式故看起来完整）。
        let req = json!({
            "model": "gemini-2.5-pro",
            "instructions": "You are Codex, a coding agent.",
            "input": [
                { "type": "message", "role": "user", "content": [{ "type": "input_text", "text": "hi" }] },
                { "type": "function_call_output", "call_id": "call_1", "output": "ok" }
            ],
            "max_output_tokens": 1024
        });

        let simplified = simplify_payload_json(&req);

        assert_eq!(
            simplified["instructions"], "You are Codex, a coding agent.",
            "instructions 必须保留: {simplified}"
        );
        assert!(
            simplified["input"].is_array(),
            "input 必须保留为数组: {simplified}"
        );
        assert_eq!(simplified["input"][0]["role"], "user");
        assert_eq!(simplified["input"][1]["type"], "function_call_output");
        assert_eq!(simplified["input"][1]["call_id"], "call_1");
        assert_eq!(simplified["input"][1]["output"], "ok");
        assert_eq!(simplified["max_output_tokens"], 1024);
    }

    #[test]
    fn test_simplify_consolidated_response_preserves_all_ops_fields() {
        let consolidated = json!({
            "_session_thinking_id": "f4379d8e-02a3-4e44-902b-1a87c4a1b30c",
            "content": "Here is the result of your query.",
            "thinking": "First, let's consider the problem deeply...",
            "thinking_signature": "sig_abcd_1234567890_very_long_valid_signature",
            "tool_calls": [
                {
                    "id": "call_99",
                    "type": "function",
                    "function": {
                        "name": "get_weather",
                        "arguments": "{\"location\":\"Beijing\"}"
                    }
                }
            ],
            "usage": {
                "input_tokens": 33724,
                "output_tokens": 35,
                "total_tokens": 33759,
                "cached_tokens": 0,
                "cache_hit_rate": "0.0%"
            }
        });

        let simplified = simplify_payload_json(&consolidated);
        assert_eq!(
            simplified["_session_thinking_id"],
            "f4379d8e-02a3-4e44-902b-1a87c4a1b30c"
        );
        assert_eq!(simplified["content"], "Here is the result of your query.");
        assert_eq!(
            simplified["thinking"],
            "First, let's consider the problem deeply..."
        );
        assert_eq!(
            simplified["thinking_signature"],
            "sig_abcd_1234567890_very_long_valid_signature"
        );
        assert_eq!(
            simplified["tool_calls"][0]["function"]["name"],
            "get_weather"
        );
        assert_eq!(
            simplified["tool_calls"][0]["function"]["arguments"],
            "{\"location\":\"Beijing\"}"
        );
        assert_eq!(simplified["usage"]["input_tokens"], 33724);
        assert_eq!(simplified["usage"]["output_tokens"], 35);
    }

    #[test]
    fn test_simplify_gemini_parts_preserves_args_and_signatures() {
        let gemini_resp = json!({
            "candidates": [
                {
                    "content": {
                        "role": "model",
                        "parts": [
                            {
                                "thought": true,
                                "text": "Analyzing the request...",
                                "thoughtSignature": "tsig_xyz_987654"
                            },
                            {
                                "functionCall": {
                                    "name": "search_code",
                                    "args": { "query": "struct Token" }
                                }
                            }
                        ]
                    }
                }
            ],
            "usageMetadata": {
                "promptTokenCount": 1500,
                "candidatesTokenCount": 80
            }
        });

        let simplified = simplify_payload_json(&gemini_resp);
        let cand = &simplified["candidates"][0]["content"]["parts"];
        assert_eq!(cand[0]["thought"], true);
        assert_eq!(cand[0]["text"], "Analyzing the request...");
        assert_eq!(cand[0]["thoughtSignature"], "tsig_xyz_987654");
        assert_eq!(cand[1]["functionCall"]["name"], "search_code");
        assert_eq!(cand[1]["functionCall"]["args"]["query"], "struct Token");
        assert_eq!(simplified["usageMetadata"]["promptTokenCount"], 1500);
    }
}
