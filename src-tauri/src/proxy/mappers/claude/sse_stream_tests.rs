// SSE stream tests (split from mod.rs).
// Test-only module.
use super::sse_stream::*;
use super::streaming::StreamingState;

use super::*;

#[test]
fn test_process_sse_line_done() {
    let mut state = StreamingState::new();
    let result = process_sse_line("data: [DONE]", &mut state, "test_id", "test@example.com");
    assert!(result.is_some());
    let chunks = result.unwrap();
    assert!(!chunks.is_empty());

    let all_text: String = chunks
        .iter()
        .map(|b| String::from_utf8(b.to_vec()).unwrap_or_default())
        .collect();
    assert!(all_text.contains("message_stop"));
}

#[test]
fn test_process_sse_line_with_text() {
    let mut state = StreamingState::new();

    let test_data = r#"data: {"candidates":[{"content":{"parts":[{"text":"Hello"}]}}],"usageMetadata":{},"modelVersion":"test","responseId":"123"}"#;

    let result = process_sse_line(test_data, &mut state, "test_id", "test@example.com");
    assert!(result.is_some());

    let chunks = result.unwrap();
    assert!(!chunks.is_empty());

    // 应该包含 message_start 和 text delta
    let all_text: String = chunks
        .iter()
        .map(|b| String::from_utf8(b.to_vec()).unwrap_or_default())
        .collect();

    assert!(all_text.contains("message_start"));
    assert!(all_text.contains("content_block_start"));
    assert!(all_text.contains("Hello"));
}

#[tokio::test]
async fn test_thinking_only_interruption_recovery() {
    use futures::StreamExt;

    // 1. 模拟一个只发送 Thinking 然后就结束的流
    let mock_stream = async_stream::stream! {
        // 发送 Thinking 块
        let thinking_json = serde_json::json!({
            "candidates": [{
                "content": {
                    "parts": [{ "text": "Thinking...", "thought": true }]
                }
            }],
            "modelVersion": "gemini-2.0-flash-thinking",
            "responseId": "msg_interrupted"
        });
        yield Ok::<_, String>(bytes::Bytes::from(format!("data: {}\n\n", thinking_json)));

        // 然后突然结束 (没有 Text, 没有 Usage, 直接 None)
    };

    // 2. 创建转换后的流
    let mut claude_stream = create_claude_sse_stream(
        Box::pin(mock_stream),
        "trace_test".to_string(),
        "test@example.com".to_string(),
        None,
        false,
        1_000,
        None,
        1,          // message_count
        None,       // client_adapter
        Vec::new(), // registered_tool_names
    );

    // 3. 收集输出
    let mut all_chunks = Vec::new();
    while let Some(result) = claude_stream.next().await {
        if let Ok(bytes) = result {
            all_chunks.push(String::from_utf8(bytes.to_vec()).unwrap());
        }
    }
    let output = all_chunks.join("");

    // 4. 验证恢复逻辑
    // 必须包含 Thinking
    assert!(output.contains("Thinking..."));

    // 必须包含恢复的系统提示
    assert!(output.contains("Recovered by Antigravity"));

    // 必须包含模拟的 Usage
    assert!(output.contains("\"usage\":"));
    assert!(output.contains("\"output_tokens\":100")); // Should contain the recovery usage
}
