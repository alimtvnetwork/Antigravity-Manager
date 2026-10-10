use super::*;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn in_memory_store_round_trips_raw_payload() {
        let store = ToolArtifactStore::new(8, Duration::from_secs(60));
        let stored = store.save(Some("call_a"), "command_output", "raw output");
        let record = store
            .get(&stored.artifact_id)
            .expect("artifact should exist");

        assert_eq!(record.call_id.as_deref(), Some("call_a"));
        assert_eq!(record.kind, "command_output");
        assert_eq!(record.raw_content, "raw output");
    }

    #[test]
    fn read_tool_artifact_raw_round_trips_via_global() {
        // MOC-235: 通用回取走 global store(test 下为内存档),save 后按 artifact_id 取回不截断全文。
        let stored = global_tool_artifact_store().save(
            Some("call_moc235"),
            "command_output",
            "FULL UNTRUNCATED PAYLOAD moc235",
        );
        assert_eq!(
            read_tool_artifact_raw(&stored.artifact_id)
                .expect("read should not error")
                .as_deref(),
            Some("FULL UNTRUNCATED PAYLOAD moc235")
        );
        // 真不存在 / 空 id → Ok(None)(非 Err);Err 仅留给瞬时 DB 读失败。
        assert!(read_tool_artifact_raw("nonexistent_moc235")
            .expect("miss is not an error")
            .is_none());
        assert!(read_tool_artifact_raw("   ")
            .expect("blank id is not an error")
            .is_none());
    }

    #[test]
    fn save_marks_persisted_only_when_db_backed() {
        // MOC-235 review #4: persisted 必须如实反映「是否进了共享 DB」—— 跨进程 reader 只能读 DB,
        // 摘要据此决定要不要告知模型可回取(内存档不告知, 否则给一个 reader 看不到的 id)。
        let mem = ToolArtifactStore::new(8, Duration::from_secs(60));
        assert!(
            !mem.save(Some("c"), "command_output", "raw").persisted,
            "无 DB 的内存档不应标 persisted"
        );

        let dir = tempfile::tempdir().unwrap();
        let (db, _warn) = ToolArtifactStore::with_db_path(
            8,
            Duration::from_secs(60),
            DEFAULT_PERSISTED_TTL,
            &dir.path().join("a.db"),
        );
        assert!(
            db.save(Some("c"), "command_output", "raw").persisted,
            "写入共享 DB 成功应标 persisted"
        );
    }

    #[test]
    fn sqlite_store_round_trips_raw_payload() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("tool_artifacts.db");
        let (store, warn) = ToolArtifactStore::with_db_path(
            8,
            Duration::from_secs(60),
            DEFAULT_PERSISTED_TTL,
            &path,
        );
        assert!(warn.is_none());

        let stored = store.save(Some("call_b"), "web_or_search", "large web payload");
        drop(store);

        let (store2, warn2) = ToolArtifactStore::with_db_path(
            8,
            Duration::from_secs(60),
            DEFAULT_PERSISTED_TTL,
            &path,
        );
        assert!(warn2.is_none());
        let record = store2
            .get(&stored.artifact_id)
            .expect("artifact should load");

        assert_eq!(record.call_id.as_deref(), Some("call_b"));
        assert_eq!(record.kind, "web_or_search");
        assert_eq!(record.raw_content, "large web payload");
    }
}
