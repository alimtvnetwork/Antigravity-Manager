use super::tests_common::rec;
use super::*;
use serde_json::json;

#[test]
fn stores_full_thought_without_truncation() {
    let store = ThinkingStore::new();
    let long = "T".repeat(50_000);
    store.record("t:s1", rec(&long, "hello world", None));
    let mut contents = vec![json!({
        "role": "model",
        "parts": [{ "text": "hello world" }]
    })];
    let n = store.restore_gemini_contents("t:s1", &mut contents);
    assert_eq!(n, 1);
    assert_eq!(
        contents[0]["parts"][0]["text"].as_str().unwrap().len(),
        50_000
    );
    assert_eq!(contents[0]["parts"][0]["thought"], true);
    assert_eq!(
        contents[0]["parts"][0]["thoughtSignature"]
            .as_str()
            .unwrap()
            .len(),
        60
    );
}

#[test]
fn matches_by_visible_text_not_index() {
    let store = ThinkingStore::new();
    store.record("t:s1", rec("think-A", "answer A", None));
    store.record("t:s1", rec("think-B", "answer B", None));

    // Client dropped turn A, only sends B (different packet shape / rewind)
    let mut contents = vec![json!({
        "role": "model",
        "parts": [{ "text": "answer B" }]
    })];
    store.restore_gemini_contents("t:s1", &mut contents);
    assert_eq!(contents[0]["parts"][0]["text"], "think-B");
}

#[test]
fn matches_tool_id_when_text_missing() {
    let store = ThinkingStore::new();
    store.record("t:s1", rec("plan", "", Some("call_1")));
    let mut contents = vec![json!({
        "role": "model",
        "parts": [{
            "functionCall": { "name": "shell", "id": "call_1", "args": {} }
        }]
    })];
    store.restore_gemini_contents("t:s1", &mut contents);
    assert_eq!(contents[0]["parts"][0]["thought"], true);
    assert_eq!(contents[0]["parts"][0]["text"], "plan");
    assert_eq!(
        contents[0]["parts"][1]["thoughtSignature"]
            .as_str()
            .unwrap()
            .len(),
        60
    );
}

#[test]
fn replaces_placeholder_dots() {
    let store = ThinkingStore::new();
    store.record("t:s1", rec("full chain", "final", None));
    let mut contents = vec![json!({
        "role": "model",
        "parts": [
            { "text": "...", "thought": true },
            { "text": "final" }
        ]
    })];
    store.restore_gemini_contents("t:s1", &mut contents);
    assert_eq!(contents[0]["parts"][0]["text"], "full chain");
}

#[test]
fn tenant_isolation_and_end_session() {
    let store = ThinkingStore::new();
    store.record("aaa:chat", rec("secret-a", "hi", None));
    store.record("bbb:chat", rec("secret-b", "hi", None));

    let mut a = vec![json!({"role":"model","parts":[{"text":"hi"}]})];
    store.restore_gemini_contents("aaa:chat", &mut a);
    assert_eq!(a[0]["parts"][0]["text"], "secret-a");

    let result = store.end_session("aaa:chat");
    assert_eq!(result.deleted_turns, 1);
    let mut a2 = vec![json!({"role":"model","parts":[{"text":"hi"}]})];
    assert_eq!(store.restore_gemini_contents("aaa:chat", &mut a2), 0);

    let mut b = vec![json!({"role":"model","parts":[{"text":"hi"}]})];
    store.restore_gemini_contents("bbb:chat", &mut b);
    assert_eq!(b[0]["parts"][0]["text"], "secret-b");
}

#[test]
fn sanitize_rejects_junk() {
    assert_eq!(sanitize_session_id("abc/../x"), "abc..x");
    assert_eq!(sanitize_session_id(""), "sid-unknown");
}

#[test]
fn keyword_forces_thinking_without_client_flag() {
    assert!(model_forces_server_thinking("gemini-3-flash"));
    assert!(model_forces_server_thinking("gemini-3-pro"));
    assert!(model_forces_server_thinking("gemini-3-flash-agent"));
    assert!(model_forces_server_thinking("gemini-pro-agent"));
    assert!(model_forces_server_thinking("claude-sonnet-4-6"));
    assert!(!model_forces_server_thinking("gpt-4o"));
    assert!(!model_forces_server_thinking("gemini-3-pro-image"));
    assert!(!model_forces_server_thinking("gemini-3.1-flash-lite"));
    assert!(!model_forces_server_thinking("gemini-3-pro-preview"));
}

#[test]
fn same_fingerprint_updates_in_place() {
    let store = ThinkingStore::new();
    let key = format!("t:s1-{}", uuid::Uuid::new_v4());
    store.record(&key, rec("short", "same", None));
    store.record(&key, rec("much longer thought", "same", None));
    let stats = store.session_stats(&key).unwrap();
    assert_eq!(stats.0, 1);
    let mut contents = vec![json!({"role":"model","parts":[{"text":"same"}]})];
    store.restore_gemini_contents(&key, &mut contents);
    assert_eq!(contents[0]["parts"][0]["text"], "much longer thought");
    store.end_session(&key);
}

#[test]
fn tool_record_does_not_pollute_earlier_text_turns() {
    let store = ThinkingStore::new();
    // Turn 2 generated thinking + tool call
    store.record(
        "t:s1",
        rec("**Inferring User's Intention**", "", Some("call_54421")),
    );

    // Turn 0: "你好！" (pure text, no thought)
    // Turn 1: "当然是真的！" (pure text, no thought)
    // Turn 2: tool call "call_54421"
    let mut contents = vec![
        json!({
            "role": "model",
            "parts": [{ "text": "你好！我是 JeikCode AI 编程助手。" }]
        }),
        json!({
            "role": "model",
            "parts": [{ "text": "当然是真的！😄" }]
        }),
        json!({
            "role": "model",
            "parts": [{
                "functionCall": { "name": "shell", "id": "call_54421", "args": {} }
            }]
        }),
    ];

    let restored = store.restore_gemini_contents("t:s1", &mut contents);
    assert_eq!(restored, 1);

    // Turn 0 must NOT have thinking injected
    assert_eq!(contents[0]["parts"].as_array().unwrap().len(), 1);
    assert_eq!(
        contents[0]["parts"][0]["text"],
        "你好！我是 JeikCode AI 编程助手。"
    );
    assert!(contents[0]["parts"][0].get("thought").is_none());

    // Turn 1 must NOT have thinking injected
    assert_eq!(contents[1]["parts"].as_array().unwrap().len(), 1);
    assert_eq!(contents[1]["parts"][0]["text"], "当然是真的！😄");
    assert!(contents[1]["parts"][0].get("thought").is_none());

    // Turn 2 MUST have thinking injected and matched with call_54421
    assert_eq!(contents[2]["parts"][0]["thought"], true);
    assert_eq!(
        contents[2]["parts"][0]["text"],
        "**Inferring User's Intention**"
    );
    assert_eq!(contents[2]["parts"][1]["functionCall"]["id"], "call_54421");
}

#[test]
fn test_thinking_with_text_and_parallel_tools() {
    let store = ThinkingStore::new();

    // Test Case 6: 有思考、有正文、有多工具并行出来
    let tool_ids = vec!["call_batch_1".to_string(), "call_batch_2".to_string()];
    let tool_names = vec!["read_file".to_string(), "grep_search".to_string()];
    // 构造符合 Google 原生特征的假签名（Base64 解码后首字节 = protobuf tag 0x12）
    let real_sig = {
        use base64::Engine;
        let mut raw = vec![0x12u8];
        raw.extend_from_slice(&[b'A'; 60]);
        base64::engine::general_purpose::STANDARD.encode(raw)
    };
    let fp = fingerprint("I will read both files in parallel", &tool_ids, &tool_names);
    store.record(
        "t:s2",
        ThinkingRecord {
            fingerprint: fp,
            thought: "Parallel execution planned".to_string(),
            signature: Some(real_sig.clone()),
            tool_ids: tool_ids.clone(),
            tool_names: tool_names.clone(),
            visible: "I will read both files in parallel".to_string(),
        },
    );

    let mut contents = vec![json!({
        "role": "model",
        "parts": [
            { "text": "I will read both files in parallel" },
            { "functionCall": { "name": "read_file", "id": "call_batch_1", "args": {} } },
            { "functionCall": { "name": "grep_search", "id": "call_batch_2", "args": {} } }
        ]
    })];

    let restored = store.restore_gemini_contents("t:s2", &mut contents);
    assert_eq!(restored, 1);

    let parts = contents[0]["parts"].as_array().unwrap();
    // Index 0: thought block —— 保持纯净文本，铁律 I4：思考块绝不携带签名
    assert_eq!(parts[0]["thought"], true);
    assert_eq!(parts[0]["text"], "Parallel execution planned");
    assert!(parts[0].get("thoughtSignature").is_none());

    // Index 1: 可见正文 = 该轮「第一个非思考 part」= 签名锚点
    assert_eq!(parts[1]["text"], "I will read both files in parallel");
    assert_eq!(
        parts[1]["thoughtSignature"], real_sig,
        "Real signature must be restored onto the anchor (first non-thought part)"
    );

    // Index 2 / 3: 工具调用不再盖章 —— 签名字段必须「缺席」，而不是空串
    assert_eq!(parts[2]["functionCall"]["id"], "call_batch_1");
    assert!(parts[2].get("thoughtSignature").is_none());
    assert_eq!(parts[3]["functionCall"]["id"], "call_batch_2");
    assert!(parts[3].get("thoughtSignature").is_none());
}
