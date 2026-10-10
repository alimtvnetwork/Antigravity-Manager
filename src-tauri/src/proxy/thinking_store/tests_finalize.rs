use super::*;
use serde_json::json;

#[test]
fn finalize_upgrades_sentinel_thought_from_tool_real_signature() {
    let real_sig = "Ep4MCpsMARFNMg9NDlK9RXXz5Mzq9mniX9KSQBBzbUx3k85w/qDgtcE+28NH+1EvPeULAprqUquvYXGMzUXGy1xJoMnqdkC4vqebuhyd2Xhs0oz+OhqcOTwLhGYOG0KBKQ87Hfw4q/sMCSgf2gz4vFMa6V6kKMJepYlPXKFJJF4ok+W6lUt3PfYln8K9Dh7wB/40iHiZ2BnJd++6hfUwu9Bz1n795S50l0yCj84EaSCDDF334Erxq7Fo";
    let mut contents = vec![json!({
        "role": "model",
        "parts": [
            {
                "text": "...",
                "thought": true,
                "thoughtSignature": SENTINEL_SIGNATURE
            },
            {
                "functionCall": {
                    "name": "read_file",
                    "id": "call_upgrade",
                    "args": {}
                },
                "thoughtSignature": real_sig
            }
        ]
    })];
    finalize_gemini_contents_thinking(&mut contents, true);
    let parts = contents[0]["parts"].as_array().unwrap();
    assert_eq!(parts[0]["thought"], true);
    assert!(parts[0].get("thoughtSignature").is_none());
    assert_eq!(parts[1]["thoughtSignature"], real_sig);
}

#[test]
fn turn_needs_restore_when_thought_signature_is_sentinel() {
    let parts = vec![
        json!({
            "text": "some real looking text that is not a placeholder",
            "thought": true,
            "thoughtSignature": SENTINEL_SIGNATURE
        }),
        json!({
            "functionCall": { "name": "shell", "id": "call_x", "args": {} },
            "thoughtSignature": SENTINEL_SIGNATURE
        }),
    ];
    assert!(
        turn_needs_restore(&parts, "some real looking text that is not a placeholder"),
        "sentinel thought signature must still request restore"
    );
}

#[test]
fn test_is_meaningful_thought_sanitizer() {
    // Placeholders & empty must fail
    assert!(!is_meaningful_thought(""));
    assert!(!is_meaningful_thought("   "));
    assert!(!is_meaningful_thought("..."));
    assert!(!is_meaningful_thought("···"));
    assert!(!is_meaningful_thought("."));

    // Pseudo-thinking tags & placeholders must fail
    assert!(!is_meaningful_thought("<think></think>"));
    assert!(!is_meaningful_thought("<think>\n\n</think>"));
    assert!(!is_meaningful_thought("Thinking Process:\n"));
    assert!(!is_meaningful_thought("[Thinking]"));
    assert!(!is_meaningful_thought("None"));
    assert!(!is_meaningful_thought("none"));
    assert!(!is_meaningful_thought("null"));
    assert!(!is_meaningful_thought("undefined"));
    assert!(!is_meaningful_thought("[Thinking]\n..."));

    // Real thoughts must pass
    assert!(is_meaningful_thought(
        "Let's analyze the problem step by step."
    ));
    assert!(is_meaningful_thought(
        "<think>First compute the square root of 16, which is 4.</think>"
    ));
    assert!(is_meaningful_thought(
        "Thinking Process:\n1. Check file existence\n2. Open file"
    ));
}

#[test]
fn test_finalize_thinking_disabled_downgrades_meaningful_thought_and_strips_placeholders() {
    let mut contents = vec![
        json!({
            "role": "model",
            "parts": [
                {
                    "text": "...",
                    "thought": true,
                    "thoughtSignature": SENTINEL_SIGNATURE
                },
                {
                    "text": "Hello, how can I help?"
                }
            ]
        }),
        json!({
            "role": "model",
            "parts": [
                {
                    "text": "Real thought: solving user query carefully.",
                    "thought": true,
                    "thoughtSignature": "some_sig"
                },
                {
                    "text": "Here is the answer."
                }
            ]
        }),
    ];

    finalize_gemini_contents_thinking(&mut contents, false);

    // Turn 1: placeholder "..." thought is dropped, only visible text survives
    let parts1 = contents[0]["parts"].as_array().unwrap();
    assert_eq!(parts1.len(), 1);
    assert_eq!(parts1[0]["text"], "Hello, how can I help?");
    assert!(parts1[0].get("thought").is_none());

    // Turn 2: meaningful thought is downgraded to text {"text": "Real thought: ..."}
    let parts2 = contents[1]["parts"].as_array().unwrap();
    assert_eq!(parts2.len(), 2);
    assert_eq!(
        parts2[0]["text"],
        "Real thought: solving user query carefully."
    );
    assert!(parts2[0].get("thought").is_none());
    assert!(parts2[0].get("thoughtSignature").is_none());
    assert_eq!(parts2[1]["text"], "Here is the answer.");
}

#[test]
fn test_finalize_thinking_disabled_cleans_user_function_response_signature() {
    let mut contents = vec![json!({
        "role": "user",
        "parts": [
            {
                "functionResponse": {
                    "name": "calc",
                    "response": { "result": 42 }
                },
                "thoughtSignature": "sig_to_be_cleaned"
            }
        ]
    })];

    finalize_gemini_contents_thinking(&mut contents, false);

    let parts = contents[0]["parts"].as_array().unwrap();
    assert_eq!(parts.len(), 1);
    assert!(parts[0].get("functionResponse").is_some());
    assert!(
        parts[0].get("thoughtSignature").is_none(),
        "thoughtSignature must be removed from functionResponse when thinking is disabled"
    );
}

#[test]
fn test_sqlite_penetration_fallback_when_memory_missing() {
    let store = ThinkingStore::new();
    let key = "t:sqlite-fallback-test";
    let tool_id = "call_fallback_999";
    let real_sig = "s".repeat(60);

    // 1. 模拟旧轮次已持久化入库 SQLite（但在内存缓存中已被淘汰或未命中）
    crate::modules::proxy_db::save_thinking_record(
        key,
        "fp_fallback",
        "This thinking was retrieved directly from SQLite!",
        Some(&real_sig),
        &[tool_id.to_string()],
        &[],
        "some visible",
    )
    .unwrap();

    // 确保内存缓存是完全清空的，逼迫触发 L2 SQLite 穿透点查
    store.clear();

    // 2. 构造客户端回传的历史请求（缺少思考块，仅占位符）
    let mut contents = vec![json!({
        "role": "model",
        "parts": [
            {
                "text": "...",
                "thought": true,
                "thoughtSignature": SENTINEL_SIGNATURE
            },
            {
                "functionCall": {
                    "name": "shell",
                    "id": tool_id,
                    "args": {}
                }
            }
        ]
    })];

    // 3. 执行思考复活：应当穿透到 SQLite 成功捞回！
    let restored = store.restore_gemini_contents(key, &mut contents);
    assert_eq!(restored, 1, "Must penetrate to SQLite and restore 1 turn!");

    let parts = contents[0]["parts"].as_array().unwrap();
    assert_eq!(parts[0]["thought"], true);
    assert_eq!(
        parts[0]["text"],
        "This thinking was retrieved directly from SQLite!"
    );
    assert!(parts[0].get("thoughtSignature").is_none());
    assert_eq!(parts[1]["thoughtSignature"], real_sig);

    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        crate::modules::proxy_db::delete_thinking_records_for_session(key),
        "delete_thinking_records_for_session",
    );
}

#[test]
fn test_canonical_json_hash_key_order_independence() {
    let a = json!({ "b": 2, "a": 1, "c": { "z": 9, "y": 8 } });
    let b = json!({ "a": 1, "c": { "y": 8, "z": 9 }, "b": 2 });
    assert_eq!(
        canonical_json_hash(Some(&a)),
        canonical_json_hash(Some(&b)),
        "Canonical JSON hash must be independent of key order"
    );
}

#[test]
fn test_compute_causal_anchor_differentiates_user_text_vs_tool_response() {
    let turn1 = json!({ "role": "user", "parts": [{ "text": "Run cargo check" }] });
    let turn2 = json!({
        "role": "user",
        "parts": [{ "functionResponse": { "name": "bash", "response": { "exit_code": 1, "output": "error" } } }]
    });
    let anchor1 = compute_causal_anchor(Some(&turn1));
    let anchor2 = compute_causal_anchor(Some(&turn2));
    assert_ne!(
        anchor1, anchor2,
        "Causal anchors for user text vs tool response must be distinct"
    );
}

#[test]
fn test_gemini_native_consecutive_identical_tools_do_not_overwrite() {
    let key = "t:test_identical_tools";
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        crate::modules::proxy_db::delete_thinking_records_for_session(key),
        "delete_thinking_records_for_session",
    );
    let store = ThinkingStore::new();

    // Turn 1: user text -> model tool call
    let mut acc1 = TurnAccumulator::with_anchor("anchor_user_turn_1");
    acc1.ingest_part(&json!({
        "text": "Thought for turn 1",
        "thought": true,
        "thoughtSignature": "s".repeat(60)
    }));
    acc1.ingest_part(&json!({
        "functionCall": { "name": "bash", "args": { "command": "pwd" } }
    }));
    store.record(key, acc1.into_record());

    // Turn 2: tool response -> model tool call (identical tool name and args!)
    let mut acc2 = TurnAccumulator::with_anchor("anchor_fr_turn_2");
    acc2.ingest_part(&json!({
        "text": "Thought for turn 2",
        "thought": true,
        "thoughtSignature": "t".repeat(60)
    }));
    acc2.ingest_part(&json!({
        "functionCall": { "name": "bash", "args": { "command": "pwd" } }
    }));
    store.record(key, acc2.into_record());

    let stats = store.session_stats(key).unwrap();
    assert_eq!(
        stats.0, 2,
        "Must store two distinct turns, not overwrite via collision"
    );

    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        crate::modules::proxy_db::delete_thinking_records_for_session(key),
        "delete_thinking_records_for_session",
    );
}
