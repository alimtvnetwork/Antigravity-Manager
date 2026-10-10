use crate::proxy::config::LogRetentionConfig;
use rusqlite::{params, Connection, OpenFlags};

use super::*;

#[cfg(test)]
pub(crate) mod thinking_pack_tests {
    use super::*;

    #[test]
    pub(crate) fn pack_roundtrip_short_and_long() {
        let short = "hello thought";
        assert_eq!(unpack_thought(&pack_thought(short)), short);
        assert!(pack_thought(short).starts_with(THOUGHT_RAW_MAGIC));

        let long = "word ".repeat(2000);
        let packed = pack_thought(&long);
        assert!(
            packed.starts_with(THOUGHT_GZIP_MAGIC),
            "long thought should gzip"
        );
        assert!(packed.len() < long.len());
        assert_eq!(unpack_thought(&packed), long);
    }

    #[test]
    pub(crate) fn unpack_legacy_utf8() {
        assert_eq!(unpack_thought(b"plain old thought"), "plain old thought");
    }

    #[test]
    pub(crate) fn persist_visible_drops_tool_turns() {
        assert_eq!(
            persist_visible(&["call_1".to_string()], "I will run the tool"),
            ""
        );
        assert_eq!(persist_visible(&[], "hello"), "hello");
    }
}

#[cfg(test)]
pub(crate) mod tool_signature_tests {
    use super::*;
    use crate::proxy::monitor::prompt_log_tests::TestDataDir;

    #[test]
    pub(crate) fn tool_signature_misses_reuse_readonly_connection() {
        let _dir = TestDataDir::new();
        assert!(load_tool_signature("missing").is_err());
        assert!(!get_proxy_db_path().unwrap().exists());
        init_db().unwrap();
        let writer = connect_db().unwrap();
        assert_eq!(
            writer
                .pragma_query_value::<i64, _>(None, "auto_vacuum", |r| r.get(0))
                .unwrap(),
            2
        );
        let before: i64 = writer
            .pragma_query_value(None, "data_version", |r| r.get(0))
            .unwrap();
        assert_eq!(load_tool_signature("missing").unwrap(), None);
        {
            let db = TOOL_SIGNATURE_DB.get().unwrap().lock().unwrap();
            let conn = &db.as_ref().unwrap().1;
            assert!(conn.is_readonly(rusqlite::DatabaseName::Main).unwrap());
            // A connection-local setting detects accidental reopening on a miss.
            conn.pragma_update(None, "cache_size", -1234).unwrap();
        }
        for _ in 0..32 {
            assert_eq!(load_tool_signature("missing").unwrap(), None);
        }
        {
            let db = TOOL_SIGNATURE_DB.get().unwrap().lock().unwrap();
            let conn = &db.as_ref().unwrap().1;
            assert_eq!(
                conn.pragma_query_value::<i64, _>(None, "cache_size", |r| r.get(0))
                    .unwrap(),
                -1234
            );
            assert!(conn.is_autocommit());
            assert_eq!(conn.total_changes(), 0);
        }
        let after: i64 = writer
            .pragma_query_value(None, "data_version", |r| r.get(0))
            .unwrap();
        assert_eq!(after, before);
        TOOL_SIGNATURE_DB.get().unwrap().lock().unwrap().take();
    }

    #[test]
    pub(crate) fn tool_signature_reads_follow_writes_and_data_dir_changes() {
        let _dir = TestDataDir::new();
        init_db().unwrap();
        let signature = "s".repeat(60);
        assert_eq!(load_tool_signature("tool").unwrap(), None);
        save_tool_signature("tool", &signature).unwrap();
        assert_eq!(
            load_tool_signature("tool").unwrap(),
            Some(signature.clone())
        );
        let replacement = "r".repeat(60);
        save_tool_signature("tool", &replacement).unwrap();
        assert_eq!(
            load_tool_signature("tool").unwrap(),
            Some(replacement.clone())
        );
        {
            let _other_dir = TestDataDir::new_nested();
            assert!(load_tool_signature("tool").is_err());
            init_db().unwrap();
            assert_eq!(load_tool_signature("tool").unwrap(), None);
            save_tool_signature("tool", &signature).unwrap();
            assert_eq!(load_tool_signature("tool").unwrap(), Some(signature));
            TOOL_SIGNATURE_DB.get().unwrap().lock().unwrap().take();
        }
        assert_eq!(load_tool_signature("tool").unwrap(), Some(replacement));
        TOOL_SIGNATURE_DB.get().unwrap().lock().unwrap().take();
    }
}

#[cfg(test)]
pub(crate) mod thinking_sqlite_tests {
    use super::*;
    use crate::proxy::monitor::prompt_log_tests::TestDataDir;

    #[test]
    pub(crate) fn test_thinking_record_deduplication_and_penetration_lookup() {
        let _dir = TestDataDir::new();

        let session_key = "test_tenant:sess-123456";
        let tool_id = "call_abc999";
        let real_sig = "s".repeat(60);

        // 1. 首次写入：实质思考 + tool_id
        save_thinking_record(
            session_key,
            "fp_turn1",
            "This is deep analytical thinking about rust code",
            Some(&real_sig),
            &[tool_id.to_string()],
            &[],
            "visible",
        )
        .unwrap();

        // 2. 二次写入相同 tool_id（例如客户端再次回传包含占位符的相同轮次）：绝不叠加新行，绝不将实质思考覆盖为占位符！
        save_thinking_record(
            session_key,
            "fp_turn1",
            "...",
            Some(&real_sig),
            &[tool_id.to_string()],
            &[],
            "visible",
        )
        .unwrap();

        // 3. 验证 SQLite 中仅存 1 行，且保留高质量思考
        let all = load_thinking_records(session_key).unwrap();
        assert_eq!(all.len(), 1, "Duplicate tool saves must be deduplicated!");
        assert_eq!(
            all[0].thought,
            "This is deep analytical thinking about rust code"
        );
        assert_eq!(all[0].signature, Some(real_sig.clone()));

        // 4. 精准穿透点查 tool_id
        let loaded = load_thinking_by_tool_id(session_key, tool_id).unwrap();
        assert!(loaded.is_some());
        let rec = loaded.unwrap();
        assert_eq!(
            rec.thought,
            "This is deep analytical thinking about rust code"
        );
        assert_eq!(rec.signature, Some(real_sig));

        // 5. 不存在的 tool_id 应当正确返回 None
        let missing = load_thinking_by_tool_id(session_key, "call_nonexistent").unwrap();
        assert!(missing.is_none());

        // 6. 纯文本指纹点查测试
        let text_fp = "fp_pure_text_1";
        save_thinking_record(
            session_key,
            text_fp,
            "Pure text reasoning",
            None,
            &[],
            &[],
            "pure text visible",
        )
        .unwrap();
        let loaded_text = load_thinking_by_fingerprint(session_key, text_fp).unwrap();
        assert!(loaded_text.is_some());
        assert_eq!(loaded_text.unwrap().thought, "Pure text reasoning");
    }

    #[test]
    pub(crate) fn test_signature_healing_and_write_back() {
        use base64::Engine;
        let _dir = TestDataDir::new();
        init_db().unwrap();

        let session_key = "test_tenant:sess-healing";
        let tool_id = "call_corrupted_1";

        // 构造一个典型的被错误解码为 UTF-8 原始 Protobuf 二进制的签名 (首字节 0x12)
        let raw_proto_bytes = [
            0x12, 0x26, 0x0a, 0x24, b'e', b'2', b'4', b'8', b'3', b'0', b'a', b'7', b'-', b'5',
            b'c', b'd', b'6', b'-', b'4', b'2', b'f', b'e', b'-', b'9', b'9', b'8', b'b', b'-',
            b'e', b'e', b'5', b'3', b'9', b'e', b'7', b'2', b'b', b'9', b'c', b'3',
        ];
        let raw_corrupted_sig = String::from_utf8(raw_proto_bytes.to_vec()).unwrap();
        let expected_base64 = base64::engine::general_purpose::STANDARD.encode(raw_proto_bytes);
        assert_eq!(
            expected_base64,
            "EiYKJGUyNDgzMGE3LTVjZDYtNDJmZS05OThiLWVlNTM5ZTcyYjljMw=="
        );

        // 1. normalize_and_heal_signature 单测
        assert_eq!(
            normalize_and_heal_signature(&raw_corrupted_sig),
            Some(expected_base64.clone())
        );
        assert_eq!(
            normalize_and_heal_signature(&expected_base64),
            Some(expected_base64.clone())
        );
        assert_eq!(normalize_and_heal_signature(SENTINEL_SIGNATURE), None);
        assert_eq!(normalize_and_heal_signature("short"), None);

        // 2. save_tool_signature 会自动自愈为 Base64 存储
        save_tool_signature(tool_id, &raw_corrupted_sig).unwrap();
        let loaded_tool_sig = load_tool_signature(tool_id).unwrap();
        assert_eq!(loaded_tool_sig, Some(expected_base64.clone()));

        // 3. 模拟底层 SQLite 已经脏存了原始二进制签名的历史数据
        let conn = connect_db().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO tool_signatures (tool_id, signature, created_at) VALUES (?1, ?2, ?3)",
            params!["call_legacy_dirty", &raw_corrupted_sig, 1000],
        ).unwrap();
        drop(conn);

        // load_tool_signature 读出时自动识别并修复，且反向写回 SQLite
        let loaded_dirty = load_tool_signature("call_legacy_dirty").unwrap();
        assert_eq!(loaded_dirty, Some(expected_base64.clone()));

        // 验证 SQLite 中确实已被写回替换为标准 Base64 格式
        let conn = connect_db().unwrap();
        let in_db: String = conn
            .query_row(
                "SELECT signature FROM tool_signatures WHERE tool_id = 'call_legacy_dirty'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(in_db, expected_base64);

        // 4. thinking_records 自愈与反向写回测试
        let think_conn = thinking_db().unwrap();
        think_conn.execute(
            "INSERT INTO thinking_records (session_key, fingerprint, thought, signature, tool_ids, tool_names, visible, created_at, primary_tool_id, causal_tool_id)
             VALUES (?1, 'fp_dirty', ?2, ?3, '[\"call_corrupted_1\"]', '[]', 'vis', 1000, 'call_corrupted_1', 'call_corrupted_1')",
            params![session_key, pack_thought("thinking content"), &raw_corrupted_sig],
        ).unwrap();
        drop(think_conn);

        // load_thinking_by_tool_id 点查时触发反向自愈写回
        let loaded_rec = load_thinking_by_tool_id(session_key, "call_corrupted_1")
            .unwrap()
            .unwrap();
        assert_eq!(loaded_rec.signature, Some(expected_base64.clone()));

        // 验证 thinking_records 表中 signature 字段已被更新为自愈后的 Base64
        let think_conn = thinking_db().unwrap();
        let sig_in_db: String = think_conn
            .query_row(
                "SELECT signature FROM thinking_records WHERE session_key = ?1 AND primary_tool_id = 'call_corrupted_1'",
                params![session_key],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(sig_in_db, expected_base64);
    }
}

#[cfg(test)]
pub(crate) mod retention_tests {
    use super::*;
    use crate::proxy::config::LogRetentionConfig;
    use rusqlite::Connection;

    #[test]
    pub(crate) fn prompt_log_disk_budget_cleanup_and_live_config_reload() {
        use crate::proxy::monitor::prompt_log_tests::{sample_log, TestDataDir};
        let _dir = TestDataDir::new();
        init_db().unwrap();
        let mut config = crate::modules::config::load_app_config().unwrap();
        assert_eq!(
            serde_json::from_str::<LogRetentionConfig>("{}")
                .unwrap()
                .max_disk_mb,
            1024
        );
        config.proxy.log_retention.max_disk_mb = 8;
        config.proxy.log_retention.max_storage_gb = 0.0;
        crate::modules::config::save_app_config(&config).unwrap();
        save_log(sample_log("old", 300_000)).unwrap();
        let conn = connect_db().unwrap();
        reclaim_space(&conn).unwrap();
        assert!(disk_bytes(&conn).unwrap() > 1024 * 1024);
        config.proxy.log_retention.max_disk_mb = 1;
        config.proxy.log_retention.max_storage_gb = 0.0;
        crate::modules::config::save_app_config(&config).unwrap();
        save_log(sample_log("new", 4096)).unwrap();
        assert!(get_log_detail("old").is_err());
        assert_eq!(
            get_log_detail("new").unwrap().response_body,
            Some("错".repeat(4096))
        );
        assert!(disk_bytes(&conn).unwrap() <= 1024 * 1024);
        save_log(sample_log("oversize", 400_000)).unwrap();
        assert!(get_log_detail("oversize").unwrap().response_body.is_none());
        let zero_budget_policy = LogRetentionConfig {
            max_disk_mb: 0,
            max_storage_gb: 0.0,
            ..config.proxy.log_retention
        };
        assert!(
            save_log_with_connection(&conn, sample_log("no-room", 100), &zero_budget_policy)
                .is_err()
        );
        assert!(get_log_detail("no-room").is_err());
    }

    #[test]
    pub(crate) fn prompt_log_legacy_headroom_rejection_preserves_history_on_retries() {
        use crate::proxy::monitor::prompt_log_tests::TestDataDir;
        let _dir = TestDataDir::new();
        let conn = Connection::open(get_proxy_db_path().unwrap()).unwrap();
        conn.execute_batch("CREATE TABLE request_logs (id TEXT PRIMARY KEY, timestamp INTEGER, method TEXT, url TEXT, status INTEGER, duration INTEGER, model TEXT, error TEXT, response_body TEXT)").unwrap();
        assert_eq!(
            conn.pragma_query_value::<i64, _>(None, "auto_vacuum", |r| r.get(0))
                .unwrap(),
            0
        );
        conn.execute_batch(
            "WITH RECURSIVE n(i) AS (VALUES(1) UNION ALL SELECT i + 1 FROM n WHERE i < 128)
             INSERT INTO request_logs (id, timestamp, response_body)
             SELECT CAST(i AS TEXT), i, zeroblob(8192) FROM n;",
        )
        .unwrap();
        reclaim_space(&conn).unwrap();
        let before = disk_bytes(&conn).unwrap();
        let budget = 2 * 1024 * 1024;
        let log_bytes = 500_000;
        assert!(before < budget);
        assert!(before + 2 * log_bytes + 64 * 1024 > budget);
        assert!(3 * log_bytes + 64 * 1024 <= budget / 5 * 4);

        for _ in 0..6 {
            assert!(make_room(&conn, budget, log_bytes).is_err());
            let counts: (i64, i64) = conn
                .query_row(
                    "SELECT COUNT(*), COUNT(response_body) FROM request_logs",
                    [],
                    |r| Ok((r.get(0)?, r.get(1)?)),
                )
                .unwrap();
            assert_eq!(counts, (128, 128));
            assert_eq!(disk_bytes(&conn).unwrap(), before);
        }
    }

    #[test]
    pub(crate) fn prompt_log_reclaims_free_pages_before_deleting_summaries() {
        use crate::proxy::monitor::prompt_log_tests::{sample_log, TestDataDir};
        let _dir = TestDataDir::new();
        init_db().unwrap();
        let conn = connect_db().unwrap();
        conn.execute_batch(
            "INSERT INTO request_logs (id, timestamp, response_body) VALUES
             ('old-1', 1, zeroblob(2097152)), ('old-2', 2, zeroblob(2097152)),
             ('old-3', 3, zeroblob(2097152));",
        )
        .unwrap();
        reclaim_space(&conn).unwrap();
        assert!(disk_bytes(&conn).unwrap() > 6 * 1024 * 1024);
        let policy = LogRetentionConfig {
            max_disk_mb: 1,
            max_storage_gb: 0.0,
            ..LogRetentionConfig::default()
        };

        save_log_with_connection(&conn, sample_log("new", 100), &policy).unwrap();

        let counts: (i64, i64) = conn
            .query_row(
                "SELECT COUNT(*), COUNT(response_body) FROM request_logs WHERE id != 'new'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(counts, (0, 0));
        assert_eq!(
            get_log_detail("new").unwrap().response_body,
            Some("错".repeat(100))
        );
        assert!(disk_bytes(&conn).unwrap() <= 1024 * 1024);
    }

    #[test]
    pub(crate) fn clears_old_bodies_and_limits_rows() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE request_logs (id TEXT PRIMARY KEY, timestamp INTEGER, request_body TEXT, upstream_request_body TEXT, response_body TEXT)").unwrap();
        let now = chrono::Utc::now().timestamp_millis();
        conn.execute(
            "INSERT INTO request_logs VALUES ('retained-with-old-body', ?1, 'request', NULL, 'response')",
            [now - 25 * 3600 * 1000],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO request_logs VALUES ('new-1', ?1, NULL, NULL, NULL)",
            [now - 30 * 3600 * 1000],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO request_logs VALUES ('deleted-1', ?1, NULL, NULL, NULL)",
            [now - 35 * 3600 * 1000],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO request_logs VALUES ('deleted-2', ?1, NULL, NULL, NULL)",
            [now - 40 * 3600 * 1000],
        )
        .unwrap();
        let policy = LogRetentionConfig {
            max_body_age_hours: 24,
            max_age_days: 30,
            max_rows: 2,
            ..LogRetentionConfig::default()
        };
        let (cleared, deleted) = apply_retention_with_connection(&conn, &policy).unwrap();
        assert_eq!(cleared, 0);
        assert_eq!(deleted, 2);
        let body: Option<String> = conn
            .query_row(
                "SELECT request_body FROM request_logs WHERE id = 'retained-with-old-body'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(body, Some("request".to_string()));
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM request_logs", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 2);
    }
}
