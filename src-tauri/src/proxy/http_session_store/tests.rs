use super::*;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn entry(text: &str) -> HttpSessionEntry {
        HttpSessionEntry {
            input_items: vec![json!({
                "id": format!("msg-{text}"),
                "type": "message",
                "role": "user",
                "content": text
            })],
            instructions: "be concise".to_string(),
            model: "gemini-3.7-flash-high".to_string(),
            last_accessed: Instant::now(),
        }
    }

    #[test]
    fn responses_store_false_restores_full_input_without_retaining_delta() {
        let history = vec![
            json!({"id": "old", "role": "user", "content": "x".repeat(32768)}),
            json!({"id": "answer", "role": "assistant", "content": "prior answer"}),
        ];
        let next = json!({"id": "new", "role": "user", "content": "y".repeat(32768)});
        let mut full_input = history.clone();
        full_input.push(next.clone());
        for replay in [vec![next], full_input.clone()] {
            let prepared =
                prepare_session_input_with_storage(history.clone(), replay, &HashMap::new(), false);
            assert!(prepared.delta.is_empty());
            assert_eq!(prepared.merged, full_input);
        }
        let prepared =
            prepare_session_input_with_storage(Vec::new(), history.clone(), &HashMap::new(), false);
        assert!(prepared.delta.is_empty());
        assert_eq!(prepared.merged, history);
    }

    #[test]
    fn responses_store_false_full_tool_replay_needs_no_global_cache() {
        let call_id = format!("uncached-{}", uuid::Uuid::new_v4());
        let input = vec![
            json!({"id":"call-item", "type":"function_call", "call_id":call_id, "name":"shell_command", "arguments":"{\"command\":\"pwd\"}"}),
            json!({"type":"function_call_output", "call_id":call_id, "output":"/synthetic/workspace"}),
            json!({"role":"user", "content":"continue"}),
        ];
        assert!(get_cached_tool_call(&call_id).is_none());
        let prepared =
            prepare_session_input_with_storage(Vec::new(), input.clone(), &HashMap::new(), false);
        assert!(prepared.delta.is_empty());
        assert_eq!(prepared.merged, input);
    }

    #[test]
    fn session_chain_stores_delta_and_materializes_history() {
        let mut store = HttpSessionStore::new();
        store.insert("resp-1".to_string(), entry("first"));
        let (root, parent) = store.get("resp-1").expect("root");
        let mut replay = root.input_items.clone();
        replay.push(json!({"id": "msg-second", "content": "second"}));
        let prepared = prepare_session_input(root.input_items, replay, &HashMap::new());
        assert_eq!(prepared.delta.len(), 1);
        assert_eq!(prepared.merged.len(), 2);
        store.insert_delta(
            "resp-2".to_string(),
            Some(parent),
            prepared.delta,
            vec![json!({"id": "out-second", "content": "answer"})],
            "be concise".to_string(),
            "gemini-3.7-flash-high".to_string(),
            Some("routing-root".to_string()),
        );

        let (previous, _) = store.get("resp-2").expect("child");
        assert_eq!(previous.input_items[0]["content"], "first");
        assert_eq!(previous.input_items.len(), 3);
        assert_eq!(store.sessions["resp-2"].node.input_delta.len(), 1);
        assert_eq!(store.sessions["resp-2"].node.response_output.len(), 1);
    }

    #[test]
    fn old_response_id_branches_share_parent() {
        let mut store = HttpSessionStore::new();
        store.insert("resp-root".to_string(), entry("root"));
        let (_, parent_a) = store.get("resp-root").expect("parent a");
        let (_, parent_b) = store.get("resp-root").expect("parent b");
        assert!(Arc::ptr_eq(&parent_a.0, &parent_b.0));

        store.insert_delta(
            "resp-a".to_string(),
            Some(parent_a),
            vec![json!({"content": "branch a"})],
            Vec::new(),
            String::new(),
            String::new(),
            Some("routing-root".to_string()),
        );
        store.insert_delta(
            "resp-b".to_string(),
            Some(parent_b),
            vec![json!({"content": "branch b"})],
            Vec::new(),
            String::new(),
            String::new(),
            Some("routing-root".to_string()),
        );

        let parent_a = store.sessions["resp-a"].node.parent.as_ref().unwrap();
        let parent_b = store.sessions["resp-b"].node.parent.as_ref().unwrap();
        assert!(Arc::ptr_eq(parent_a, parent_b));
        assert_eq!(
            store.sessions["resp-a"].node.routing_session_id,
            "routing-root"
        );
        assert_eq!(
            store.sessions["resp-b"].node.routing_session_id,
            "routing-root"
        );
    }

    #[test]
    fn prepare_session_input_prevents_duplication_on_full_history_replay_without_ids() {
        // Test case when client resends full history without IDs:
        // history has 2 messages, new_input has 3 messages (the same 2 + 1 new), but no "id" field.
        let history = vec![
            json!({"role": "user", "type": "message", "content": "hello"}),
            json!({"role": "assistant", "type": "message", "content": "hi there"}),
        ];
        let new_input = vec![
            json!({"role": "user", "type": "message", "content": "hello"}),
            json!({"role": "assistant", "type": "message", "content": "hi there"}),
            json!({"role": "user", "type": "message", "content": "next question"}),
        ];

        let prepared = prepare_session_input(history, new_input, &HashMap::new());
        // Delta should be the 1 new message, not all 3 messages!
        assert_eq!(prepared.delta.len(), 1);
        assert_eq!(prepared.delta[0]["content"], "next question");
        // Merged history should be 3 messages, NOT 5 (2 + 3)!
        assert_eq!(prepared.merged.len(), 3);
    }

    #[test]
    fn prepare_session_input_fallback_avoids_duplication_when_unmatched() {
        let history = vec![
            json!({"role": "user", "type": "message", "content": "msg 1"}),
            json!({"role": "assistant", "type": "message", "content": "msg 2"}),
        ];
        // Client sends 2 different messages (length >= history)
        let new_input = vec![
            json!({"role": "user", "type": "message", "content": "different 1"}),
            json!({"role": "assistant", "type": "message", "content": "different 2"}),
        ];

        let prepared = prepare_session_input(history, new_input, &HashMap::new());
        // Should not duplicate to 4 items
        assert_eq!(prepared.merged.len(), 2);
    }
}
