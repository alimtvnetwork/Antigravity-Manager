use super::tests_common::rec;
use super::*;
use serde_json::json;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::time::Instant;

#[test]
fn fingerprint_matches_legacy_whitespace_join() {
    let visible = "  hello\n\tworld  foo";
    let ids: Vec<String> = vec!["call_1".to_string()];
    let names: Vec<String> = vec!["shell".to_string()];
    let mut hasher = Sha256::new();
    let norm: String = visible.split_whitespace().collect::<Vec<_>>().join(" ");
    hasher.update(norm.as_bytes());
    hasher.update([0xff]);
    hasher.update(ids[0].as_bytes());
    hasher.update([0xfe]);
    hasher.update([0xfd]);
    hasher.update(names[0].as_bytes());
    hasher.update([0xfc]);
    let hex = format!("{:x}", hasher.finalize());
    assert_eq!(fingerprint(visible, &ids, &names), hex[..16].to_string());
}

#[test]
fn placeholder_thoughts_are_not_recorded() {
    let store = ThinkingStore::new();
    let key = "t:placeholder-skip";
    store.record(
        key,
        ThinkingRecord {
            fingerprint: fingerprint("visible answer", &[], &[]),
            thought: "...".to_string(),
            signature: None,
            tool_ids: vec![],
            tool_names: vec![],
            visible: "visible answer".to_string(),
        },
    );
    store.record(
        key,
        ThinkingRecord {
            fingerprint: fingerprint("visible answer", &[], &[]),
            thought: "...".to_string(),
            signature: Some(SENTINEL_SIGNATURE.to_string()),
            tool_ids: vec![],
            tool_names: vec![],
            visible: "visible answer".to_string(),
        },
    );
    assert!(
        store.session_stats(key).is_none(),
        "placeholder / sentinel-only thoughts must not create store turns"
    );
}

#[test]
fn ingest_placeholders_does_not_duplicate_history() {
    let store = ThinkingStore::new();
    let key = "t:ingest-no-dup";
    for i in 0..20 {
        store.record(
            key,
            rec(
                &format!("thought-{i}"),
                &format!("answer {i}"),
                Some(&format!("call_{i}")),
            ),
        );
    }
    let (before, _) = store.session_stats(key).unwrap();
    assert_eq!(before, 20);

    let contents: Vec<Value> = (0..20)
        .map(|i| {
            json!({
                "role": "model",
                "parts": [
                    { "text": "...", "thought": true, "thoughtSignature": SENTINEL_SIGNATURE },
                    { "text": format!("answer {i}") },
                    { "functionCall": { "name": "shell", "id": format!("call_{i}"), "args": {} } }
                ]
            })
        })
        .collect();

    store.ingest_from_contents(key, &contents);
    let (after, _) = store.session_stats(key).unwrap();
    assert_eq!(
        after, 20,
        "placeholder history must not be appended as new turns"
    );
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        crate::modules::proxy_db::delete_thinking_records_for_session(key),
        "delete_thinking_records_for_session",
    );
}

#[test]
fn ingest_real_client_thinking_is_idempotent() {
    let store = ThinkingStore::new();
    let key = "t:ingest-real";
    let sig = "s".repeat(60);
    let contents = vec![json!({
        "role": "model",
        "parts": [
            { "text": "full chain of thought here", "thought": true, "thoughtSignature": sig },
            { "text": "hello world" }
        ]
    })];
    store.ingest_from_contents(key, &contents);
    store.ingest_from_contents(key, &contents);
    let (turns, _) = store.session_stats(key).unwrap();
    assert_eq!(turns, 1);
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        crate::modules::proxy_db::delete_thinking_records_for_session(key),
        "delete_thinking_records_for_session",
    );
}

#[test]
fn restore_large_visible_text_is_linear() {
    let store = ThinkingStore::new();
    let key = "t:big-visible";
    let blob = "word ".repeat(8_000); // ~40KB per turn
    for i in 0..30 {
        let visible = format!("head-{i} {blob}");
        store.record(key, rec(&format!("thought-{i}"), &visible, None));
    }
    // Truncated visibles force Phase 3 prefix matching (the old quadratic path).
    let mut contents: Vec<Value> = (0..30)
        .map(|i| {
            json!({
                "role": "model",
                "parts": [{ "text": format!("head-{i}") }]
            })
        })
        .collect();
    let start = Instant::now();
    let n = store.restore_gemini_contents(key, &mut contents);
    let elapsed = start.elapsed();
    assert_eq!(n, 30);
    assert!(
        elapsed.as_millis() < 800,
        "restore of ~1.2MB visible text took {elapsed:?}; matching must not be quadratic"
    );
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        crate::modules::proxy_db::delete_thinking_records_for_session(key),
        "delete_thinking_records_for_session",
    );
}

#[test]
fn placeholder_history_fills_in_order_without_scramble() {
    let store = ThinkingStore::new();
    let key = "t:fill-order";
    for i in 0..12 {
        store.record(
            key,
            rec(
                &format!("THOUGHT-BLOCK-{i}"),
                &format!("answer {i}"),
                Some(&format!("call_{i}")),
            ),
        );
    }
    let mut contents: Vec<Value> = (0..12)
        .map(|i| {
            json!({
                "role": "model",
                "parts": [
                    { "text": "...", "thought": true, "thoughtSignature": SENTINEL_SIGNATURE },
                    { "text": format!("answer {i}") },
                    { "functionCall": { "name": "shell", "id": format!("call_{i}"), "args": { "n": i } } }
                ]
            })
        })
        .collect();

    assert!(!contents_have_capturable_thought(&contents));
    let n = store.restore_gemini_contents(key, &mut contents);
    assert_eq!(n, 12);
    for i in 0..12 {
        let parts = contents[i]["parts"].as_array().unwrap();
        assert_eq!(
            parts[0]["thought"], true,
            "thought must stay at parts[0] for turn {i}"
        );
        assert_eq!(parts[0]["text"], format!("THOUGHT-BLOCK-{i}"));
        assert_eq!(parts[0]["thoughtSignature"].as_str().unwrap().len(), 60);
        assert_eq!(parts[1]["text"], format!("answer {i}"));
        assert_eq!(parts[2]["functionCall"]["id"], format!("call_{i}"));
        assert_eq!(parts[2]["functionCall"]["args"]["n"], i);
        assert_eq!(parts[2]["thoughtSignature"].as_str().unwrap().len(), 60);
    }
    store.prune_orphaned_records(key, &contents);
    let (turns, _) = store.session_stats(key).unwrap();
    assert_eq!(turns, 12, "placeholder fill must not prune live tool turns");
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        crate::modules::proxy_db::delete_thinking_records_for_session(key),
        "delete_thinking_records_for_session",
    );
}

#[test]
fn signed_function_call_is_not_mistaken_for_thinking_block() {
    let real_sig = "Ep4MCpsMARFNMg9NDlK9RXXz5Mzq9mniX9KSQBBzbUx3k85w/qDgtcE+28NH+1EvPeULAprqUquvYXGMzUXGy1xJoMnqdkC4vqebuhyd2Xhs0oz+OhqcOTwLhGYOG0KBKQ87Hfw4q/sMCSgf2gz4vFMa6V6kKMJepYlPXKFJJF4ok+W6lUt3PfYln8K9Dh7wB/40iHiZ2BnJd++6hfUwu9Bz1n795S50l0yCj84EaSCDDF334Erxq7Fo";

    // Case 1: thinking enabled, only signed functionCall → must prepend real thought block
    let mut contents = vec![json!({
        "role": "model",
        "parts": [
            {
                "functionCall": {
                    "name": "read_file",
                    "id": "call_1",
                    "args": { "path": "a.rs" }
                },
                "thoughtSignature": real_sig
            }
        ]
    })];
    finalize_gemini_contents_thinking(&mut contents, true);
    let parts = contents[0]["parts"].as_array().unwrap();
    // 【2026-09-27】无思考文本时不再注入 "..." 占位思考块（官方标准形态：
    // 「无思考块 + 锚点带签名」，9/24 轮）。签名落锚点 fc。
    assert_eq!(
        parts.len(),
        1,
        "no placeholder thinking block should be injected"
    );
    assert!(parts[0].get("functionCall").is_some());
    assert_eq!(parts[0]["thoughtSignature"], real_sig);

    // Case 2: thinking disabled → signed functionCall must survive and preserve thoughtSignature for Gemini AST validator
    let mut contents_off = vec![json!({
        "role": "model",
        "parts": [
            {
                "functionCall": {
                    "name": "bash",
                    "id": "call_9",
                    "args": {}
                },
                "thoughtSignature": real_sig
            }
        ]
    })];
    finalize_gemini_contents_thinking(&mut contents_off, false);
    let parts_off = contents_off[0]["parts"].as_array().unwrap();
    assert_eq!(
        parts_off.len(),
        1,
        "functionCall must not be dropped when thinking is off"
    );
    assert!(parts_off[0].get("functionCall").is_some());
    assert_eq!(
        parts_off[0]["thoughtSignature"], real_sig,
        "Gemini native model requires functionCall to retain its signature even when thinking is off"
    );
}
