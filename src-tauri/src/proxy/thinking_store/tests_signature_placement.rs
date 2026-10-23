//! Official message alignment regression test: the signature anchor points under the five part arrangements must be consistent with the real machine.
//!
//! Invariants summarized based on 3 official Antigravity messages and 23 real signatures
//! (See `.workbuddy/outputs/correct-assembly-spec.md`):
//!   1. Signatures only appear in the model round, with a maximum of 1 in each round;
//!   2. Anchor point = the first `thought != true` part of this round;
//!   3. `thought: true` / `functionResponse` / non-anchor - the field must be "absent";
//!   4. Missing signatures are tolerated by upstream and sentinels are never created.
use super::*;
use serde_json::json;

/// Construct a fake signature that conforms to Google's native characteristics (first byte after Base64 decoding = protobuf tag 0x12)
fn gemini_sig(seed: u8) -> String {
    use base64::Engine;
    let mut raw = vec![0x12u8, seed];
    raw.extend_from_slice(&[b'A'; 60]);
    base64::engine::general_purpose::STANDARD.encode(raw)
}

#[test]
fn is_thought_part_only_accepts_explicit_flag() {
    // Signed text without the thought flag - must not be considered a thought block (the core error of the old heuristic)
    assert!(!is_thought_part(
        &json!({ "text": "answer", "thoughtSignature": "Eabc" })
    ));
    // It does not count with functionCall
    assert!(!is_thought_part(
        &json!({ "functionCall": { "name": "x" }, "thoughtSignature": "Eabc" })
    ));
    // Only explicit thought: true counts
    assert!(is_thought_part(
        &json!({ "text": "reasoning", "thought": true })
    ));
    // Explicit false does not count
    assert!(!is_thought_part(&json!({ "text": "x", "thought": false })));
}

#[test]
fn arrangement_a_thought_plus_text_keeps_signature_on_text() {
    // Official f81eae5c contents[1] arrangement
    let sig = gemini_sig(1);
    let mut parts = vec![
        json!({ "text": "reasoning", "thought": true }),
        json!({ "text": "visible answer", "thoughtSignature": sig }),
    ];
    let placed = place_turn_signature(&mut parts, None);

    assert_eq!(placed.as_deref(), Some(sig.as_str()));
    assert_eq!(parts.len(), 2, "Never rearrange or add or delete parts");
    assert!(
        parts[0].get("thoughtSignature").is_none(),
        "Thought blocks are never signed"
    );
    assert_eq!(parts[1]["thoughtSignature"], sig, "anchor = parts[1]");
}

#[test]
fn arrangement_b_parallel_calls_signs_only_the_first() {
    // Official f81eae5c contents[3]: 3 concurrent functionCalls, signature only in parts[0]
    let sig = gemini_sig(2);
    let mut parts = vec![
        json!({ "functionCall": { "id": "a", "name": "run_command" }, "thoughtSignature": sig }),
        json!({ "functionCall": { "id": "b", "name": "view_file" } }),
        json!({ "functionCall": { "id": "c", "name": "view_file" } }),
    ];
    place_turn_signature(&mut parts, None);

    assert_eq!(parts.len(), 3, "Never insert fake thought blocks");
    assert_eq!(parts[0]["thoughtSignature"], sig);
    assert!(
        parts[1].get("thoughtSignature").is_none(),
        "Field must be absent and not empty string"
    );
    assert!(parts[2].get("thoughtSignature").is_none());
}

#[test]
fn arrangement_c_function_responses_never_signed() {
    let sig = gemini_sig(3);
    let mut parts = vec![
        json!({ "functionResponse": { "id": "a", "name": "x" } }),
        json!({ "functionResponse": { "id": "b", "name": "y" } }),
    ];
    // functionResponse is never written even if a backfill source is provided
    let placed = place_turn_signature(&mut parts, Some(sig.as_str()));

    assert!(placed.is_none());
    assert!(parts[0].get("thoughtSignature").is_none());
    assert!(parts[1].get("thoughtSignature").is_none());
}

#[test]
fn arrangement_d_pure_text_keeps_its_own_signature() {
    let sig = gemini_sig(4);
    let mut parts = vec![json!({ "text": "hello", "thoughtSignature": sig })];
    let placed = place_turn_signature(&mut parts, None);

    assert_eq!(placed.as_deref(), Some(sig.as_str()));
    assert_eq!(parts[0]["thoughtSignature"], sig);
}

#[test]
fn unsigned_anchor_stays_absent_when_no_source_available() {
    // Official baogao.txt contents[17]: In flying functionCall, it is unsigned
    let mut parts = vec![json!({ "functionCall": { "id": "call_x", "name": "bash" } })];
    let placed = place_turn_signature(&mut parts, None);

    assert!(placed.is_none());
    assert!(
        parts[0].get("thoughtSignature").is_none(),
        "Sentinels will never be invented. Sentinels appear 0 times in official reports."
    );
}

#[test]
fn fallback_signature_is_placed_on_anchor() {
    // Cross-protocol path: The anchor has no signature, but the library has a real signature
    let sig = gemini_sig(5);
    let mut parts = vec![json!({ "functionCall": { "id": "call_x", "name": "bash" } })];
    let placed = place_turn_signature(&mut parts, Some(sig.as_str()));

    assert_eq!(placed.as_deref(), Some(sig.as_str()));
    assert_eq!(parts[0]["thoughtSignature"], sig);
}

#[test]
fn sentinel_is_never_accepted_as_a_source() {
    let mut parts = vec![json!({ "functionCall": { "id": "call_x", "name": "bash" } })];
    let placed = place_turn_signature(&mut parts, Some(SENTINEL_SIGNATURE));

    assert!(
        placed.is_none(),
        "Sentinel is not a legitimate backfill source"
    );
    assert!(parts[0].get("thoughtSignature").is_none());
}

#[test]
fn non_anchor_signatures_are_purged_to_absence() {
    let sig_a = gemini_sig(6);
    let sig_b = gemini_sig(7);
    let mut parts = vec![
        json!({ "functionCall": { "id": "a", "name": "x" }, "thoughtSignature": sig_a }),
        json!({ "functionCall": { "id": "b", "name": "y" }, "thoughtSignature": sig_b }),
        json!({ "text": "trailing", "thought_signature": sig_b }),
    ];
    place_turn_signature(&mut parts, None);

    assert_eq!(
        parts[0]["thoughtSignature"], sig_a,
        "Anchors retain their own true signature"
    );
    assert!(parts[1].get("thoughtSignature").is_none());
    assert!(parts[2].get("thoughtSignature").is_none());
    assert!(
        parts[2].get("thought_signature").is_none(),
        "Snake fields must be cleared"
    );
}

#[test]
fn all_thought_turn_has_no_anchor() {
    let sig = gemini_sig(8);
    let mut parts = vec![
        json!({ "text": "r1", "thought": true }),
        json!({ "text": "r2", "thought": true }),
    ];
    let placed = place_turn_signature(&mut parts, Some(sig.as_str()));

    assert!(
        placed.is_none(),
        "If you think the whole round, there will be no anchor point in this round."
    );
    assert!(parts[0].get("thoughtSignature").is_none());
    assert!(parts[1].get("thoughtSignature").is_none());
}

#[test]
fn test_tool_id_normalization_in_restore() {
    let store = ThinkingStore::new();
    let session_key = "test_norm_tool_id_session";
    let sig = gemini_sig(9);

    // Record saved with canonical call_573077
    // Record saved with canonical call_573077
    store.record(
        session_key,
        ThinkingRecord {
            fingerprint: "fp1".to_string(),
            thought: "I need to read this file".to_string(),
            signature: Some(sig.clone()),
            tool_ids: vec!["call_573077".to_string()],
            tool_names: vec!["default_api:read".to_string()],
            visible: String::new(),
        },
    );

    // Incoming client contents stripped underscore: call573077
    let mut contents = vec![
        json!({
            "role": "user",
            "parts": [{ "text": "read the file" }]
        }),
        json!({
            "role": "model",
            "parts": [{
                "functionCall": {
                    "name": "default_api:read",
                    "id": "call573077",
                    "args": { "path": "test.txt" }
                }
            }]
        }),
    ];

    let count = store.restore_gemini_contents_with_model(
        session_key,
        &mut contents,
        Some("gemini-2.5-flash"),
    );
    assert_eq!(count, 1);
    let model_parts = contents[1]["parts"].as_array().unwrap();
    // Signature should be attached to the anchor
    assert_eq!(model_parts[0]["thoughtSignature"], sig);
}
