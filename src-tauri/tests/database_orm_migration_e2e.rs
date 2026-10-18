//! End-to-End Integration Tests for Database ORM and Split-DB Migration Strategy.
//! Verifies:
//! 1. Legacy snake_case data export into unified pre-migration snapshot.
//! 2. Zero-Account-Loss verification via SHA-256 hash before and after migration.
//! 3. Fresh PascalCase Split-DB schema initialization at SchemaVersion = 1.
//! 4. Zero-Data-Loss restoration into PascalCase entities.
//! 5. FIFO prompt queue and joined multi-table Views.
//! 6. Compile-time checked Diesel ORM model compatibility.

use antigravity_tools_lib::modules::db_migration::*;
use antigravity_tools_lib::modules::db_orm::diesel_models::*;
use antigravity_tools_lib::modules::db_orm::views::*;
use antigravity_tools_lib::modules::db_orm::*;
use rusqlite::Connection;
use serde_json::json;
use std::fs;
use tempfile::tempdir;

#[test]
fn test_database_orm_migration_e2e_full_lifecycle() {
    let tmp = tempdir().expect("Failed to create tempdir for e2e test");
    let base_dir = tmp.path();

    let backup_dir = base_dir.join("backup-prompts");
    fs::create_dir_all(&backup_dir).unwrap();

    let errors_db_path = base_dir.join("errors.db");
    let repo_prompts_db_path = base_dir.join("repo_prompts.db");
    let backup_prompts_db_path = backup_dir.join("backup-prompts.db");
    let task_history_db_path = base_dir.join("task-history.db");

    // =========================================================================
    // Step 1: Seed Legacy snake_case SQLite Databases
    // =========================================================================
    {
        // 1.1 Legacy errors.db
        let conn = Connection::open(&errors_db_path).unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE errors (
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
            INSERT INTO errors VALUES (
                'err-legacy-1', 'E1001', 'error', 'Database connection refused',
                'at legacy_conn:42', 'proxy', 'http_request', '{"client":"gemini"}', 1740000000
            );
            INSERT INTO errors VALUES (
                'err-legacy-2', 'E2002', 'warn', 'Token refresh slow',
                NULL, 'oauth', 'token_refresh', NULL, 1740000050
            );
            "#,
        )
        .unwrap();
    }

    {
        // 1.2 Legacy repo_prompts.db
        let conn = Connection::open(&repo_prompts_db_path).unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE running_projects (
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
                running_project_id TEXT NOT NULL,
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
            INSERT INTO running_projects VALUES (
                'proj-legacy-1', 'default', 'antigravity-manager', 'd:/work/antigravity-manager',
                'C:/storage/agm', 1, 1740000000, 1740000000
            );
            -- Insert prompt 2 earlier than prompt 1 to verify FIFO sorting later
            INSERT INTO active_prompts VALUES (
                'prompt-legacy-2', 'proj-legacy-1', 'default', 'd:/work/antigravity-manager',
                'Second queued prompt', 'gemini-2.5-pro', 'sess-2', 'queued', 1740000200, 1740000200, NULL
            );
            INSERT INTO active_prompts VALUES (
                'prompt-legacy-1', 'proj-legacy-1', 'default', 'd:/work/antigravity-manager',
                'First queued prompt', 'gemini-2.5-flash', 'sess-1', 'queued', 1740000100, 1740000100, NULL
            );
            "#,
        )
        .unwrap();
    }

    {
        // 1.3 Legacy backup-prompts.db
        let conn = Connection::open(&backup_prompts_db_path).unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE backup_batches (
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
                original_prompt_id TEXT NOT NULL,
                project_name TEXT NOT NULL,
                project_path TEXT NOT NULL,
                running_project_id TEXT NOT NULL,
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
                instance_id TEXT NOT NULL DEFAULT 'default'
            );
            INSERT INTO backup_batches VALUES (
                'batch-legacy-1', 1740000000, 1, 'backup_2026.json', 1, 0
            );
            INSERT INTO prompt_backups VALUES (
                'pb-legacy-1', 'batch-legacy-1', 'prompt-legacy-1', 'antigravity-manager',
                'd:/work/antigravity-manager', 'proj-legacy-1', 'conv-1', 'E2E Task', 1,
                'First queued prompt', 0, NULL, 'queued', 1740000100, 0, NULL, 'default'
            );
            "#,
        )
        .unwrap();
    }

    {
        // 1.4 Legacy task-history.db
        let conn = Connection::open(&task_history_db_path).unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE task_history (
                id TEXT PRIMARY KEY,
                action TEXT NOT NULL,
                action_code INTEGER,
                status TEXT NOT NULL,
                subject TEXT NOT NULL,
                detail TEXT DEFAULT '',
                instance_id TEXT DEFAULT 'default',
                split_path TEXT DEFAULT '',
                from_email TEXT DEFAULT '',
                to_email TEXT DEFAULT '',
                created_at INTEGER NOT NULL,
                finished_at INTEGER
            );
            INSERT INTO task_history VALUES (
                'th-legacy-1', 'switch_account', 101, 'completed', 'Auto switch account',
                'Switched to profile 2', 'default', '', 'sys@agm.local', 'user@agm.local',
                1740000000, 1740000005
            );
            "#,
        )
        .unwrap();
    }

    // =========================================================================
    // Step 2: Seed accounts.json (Zero-Account-Loss Safeguard)
    // =========================================================================
    let accounts_path = base_dir.join("accounts.json");
    let sample_accounts = json!([
        {
            "id": "acc-prod-01",
            "email": "user1@example.com",
            "tier": "pro",
            "created_at": 1740000000
        },
        {
            "id": "acc-prod-02",
            "email": "user2@example.com",
            "tier": "enterprise",
            "created_at": 1740000100
        }
    ]);
    fs::write(
        &accounts_path,
        serde_json::to_string_pretty(&sample_accounts).unwrap(),
    )
    .unwrap();

    // Verify initial account snapshot capture
    let pre_acc_snap = capture_accounts_snapshot_in(Some(base_dir)).unwrap();
    assert_eq!(pre_acc_snap.account_count, 2);
    assert_eq!(
        pre_acc_snap.emails,
        vec!["user1@example.com", "user2@example.com"]
    );
    assert!(!pre_acc_snap.sha256.is_empty());

    // =========================================================================
    // Step 3: Export Single-File Pre-Migration Snapshot
    // =========================================================================
    let snapshot_file = base_dir.join("pre_migration_v4_snapshot.json");
    let export_result = export_pre_migration_snapshot_in(Some(&snapshot_file), Some(base_dir));
    assert!(
        export_result.is_ok(),
        "Snapshot export failed: {:?}",
        export_result.err()
    );

    let snapshot_path = export_result.unwrap();
    let snapshot_raw = fs::read(&snapshot_path).unwrap();
    let snapshot: PreMigrationSnapshot = serde_json::from_slice(&snapshot_raw).unwrap();

    assert_eq!(snapshot.accounts_verification.account_count, 2);
    assert_eq!(
        snapshot
            .databases
            .get("errors.db")
            .unwrap()
            .get("errors")
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        snapshot
            .databases
            .get("repo_prompts.db")
            .unwrap()
            .get("running_projects")
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        snapshot
            .databases
            .get("repo_prompts.db")
            .unwrap()
            .get("active_prompts")
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        snapshot
            .databases
            .get("prompt-backups.db")
            .unwrap()
            .get("backup_batches")
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        snapshot
            .databases
            .get("prompt-backups.db")
            .unwrap()
            .get("prompt_backups")
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        snapshot
            .databases
            .get("task-history.db")
            .unwrap()
            .get("task_history")
            .unwrap()
            .len(),
        1
    );

    // =========================================================================
    // Step 4: Remove legacy DBs so restore creates clean fresh PascalCase schemas
    // =========================================================================
    let _ = fs::remove_file(&errors_db_path);
    let _ = fs::remove_file(&repo_prompts_db_path);
    let _ = fs::remove_file(&backup_prompts_db_path);
    let _ = fs::remove_file(&task_history_db_path);

    // =========================================================================
    // Step 5: Lossless Data Restoration into PascalCase Schemas
    // =========================================================================
    let restore_result = restore_from_snapshot(&snapshot_file, Some(base_dir));
    assert!(
        restore_result.is_ok(),
        "Restore failed: {:?}",
        restore_result.err()
    );

    let summary = restore_result.unwrap();
    assert_eq!(summary.total_imported, 7);

    // =========================================================================
    // Step 6: Verify Zero Account Loss
    // =========================================================================
    let acc_check = verify_zero_account_loss_in(&snapshot.accounts_verification, Some(base_dir));
    assert!(
        acc_check.is_ok(),
        "Account verification must pass with 100% SHA-256 equivalence"
    );

    // =========================================================================
    // Step 7: Verify SQLite Views & FIFO Ordering
    // =========================================================================
    {
        // 7.1 Verify VwRecentErrors on errors.db
        let conn = Connection::open(&errors_db_path).unwrap();
        let errors = ViewQueryBuilder::<VwRecentErrors>::select()
            .order_by("CreatedAt", Order::Asc)
            .fetch_all(&conn)
            .expect("Fetch VwRecentErrors must succeed");
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].error_code, "E1001");
        assert_eq!(errors[1].error_code, "E2002");
    }

    {
        // 7.2 Verify VwActivePromptsWithProjects and VwQueuedPromptsFifo on repo-prompts.db
        let target_repo_db = base_dir.join("repo-prompts.db");
        let conn = Connection::open(&target_repo_db).unwrap();

        // Joined View
        let joined = ViewQueryBuilder::<VwActivePromptsWithProjects>::select()
            .fetch_all(&conn)
            .expect("Fetch VwActivePromptsWithProjects must succeed");
        assert_eq!(joined.len(), 2);
        assert_eq!(joined[0].repo_name.as_deref(), Some("antigravity-manager"));
        assert_eq!(joined[0].is_project_running, Some(true));

        // Strict FIFO View: prompt-legacy-1 (created at 1740000100) MUST come before prompt-legacy-2 (1740000200)
        let fifo = ViewQueryBuilder::<VwQueuedPromptsFifo>::select()
            .fetch_all(&conn)
            .expect("Fetch VwQueuedPromptsFifo must succeed");
        assert_eq!(fifo.len(), 2);
        assert_eq!(fifo[0].active_prompt_id, "prompt-legacy-1");
        assert_eq!(fifo[1].active_prompt_id, "prompt-legacy-2");
        assert_eq!(fifo[0].prompt_content, "First queued prompt");
        assert_eq!(fifo[1].prompt_content, "Second queued prompt");
    }

    {
        // 7.3 Verify VwUnrestoredBackups on prompt-backups.db
        let target_backup_db = base_dir.join("backup-prompts").join("prompt-backups.db");
        let conn = Connection::open(&target_backup_db).unwrap();
        let backups = ViewQueryBuilder::<VwUnrestoredBackups>::select()
            .fetch_all(&conn)
            .expect("Fetch VwUnrestoredBackups must succeed");
        assert_eq!(backups.len(), 1);
        assert_eq!(backups[0].prompt_text, "First queued prompt");
        assert_eq!(backups[0].batch_file_path, "backup_2026.json");
    }

    {
        // 7.4 Verify VwRecentTaskHistory on task-history.db
        let target_th_db = base_dir.join("task-history.db");
        let conn = Connection::open(&target_th_db).unwrap();
        let history = ViewQueryBuilder::<VwRecentTaskHistory>::select()
            .fetch_all(&conn)
            .expect("Fetch VwRecentTaskHistory must succeed");
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].action, "switch_account");
        assert_eq!(history[0].action_code, Some(101));
    }

    // =========================================================================
    // Step 8: Verify Diesel Compile-Time Model Bidirectional Conversion
    // =========================================================================
    {
        let original_error = ErrorLog {
            error_log_id: "err-diesel-test".to_string(),
            error_code: "E9001".to_string(),
            severity: "critical".to_string(),
            message: "Diesel compile-time type checked".to_string(),
            stack_trace: None,
            source: Some("diesel".to_string()),
            trigger_action: None,
            metadata_json: None,
            created_at: 1740000999,
        };

        let diesel_model: DieselErrorLog = original_error.clone().into();
        assert_eq!(diesel_model.error_log_id, "err-diesel-test");
        assert_eq!(diesel_model.error_code, "E9001");

        let converted_back: ErrorLog = diesel_model.into();
        assert_eq!(converted_back, original_error);
    }
}
