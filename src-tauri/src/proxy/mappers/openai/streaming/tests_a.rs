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
async fn responses_store_false_emits_complete_tool_call_without_caching_it() {
    let (_, events) = collect_codex_stream_with_cache(vec![json!({
        "response": {"candidates": [{
            "finishReason": "STOP",
            "content": {"parts": [{"functionCall": {"name":"shell_command", "args":{"command":"pwd"}}}]}
        }]}
    })], false).await;
    let completed = events
        .iter()
        .find(|event| event["type"] == "response.completed")
        .expect("completed response");
    let call = completed["response"]["output"]
        .as_array()
        .expect("output")
        .iter()
        .find(|item| item["type"] == "function_call")
        .expect("function call");
    let call_id = call["call_id"].as_str().expect("call id");
    assert_eq!(call["name"], "shell_command");
    assert_eq!(
        serde_json::from_str::<Value>(call["arguments"].as_str().expect("arguments"))
            .expect("JSON arguments"),
        json!({"command":"pwd"})
    );
    assert!(crate::proxy::handlers::openai::get_cached_tool_call(call_id).is_none());
    assert!(events
        .iter()
        .any(|event| event["type"] == "response.output_item.done"
            && event["item"]["call_id"] == call_id));
}

#[tokio::test]
async fn codex_response_id_matches_saved_session_key() {
    let response_id = format!("resp-test-{}", uuid::Uuid::new_v4());
    let upstream = vec![Ok::<Bytes, String>(Bytes::from(
        "data: {\"response\":{\"candidates\":[{\"finishReason\":\"STOP\",\"content\":{\"parts\":[{\"thought\":true,\"text\":\"Checking.\"},{\"text\":\"done\"}]}}]}}\n\n",
    ))];
    let (completion_tx, completion_rx) =
        tokio::sync::oneshot::channel::<(Vec<Value>, tokio::sync::oneshot::Sender<()>)>();
    let save_response_id = response_id.clone();
    let save_task = tokio::spawn(async move {
        let (outputs, ack_tx) = completion_rx.await.expect("completed output");
        crate::proxy::http_session_store::save_session_delta(
            save_response_id,
            None,
            vec![json!({"id": "user-1", "role": "user", "content": "hello"})],
            outputs,
            String::new(),
            "gemini-pro-agent".to_string(),
            "routing-test-session".to_string(),
        )
        .await;
        ack_tx.send(()).expect("acknowledge session save");
    });
    let mut stream = create_codex_sse_stream(
        Box::pin(stream::iter(upstream)),
        "gemini-pro-agent".to_string(),
        "test-session".to_string(),
        1,
        0,
        response_id.clone(),
        Some(completion_tx),
        true,
    );

    let mut raw = String::new();
    let mut saw_completed = false;
    while let Some(item) = stream.next().await {
        let item = item.expect("stream item");
        let text = String::from_utf8_lossy(&item);
        if text.contains("event: response.completed") {
            saw_completed = true;
            let restored = crate::proxy::http_session_store::get_session(&response_id)
                .await
                .expect("session exists when response.completed is visible");
            assert_eq!(restored.input_items[0]["id"], "user-1");
            let completed: Value = serde_json::from_str(
                text.lines()
                    .find_map(|line| line.strip_prefix("data: "))
                    .expect("completion data"),
            )
            .expect("completion JSON");
            assert_eq!(restored.input_items[1]["type"], "reasoning");
            assert_eq!(restored.input_items[1]["summary"][0]["text"], "Checking.");
            assert_eq!(
                &restored.input_items[1..],
                completed["response"]["output"].as_array().expect("output")
            );
            assert!(restored
                .input_items
                .iter()
                .any(|item| item["role"] == "assistant"));
        }
        raw.push_str(&text);
    }
    save_task.await.expect("session save task");

    assert!(saw_completed);
    assert!(raw.contains(&format!("\"id\":\"{response_id}\"")));
}

#[test]
fn response_branches_store_signatures_under_their_own_response_ids() {
    let branch_a = format!("resp-signature-a-{}", uuid::Uuid::new_v4());
    let branch_b = format!("resp-signature-b-{}", uuid::Uuid::new_v4());
    let signature_a = "a".repeat(64);
    let signature_b = "b".repeat(64);
    store_thought_signature(&signature_a, &branch_a, 1);
    store_thought_signature(&signature_b, &branch_b, 1);

    assert_eq!(
        crate::proxy::SignatureCache::global().get_session_signature(&branch_a),
        Some(signature_a)
    );
    assert_eq!(
        crate::proxy::SignatureCache::global().get_session_signature(&branch_b),
        Some(signature_b)
    );
}

#[tokio::test]
async fn test_codex_reasoning_summary_and_tool_are_distinct_output_items() {
    let (raw, events) = collect_codex_stream(vec![
        json!({
            "response": {
                "candidates": [{
                    "content": {"parts": [{"text": "Inspecting the workspace.", "thought": true}]}
                }]
            }
        }),
        json!({
            "response": {
                "candidates": [{
                    "content": {"parts": [{
                        "functionCall": {"name": "shell_command", "args": {"command": "Get-ChildItem"}}
                    }]}
                }]
            }
        }),
        json!({
            "response": {
                "candidates": [{
                    "finishReason": "STOP",
                    "content": {"parts": [{"text": ""}]}
                }]
            }
        }),
    ])
    .await;

    assert!(raw.starts_with("event: response.created\ndata: "));
    let names: Vec<&str> = events
        .iter()
        .filter_map(|event| event["type"].as_str())
        .collect();
    assert_eq!(names[0], "response.created");
    assert_eq!(names[1], "response.in_progress");
    assert!(!names.contains(&"response.output_text.delta"));
    assert_eq!(
        &names[3..8],
        &[
            "response.reasoning_summary_part.added",
            "response.reasoning_summary_text.delta",
            "response.reasoning_summary_text.done",
            "response.reasoning_summary_part.done",
            "response.output_item.done",
        ]
    );
    assert!(names.contains(&"response.function_call_arguments.delta"));
    assert_eq!(names.last().copied(), Some("response.completed"));

    for (expected, event) in events.iter().enumerate() {
        assert_eq!(event["sequence_number"], expected as u64);
    }
    assert!(events
        .iter()
        .filter(|event| event["type"]
            .as_str()
            .is_some_and(|name| name.starts_with("response.reasoning_summary_")))
        .all(|event| event["summary_index"] == 0
            && event["output_index"] == 0
            && event["item_id"] == events[2]["item"]["id"]));

    let added: Vec<&Value> = events
        .iter()
        .filter(|event| event["type"] == "response.output_item.added")
        .collect();
    assert_eq!(added.len(), 2);
    assert_eq!(added[0]["item"]["type"], "reasoning");
    assert_eq!(added[0]["item"]["summary"], json!([]));
    assert_eq!(added[0]["item"]["status"], "in_progress");
    assert!(added[0]["item"]["id"]
        .as_str()
        .is_some_and(|id| id.starts_with("rs_")));
    assert_eq!(added[0]["output_index"], 0);
    assert_eq!(added[1]["item"]["type"], "function_call");
    assert_eq!(added[1]["output_index"], 1);

    let completed = events.last().expect("terminal event");
    let output = completed["response"]["output"]
        .as_array()
        .expect("completed output");
    assert_eq!(output.len(), 2);
    assert_eq!(output[0]["type"], "reasoning");
    assert_eq!(output[0]["summary"][0]["text"], "Inspecting the workspace.");
    assert_eq!(output[0], events[7]["item"]);
    assert_eq!(events[8]["item"], added[1]["item"]);
    assert_eq!(events[9]["item_id"], output[1]["id"]);
    assert_eq!(added[1]["item"]["call_id"], output[1]["call_id"]);
    assert!(!raw.contains("**Thinking**"));
    assert_eq!(output[1]["type"], "function_call");
}

#[tokio::test]
async fn test_codex_final_message_is_promoted_and_persisted() {
    let (_, events) = collect_codex_stream(vec![
        json!({"candidates": [{"content": {"parts": [{"text": "Checking ", "thought": true}]}}]}),
        json!({"candidates": [{"content": {"parts": [{"text": "results.", "thought": true}]}}]}),
        json!({
            "candidates": [{
                "content": {"parts": [{"text": "The task is complete."}]}
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

    let added = events
        .iter()
        .find(|event| {
            event["type"] == "response.output_item.added" && event["item"]["type"] == "message"
        })
        .expect("message added");
    assert_eq!(added["item"]["phase"], "commentary");

    let done = events
        .iter()
        .find(|event| {
            event["type"] == "response.output_item.done" && event["item"]["type"] == "message"
        })
        .expect("message done");
    assert_eq!(done["item"]["phase"], "final_answer");
    assert_eq!(done["item"]["content"][0]["text"], "The task is complete.");
    assert_eq!(done["item"]["content"][0]["annotations"], json!([]));
    assert_eq!(
        events[3]["part"],
        json!({"type": "summary_text", "text": ""})
    );
    assert_eq!(events[4]["delta"], "Checking ");
    assert_eq!(events[5]["delta"], "results.");
    assert_eq!(events[4]["item_id"], events[5]["item_id"]);
    assert_eq!(events[6]["text"], "Checking results.");
    assert_eq!(
        events[7]["part"],
        json!({"type": "summary_text", "text": "Checking results."})
    );
    assert_eq!(events[8]["type"], "response.output_item.done");
    assert_eq!(events[9]["item"], added["item"]);

    let terminal = events.last().expect("terminal event");
    assert_eq!(terminal["type"], "response.completed");
    assert_eq!(terminal["response"]["status"], "completed");
    assert_eq!(terminal["response"]["output"][0], events[8]["item"]);
    assert_eq!(terminal["response"]["output"][1], done["item"]);
    assert_eq!(terminal["response"]["output"][1]["phase"], "final_answer");
    assert_eq!(
        terminal["response"]["output"][1]["content"][0]["text"],
        "The task is complete."
    );
}

#[tokio::test]
async fn test_codex_reasoning_summary_is_closed_at_stream_end() {
    let (_, events) = collect_codex_stream(vec![json!({
        "candidates": [{"content": {"parts": [{"text": "Checking.", "thought": true}]}}]
    })])
    .await;
    assert_eq!(events[5]["type"], "response.reasoning_summary_text.done");
    assert_eq!(events[5]["text"], "Checking.");
    assert_eq!(events[6]["type"], "response.reasoning_summary_part.done");
    assert_eq!(
        events[6]["part"],
        json!({"type": "summary_text", "text": "Checking."})
    );
    assert_eq!(events[7]["type"], "response.output_item.done");
    assert_eq!(events[7]["item"]["type"], "reasoning");
    assert_eq!(events[7]["item"]["status"], "completed");
    assert_eq!(events[7]["item"]["summary"], json!([events[6]["part"]]));
    let terminal = events.last().expect("terminal event");
    assert_eq!(terminal["response"]["output"], json!([events[7]["item"]]));
    assert_eq!(terminal["type"], "response.incomplete");
    assert_eq!(
        terminal["response"]["incomplete_details"]["reason"],
        "interrupted"
    );
    assert_eq!(terminal["response"]["error"]["code"], "empty_response");
}

#[tokio::test]
async fn test_codex_late_thought_does_not_mix_into_final_answer() {
    let (raw, events) = collect_codex_stream(vec![
        json!({
            "candidates": [{
                "content": {"parts": [{"text": "The task is complete."}]}
            }]
        }),
        json!({
            "candidates": [{
                "content": {"parts": [{"text": "late private thought", "thought": true}]}
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

    assert!(!raw.contains("late private thought"));
    assert!(events.iter().all(|event| {
        !(event["type"] == "response.output_item.added"
            && event["item"]["id"]
                .as_str()
                .is_some_and(|id| id.starts_with("rs_")))
    }));
    let terminal = events.last().expect("terminal event");
    assert_eq!(terminal["type"], "response.completed");
    assert_eq!(
        terminal["response"]["output"][0]["content"][0]["text"],
        "The task is complete."
    );
}
