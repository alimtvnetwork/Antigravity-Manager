//! Unit tests for the type-safe Rust ORM layer.

use super::*;
use rusqlite::Connection;

#[test]
fn test_error_log_orm_crud_and_view() {
    let conn = Connection::open_in_memory().unwrap();
    init_split_db_schema(&conn, "errors.db").unwrap();

    let err = ErrorLog {
        error_log_id: "err-test-001".to_string(),
        error_code: "E1001".to_string(),
        severity: "error".to_string(),
        message: "Test database failure".to_string(),
        stack_trace: Some("at test_error_log_orm:line 10".to_string()),
        source: Some("unit-test".to_string()),
        trigger_action: Some("test_run".to_string()),
        metadata_json: Some(r#"{"env": "test"}"#.to_string()),
        created_at: 1740000000,
    };

    // 1. Insert entity
    QueryBuilder::insert(&conn, &err).expect("Insert must succeed");

    // 2. Count records
    let count = QueryBuilder::<ErrorLog>::count(&conn).unwrap();
    assert_eq!(count, 1);

    // 3. Find by ID
    let found = QueryBuilder::<ErrorLog>::find_by_id(&conn, &"err-test-001".to_string())
        .expect("Find by ID should succeed")
        .expect("Entity must be found");
    assert_eq!(found.error_code, "E1001");
    assert_eq!(found.message, "Test database failure");

    // 4. Update entity
    let mut updated = found.clone();
    updated.message = "Updated error message".to_string();
    QueryBuilder::update(&conn, &updated).expect("Update must succeed");

    let re_read = QueryBuilder::<ErrorLog>::find_by_id(&conn, &"err-test-001".to_string())
        .unwrap()
        .unwrap();
    assert_eq!(re_read.message, "Updated error message");

    // 5. Query via View
    let views = ViewQueryBuilder::<VwRecentErrors>::select()
        .order_by("CreatedAt", Order::Desc)
        .fetch_all(&conn)
        .expect("View fetch must succeed");
    assert_eq!(views.len(), 1);
    assert_eq!(views[0].error_code, "E1001");
    assert_eq!(views[0].message, "Updated error message");

    // 6. Delete entity
    let deleted =
        QueryBuilder::<ErrorLog>::delete_by_id(&conn, &"err-test-001".to_string()).unwrap();
    assert!(deleted);
    let after_delete =
        QueryBuilder::<ErrorLog>::find_by_id(&conn, &"err-test-001".to_string()).unwrap();
    assert!(after_delete.is_none());
}

#[test]
fn test_running_projects_and_active_prompts_joined_view() {
    let conn = Connection::open_in_memory().unwrap();
    init_split_db_schema(&conn, "repo-prompts.db").unwrap();

    let project = RunningProject {
        running_project_id: "proj-1".to_string(),
        instance_id: "default".to_string(),
        repo_name: "Antigravity-Manager".to_string(),
        repo_path: "d:/work/Antigravity-Manager".to_string(),
        workspace_storage_path: Some("C:/workspaces/agm".to_string()),
        is_running: true,
        last_detected_at: 1740000000,
        updated_at: 1740000000,
    };
    QueryBuilder::insert(&conn, &project).unwrap();

    let prompt = ActivePrompt {
        active_prompt_id: "prompt-1".to_string(),
        running_project_id: "proj-1".to_string(),
        instance_id: "default".to_string(),
        repo_path: "d:/work/Antigravity-Manager".to_string(),
        prompt_content: "Execute ORM migration".to_string(),
        model: Some("gemini-2.5-pro".to_string()),
        session_id: Some("sess-1".to_string()),
        status: "queued".to_string(),
        created_at: 1740000100,
        updated_at: 1740000100,
        image_payload: None,
    };
    QueryBuilder::insert(&conn, &prompt).unwrap();

    // Query pre-defined view VwActivePromptsWithProjects
    let list = ViewQueryBuilder::<VwActivePromptsWithProjects>::select()
        .fetch_all(&conn)
        .expect("Fetch view must succeed");
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].repo_name.as_deref(), Some("Antigravity-Manager"));
    assert_eq!(list[0].prompt_content, "Execute ORM migration");
    assert_eq!(list[0].is_project_running, Some(true));

    // Query FIFO queued view
    let fifo = ViewQueryBuilder::<VwQueuedPromptsFifo>::select()
        .fetch_all(&conn)
        .expect("Fetch FIFO view must succeed");
    assert_eq!(fifo.len(), 1);
    assert_eq!(fifo[0].active_prompt_id, "prompt-1");
}

#[test]
fn test_prompt_backup_and_batch_views() {
    let conn = Connection::open_in_memory().unwrap();
    init_split_db_schema(&conn, "prompt-backups.db").unwrap();

    let batch = BackupBatch {
        backup_batch_id: "batch-1".to_string(),
        created_at: 1740000000,
        prompts_count: 1,
        file_path: "backup_01.json".to_string(),
        retention_days: 1,
        is_fully_restored: false,
    };
    QueryBuilder::insert(&conn, &batch).unwrap();

    let backup = PromptBackup {
        prompt_backup_id: "pb-1".to_string(),
        backup_batch_id: "batch-1".to_string(),
        original_prompt_id: "prompt-1".to_string(),
        project_name: "Antigravity-Manager".to_string(),
        project_path: "d:/work/Antigravity-Manager".to_string(),
        running_project_id: "proj-1".to_string(),
        conversation_id: "conv-1".to_string(),
        conversation_name: Some("Main Task".to_string()),
        sequence_id: 1,
        prompt_text: "Migrate DB to PascalCase".to_string(),
        has_images: false,
        images_payload: None,
        status: "queued".to_string(),
        created_at: 1740000200,
        is_restored: false,
        restored_at: None,
        instance_id: "default".to_string(),
    };
    QueryBuilder::insert(&conn, &backup).unwrap();

    // Query unrestored view
    let unrestored = ViewQueryBuilder::<VwUnrestoredBackups>::select()
        .fetch_all(&conn)
        .unwrap();
    assert_eq!(unrestored.len(), 1);
    assert_eq!(unrestored[0].prompt_text, "Migrate DB to PascalCase");
    assert_eq!(unrestored[0].has_images, false);

    // Query latest batches view
    let batches = ViewQueryBuilder::<VwLatestBackupBatches>::select()
        .fetch_all(&conn)
        .unwrap();
    assert_eq!(batches.len(), 1);
    assert_eq!(batches[0].total_backups_count, 1);
    assert_eq!(batches[0].restored_count, 0);
}
