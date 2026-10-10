//! 官方报文对齐回归测试：五种 part 排列下签名锚点必须与真机一致。
//!
//! 依据 3 份官方 Antigravity 报文、23 处真实签名归纳出的不变量
//! （见 `.workbuddy/outputs/correct-assembly-spec.md`）：
//!   1. 签名只出现在 model 轮，每轮至多 1 个；
//!   2. 锚点 = 该轮第一个 `thought != true` 的 part；
//!   3. `thought: true` / `functionResponse` / 非锚点 —— 字段必须「缺席」；
//!   4. 缺失签名被上游容忍，绝不发明哨兵。
use super::*;
use serde_json::json;

/// 构造符合 Google 原生特征的假签名（Base64 解码后首字节 = protobuf tag 0x12）
fn gemini_sig(seed: u8) -> String {
    use base64::Engine;
    let mut raw = vec![0x12u8, seed];
    raw.extend_from_slice(&[b'A'; 60]);
    base64::engine::general_purpose::STANDARD.encode(raw)
}

#[test]
fn is_thought_part_only_accepts_explicit_flag() {
    // 带签名但没有 thought 标志的正文 —— 绝不能判为思考块（旧启发式的核心错误）
    assert!(!is_thought_part(
        &json!({ "text": "answer", "thoughtSignature": "Eabc" })
    ));
    // 带 functionCall 也不算
    assert!(!is_thought_part(
        &json!({ "functionCall": { "name": "x" }, "thoughtSignature": "Eabc" })
    ));
    // 只有显式 thought: true 才算
    assert!(is_thought_part(
        &json!({ "text": "reasoning", "thought": true })
    ));
    // 显式 false 不算
    assert!(!is_thought_part(&json!({ "text": "x", "thought": false })));
}

#[test]
fn arrangement_a_thought_plus_text_keeps_signature_on_text() {
    // 官方 f81eae5c contents[1] 的排列
    let sig = gemini_sig(1);
    let mut parts = vec![
        json!({ "text": "reasoning", "thought": true }),
        json!({ "text": "visible answer", "thoughtSignature": sig }),
    ];
    let placed = place_turn_signature(&mut parts, None);

    assert_eq!(placed.as_deref(), Some(sig.as_str()));
    assert_eq!(parts.len(), 2, "绝不能重排或增删 part");
    assert!(
        parts[0].get("thoughtSignature").is_none(),
        "思考块绝不带签名"
    );
    assert_eq!(parts[1]["thoughtSignature"], sig, "锚点 = parts[1]");
}

#[test]
fn arrangement_b_parallel_calls_signs_only_the_first() {
    // 官方 f81eae5c contents[3]：3 个并发 functionCall，签名只在 parts[0]
    let sig = gemini_sig(2);
    let mut parts = vec![
        json!({ "functionCall": { "id": "a", "name": "run_command" }, "thoughtSignature": sig }),
        json!({ "functionCall": { "id": "b", "name": "view_file" } }),
        json!({ "functionCall": { "id": "c", "name": "view_file" } }),
    ];
    place_turn_signature(&mut parts, None);

    assert_eq!(parts.len(), 3, "绝不插入假思考块");
    assert_eq!(parts[0]["thoughtSignature"], sig);
    assert!(
        parts[1].get("thoughtSignature").is_none(),
        "字段必须缺席而非空串"
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
    // 即便提供回填来源，也绝不写入 functionResponse
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
    // 官方 baogao.txt contents[17]：在飞 functionCall 就是无签名的
    let mut parts = vec![json!({ "functionCall": { "id": "call_x", "name": "bash" } })];
    let placed = place_turn_signature(&mut parts, None);

    assert!(placed.is_none());
    assert!(
        parts[0].get("thoughtSignature").is_none(),
        "绝不发明哨兵，官方报文里哨兵出现 0 次"
    );
}

#[test]
fn fallback_signature_is_placed_on_anchor() {
    // 跨协议路径：锚点无签名，但库里有真实签名
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

    assert!(placed.is_none(), "哨兵不是合法回填来源");
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

    assert_eq!(parts[0]["thoughtSignature"], sig_a, "锚点保留自己的真签名");
    assert!(parts[1].get("thoughtSignature").is_none());
    assert!(parts[2].get("thoughtSignature").is_none());
    assert!(
        parts[2].get("thought_signature").is_none(),
        "蛇形字段必须被清除"
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

    assert!(placed.is_none(), "整轮皆思考则本轮无锚点");
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
