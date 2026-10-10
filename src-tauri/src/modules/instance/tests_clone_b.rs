//! Instance clone-tree tests.
use super::*;
use std::fs;
use std::path::PathBuf;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(test)]
mod clone_tree_tests {
    use super::*;

    #[test]
    fn test_sanitize_cloned_instance_summaries() {
        let test_inst_id = format!(
            "test_inst_sanitize_{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        );
        let instances_dir = get_instances_dir().expect("get_instances_dir should succeed");
        let inst_gemini_dir = instances_dir
            .join(&test_inst_id)
            .join("home")
            .join(".gemini")
            .join("antigravity");
        fs::create_dir_all(&inst_gemini_dir).expect("create_dir_all should succeed");

        let db_path = inst_gemini_dir.join("conversation_summaries.db");
        let conn = rusqlite::Connection::open(&db_path).expect("open db should succeed");
        conn.execute(
            "CREATE TABLE conversation_summaries (
                conversation_id TEXT PRIMARY KEY,
                not_fully_idle INTEGER,
                status TEXT
            )",
            [],
        )
        .expect("create table should succeed");

        conn.execute(
            "INSERT INTO conversation_summaries (conversation_id, not_fully_idle, status)
             VALUES ('conv-1', 1, 'CASCADE_RUN_STATUS_RUNNING')",
            [],
        )
        .expect("insert row should succeed");
        drop(conn);

        let res = sanitize_cloned_instance_summaries(&test_inst_id);
        assert!(res.is_ok());

        let conn2 = rusqlite::Connection::open(&db_path).expect("reopen db should succeed");
        let (idle, status): (i32, String) = conn2
            .query_row(
                "SELECT not_fully_idle, status FROM conversation_summaries WHERE conversation_id = 'conv-1'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .expect("query row should succeed");

        assert_eq!(idle, 0);
        assert_eq!(status, "IDLE");

        // Justification: test cleanup; absence is the normal case
        crate::error::record_ignored(
            fs::remove_dir_all(instances_dir.join(&test_inst_id)),
            "clean test dir",
        );
    }

    #[test]
    fn test_get_macos_candidate_paths_includes_user_applications() {
        let paths = get_macos_candidate_paths();
        assert!(paths
            .iter()
            .any(|p| p == &PathBuf::from("/Applications/Antigravity.app")));
        if let Some(home) = dirs::home_dir() {
            let user_app = home.join("Applications").join("Antigravity.app");
            let user_macos_bin = user_app.join("Contents").join("MacOS").join("Antigravity");
            assert!(paths.contains(&user_app));
            assert!(paths.contains(&user_macos_bin));
        }
    }

    #[test]
    fn test_resolve_instance_exe_name() {
        let default_exe = resolve_instance_exe_name("default", None);
        #[cfg(target_os = "windows")]
        assert!(default_exe.ends_with(".exe"));
        #[cfg(not(target_os = "windows"))]
        assert!(!default_exe.is_empty());

        let custom_exe = resolve_instance_exe_name("custom-1", Some("/opt/bin/my-ide"));
        assert_eq!(custom_exe, "my-ide");

        let fallback_exe = resolve_instance_exe_name("inst-2", None);
        #[cfg(target_os = "windows")]
        assert_eq!(fallback_exe, "Antigravity-inst-2.exe");
        #[cfg(not(target_os = "windows"))]
        assert_eq!(fallback_exe, "Antigravity-inst-2");
    }

    #[test]
    fn test_child_pid_promotion_in_smart_cache() {
        let test_inst = "test-child-promo-inst";
        let dead_primary = 99991u32;
        let live_child = 99992u32;

        if let Ok(mut guard) = MOCK_PID_ALIVE.write() {
            let mut set = std::collections::HashSet::new();
            set.insert(live_child);
            *guard = Some(set);
        }

        let now = chrono::Utc::now().timestamp();
        let record = InstanceProcessRecord {
            instance_id: test_inst.to_string(),
            pid: dead_primary,
            pids: vec![dead_primary, live_child],
            primary_pid: Some(dead_primary),
            data_dir: "/mock/test_data_dir".to_string(),
            launched_at: now,
            last_verified_at: now,
            is_alive: true,
            command_line: None,
        };
        update_cached_instance_process(record);

        let result = check_cached_pid_alive(test_inst, test_inst);
        assert!(result.is_some());
        let (is_alive, primary_pid, pids) = result.unwrap();
        assert!(is_alive);
        assert_eq!(primary_pid, Some(live_child));
        assert_eq!(pids, vec![live_child]);

        let cached = get_cached_instance_process(test_inst).unwrap();
        assert_eq!(cached.primary_pid, Some(live_child));
        assert_eq!(cached.pid, live_child);

        if let Ok(mut guard) = MOCK_PID_ALIVE.write() {
            *guard = None;
        }
        invalidate_instance_process_cache(test_inst);
    }

    #[test]
    fn test_ensure_instance_running_smart_reopen_guard() {
        let test_inst = "test-reopen-guard-inst";
        let live_pid = 99993u32;

        if let Ok(mut guard) = MOCK_PID_ALIVE.write() {
            let mut set = std::collections::HashSet::new();
            set.insert(live_pid);
            *guard = Some(set);
        }

        let now = chrono::Utc::now().timestamp();
        let record = InstanceProcessRecord {
            instance_id: test_inst.to_string(),
            pid: live_pid,
            pids: vec![live_pid],
            primary_pid: Some(live_pid),
            data_dir: "/mock/reopen_guard_dir".to_string(),
            launched_at: now,
            last_verified_at: now,
            is_alive: true,
            command_line: None,
        };
        update_cached_instance_process(record);

        let res = ensure_instance_running_smart(test_inst, None);
        assert!(res.is_ok());
        let (is_running, primary_pid) = res.unwrap();
        assert!(is_running);
        assert_eq!(primary_pid, Some(live_pid));

        if let Ok(mut guard) = MOCK_PID_ALIVE.write() {
            *guard = None;
        }
        invalidate_instance_process_cache(test_inst);
    }
}
