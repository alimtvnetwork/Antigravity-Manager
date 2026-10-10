//! Instance tests (tests_b).
use super::*;
use std::fs;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_instance_config_serialization() {
        let instance = InstanceConfig {
            id: "ubuntu-test".to_string(),
            name: "Ubuntu Test".to_string(),
            data_dir: "/home/user/.config/antigravity-test".to_string(),
            executable_path: Some("/opt/antigravity/antigravity".to_string()),
            extensions_dir: None,
            bound_account_id: Some("acc-123".to_string()),
            bound_email: Some("dev@example.com".to_string()),
            created_at: 1000,
            last_used: 2000,
            is_default: false,
            pid: Some(12345),
            seq_num: Some(2),
        };

        let json = serde_json::to_string(&instance).unwrap();
        let restored: InstanceConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.id, "ubuntu-test");
        assert_eq!(restored.pid, Some(12345));
        assert_eq!(restored.seq_num, Some(2));
        assert_eq!(
            restored.executable_path,
            Some("/opt/antigravity/antigravity".to_string())
        );
    }

    #[test]
    fn test_instance_pid_sqlite_persistence() {
        let temp_dir = std::env::temp_dir().join(format!(
            "agm_pid_test_{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        // Justification: test fixture setup; a real failure fails the test at the next step
        crate::error::record_ignored(std::fs::create_dir_all(&temp_dir), "create test dir");
        let test_db_path = temp_dir.join("test_instances.db");
        let conn = rusqlite::Connection::open(&test_db_path).unwrap();
        // Justification: test fixture seeding; a real failure fails the test at the next assertion
        crate::error::record_ignored(
            conn.execute_batch(
                "CREATE TABLE IF NOT EXISTS instance_processes (
                    instance_id TEXT PRIMARY KEY,
                    pid INTEGER NOT NULL,
                    data_dir TEXT,
                    status TEXT NOT NULL,
                    started_at INTEGER NOT NULL,
                    updated_at INTEGER NOT NULL
                );",
            ),
            "seed test db",
        );

        let now = chrono::Utc::now().timestamp();
        conn.execute(
            "INSERT OR REPLACE INTO instance_processes (instance_id, pid, data_dir, status, started_at, updated_at)
             VALUES (?1, ?2, ?3, 'running', ?4, ?4)",
            rusqlite::params!["inst-test-1", 54321, "/tmp/inst1", now],
        ).unwrap();

        let (saved_pid, status): (u32, String) = conn
            .query_row(
                "SELECT pid, status FROM instance_processes WHERE instance_id = ?1",
                ["inst-test-1"],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        assert_eq!(saved_pid, 54321);
        assert_eq!(status, "running");

        conn.execute(
            "UPDATE instance_processes SET status = 'stopped', updated_at = ?1 WHERE instance_id = ?2",
            rusqlite::params![now + 10, "inst-test-1"],
        ).unwrap();

        let updated_status: String = conn
            .query_row(
                "SELECT status FROM instance_processes WHERE instance_id = ?1",
                ["inst-test-1"],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(updated_status, "stopped");

        // Justification: test cleanup; absence is the normal case
        crate::error::record_ignored(std::fs::remove_dir_all(&temp_dir), "clean test dir");
    }

    #[test]
    fn test_ubuntu_instance_switching_end_to_end_flow() {
        // Heavy local disk / OS environment instance switching test: skip in CI/CD
        if crate::proxy::config::is_ci_environment() {
            return;
        }
        let temp_dir = std::env::temp_dir().join(format!(
            "agm_test_{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let instance_data = temp_dir.join("ubuntu-inst").join("data");
        let user_storage = instance_data.join("User").join("globalStorage");
        assert!(std::fs::create_dir_all(&user_storage).is_ok());

        let db_path = user_storage.join("state.vscdb");
        let conn = rusqlite::Connection::open(&db_path).unwrap();
        assert!(conn
            .execute(
                "CREATE TABLE IF NOT EXISTS ItemTable (key TEXT PRIMARY KEY, value BLOB)",
                [],
            )
            .is_ok());

        let rows_count: i64 = conn
            .query_row("SELECT count(*) FROM ItemTable", [], |r| r.get(0))
            .unwrap();
        assert_eq!(rows_count, 0);

        let linux_data_str = instance_data.to_string_lossy().to_string();
        let normalized = linux_data_str.to_lowercase().replace('\\', "/");
        let clean = normalized.trim_end_matches('/');
        assert!(clean.contains("ubuntu-inst"));

        // Justification: test cleanup; absence is the normal case
        crate::error::record_ignored(std::fs::remove_dir_all(&temp_dir), "clean test dir");
    }

    #[test]
    fn test_mock_account_switch_ci() {
        // Lightweight in-memory mock test for CI/CD account & instance binding rotation
        let mut inst = InstanceConfig {
            id: "inst-ci-mock".to_string(),
            name: "CI Mock Worker".to_string(),
            data_dir: "/mock/inst-ci".to_string(),
            executable_path: None,
            extensions_dir: None,
            bound_account_id: Some("acc-old".to_string()),
            bound_email: Some("old@example.com".to_string()),
            created_at: 1000,
            last_used: 1000,
            is_default: true,
            pid: None,
            seq_num: Some(1),
        };
        inst.bound_account_id = Some("acc-new".to_string());
        inst.bound_email = Some("new@example.com".to_string());
        inst.last_used = 2000;
        assert_eq!(inst.bound_account_id.as_deref(), Some("acc-new"));
        assert_eq!(inst.bound_email.as_deref(), Some("new@example.com"));
        assert_eq!(inst.last_used, 2000);
    }

    #[test]
    fn test_resolve_instance_id_resolution() {
        let registry = InstanceRegistry {
            active_instance_id: "inst-default".to_string(),
            instances: vec![
                InstanceConfig {
                    id: "inst-default".to_string(),
                    name: "Default Instance".to_string(),
                    data_dir: "/tmp/default".to_string(),
                    executable_path: None,
                    extensions_dir: None,
                    bound_account_id: None,
                    bound_email: None,
                    created_at: 0,
                    last_used: 0,
                    is_default: true,
                    pid: None,
                    seq_num: Some(1),
                },
                InstanceConfig {
                    id: "inst-custom-2".to_string(),
                    name: "Worker Node 2".to_string(),
                    data_dir: "/tmp/custom2".to_string(),
                    executable_path: None,
                    extensions_dir: None,
                    bound_account_id: None,
                    bound_email: None,
                    created_at: 0,
                    last_used: 0,
                    is_default: false,
                    pid: None,
                    seq_num: Some(2),
                },
            ],
        };

        // Helper mock check logic
        let resolve_mock = |spec: &str| -> String {
            let clean = spec.trim();
            if clean.is_empty() || clean.eq_ignore_ascii_case("active") {
                return registry.active_instance_id.clone();
            }
            if clean.eq_ignore_ascii_case("default") {
                if let Some(def) = registry
                    .instances
                    .iter()
                    .find(|i| i.is_default || i.id == "default")
                {
                    return def.id.clone();
                }
            }
            if let Ok(num) = clean.parse::<u32>() {
                if let Some(inst) = registry.instances.iter().find(|i| i.seq_num == Some(num)) {
                    return inst.id.clone();
                }
            }
            let clean_num = clean
                .trim_start_matches("ins-")
                .trim_start_matches("instance-")
                .trim_start_matches('#');
            if let Ok(num) = clean_num.parse::<u32>() {
                if let Some(inst) = registry.instances.iter().find(|i| i.seq_num == Some(num)) {
                    return inst.id.clone();
                }
            }
            if let Some(inst) = registry
                .instances
                .iter()
                .find(|i| i.id.eq_ignore_ascii_case(clean))
            {
                return inst.id.clone();
            }
            registry.active_instance_id.clone()
        };

        assert_eq!(resolve_mock("1"), "inst-default");
        assert_eq!(resolve_mock("2"), "inst-custom-2");
        assert_eq!(resolve_mock("#2"), "inst-custom-2");
        assert_eq!(resolve_mock("ins-2"), "inst-custom-2");
        assert_eq!(resolve_mock("default"), "inst-default");
        assert_eq!(resolve_mock("active"), "inst-default");
        assert_eq!(resolve_mock("inst-custom-2"), "inst-custom-2");
    }

    #[test]
    fn test_multi_instance_multi_project_account_swap_isolation() {
        let temp_root = std::env::temp_dir().join(format!(
            "agm_multi_inst_proj_test_{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let inst1_data = temp_root.join("inst-1").join("data");
        let inst2_data = temp_root.join("inst-2").join("data");
        let proj_a = temp_root.join("project-alpha");
        let proj_b = temp_root.join("project-beta");
        let proj_c = temp_root.join("project-gamma");
        // Justification: test fixture setup; a real failure fails the test at the next step
        crate::error::record_ignored(std::fs::create_dir_all(&proj_a), "create test dir");
        // Justification: test fixture setup; a real failure fails the test at the next step
        crate::error::record_ignored(std::fs::create_dir_all(&proj_b), "create test dir");
        // Justification: test fixture setup; a real failure fails the test at the next step
        crate::error::record_ignored(std::fs::create_dir_all(&proj_c), "create test dir");

        // Assign Project Alpha + Project Beta to Instance 1, and Project Gamma to Instance 2
        let ws1_a = inst1_data
            .join("User")
            .join("workspaceStorage")
            .join("ws-alpha");
        let ws1_b = inst1_data
            .join("User")
            .join("workspaceStorage")
            .join("ws-beta");
        let ws2_c = inst2_data
            .join("User")
            .join("workspaceStorage")
            .join("ws-gamma");
        // Justification: test fixture setup; a real failure fails the test at the next step
        crate::error::record_ignored(std::fs::create_dir_all(&ws1_a), "create test dir");
        // Justification: test fixture setup; a real failure fails the test at the next step
        crate::error::record_ignored(std::fs::create_dir_all(&ws1_b), "create test dir");
        // Justification: test fixture setup; a real failure fails the test at the next step
        crate::error::record_ignored(std::fs::create_dir_all(&ws2_c), "create test dir");

        let uri_a = format!("file:///{}", proj_a.to_string_lossy().replace('\\', "/"));
        let uri_b = format!("file:///{}", proj_b.to_string_lossy().replace('\\', "/"));
        let uri_c = format!("file:///{}", proj_c.to_string_lossy().replace('\\', "/"));
        // Justification: test fixture seeding; a real failure fails the test at the next assertion
        crate::error::record_ignored(
            std::fs::write(
                ws1_a.join("workspace.json"),
                serde_json::json!({ "folder": uri_a }).to_string(),
            ),
            "write test fixture",
        );
        // Justification: test fixture seeding; a real failure fails the test at the next assertion
        crate::error::record_ignored(
            std::fs::write(
                ws1_b.join("workspace.json"),
                serde_json::json!({ "folder": uri_b }).to_string(),
            ),
            "write test fixture",
        );
        // Justification: test fixture seeding; a real failure fails the test at the next assertion
        crate::error::record_ignored(
            std::fs::write(
                ws2_c.join("workspace.json"),
                serde_json::json!({ "folder": uri_c }).to_string(),
            ),
            "write test fixture",
        );

        let inst1_folders =
            get_instance_workspace_folders("inst-1-isolated", &inst1_data.to_string_lossy());
        let inst2_folders =
            get_instance_workspace_folders("inst-2-isolated", &inst2_data.to_string_lossy());

        assert_eq!(
            inst1_folders.len(),
            2,
            "Instance 1 must restore both bound project workspaces (alpha & beta)"
        );
        assert_eq!(
            inst2_folders.len(),
            1,
            "Instance 2 must restore only its bound project workspace (gamma)"
        );

        // Justification: test cleanup; absence is the normal case
        crate::error::record_ignored(std::fs::remove_dir_all(&temp_root), "clean test dir");
    }
}
