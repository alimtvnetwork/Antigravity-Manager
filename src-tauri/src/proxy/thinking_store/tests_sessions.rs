use super::tests_common::rec;
use super::*;
use axum::http::HeaderMap;
use serde_json::json;
use std::sync::Arc;

#[test]
fn test_sqlite_persistence_and_recovery() {
    let store_key = "test_session_sqlite_recovery_unique";
    let fp = fingerprint(
        "Persisted visible text",
        &["call_persisted_999".to_string()],
        &["bash".to_string()],
    );
    // 构造符合 Google 原生特征的假签名（Base64 解码后首字节 = protobuf tag 0x12）
    let real_sig = {
        use base64::Engine;
        let mut raw = vec![0x12u8];
        raw.extend_from_slice(&[b'A'; 60]);
        base64::engine::general_purpose::STANDARD.encode(raw)
    };
    let rec = ThinkingRecord {
        fingerprint: fp,
        thought: "Thought restored from SQLite".to_string(),
        signature: Some(real_sig.clone()),
        tool_ids: vec!["call_persisted_999".to_string()],
        tool_names: vec!["bash".to_string()],
        visible: "Persisted visible text".to_string(),
    };

    let db_res = crate::modules::proxy_db::save_thinking_record(
        store_key,
        &rec.fingerprint,
        &rec.thought,
        rec.signature.as_deref(),
        &rec.tool_ids,
        &rec.tool_names,
        &rec.visible,
    );
    if let Err(e) = db_res {
        eprintln!("Skipping DB test if DB not initialized: {}", e);
        return;
    }

    let store = ThinkingStore::new();

    let mut contents = vec![json!({
        "role": "model",
        "parts": [
            { "text": "Persisted visible text" },
            { "functionCall": { "name": "bash", "id": "call_persisted_999", "args": {} } }
        ]
    })];

    let restored = store.restore_gemini_contents(store_key, &mut contents);
    assert_eq!(restored, 1);

    let parts = contents[0]["parts"].as_array().unwrap();
    assert_eq!(parts[0]["thought"], true);
    assert_eq!(parts[0]["text"], "Thought restored from SQLite");
    // 锚点 = 该轮第一个非思考 part（这里是可见正文）：真实签名归位到它上面，
    // 工具调用不再被盖章。
    assert_eq!(parts[1]["thoughtSignature"], real_sig);
    assert!(parts[2].get("thoughtSignature").is_none());
}

#[test]
fn sqlite_merges_latest_chunk_but_keeps_older_same_fingerprint_turn() {
    let store_key = "test_session_fp_hello_isolation";
    if crate::modules::proxy_db::delete_thinking_records_for_session(store_key).is_err() {
        return;
    }

    let sig = "sig_hello_12345678901234567890123456789012345678901234567890";
    let fp_hello = fingerprint("你好", &[], &[]);
    let fp_other = fingerprint("other", &["call_x".to_string()], &["bash".to_string()]);

    let save = |fp: &str, thought: &str, visible: &str, ids: &[String], names: &[String]| {
        crate::modules::proxy_db::save_thinking_record(
            store_key,
            fp,
            thought,
            Some(sig),
            ids,
            names,
            visible,
        )
    };

    assert!(save(&fp_hello, "thought-1", "你好", &[], &[]).is_ok());
    assert!(save(&fp_hello, "thought-1-longer", "你好", &[], &[]).is_ok());
    assert!(save(
        &fp_other,
        "thought-tool",
        "other",
        &["call_x".to_string()],
        &["bash".to_string()]
    )
    .is_ok());
    assert!(save(&fp_hello, "thought-3", "你好", &[], &[]).is_ok());

    let rows = crate::modules::proxy_db::load_thinking_records(store_key).unwrap_or_default();
    assert_eq!(
        rows.len(),
        3,
        "older 你好 turn must not be overwritten: {rows:?}"
    );
    assert_eq!(rows[0].thought, "thought-1-longer");
    assert_eq!(rows[1].thought, "thought-tool");
    assert_eq!(rows[2].thought, "thought-3");
    // Justification: best-effort persistence/sync/notification; failure logged, in-memory state remains authoritative for this run
    crate::error::record_ignored(
        crate::modules::proxy_db::delete_thinking_records_for_session(store_key),
        "delete_thinking_records_for_session",
    );
}

#[test]
fn test_explicit_session_id_and_query_extraction() {
    // 1. Query parameter extraction
    let sid =
        extract_session_from_query_str("model=gemini-2.5-pro&session_id=win_alpha_101&temp=0.7");
    assert_eq!(sid.as_deref(), Some("win_alpha_101"));

    let sid_alias = extract_session_from_query_str("channel=proj_beta");
    assert_eq!(sid_alias.as_deref(), Some("proj_beta"));

    // 2. Header extraction: Claude Code / Cursor / VSCode
    let mut headers = HeaderMap::new();
    headers.insert("x-cursor-session-id", "cursor-tab-99".parse().unwrap());
    let extracted = explicit_session_id_with_query(&headers, None, None);
    assert_eq!(extracted.as_deref(), Some("cursor-tab-99"));

    // 3. Body & Metadata extraction
    let body = json!({
        "metadata": {
            "conversation_id": "meta-conv-888"
        }
    });
    let empty_headers = HeaderMap::new();
    let extracted2 = explicit_session_id_with_query(&empty_headers, Some(&body), None);
    assert_eq!(extracted2.as_deref(), Some("meta-conv-888"));
}

#[test]
fn test_wildcard_client_session_headers() {
    let uuid = "c17b6d3c-e808-4874-8f16-b5dd4b6a2179";

    let mut atom = HeaderMap::new();
    atom.insert("x-atomcode-session-id", uuid.parse().unwrap());
    assert_eq!(
        explicit_session_id_with_query(&atom, None, None).as_deref(),
        Some(uuid)
    );

    let mut jeik = HeaderMap::new();
    jeik.insert("x-jeikcode-sessionid", uuid.parse().unwrap());
    assert_eq!(
        explicit_session_id_with_query(&jeik, None, None).as_deref(),
        Some(uuid)
    );

    let mut multi = HeaderMap::new();
    multi.insert("x-api-key", "secret".parse().unwrap());
    multi.insert("x-atomcode-session-id", uuid.parse().unwrap());
    multi.insert("x-jeikcode-sessionid", uuid.parse().unwrap());
    multi.insert("x-session-id", uuid.parse().unwrap());
    assert_eq!(
        explicit_session_id_with_query(&multi, None, None).as_deref(),
        Some(uuid)
    );

    let mut custom = HeaderMap::new();
    custom.insert("x-windsurf-session-id", "wind-tab-1".parse().unwrap());
    assert_eq!(
        explicit_session_id_with_query(&custom, None, None).as_deref(),
        Some("wind-tab-1")
    );

    let mut ignored = HeaderMap::new();
    ignored.insert("x-request-id", "req-should-not-win".parse().unwrap());
    ignored.insert("x-api-key", "secret".parse().unwrap());
    assert_eq!(explicit_session_id_with_query(&ignored, None, None), None);
}

#[test]
fn product_session_headers_win_over_generic_x_session_id() {
    let mut jeik = HeaderMap::new();
    jeik.insert("x-session-id", "generic-session".parse().unwrap());
    jeik.insert("x-jeikcode-sessionid", "jeik-session".parse().unwrap());
    assert_eq!(
        explicit_session_id_with_query(&jeik, None, None).as_deref(),
        Some("jeik-session")
    );

    let mut atom = HeaderMap::new();
    atom.insert("x-session-id", "generic-session".parse().unwrap());
    atom.insert("x-atomcode-session-id", "atom-session".parse().unwrap());
    assert_eq!(
        explicit_session_id_with_query(&atom, None, None).as_deref(),
        Some("atom-session")
    );

    let mut wildcard = HeaderMap::new();
    wildcard.insert("x-session-id", "generic-session".parse().unwrap());
    wildcard.insert("x-windsurf-session-id", "wind-session".parse().unwrap());
    assert_eq!(
        explicit_session_id_with_query(&wildcard, None, None).as_deref(),
        Some("wind-session")
    );

    let mut only_generic = HeaderMap::new();
    only_generic.insert("x-session-id", "generic-session".parse().unwrap());
    assert_eq!(
        explicit_session_id_with_query(&only_generic, None, None).as_deref(),
        Some("generic-session")
    );
}

#[test]
fn test_3d_orthogonal_blended_session_stability_and_isolation() {
    // 1. 同一对话多轮聊天：相同 Headers 与相同的 Anchor -> 100% 相同稳定
    let mut headers = HeaderMap::new();
    headers.insert(
        "x-claude-code-session-id",
        "53696541-0a6e-4be0-801e-2ee7a5601831".parse().unwrap(),
    );
    let scope_turn1 = SessionScope::from_headers(&headers, "sid-main-conversation-root");
    let scope_turn2 = SessionScope::from_headers(&headers, "sid-main-conversation-root");
    assert_eq!(scope_turn1.client_id, scope_turn2.client_id);
    assert_eq!(scope_turn1.store_key, scope_turn2.store_key);

    // 2. 主 Agent 与 Subagent 在同一 CLI 进程下（相同 x-claude-code-session-id 但不同 Anchor） -> 绝对隔离！
    let scope_subagent = SessionScope::from_headers(&headers, "sid-subagent-distinct-prompt");
    assert_ne!(scope_turn1.client_id, scope_subagent.client_id);
    assert_ne!(scope_turn1.store_key, scope_subagent.store_key);

    // 3. 不同租户多用户并发（不同 Authorization / API Key） -> 绝对隔离！
    let mut headers_user_a = headers.clone();
    headers_user_a.insert("authorization", "Bearer user-token-aaa".parse().unwrap());
    let mut headers_user_b = headers.clone();
    headers_user_b.insert("authorization", "Bearer user-token-bbb".parse().unwrap());
    let scope_user_a = SessionScope::from_headers(&headers_user_a, "sid-main-conversation-root");
    let scope_user_b = SessionScope::from_headers(&headers_user_b, "sid-main-conversation-root");
    assert_ne!(scope_user_a.store_key, scope_user_b.store_key);

    // 4. Header 乱序注入时哈希绝对一致（BTreeMap 保证确定性排序）
    let mut headers_order1 = HeaderMap::new();
    headers_order1.insert("x-atomcode-session-id", "uuid-123".parse().unwrap());
    headers_order1.insert("x-jeikcode-sessionid", "uuid-456".parse().unwrap());

    let mut headers_order2 = HeaderMap::new();
    headers_order2.insert("x-jeikcode-sessionid", "uuid-456".parse().unwrap());
    headers_order2.insert("x-atomcode-session-id", "uuid-123".parse().unwrap());

    let scope_ord1 = SessionScope::from_headers(&headers_order1, "anchor-1");
    let scope_ord2 = SessionScope::from_headers(&headers_order2, "anchor-1");
    assert_eq!(scope_ord1.client_id, scope_ord2.client_id);

    // 5. 跨协议相同显式会话头（如 Claude 切到 OpenAI）：相同会话锚点下 store_key 绝对一致共享，不同锚点下强隔离
    let mut claude_headers = HeaderMap::new();
    claude_headers.insert("x-session-id", "conv-uuid-999".parse().unwrap());
    claude_headers.insert("x-api-key", "secret-token".parse().unwrap());

    let mut openai_headers = HeaderMap::new();
    openai_headers.insert("x-session-id", "conv-uuid-999".parse().unwrap());
    openai_headers.insert("authorization", "Bearer secret-token".parse().unwrap());

    let scope_claude = SessionScope::from_headers(&claude_headers, "shared-anchor");
    let scope_openai = SessionScope::from_headers(&openai_headers, "shared-anchor");

    assert_eq!(
        scope_claude.store_key, scope_openai.store_key,
        "Cross-protocol requests in the same session must share the identical store_key"
    );
}

#[test]
fn tail_first_match_uses_latest_record_for_latest_incomplete_turn() {
    let store = ThinkingStore::new();
    let key = "t:tail-hello";
    store.record(key, rec("think-turn1", "你好", None));
    store.record(key, rec("think-turn2", "你好", None));

    let sig = "s".repeat(60);
    let mut contents = vec![
        json!({
            "role": "model",
            "parts": [
                { "text": "think-turn1", "thought": true, "thoughtSignature": sig },
                { "text": "你好" }
            ]
        }),
        json!({
            "role": "model",
            "parts": [{ "text": "你好" }]
        }),
    ];
    let restored = store.restore_gemini_contents(key, &mut contents);
    assert_eq!(restored, 1);
    assert_eq!(contents[0]["parts"][0]["text"], "think-turn1");
    assert_eq!(contents[1]["parts"][0]["thought"], true);
    assert_eq!(
        contents[1]["parts"][0]["text"], "think-turn2",
        "latest incomplete turn must take the latest matching record, not the first 你好"
    );
}

#[test]
fn concurrent_sessions_do_not_mix_thinking() {
    use std::thread;

    let store = Arc::new(ThinkingStore::new());
    let handles: Vec<_> = (0..48)
        .map(|i| {
            let store = Arc::clone(&store);
            thread::spawn(move || {
                let key = format!("t:conc-{i}");
                let thought = format!("thought-{i}");
                let visible = format!("hello-{i}");
                store.record(&key, rec(&thought, &visible, None));
                let mut contents = vec![json!({
                    "role": "model",
                    "parts": [{ "text": visible }]
                })];
                let n = store.restore_gemini_contents(&key, &mut contents);
                assert_eq!(n, 1);
                assert_eq!(contents[0]["parts"][0]["text"], thought);
            })
        })
        .collect();
    for h in handles {
        h.join().expect("session thread panicked");
    }
}

#[test]
fn capture_from_client_history_and_prune_compressed_turns() {
    let store = ThinkingStore::new();
    let session_key = format!("t:compress-{}", uuid::Uuid::new_v4());
    let key = &session_key;
    store.record(
        key,
        rec("thought-old-1", "old visible one", Some("call_old_1")),
    );
    store.record(
        key,
        rec("thought-old-2", "old visible two", Some("call_old_2")),
    );
    store.record(
        key,
        rec("thought-old-3", "old visible three", Some("call_old_3")),
    );
    store.record(
        key,
        rec("thought-keep", "kept latest answer", Some("call_keep")),
    );
    store.record(key, rec("thought-tail", "newest unused", None));

    // Client /compact dropped the first three turns; only the latest kept turn remains.
    let mut contents = vec![json!({
        "role": "model",
        "parts": [
            { "text": "kept latest answer" },
            { "functionCall": { "name": "shell", "id": "call_keep", "args": {} } }
        ]
    })];

    let restored = store.restore_gemini_contents(key, &mut contents);
    assert_eq!(restored, 1);
    store.prune_orphaned_records(key, &contents);
    let (turns, _) = store.session_stats(key).unwrap();
    assert!(
        turns <= 3,
        "orphaned compressed turns should be pruned, got {turns}"
    );
    let parts = contents[0]["parts"].as_array().unwrap();
    assert_eq!(parts[0]["text"], "thought-keep");
    store.end_session(key);
}
