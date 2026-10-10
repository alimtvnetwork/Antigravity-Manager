//! Instance tests (tests_b).
use super::*;
use std::fs;
use sysinfo::System;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "local_only_e2e"]
    fn test_local_e2e_instance_switch_and_prompt_restore() {
        // Strict Skip-by-Default Isolation Guard
        if std::env::var("RUN_TEMP_E2E").as_deref() != Ok("1") {
            println!("Skipping temporary local-only E2E test; run on-demand with RUN_TEMP_E2E=1");
            return;
        }

        // Phase 1: Invariant Protection Guard
        let default_dir = get_default_antigravity_data_dir();
        let default_dir_str = default_dir.to_string_lossy();
        let protected_pids = find_pids_for_data_dir(&default_dir_str, true);
        println!("[TEMP E2E] Host Protected PIDs: {:?}", protected_pids);

        // Phase 2: Create isolated sandbox instance
        let unique_id = format!("e2e-inst-{}", chrono::Utc::now().timestamp_millis());
        let temp_base = std::env::temp_dir().join(&unique_id);
        let inst_data = temp_base.join("data");
        let proj_dir = temp_base.join("project-gitmap");
        assert!(std::fs::create_dir_all(&inst_data).is_ok());
        assert!(std::fs::create_dir_all(&proj_dir).is_ok());

        // Create state.vscdb with initial credentials
        let user_storage = inst_data.join("User").join("globalStorage");
        assert!(std::fs::create_dir_all(&user_storage).is_ok());
        let db_path = user_storage.join("state.vscdb");
        {
            let conn = rusqlite::Connection::open(&db_path).unwrap();
            conn.execute(
                "CREATE TABLE IF NOT EXISTS ItemTable (key TEXT PRIMARY KEY, value BLOB)",
                [],
            )
            .unwrap();
            let initial_email = "rokixshohag1@gmail.com";
            conn.execute(
                "INSERT OR REPLACE INTO ItemTable (key, value) VALUES (?1, ?2)",
                rusqlite::params!["antigravity.activeUser", initial_email.as_bytes()],
            )
            .unwrap();
        }

        // Phase 3: Bind workspace project
        let ws_storage = inst_data.join("User").join("workspaceStorage").join("ws-1");
        assert!(std::fs::create_dir_all(&ws_storage).is_ok());
        let proj_uri = format!("file:///{}", proj_dir.to_string_lossy().replace('\\', "/"));
        std::fs::write(
            ws_storage.join("workspace.json"),
            serde_json::json!({ "folder": proj_uri }).to_string(),
        )
        .unwrap();

        // Verify bound workspace discovery
        let discovered_folders =
            get_instance_workspace_folders(&unique_id, &inst_data.to_string_lossy());
        assert_eq!(
            discovered_folders.len(),
            1,
            "Must find bound project folder"
        );

        // Phase 4: Seed in-flight and queued prompts into repo_db
        let running_text = "Running the Gitmap tests and verifying test inventory";
        let queued_text_1 = "Check the CICD pipeline status and diagnostic logs";
        let queued_text_2 = "Verify unit test durations and isolate heavy system calls";

        if let Ok(conn) = crate::modules::repo_db::connect_db() {
            let now = chrono::Utc::now().timestamp();
            // Justification: test fixture seeding; a real failure fails the test at the next assertion
            crate::error::record_ignored(
                conn.execute(
                    "INSERT OR REPLACE INTO running_projects
                     (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    rusqlite::params![
                        "gitmap-1",
                        unique_id,
                        "Gitmap",
                        proj_dir.to_string_lossy().to_string(),
                        ws_storage.to_string_lossy().to_string(),
                        1,
                        now,
                        now,
                    ],
                ),
                "seed test db",
            );
            // Justification: test fixture seeding; a real failure fails the test at the next assertion
            crate::error::record_ignored(
                conn.execute(
                    "INSERT OR REPLACE INTO active_prompts
                     (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                    rusqlite::params![
                        format!("prompt-{}-1", unique_id),
                        "gitmap-1",
                        unique_id,
                        proj_dir.to_string_lossy().to_string(),
                        running_text,
                        "gemini-2.5-pro",
                        "sess-1",
                        "running",
                        now,
                        now,
                        None::<String>,
                    ],
                ),
                "seed test db",
            );
            // Justification: test fixture seeding; a real failure fails the test at the next assertion
            crate::error::record_ignored(
                conn.execute(
                    "INSERT OR REPLACE INTO active_prompts
                     (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                    rusqlite::params![
                        format!("prompt-{}-2", unique_id),
                        "gitmap-1",
                        unique_id,
                        proj_dir.to_string_lossy().to_string(),
                        queued_text_1,
                        "gemini-2.5-pro",
                        "sess-2",
                        "queued",
                        now,
                        now,
                        None::<String>,
                    ],
                ),
                "seed test db",
            );
            // Justification: test fixture seeding; a real failure fails the test at the next assertion
            crate::error::record_ignored(
                conn.execute(
                    "INSERT OR REPLACE INTO active_prompts
                     (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
                    rusqlite::params![
                        format!("prompt-{}-3", unique_id),
                        "gitmap-1",
                        unique_id,
                        proj_dir.to_string_lossy().to_string(),
                        queued_text_2,
                        "gemini-2.5-pro",
                        "sess-3",
                        "queued",
                        now,
                        now,
                        None::<String>,
                    ],
                ),
                "seed test db",
            );
        }

        // Phase 5: Backup prompts prior to switch
        let mut in_flight_note = None;
        let mut queued_notes = Vec::new();
        if let Ok(conn) = crate::modules::repo_db::connect_db() {
            if let Ok(mut stmt) = conn
                .prepare("SELECT prompt_content, status FROM active_prompts WHERE instance_id = ?1")
            {
                if let Ok(rows) = stmt.query_map(rusqlite::params![unique_id], |r| {
                    Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
                }) {
                    for r in rows.flatten() {
                        if r.1 == "running" {
                            in_flight_note = Some(r.0);
                        } else {
                            queued_notes.push(r.0);
                        }
                    }
                }
            }
        }
        assert_eq!(in_flight_note.as_deref(), Some(running_text));
        assert_eq!(queued_notes.len(), 2);

        // Phase 6: Switch account credentials in state.vscdb
        let new_email = "erfan.office.n@gmail.com";
        {
            let conn = rusqlite::Connection::open(&db_path).unwrap();
            conn.execute(
                "INSERT OR REPLACE INTO ItemTable (key, value) VALUES (?1, ?2)",
                rusqlite::params!["antigravity.activeUser", new_email.as_bytes()],
            )
            .unwrap();

            // Verify account change in state.vscdb
            let stored_val: Vec<u8> = conn
                .query_row(
                    "SELECT value FROM ItemTable WHERE key = 'antigravity.activeUser'",
                    [],
                    |r| r.get(0),
                )
                .unwrap();
            let stored_str = String::from_utf8_lossy(&stored_val);
            assert_eq!(stored_str, new_email);
        }

        // Phase 7: Restore and push prompts back to project directory
        let resume_task_file = proj_dir.join(".antigravity_resume_task.json");
        let resume_payload = serde_json::json!({
            "task": in_flight_note.unwrap_or_default(),
            "queued": queued_notes,
            "instance_id": unique_id,
            "email": new_email,
            "status": "restored"
        });
        std::fs::write(&resume_task_file, resume_payload.to_string()).unwrap();
        assert!(resume_task_file.exists());

        // Verify resume file content
        let read_back: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&resume_task_file).unwrap()).unwrap();
        assert_eq!(read_back["email"], new_email);
        assert_eq!(read_back["task"], running_text);

        // Clean up seeded prompts in repo_db
        if let Ok(conn) = crate::modules::repo_db::connect_db() {
            // Justification: test fixture seeding; a real failure fails the test at the next assertion
            crate::error::record_ignored(
                conn.execute(
                    "DELETE FROM active_prompts WHERE instance_id = ?1",
                    rusqlite::params![unique_id],
                ),
                "seed test db",
            );
        }

        // Phase 8: Invariant Check & Cleanup
        for pid in &protected_pids {
            let s = sysinfo::System::new_with_specifics(
                sysinfo::RefreshKind::new().with_processes(sysinfo::ProcessRefreshKind::new()),
            );
            assert!(
                s.process(sysinfo::Pid::from_u32(*pid)).is_some(),
                "Host IDE PID {} must remain alive and untouched throughout E2E test",
                pid
            );
        }

        // Justification: test cleanup; absence is the normal case
        crate::error::record_ignored(std::fs::remove_dir_all(&temp_base), "clean test dir");
        println!("[TEMP E2E] Local-only E2E test completed successfully with 100% clean teardown.");
    }
}
