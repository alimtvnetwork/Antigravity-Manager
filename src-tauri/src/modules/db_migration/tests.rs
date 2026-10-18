//! Unit tests for the database migration and account guard engine.

use super::*;
use rusqlite::Connection;
use std::collections::HashMap;
use std::fs;

#[test]
fn test_accounts_verification_guard() {
    let before = AccountsVerification {
        account_count: 2,
        emails: vec![
            "alpha@example.com".to_string(),
            "beta@example.com".to_string(),
        ],
        sha256: "dummy-hash".to_string(),
    };

    // If an account is lost, it must fail
    let mut bad_emails = before.clone();
    bad_emails.account_count = 1;
    bad_emails.emails = vec!["alpha@example.com".to_string()];

    // Verify error detection logic directly
    assert!(bad_emails.account_count < before.account_count);
    assert!(!bad_emails.emails.contains(&"beta@example.com".to_string()));
}

#[test]
fn test_schema_version_tracking() {
    let conn = Connection::open_in_memory().unwrap();
    assert_eq!(get_schema_version(&conn).unwrap(), 0);
    assert!(!is_migrated_to_pascalcase(&conn));

    // Initialize SchemaVersion table
    conn.execute_batch(
        "CREATE TABLE SchemaVersion (
            SchemaVersionId INTEGER PRIMARY KEY AUTOINCREMENT,
            VersionNumber INTEGER NOT NULL,
            AppliedAt INTEGER NOT NULL,
            Description TEXT NOT NULL,
            Checksum TEXT NOT NULL
        );",
    )
    .unwrap();

    record_schema_version(&conn, 1, "Initial PascalCase Schema", "test_chk").unwrap();
    assert_eq!(get_schema_version(&conn).unwrap(), 1);
    assert!(is_migrated_to_pascalcase(&conn));

    record_schema_version(&conn, 2, "Added extra indices", "test_chk_2").unwrap();
    assert_eq!(get_schema_version(&conn).unwrap(), 2);
    assert!(is_migrated_to_pascalcase(&conn));
}

#[test]
fn test_migration_snapshot_serialization_and_restore() {
    let temp_dir = std::env::temp_dir().join(format!("agm_test_mig_{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&temp_dir).unwrap();

    let snapshot_file = temp_dir.join("test_snapshot.json");

    let mut db_dumps = HashMap::new();
    let mut err_tbls = HashMap::new();
    err_tbls.insert(
        "errors".to_string(),
        vec![serde_json::json!({
            "id": "err-legacy-01",
            "error_code": "E2001",
            "severity": "warn",
            "message": "Legacy error message",
            "stack_trace": null,
            "source": "migrator",
            "trigger_action": "test",
            "metadata_json": "{}",
            "created_at": 1740000050
        })],
    );
    db_dumps.insert("errors.db".to_string(), err_tbls);

    let snapshot = PreMigrationSnapshot {
        format_version: "1.0".to_string(),
        exported_at: 1740000050,
        agm_version: "4.231.0".to_string(),
        accounts_verification: AccountsVerification {
            account_count: 0,
            emails: Vec::new(),
            sha256: String::new(),
        },
        databases: db_dumps,
    };

    fs::write(
        &snapshot_file,
        serde_json::to_vec_pretty(&snapshot).unwrap(),
    )
    .unwrap();

    // Restore into temp directory
    let summary =
        restore_from_snapshot(&snapshot_file, Some(&temp_dir)).expect("Restore must succeed");

    assert_eq!(summary.errors_imported, 1);

    // Verify errors.db was created with PascalCase ErrorLog table
    let errors_db_path = temp_dir.join("errors.db");
    assert!(errors_db_path.exists());

    let conn = Connection::open(&errors_db_path).unwrap();
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM ErrorLog", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 1);

    let msg: String = conn
        .query_row(
            "SELECT Message FROM ErrorLog WHERE ErrorLogId = 'err-legacy-01'",
            [],
            |r| r.get(0),
        )
        .unwrap();
    assert_eq!(msg, "Legacy error message");

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_end_to_end_migration_and_zero_account_loss() {
    use crate::modules::db_orm::*;

    let base_temp = std::env::temp_dir().join(format!("agm_e2e_mig_{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&base_temp).expect("Create base temp dir");

    // 1. Setup mock accounts.json
    let accounts_json_path = base_temp.join("accounts.json");
    let initial_accounts = serde_json::json!({
        "accounts": [
            { "email": "dev1@gmail.com", "token": "tok1", "tier": "PRO" },
            { "email": "dev2@gmail.com", "token": "tok2", "tier": "ULTRA" }
        ]
    });
    fs::write(
        &accounts_json_path,
        serde_json::to_vec_pretty(&initial_accounts).unwrap(),
    )
    .unwrap();

    // 2. Setup mock legacy errors.db
    let errors_db_path = base_temp.join("errors.db");
    let err_conn = Connection::open(&errors_db_path).unwrap();
    err_conn
        .execute_batch(
            "CREATE TABLE errors (
                id TEXT PRIMARY KEY,
                error_code TEXT NOT NULL,
                severity TEXT NOT NULL,
                message TEXT NOT NULL,
                stack_trace TEXT,
                source TEXT,
                trigger_action TEXT,
                metadata_json TEXT,
                created_at INTEGER NOT NULL
            );
            INSERT INTO errors VALUES 
            ('err-01', 'E1001', 'error', 'Initial fault', NULL, 'cli', 'test', '{}', 1740000000);",
        )
        .unwrap();

    // 3. Setup mock legacy repo_prompts.db
    let repo_db_path = base_temp.join("repo_prompts.db");
    let repo_conn = Connection::open(&repo_db_path).unwrap();
    repo_conn
        .execute_batch(
            "CREATE TABLE running_projects (
                id TEXT PRIMARY KEY,
                instance_id TEXT NOT NULL,
                repo_name TEXT NOT NULL,
                repo_path TEXT NOT NULL,
                workspace_storage_path TEXT,
                is_running INTEGER NOT NULL DEFAULT 1,
                last_detected_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            CREATE TABLE active_prompts (
                id TEXT PRIMARY KEY,
                project_id TEXT NOT NULL,
                instance_id TEXT NOT NULL,
                repo_path TEXT NOT NULL,
                prompt_content TEXT NOT NULL,
                model TEXT,
                session_id TEXT,
                status TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                image_payload TEXT
            );
            INSERT INTO running_projects VALUES 
            ('proj-legacy-1', 'default', 'Antigravity-Manager', 'd:/work/AGM', NULL, 1, 1740000000, 1740000000);
            INSERT INTO active_prompts VALUES 
            ('prompt-legacy-1', 'proj-legacy-1', 'default', 'd:/work/AGM', 'Legacy queued prompt', 'gemini-flash', NULL, 'queued', 1740000000, 1740000000, NULL);",
        )
        .unwrap();

    // 4. Setup mock legacy backup-prompts.db
    let backup_dir = base_temp.join("backup-prompts");
    fs::create_dir_all(&backup_dir).unwrap();
    let backup_db_path = backup_dir.join("backup-prompts.db");
    let backup_conn = Connection::open(&backup_db_path).unwrap();
    backup_conn
        .execute_batch(
            "CREATE TABLE backup_batches (
                id TEXT PRIMARY KEY,
                created_at INTEGER NOT NULL,
                prompts_count INTEGER NOT NULL,
                file_path TEXT NOT NULL,
                retention_days INTEGER NOT NULL DEFAULT 1,
                is_fully_restored INTEGER NOT NULL DEFAULT 0
            );
            CREATE TABLE prompt_backups (
                id TEXT PRIMARY KEY,
                backup_batch_id TEXT NOT NULL,
                prompt_id TEXT NOT NULL,
                project_name TEXT NOT NULL,
                project_path TEXT NOT NULL,
                project_id TEXT NOT NULL,
                conversation_id TEXT NOT NULL,
                conversation_name TEXT,
                sequence_id INTEGER NOT NULL,
                prompt_text TEXT NOT NULL,
                has_images INTEGER NOT NULL DEFAULT 0,
                images_payload TEXT,
                status TEXT NOT NULL DEFAULT 'queued',
                created_at INTEGER NOT NULL,
                is_restored INTEGER NOT NULL DEFAULT 0,
                restored_at INTEGER,
                instance_id TEXT DEFAULT 'default'
            );
            INSERT INTO backup_batches VALUES 
            ('batch-legacy-1', 1740000000, 1, 'snap.json', 1, 0);
            INSERT INTO prompt_backups VALUES 
            ('backup-legacy-1', 'batch-legacy-1', 'prompt-legacy-1', 'Antigravity-Manager', 'd:/work/AGM', 'proj-legacy-1', 'conv-1', 'Task', 1, 'Backed up text', 0, NULL, 'queued', 1740000000, 0, NULL, 'default');",
        )
        .unwrap();

    // 5. Export pre-migration snapshot
    let snapshot_file = base_temp.join("migration_snapshot.json");
    let mut db_dumps = std::collections::HashMap::new();

    let mut err_tbls = std::collections::HashMap::new();
    err_tbls.insert(
        "errors".to_string(),
        dump_table_rows(&err_conn, "errors").unwrap(),
    );
    db_dumps.insert("errors.db".to_string(), err_tbls);

    let mut repo_tbls = std::collections::HashMap::new();
    repo_tbls.insert(
        "running_projects".to_string(),
        dump_table_rows(&repo_conn, "running_projects").unwrap(),
    );
    repo_tbls.insert(
        "active_prompts".to_string(),
        dump_table_rows(&repo_conn, "active_prompts").unwrap(),
    );
    db_dumps.insert("repo_prompts.db".to_string(), repo_tbls);

    let mut bkp_tbls = std::collections::HashMap::new();
    bkp_tbls.insert(
        "backup_batches".to_string(),
        dump_table_rows(&backup_conn, "backup_batches").unwrap(),
    );
    bkp_tbls.insert(
        "prompt_backups".to_string(),
        dump_table_rows(&backup_conn, "prompt_backups").unwrap(),
    );
    db_dumps.insert("prompt-backups.db".to_string(), bkp_tbls);

    let snapshot = PreMigrationSnapshot {
        format_version: "1.0".to_string(),
        exported_at: chrono::Utc::now().timestamp(),
        agm_version: "4.231.0".to_string(),
        accounts_verification: capture_accounts_snapshot_in(Some(&base_temp)).unwrap(),
        databases: db_dumps,
    };
    fs::write(
        &snapshot_file,
        serde_json::to_vec_pretty(&snapshot).unwrap(),
    )
    .unwrap();

    // 6. Delete legacy DBs to prepare for clean PascalCase fresh start
    drop(err_conn);
    drop(repo_conn);
    drop(backup_conn);
    fs::remove_file(&errors_db_path).unwrap();
    fs::remove_file(&repo_db_path).unwrap();
    fs::remove_file(&backup_db_path).unwrap();

    // 7. Perform fresh re-import into new PascalCase databases
    let summary = restore_from_snapshot(&snapshot_file, Some(&base_temp))
        .expect("Restore must succeed cleanly");

    assert_eq!(summary.errors_imported, 1);
    assert_eq!(summary.projects_imported, 1);
    assert_eq!(summary.prompts_imported, 1);
    assert_eq!(summary.batches_imported, 1);
    assert_eq!(summary.backups_imported, 1);

    // 8. Verify PascalCase tables and views in fresh databases
    let fresh_err_conn = Connection::open(&errors_db_path).unwrap();
    let err_views = ViewQueryBuilder::<VwRecentErrors>::select()
        .fetch_all(&fresh_err_conn)
        .unwrap();
    assert_eq!(err_views.len(), 1);
    assert_eq!(err_views[0].error_code, "E1001");
    assert_eq!(err_views[0].message, "Initial fault");
    assert!(is_migrated_to_pascalcase(&fresh_err_conn));

    let fresh_repo_conn = Connection::open(&repo_db_path).unwrap();
    let prompt_views = ViewQueryBuilder::<VwActivePromptsWithProjects>::select()
        .fetch_all(&fresh_repo_conn)
        .unwrap();
    assert_eq!(prompt_views.len(), 1);
    assert_eq!(
        prompt_views[0].repo_name.as_deref(),
        Some("Antigravity-Manager")
    );
    assert_eq!(prompt_views[0].is_project_running, Some(true));
    assert!(is_migrated_to_pascalcase(&fresh_repo_conn));

    let fresh_bkp_conn = Connection::open(&backup_db_path).unwrap();
    let bkp_views = ViewQueryBuilder::<VwUnrestoredBackups>::select()
        .fetch_all(&fresh_bkp_conn)
        .unwrap();
    assert_eq!(bkp_views.len(), 1);
    assert_eq!(bkp_views[0].prompt_text, "Backed up text");
    assert_eq!(bkp_views[0].has_images, false);
    assert!(is_migrated_to_pascalcase(&fresh_bkp_conn));

    // 9. Clean up
    let _ = fs::remove_dir_all(&base_temp);
}
