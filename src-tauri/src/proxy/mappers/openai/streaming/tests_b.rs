// OpenAI streaming tests (split from streaming.rs).
// Test-only module, declared #[cfg(test)] in mod.rs.
use super::*;
use futures::stream;
use serde_json::json;

async fn collect_codex_stream(chunks: Vec<Value>) -> (String, Vec<Value>) {
    collect_codex_stream_with_cache(chunks, true).await
}

async fn collect_codex_stream_with_cache(
    chunks: Vec<Value>,
    cache_tool_calls: bool,
) -> (String, Vec<Value>) {
    let items: Vec<Result<Bytes, String>> = chunks
        .into_iter()
        .map(|chunk| Ok(Bytes::from(format!("data: {chunk}\n\n"))))
        .collect();
    let mut stream = create_codex_sse_stream(
        Box::pin(stream::iter(items)),
        "gemini-pro-agent".to_string(),
        "test-codex-session".to_string(),
        0,
        0,
        "resp-test-codex-session".to_string(),
        None,
        cache_tool_calls,
    );

    let mut raw = String::new();
    while let Some(item) = stream.next().await {
        raw.push_str(&String::from_utf8_lossy(&item.expect("codex stream item")));
    }
    let events = raw
        .lines()
        .filter_map(|line| line.strip_prefix("data: "))
        .filter_map(|data| serde_json::from_str::<Value>(data).ok())
        .collect();
    (raw, events)
}

#[tokio::test]
async fn test_codex_tool_round_message_stays_commentary() {
    let (_, events) = collect_codex_stream(vec![
        json!({
            "candidates": [{
                "content": {"parts": [{"text": "I will inspect the files first."}]}
            }]
        }),
        json!({
            "candidates": [{
                "content": {"parts": [{
                    "functionCall": {"name": "shell_command", "args": {"command": "Get-ChildItem"}}
                }]}
            }]
        }),
        json!({
            "candidates": [{
                "finishReason": "STOP",
                "content": {"parts": [{"text": ""}]}
            }]
        }),
    ])
    .await;

    let message_done = events
        .iter()
        .find(|event| {
            event["type"] == "response.output_item.done" && event["item"]["type"] == "message"
        })
        .expect("message done");
    assert_eq!(message_done["item"]["phase"], "commentary");

    let terminal = events.last().expect("terminal event");
    assert_eq!(terminal["type"], "response.completed");
    let output = terminal["response"]["output"]
        .as_array()
        .expect("completed output");
    assert_eq!(output[0]["type"], "message");
    assert_eq!(output[0]["phase"], "commentary");
    assert_eq!(output[1]["type"], "function_call");
}

#[tokio::test]
async fn test_codex_empty_stop_is_incomplete_instead_of_blank_final_answer() {
    let (_, events) = collect_codex_stream(vec![json!({
        "candidates": [{
            "finishReason": "STOP",
            "content": {"parts": [{"text": ""}]}
        }]
    })])
    .await;

    assert!(events.iter().all(|event| {
        !(event["type"] == "response.output_item.added" && event["item"]["type"] == "message")
    }));
    let terminal = events.last().expect("terminal event");
    assert_eq!(terminal["type"], "response.incomplete");
    assert_eq!(terminal["response"]["status"], "incomplete");
    assert_eq!(terminal["response"]["error"]["code"], "empty_response");
}

#[tokio::test]
async fn test_openai_streaming_usage_only_at_end() {
    // Chunk 1: Partial content, no usage
    let chunk1_json = json!({
        "candidates": [{
            "content": {
                "parts": [{ "text": "Hello" }]
            }
        }]
    });

    // Chunk 2: Finish reason + Usage metadata
    let chunk2_json = json!({
        "candidates": [{
            "finishReason": "STOP",
            "content": {
                "parts": [{ "text": "" }]
            }
        }],
        "usageMetadata": {
            "promptTokenCount": 5,
            "candidatesTokenCount": 2,
            "totalTokenCount": 7
        }
    });

    // Use a helper to create the stream items compatible with the required signature
    let items: Vec<Result<Bytes, reqwest::Error>> = vec![
        Ok(Bytes::from(format!("data: {}\n\n", chunk1_json))),
        Ok(Bytes::from(format!("data: {}\n\n", chunk2_json))),
    ];

    let gemini_stream = Box::pin(stream::iter(items));

    let mut openai_stream = create_openai_sse_stream(
        gemini_stream,
        "gemini-1.5-flash".to_string(),
        "test-session".to_string(),
        0,
        None,
        false,
    );

    let mut chunks = Vec::new();
    while let Some(result) = openai_stream.next().await {
        if let Ok(bytes) = result {
            let s = String::from_utf8_lossy(&bytes).to_string();
            for line in s.lines() {
                if line.starts_with("data: ") && !line.contains("[DONE]") {
                    chunks.push(line.to_string());
                }
            }
        }
    }

    let mut found_usage = false;
    let mut found_finish = false;

    for (i, chunk_str) in chunks.iter().enumerate() {
        let json_str = chunk_str.trim_start_matches("data: ").trim();
        let json: Value = serde_json::from_str(json_str).unwrap();

        if i < chunks.len() - 1 {
            assert!(
                json.get("usage").is_none(),
                "Usage should not be in intermediate chunks. Found in chunk {}",
                i
            );
        } else {
            if let Some(usage) = json.get("usage") {
                found_usage = true;
                assert_eq!(usage["prompt_tokens"], 5);
                assert_eq!(usage["completion_tokens"], 2);
                assert_eq!(usage["total_tokens"], 7);
            }
            if let Some(choices) = json.get("choices") {
                if let Some(choice) = choices.get(0) {
                    if let Some(finish_reason) = choice.get("finish_reason") {
                        if finish_reason.as_str() == Some("stop") {
                            found_finish = true;
                        }
                    }
                }
            }
        }
    }
    assert!(found_usage, "Usage should be found in the last chunk");
    assert!(found_finish, "Finish reason should be strictly 'stop'");
}

#[tokio::test]
async fn test_openai_streaming_with_include_usage_true() {
    let chunk1_json = json!({
        "candidates": [{
            "content": {
                "parts": [{ "text": "Hello" }]
            }
        }]
    });

    let chunk2_json = json!({
        "candidates": [{
            "finishReason": "STOP",
            "content": {
                "parts": [{ "text": " world" }]
            }
        }],
        "usageMetadata": {
            "promptTokenCount": 10,
            "candidatesTokenCount": 5,
            "totalTokenCount": 15
        }
    });

    let items: Vec<Result<Bytes, reqwest::Error>> = vec![
        Ok(Bytes::from(format!("data: {}\n\n", chunk1_json))),
        Ok(Bytes::from(format!("data: {}\n\n", chunk2_json))),
    ];

    let gemini_stream = Box::pin(stream::iter(items));

    let mut openai_stream = create_openai_sse_stream(
        gemini_stream,
        "gemini-1.5-flash".to_string(),
        "test-session".to_string(),
        0,
        None,
        true, // include_usage = true
    );

    let mut chunks = Vec::new();
    while let Some(result) = openai_stream.next().await {
        if let Ok(bytes) = result {
            let s = String::from_utf8_lossy(&bytes).to_string();
            for line in s.lines() {
                if line.starts_with("data: ") && !line.contains("[DONE]") {
                    chunks.push(line.to_string());
                }
            }
        }
    }

    // With include_usage: true, the last chunk before [DONE] MUST have choices: [] and usage
    assert!(
        chunks.len() >= 3,
        "Expected at least 3 chunks: partial, finish, usage"
    );
    let last_chunk: Value =
        serde_json::from_str(chunks.last().unwrap().trim_start_matches("data: ").trim()).unwrap();
    assert_eq!(last_chunk["choices"], json!([]));
    assert!(
        last_chunk.get("usage").is_some(),
        "Standalone usage chunk must contain usage"
    );
    let usage = &last_chunk["usage"];
    assert_eq!(usage["prompt_tokens"], 10);
    assert_eq!(usage["completion_tokens"], 5);
    assert_eq!(usage["total_tokens"], 15);
}

#[tokio::test]
async fn test_hermes_stream_without_include_usage_never_emits_empty_choices() {
    // Simulates real Gemini streaming where finishReason arrives in chunk 1,
    // and usageMetadata arrives in chunk 2 without candidates.
    let chunk1_json = json!({
        "candidates": [{
            "finishReason": "STOP",
            "content": {
                "parts": [{ "text": "Task complete." }]
            }
        }]
    });

    let chunk2_json = json!({
        "usageMetadata": {
            "promptTokenCount": 20,
            "candidatesTokenCount": 8,
            "totalTokenCount": 28
        }
    });

    let items: Vec<Result<Bytes, reqwest::Error>> = vec![
        Ok(Bytes::from(format!("data: {}\n\n", chunk1_json))),
        Ok(Bytes::from(format!("data: {}\n\n", chunk2_json))),
    ];

    let gemini_stream = Box::pin(stream::iter(items));

    // Hermes / standard OpenAI Python SDK default: include_usage = false
    let mut openai_stream = create_openai_sse_stream(
        gemini_stream,
        "gemini-1.5-flash".to_string(),
        "hermes-session".to_string(),
        0,
        None,
        false,
    );

    let mut chunks = Vec::new();
    while let Some(result) = openai_stream.next().await {
        if let Ok(bytes) = result {
            let s = String::from_utf8_lossy(&bytes).to_string();
            for line in s.lines() {
                if line.starts_with("data: ") && !line.contains("[DONE]") {
                    chunks.push(line.to_string());
                }
            }
        }
    }

    // CRITICAL: Ensure NO chunk has empty choices: []!
    // Hermes iterates `chunk.choices[0]`. An empty choices: [] chunk crashes Hermes with IndexError!
    for chunk_str in &chunks {
        let json_str = chunk_str.trim_start_matches("data: ").trim();
        let json: Value = serde_json::from_str(json_str).unwrap();
        if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
            assert!(
                !choices.is_empty(),
                "Crash hazard! Found empty choices: [] chunk when include_usage=false: {}",
                json_str
            );
            // Verify choices[0] can be accessed without panic
            assert!(choices.get(0).is_some());
        }
    }
}

#[tokio::test]
async fn test_openai_streaming_reasoning_content() {
    // Chunk with thought part
    let chunk_json = json!({
        "candidates": [{
            "content": {
                "parts": [
                    { "text": "Thinking...", "thought": true },
                    { "text": "Hello world" }
                ]
            }
        }]
    });

    let items: Vec<Result<Bytes, reqwest::Error>> =
        vec![Ok(Bytes::from(format!("data: {}\n\n", chunk_json)))];

    let gemini_stream = Box::pin(stream::iter(items));

    let mut openai_stream = create_openai_sse_stream(
        gemini_stream,
        "gemini-1.5-flash".to_string(),
        "test-session".to_string(),
        0,
        None,
        false,
    );

    let mut chunks = Vec::new();
    while let Some(result) = openai_stream.next().await {
        if let Ok(bytes) = result {
            let s = String::from_utf8_lossy(&bytes).to_string();
            for line in s.lines() {
                if line.starts_with("data: ") && !line.contains("[DONE]") {
                    chunks.push(line.to_string());
                }
            }
        }
    }

    let mut has_reasoning = false;
    let mut has_content = false;

    for chunk_str in &chunks {
        let json_str = chunk_str.trim_start_matches("data: ").trim();
        let json: Value = serde_json::from_str(json_str).unwrap();

        if let Some(choices) = json.get("choices") {
            if let Some(choice) = choices.get(0) {
                if let Some(delta) = choice.get("delta") {
                    if let Some(rc) = delta.get("reasoning_content") {
                        assert_eq!(rc.as_str().unwrap(), "Thinking...");
                        has_reasoning = true;
                        // content should be null or not match thinking process
                        if let Some(content) = delta.get("content") {
                            assert!(content.is_null());
                        }
                    }
                    if let Some(c) = delta.get("content") {
                        if c.is_string() {
                            assert_eq!(c.as_str().unwrap(), "Hello world");
                            has_content = true;
                            assert!(delta.get("reasoning_content").is_none());
                        }
                    }
                }
            }
        }
    }

    assert!(has_reasoning, "Should stream reasoning_content");
    assert!(has_content, "Should stream content");
}

#[tokio::test]
async fn test_streaming_malformed_function_call_never_injects_hardcoded_online_prompt() {
    let chunk_json = json!({
        "candidates": [{
            "content": {
                "parts": [
                    { "text": "Reasoning about weather...", "thought": true }
                ]
            },
            "finishReason": "MALFORMED_FUNCTION_CALL"
        }]
    });

    let items: Vec<Result<Bytes, reqwest::Error>> =
        vec![Ok(Bytes::from(format!("data: {}\n\n", chunk_json)))];

    let gemini_stream = Box::pin(stream::iter(items));

    let mut openai_stream = create_openai_sse_stream(
        gemini_stream,
        "gemini-3.7-flash".to_string(),
        "test-malformed-session".to_string(),
        0,
        None,
        false,
    );

    let mut all_content = String::new();
    let mut final_finish_reason: Option<String> = None;

    while let Some(result) = openai_stream.next().await {
        if let Ok(bytes) = result {
            let s = String::from_utf8_lossy(&bytes).to_string();
            for line in s.lines() {
                if line.starts_with("data: ") && !line.contains("[DONE]") {
                    let json_str = line.trim_start_matches("data: ").trim();
                    if let Ok(json) = serde_json::from_str::<Value>(json_str) {
                        if let Some(choices) = json.get("choices").and_then(|c| c.as_array()) {
                            if let Some(choice) = choices.first() {
                                if let Some(delta) = choice.get("delta") {
                                    if let Some(c) = delta.get("content").and_then(|c| c.as_str()) {
                                        all_content.push_str(c);
                                    }
                                }
                                if let Some(fr) =
                                    choice.get("finish_reason").and_then(|f| f.as_str())
                                {
                                    final_finish_reason = Some(fr.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // 验证绝对不会被注入任何臆测性的天气/联网假文本
    assert!(
        all_content.is_empty(),
        "Expected empty content, got: {}",
        all_content
    );
    assert_eq!(final_finish_reason, Some("stop".to_string()));
}
