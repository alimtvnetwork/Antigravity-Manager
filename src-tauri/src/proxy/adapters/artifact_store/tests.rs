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
        // MOC-235: Universal retrieval from the global store (internal archive under test), save and retrieve the full text according to artifact_id without truncating.
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
        // True does not exist / empty id → Ok (None) (not Err); Err is only reserved for instantaneous DB read failure.
        assert!(read_tool_artifact_raw("nonexistent_moc235")
            .expect("miss is not an error")
            .is_none());
        assert!(read_tool_artifact_raw("   ")
            .expect("blank id is not an error")
            .is_none());
    }

    #[test]
    fn save_marks_persisted_only_when_db_backed() {
        // MOC-235 review #4: persisted must truthfully reflect "whether the shared DB has been entered" - the cross-process reader can only read the DB,
        // Based on this, the summary decides whether to inform the model that it can be retrieved (the internal archive does not inform the model, otherwise it will give an ID that the reader cannot see).
        let mem = ToolArtifactStore::new(8, Duration::from_secs(60));
        assert!(
            !mem.save(Some("c"), "command_output", "raw").persisted,
            "Internal archives without DB should not be marked persisted"
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
            "Successful writing to the shared DB should be marked persisted"
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
