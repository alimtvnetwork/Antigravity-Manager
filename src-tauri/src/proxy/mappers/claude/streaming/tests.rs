// Streaming tests (split from streaming.rs).
// Test-only module, declared #[cfg(test)] in mod.rs.
use super::{PartProcessor, SignatureManager, StreamingState};
use crate::proxy::mappers::claude::models::{FunctionCall, GeminiPart};
use serde_json::json;

#[test]
fn test_signature_manager() {
    let mut mgr = SignatureManager::new();
    assert!(!mgr.has_pending());

    mgr.store(Some("sig123".to_string()));
    assert!(mgr.has_pending());

    let sig = mgr.consume();
    assert_eq!(sig, Some("sig123".to_string()));
    assert!(!mgr.has_pending());
}

#[test]
fn test_streaming_state_emit() {
    let state = StreamingState::new();
    let chunk = state.emit("test_event", json!({"foo": "bar"}));

    let s = String::from_utf8(chunk.to_vec()).unwrap();
    assert!(s.contains("event: test_event"));
    assert!(s.contains("\"foo\":\"bar\""));
}

#[test]
fn test_process_function_call_deltas() {
    let mut state = StreamingState::new();
    let mut processor = PartProcessor::new(&mut state);

    let fc = FunctionCall {
        name: "test_tool".to_string(),
        args: Some(json!({"arg": "value"})),
        id: Some("call_123".to_string()),
    };

    // Create a dummy GeminiPart with function_call
    let part = GeminiPart {
        text: None,
        function_call: Some(fc),
        inline_data: None,
        thought: None,
        thought_signature: None,
        function_response: None,
    };

    let chunks = processor.process(&part);
    let output = chunks
        .iter()
        .map(|b| String::from_utf8(b.to_vec()).unwrap())
        .collect::<Vec<_>>()
        .join("");

    // Verify sequence:
    // 1. content_block_start with empty input
    assert!(output.contains(r#""type":"content_block_start""#));
    assert!(output.contains(r#""name":"test_tool""#));
    assert!(output.contains(r#""input":{}"#));

    // 2. input_json_delta with serialized args
    assert!(output.contains(r#""type":"content_block_delta""#));
    assert!(output.contains(r#""type":"input_json_delta""#));
    // partial_json should contain escaped JSON string
    assert!(output.contains(r#"partial_json":"{\"arg\":\"value\"}"#));

    // 3. content_block_stop
    assert!(output.contains(r#""type":"content_block_stop""#));
}

/// [FIX #Bug4] Tool with args=None MUST still emit input_json_delta with "{}"
#[test]
fn test_process_function_call_no_args_emits_empty_delta() {
    let mut state = StreamingState::new();
    let mut processor = PartProcessor::new(&mut state);

    let fc = FunctionCall {
        name: "EnterPlanMode".to_string(),
        args: None, // No args at all
        id: Some("call_no_args".to_string()),
    };

    let part = GeminiPart {
        text: None,
        function_call: Some(fc),
        inline_data: None,
        thought: None,
        thought_signature: None,
        function_response: None,
    };

    let chunks = processor.process(&part);
    let output = chunks
        .iter()
        .map(|b| String::from_utf8(b.to_vec()).unwrap())
        .collect::<Vec<_>>()
        .join("");

    // Must have tool_use block start
    assert!(output.contains(r#""type":"content_block_start""#));
    assert!(output.contains(r#""name":"EnterPlanMode""#));

    // [FIX #Bug4] Must emit input_json_delta even for None args
    assert!(
        output.contains(r#""type":"input_json_delta""#),
        "MUST emit input_json_delta even when args is None; output={}",
        &output[..output.len().min(600)]
    );
    assert!(
        output.contains(r#""partial_json":"{}""#),
        "input_json_delta must be empty JSON object for None args"
    );

    // Must close block
    assert!(output.contains(r#""type":"content_block_stop""#));
    assert!(state.used_tool);
}

/// [FIX #Bug2] Tool with args=Some({}) (empty obj) MUST still emit input_json_delta
#[test]
fn test_process_function_call_empty_args_emits_delta() {
    let mut state = StreamingState::new();
    let mut processor = PartProcessor::new(&mut state);

    let fc = FunctionCall {
        name: "SomeTool".to_string(),
        args: Some(json!({})), // Empty args object
        id: Some("call_empty".to_string()),
    };

    let part = GeminiPart {
        text: None,
        function_call: Some(fc),
        inline_data: None,
        thought: None,
        thought_signature: None,
        function_response: None,
    };

    let chunks = processor.process(&part);
    let output = chunks
        .iter()
        .map(|b| String::from_utf8(b.to_vec()).unwrap())
        .collect::<Vec<_>>()
        .join("");

    // [FIX #Bug2] Must emit input_json_delta for empty object args
    assert!(
        output.contains(r#""type":"input_json_delta""#),
        "Must emit input_json_delta for empty args object; output={}",
        &output[..output.len().min(600)]
    );
    assert!(state.used_tool);
}

// -------------------------------------------------------------------------
// [FIX #3379] Tests: call:default_api:* leakage recovery
// -------------------------------------------------------------------------

/// Helper: build a PartProcessor with registered tool names
fn make_processor_with_tools<'a>(
    state: &'a mut StreamingState,
    tools: Vec<&str>,
) -> PartProcessor<'a> {
    state.set_registered_tool_names(tools.into_iter().map(|s| s.to_string()).collect());
    PartProcessor::new(state)
}

/// Collect all SSE chunks into a single string
fn chunks_to_string(chunks: &[bytes::Bytes]) -> String {
    chunks
        .iter()
        .map(|b| String::from_utf8_lossy(b).to_string())
        .collect::<Vec<_>>()
        .join("")
}

#[test]
fn test_3379_positive_recovery_standard_json() {
    // Positive: registered tool, standard JSON args, no prose → should recover
    let mut state = StreamingState::new();
    let mut processor = make_processor_with_tools(&mut state, vec!["Read"]);

    let text = r#"call:default_api:Read{"file_path":"/tmp/foo.txt","limit":100}"#;
    let part = GeminiPart {
        text: Some(text.to_string()),
        function_call: None,
        inline_data: None,
        thought: None,
        thought_signature: None,
        function_response: None,
    };
    let chunks = processor.process(&part);
    let output = chunks_to_string(&chunks);

    // Should produce a tool_use block, not a text_delta
    assert!(
        output.contains(r#""type":"content_block_start""#),
        "Expected tool_use block_start, got: {}",
        output
    );
    assert!(
        output.contains(r#""name":"Read""#),
        "Expected tool name Read"
    );
    assert!(
        !output.contains("text_delta"),
        "Must NOT produce text_delta for recovered call"
    );
    assert!(state.used_tool, "used_tool must be true after recovery");
}

#[test]
fn test_3379_negative_tool_not_registered() {
    // G4 guard: tool name not in whitelist → text_delta
    let mut state = StreamingState::new();
    let mut processor = make_processor_with_tools(&mut state, vec!["Write"]);

    let text = r#"call:default_api:Read{"file_path":"/tmp/foo.txt"}"#;
    let part = GeminiPart {
        text: Some(text.to_string()),
        function_call: None,
        inline_data: None,
        thought: None,
        thought_signature: None,
        function_response: None,
    };
    let chunks = processor.process(&part);
    let output = chunks_to_string(&chunks);

    assert!(
        output.contains("text_delta"),
        "Unregistered tool must fall through to text_delta"
    );
    assert!(!state.used_tool, "used_tool must remain false");
}

#[test]
fn test_3379_negative_no_tools_registered() {
    // G1 guard: no tools in request → text_delta
    let mut state = StreamingState::new(); // no registered_tool_names
    let mut processor = PartProcessor::new(&mut state);

    let text = r#"call:default_api:Read{"file_path":"/tmp/foo.txt"}"#;
    let part = GeminiPart {
        text: Some(text.to_string()),
        function_call: None,
        inline_data: None,
        thought: None,
        thought_signature: None,
        function_response: None,
    };
    let chunks = processor.process(&part);
    let output = chunks_to_string(&chunks);

    assert!(
        output.contains("text_delta"),
        "Empty registered_tool_names must fall through to text_delta"
    );
}

#[test]
fn test_3379_negative_surrounding_prose() {
    // G5 guard: text not solely the call expression → text_delta
    let mut state = StreamingState::new();
    let mut processor = make_processor_with_tools(&mut state, vec!["Read"]);

    let text = "Here is what I am doing: call:default_api:Read{\"file_path\":\"/tmp/foo.txt\"}";
    let part = GeminiPart {
        text: Some(text.to_string()),
        function_call: None,
        inline_data: None,
        thought: None,
        thought_signature: None,
        function_response: None,
    };
    let chunks = processor.process(&part);
    let output = chunks_to_string(&chunks);

    assert!(
        output.contains("text_delta"),
        "Prose-wrapped call must fall through to text_delta"
    );
    assert!(!state.used_tool);
}

#[test]
fn test_3379_negative_broken_json_args() {
    // G6 guard: args not valid JSON → text_delta
    let mut state = StreamingState::new();
    let mut processor = make_processor_with_tools(&mut state, vec!["Read"]);

    let text = "call:default_api:Read{file_path: NOT JSON !!!}";
    let part = GeminiPart {
        text: Some(text.to_string()),
        function_call: None,
        inline_data: None,
        thought: None,
        thought_signature: None,
        function_response: None,
    };
    let chunks = processor.process(&part);
    let output = chunks_to_string(&chunks);

    assert!(
        output.contains("text_delta"),
        "Broken JSON args must fall through to text_delta"
    );
    assert!(!state.used_tool);
}

#[test]
fn test_3379_negative_text_delta_already_emitted() {
    // G7 guard: text_delta already emitted this turn → skip recovery
    let mut state = StreamingState::new();
    state.set_registered_tool_names(vec!["Read".to_string()]);
    // Simulate a prior text_delta having been emitted
    state.text_delta_emitted_this_turn = true;

    let mut processor = PartProcessor::new(&mut state);
    let text = r#"call:default_api:Read{"file_path":"/tmp/foo.txt"}"#;
    let part = GeminiPart {
        text: Some(text.to_string()),
        function_call: None,
        inline_data: None,
        thought: None,
        thought_signature: None,
        function_response: None,
    };
    let chunks = processor.process(&part);
    let output = chunks_to_string(&chunks);

    assert!(
        output.contains("text_delta"),
        "G7 must prevent recovery after text_delta was emitted"
    );
    assert!(!state.used_tool);
}

#[test]
fn test_3379_parse_loose_json_standard() {
    // parse_loose_json_args: standard JSON passes Phase 1
    let result = PartProcessor::parse_loose_json_args(r#"{"file_path":"/tmp/foo","limit":100}"#);
    assert!(result.is_some());
    let v = result.unwrap();
    assert_eq!(v["file_path"], "/tmp/foo");
    assert_eq!(v["limit"], 100);
}

#[test]
fn test_3379_parse_loose_json_empty() {
    // Empty args → valid empty object
    let result = PartProcessor::parse_loose_json_args("");
    assert_eq!(result, Some(serde_json::json!({})));
}

#[test]
fn test_3379_regression_native_function_call_unaffected() {
    // Regression: native functionCall must still produce tool_use unaffected
    let mut state = StreamingState::new();
    state.set_registered_tool_names(vec!["Read".to_string()]);
    let mut processor = PartProcessor::new(&mut state);

    let fc = FunctionCall {
        name: "Read".to_string(),
        args: Some(json!({"file_path": "/tmp/native.txt"})),
        id: Some("call_native_001".to_string()),
    };
    let part = GeminiPart {
        text: None,
        function_call: Some(fc),
        inline_data: None,
        thought: None,
        thought_signature: None,
        function_response: None,
    };
    let chunks = processor.process(&part);
    let output = chunks_to_string(&chunks);

    assert!(
        output.contains(r#""type":"content_block_start""#),
        "Native functionCall must still produce tool_use"
    );
    assert!(output.contains(r#""name":"Read""#));
    assert!(!output.contains("text_delta"));
    assert!(state.used_tool);
}
