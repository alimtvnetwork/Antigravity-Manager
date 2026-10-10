use super::*;
use serde_json::json;

#[test]
fn test_prune_orphaned_records_zero_phase_shift() {
    let key = "t:test_phase_shift_prune";
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        crate::modules::proxy_db::delete_thinking_records_for_session(key),
        "delete_thinking_records_for_session",
    );
    let store = ThinkingStore::new();

    for i in 0..6 {
        let user_turn = json!({
            "role": "user",
            "parts": [{ "text": format!("user msg {i}") }]
        });
        let anchor = compute_causal_anchor(Some(&user_turn));
        let mut acc = TurnAccumulator::with_anchor(&anchor);
        acc.ingest_part(&json!({
            "text": format!("Thought turn {i}"),
            "thought": true,
            "thoughtSignature": format!("sig_{:0>60}", i)
        }));
        acc.ingest_part(&json!({
            "functionCall": { "name": "bash", "args": { "step": i } }
        }));
        store.record(key, acc.into_record());
    }

    // Client compresses away turns 0 and 1; only turns 2..5 remain
    let mut contents = Vec::new();
    for i in 2..6 {
        contents.push(json!({
            "role": "user",
            "parts": [{ "text": format!("user msg {i}") }]
        }));
        contents.push(json!({
            "role": "model",
            "parts": [
                { "text": "...", "thought": true, "thoughtSignature": SENTINEL_SIGNATURE },
                { "functionCall": { "name": "bash", "args": { "step": i } } }
            ]
        }));
    }

    let restored = store.restore_gemini_contents(key, &mut contents);
    assert_eq!(restored, 4, "Must restore 4 compressed turns");

    // Verify ZERO PHASE SHIFT: turn 2 must have "Thought turn 2", NOT "Thought turn 4"
    for (idx, step) in (2..6).enumerate() {
        let content_idx = idx * 2 + 1;
        let parts = contents[content_idx]["parts"].as_array().unwrap();
        assert_eq!(
            parts[0]["text"],
            format!("Thought turn {step}"),
            "Turn {step} must strictly match its own thought without phase shift"
        );
        assert_eq!(
            parts[0]["thoughtSignature"],
            format!("sig_{:0>60}", step),
            "Turn {step} must strictly match its own thoughtSignature"
        );
    }

    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        crate::modules::proxy_db::delete_thinking_records_for_session(key),
        "delete_thinking_records_for_session",
    );
}

#[test]
fn test_phase0_signature_direct_matching_recovers_truncated_text() {
    let key = "t:test_phase0_sig";
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        crate::modules::proxy_db::delete_thinking_records_for_session(key),
        "delete_thinking_records_for_session",
    );
    let store = ThinkingStore::new();
    let real_sig = "s".repeat(60);

    let mut acc = TurnAccumulator::with_anchor("anchor_1");
    acc.ingest_part(&json!({
        "text": "Detailed multi-step chain of thought",
        "thought": true,
        "thoughtSignature": real_sig
    }));
    acc.ingest_part(&json!({
        "functionCall": { "name": "read_file", "args": { "path": "main.rs" } }
    }));
    store.record(key, acc.into_record());

    let mut contents = vec![
        json!({ "role": "user", "parts": [{ "text": "start" }] }),
        json!({
            "role": "model",
            "parts": [
                { "text": "...", "thought": true, "thoughtSignature": real_sig },
                { "functionCall": { "name": "read_file", "args": { "path": "main.rs" } } }
            ]
        }),
    ];

    let restored = store.restore_gemini_contents(key, &mut contents);
    assert_eq!(restored, 1);
    assert_eq!(
        contents[1]["parts"][0]["text"], "Detailed multi-step chain of thought",
        "Phase 0 must recover truncated thought using real signature"
    );

    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        crate::modules::proxy_db::delete_thinking_records_for_session(key),
        "delete_thinking_records_for_session",
    );
}

#[test]
fn test_sqlite_l2_penetration_by_signature_and_tool() {
    let key = "t:test_sqlite_l2_sig_rescue";
    let real_sig = format!("sig_l2_{}", "x".repeat(53));
    let tool_id = "call_bash_anchor123_hash456_0";

    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        crate::modules::proxy_db::save_thinking_record(
            key,
            "fp_l2_sig_test",
            "Rescued thought from SQLite via signature",
            Some(&real_sig),
            &[tool_id.to_string()],
            &["bash".to_string()],
            "",
        ),
        "save_thinking_record",
    );

    let store = ThinkingStore::new();
    // Clear memory to force L2 penetration
    store.clear();

    let mut contents = vec![
        json!({ "role": "user", "parts": [{ "text": "hello" }] }),
        json!({
            "role": "model",
            "parts": [
                { "text": "...", "thought": true, "thoughtSignature": real_sig },
                { "functionCall": { "name": "bash", "id": tool_id, "args": {} } }
            ]
        }),
    ];

    let restored = store.restore_gemini_contents(key, &mut contents);
    assert_eq!(
        restored, 1,
        "Must penetrate to SQLite and restore via signature"
    );
    assert_eq!(
        contents[1]["parts"][0]["text"],
        "Rescued thought from SQLite via signature"
    );

    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        crate::modules::proxy_db::delete_thinking_records_for_session(key),
        "delete_thinking_records_for_session",
    );
}

#[test]
fn test_is_likely_gemini_signature_validation() {
    // 1. 官方哨兵
    assert!(is_likely_gemini_signature(SENTINEL_SIGNATURE));

    // 2. 真实 Gemini 原生签名 (首字节 0x12, 以 'E' 开头)
    let gemini_sig = "Ep4KCpsKAWkUfRMa5ZYMDdlPjxrQTLzVZ6MZeopI88888888888888888888888888888888";
    assert!(is_likely_gemini_signature(gemini_sig));

    // 3. Google Vertex AI 双层 Base64 包装 (以 'R' 开头, 解开是 'E...' 且首字节 0x12)
    use base64::Engine;
    let vertex_wrapped = base64::engine::general_purpose::STANDARD.encode(gemini_sig.as_bytes());
    assert!(vertex_wrapped.starts_with('R'));
    assert!(is_likely_gemini_signature(&vertex_wrapped));

    // 4. 异构外部 Claude 签名 (以 '3', 'l', 'A', 'R' 开头, 解码后非 0x12)
    let claude_sig_1 = "3mgp11XmVXq9InniGA4VAKd7c97NqFw+dWZt79Uz/w9znho88gSM76jv2bZmir7wI86Ixpha7eWdGuznAot4PNbe3+V9bgMTIEyUarn4MLAiiFVb830ZlM+H5ukQwXdD2Zv8nUSmmZTYinpLPGha8TORZAfpU1FJEvwyECel5+W7kc9kpTWrd8DqRNBTOz5EDtvoatiZgKv5SqInhGXK74SJ+PRIC6fNXvYG082HR6TsVxvVYaerz8A40rloIVTxRNK43h3Ecs1boxY4PZqBT8Yhl2qn/iZ+4Xt7FNkI0DAuS9iK0HYKMC4yw0OqKx/LeU+WFZlyc6hGm1BkzLY6yG97MH7kmJ0OPlBWgWFaTeL/uXuGJX6QkKObXN+phoq+kkF2vdFt/mdJMbdgfmSCVQ9037hGBhOHm0zN50KLkp1SxuAY1oWc+lDcI4ufWoyn";
    let claude_sig_2 = "ls29VsBy+VBvzrVBmB2gNmOCoaeJkn19qz8jP8jExGpDc0IxRaV1V9/+cQ4O00000000000000000000000000000000";
    let claude_sig_3 = "A1nvLg9Twun3bBCb1BKLmSNA6MRxaLE2GdEocv6bwuhNKfUmBB2YMvvmaVyO00000000000000000000000000000000";
    assert!(!is_likely_gemini_signature(claude_sig_1));
    assert!(!is_likely_gemini_signature(claude_sig_2));
    assert!(!is_likely_gemini_signature(claude_sig_3));

    // 5. 过短签名
    assert!(!is_likely_gemini_signature("short_sig"));
}

#[test]
fn test_is_claude_signature_validation() {
    use base64::Engine;
    // 1. 构建合法的原始 Claude 签名（单层 Base64 解码后包含 b"claude"）
    let inner_claude_payload =
        b"\x12\xb2\x02\n\x92\x01\x08\x12\x10\x02\x18\x02*@claude-opus-4-6-signature-data";
    let raw_claude_sig = base64::engine::general_purpose::STANDARD.encode(inner_claude_payload);
    assert!(is_claude_signature(&raw_claude_sig));

    // 2. 构建 Google Vertex AI 双层包装后的 Claude 签名
    let wrapped_claude_sig =
        base64::engine::general_purpose::STANDARD.encode(raw_claude_sig.as_bytes());
    assert!(is_claude_signature(&wrapped_claude_sig));

    // 3. Gemini 签名绝不是 Claude 签名
    let gemini_sig = "Ep4KCpsKAWkUfRMa5ZYMDdlPjxrQTLzVZ6MZeopI88888888888888888888888888888888";
    assert!(!is_claude_signature(gemini_sig));

    // 4. 空与哨兵
    assert!(!is_claude_signature(""));
    assert!(!is_claude_signature(SENTINEL_SIGNATURE));
}

#[test]
fn test_ensure_google_claude_thought_signature_does_not_inflate() {
    use base64::Engine;
    // 原始 Claude 签名：包装一次
    let inner_claude_payload =
        b"\x12\xb2\x02\n\x92\x01\x08\x12\x10\x02\x18\x02*@claude-opus-4-6-signature-data";
    let raw_claude_sig = base64::engine::general_purpose::STANDARD.encode(inner_claude_payload);
    let wrapped_once = ensure_google_claude_thought_signature(&raw_claude_sig);
    assert_ne!(wrapped_once, raw_claude_sig);

    // 已包装的 Claude 签名：幂等，绝对不再包装！
    let wrapped_twice = ensure_google_claude_thought_signature(&wrapped_once);
    assert_eq!(
        wrapped_once, wrapped_twice,
        "Claude signature must be idempotent, no double wrapping"
    );

    // 非 Claude 签名 (Gemini 签名)：绝不能包装！彻底杜绝几何级膨胀
    let gemini_sig = "Ep4KCpsKAWkUfRMa5ZYMDdlPjxrQTLzVZ6MZeopI88888888888888888888888888888888";
    let gemini_out = ensure_google_claude_thought_signature(gemini_sig);
    assert_eq!(
        gemini_out, gemini_sig,
        "Non-Claude signature must NOT be base64 wrapped"
    );
}
