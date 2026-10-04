//! Integration Tests for Instance Cloning, Settings Synchronization, and Security Presets
//! Run manually with: cargo test --test instance_cloning_and_sync_test -- --ignored

use antigravity_tools_lib::modules::db::sanitize_session;
use antigravity_tools_lib::modules::instance::{
    copy_dir_recursive, safe_clone_sqlite_db, GEMINI_CLONE_DIRS, REQUIRED_IDE_REL_PATHS,
};
use rusqlite::Connection;
use std::fs;
use tempfile::tempdir;

#[test]
#[ignore = "local-only e2e test, run manually via cargo test --test instance_cloning_and_sync_test -- --ignored"]
fn test_required_ide_rel_paths_and_gemini_clone_dirs_manifests() {
    assert!(REQUIRED_IDE_REL_PATHS.contains(&"User/settings.json"));
    assert!(REQUIRED_IDE_REL_PATHS.contains(&"User/keybindings.json"));
    assert!(REQUIRED_IDE_REL_PATHS.contains(&"User/security_presets.json"));
    assert!(REQUIRED_IDE_REL_PATHS.contains(&"User/antigravity_policies.json"));
    assert!(REQUIRED_IDE_REL_PATHS.contains(&"User/snippets"));
    assert!(REQUIRED_IDE_REL_PATHS.contains(&"User/globalStorage"));
    assert!(REQUIRED_IDE_REL_PATHS.contains(&"User/workspaceStorage"));

    assert!(GEMINI_CLONE_DIRS.contains(&"antigravity"));
    assert!(GEMINI_CLONE_DIRS.contains(&"antigravity-ide"));
    assert!(GEMINI_CLONE_DIRS.contains(&"antigravity-cli"));
    assert!(GEMINI_CLONE_DIRS.contains(&"policies"));
    assert!(GEMINI_CLONE_DIRS.contains(&"config"));
}

#[test]
#[ignore = "local-only e2e test, run manually via cargo test --test instance_cloning_and_sync_test -- --ignored"]
fn test_safe_clone_sqlite_db_and_sanitize_session_preserves_enterprise_prefs() {
    let temp = tempdir().expect("Failed to create tempdir");
    let src_db = temp.path().join("src_state.vscdb");
    let dst_db = temp.path().join("dst_state.vscdb");

    // Initialize source sqlite database with WAL mode and test records
    {
        let conn = Connection::open(&src_db).expect("Failed to open src_db");
        let _ = conn.pragma_update(None, "journal_mode", "WAL");
        conn.execute(
            "CREATE TABLE ItemTable (key TEXT UNIQUE ON CONFLICT REPLACE, value BLOB)",
            [],
        )
        .expect("Failed to create ItemTable");

        conn.execute(
            "INSERT INTO ItemTable (key, value) VALUES (?1, ?2)",
            [
                "antigravityUnifiedStateSync.oauthToken",
                "test-oauth-token-val",
            ],
        )
        .expect("Failed to insert oauthToken");

        conn.execute(
            "INSERT INTO ItemTable (key, value) VALUES (?1, ?2)",
            ["antigravityUnifiedStateSync.userStatus", "test-user-status"],
        )
        .expect("Failed to insert userStatus");

        conn.execute(
            "INSERT INTO ItemTable (key, value) VALUES (?1, ?2)",
            [
                "antigravityUnifiedStateSync.enterprisePreferences",
                "test-enterprise-preference-val",
            ],
        )
        .expect("Failed to insert enterprisePreferences");

        conn.execute(
            "INSERT INTO ItemTable (key, value) VALUES (?1, ?2)",
            ["workbench.colorTheme", "Antigravity Dark"],
        )
        .expect("Failed to insert colorTheme");
    }

    // Safely clone the sqlite database
    let clone_result = safe_clone_sqlite_db(&src_db, &dst_db);
    assert!(
        clone_result.is_ok(),
        "safe_clone_sqlite_db failed: {:?}",
        clone_result
    );
    assert!(dst_db.exists(), "Destination DB file was not created");

    // Verify cloned database contains all records
    {
        let conn_dst = Connection::open(&dst_db).expect("Failed to open dst_db");
        let theme: String = conn_dst
            .query_row(
                "SELECT value FROM ItemTable WHERE key = 'workbench.colorTheme'",
                [],
                |row| row.get(0),
            )
            .expect("Theme not found in cloned DB");
        assert_eq!(theme, "Antigravity Dark");
    }

    // Sanitize destination session and verify enterprise preferences are preserved
    let sanitize_res = sanitize_session(&dst_db);
    assert!(
        sanitize_res.is_ok(),
        "sanitize_session failed: {:?}",
        sanitize_res
    );

    {
        let conn_dst = Connection::open(&dst_db).expect("Failed to open dst_db after sanitize");
        let oauth: Result<String, _> = conn_dst.query_row(
            "SELECT value FROM ItemTable WHERE key = 'antigravityUnifiedStateSync.oauthToken'",
            [],
            |row| row.get(0),
        );
        assert!(
            oauth.is_err(),
            "oauthToken should have been deleted by sanitize_session"
        );

        let user_status: Result<String, _> = conn_dst.query_row(
            "SELECT value FROM ItemTable WHERE key = 'antigravityUnifiedStateSync.userStatus'",
            [],
            |row| row.get(0),
        );
        assert!(
            user_status.is_err(),
            "userStatus should have been deleted by sanitize_session"
        );

        let enterprise_prefs: String = conn_dst
            .query_row(
                "SELECT value FROM ItemTable WHERE key = 'antigravityUnifiedStateSync.enterprisePreferences'",
                [],
                |row| row.get(0),
            )
            .expect("enterprisePreferences MUST be preserved by sanitize_session");
        assert_eq!(enterprise_prefs, "test-enterprise-preference-val");
    }
}

#[test]
#[ignore = "local-only e2e test, run manually via cargo test --test instance_cloning_and_sync_test -- --ignored"]
fn test_copy_instance_copies_settings_keybindings_presets_and_workspaces() {
    let temp = tempdir().expect("Failed to create tempdir");
    let src_data = temp.path().join("source_inst").join("data");
    let dst_data = temp.path().join("dest_inst").join("data");

    // 1. Setup Source directory tree
    let src_user = src_data.join("User");
    fs::create_dir_all(&src_user).expect("Failed to create src_user");

    // Settings
    fs::write(
        src_user.join("settings.json"),
        r#"{"workbench.colorTheme": "Default Dark Modern", "antigravity.turboMode": true}"#,
    )
    .expect("Failed to write settings.json");

    // Keybindings
    fs::write(
        src_user.join("keybindings.json"),
        r#"[{"key": "ctrl+k ctrl+s", "command": "workbench.action.openGlobalKeybindings"}]"#,
    )
    .expect("Failed to write keybindings.json");

    // Security presets
    fs::write(
        src_user.join("security_presets.json"),
        r#"{"strictMode": true, "allowInsecureCertificates": false}"#,
    )
    .expect("Failed to write security_presets.json");

    // Antigravity policies
    fs::write(
        src_user.join("antigravity_policies.json"),
        r#"{"enforceLocalExecution": true, "maxBatchSize": 8}"#,
    )
    .expect("Failed to write antigravity_policies.json");

    // Snippets
    let src_snippets = src_user.join("snippets");
    fs::create_dir_all(&src_snippets).expect("Failed to create src_snippets");
    fs::write(
        src_snippets.join("rust.json"),
        r#"{"fn": {"prefix": "fn", "body": ["fn $1() {", "\t$0", "}"]}}"#,
    )
    .expect("Failed to write snippets rust.json");

    // Workspace storage
    let src_ws = src_user.join("workspaceStorage").join("ws_main_app");
    fs::create_dir_all(&src_ws).expect("Failed to create src_ws");
    fs::write(
        src_ws.join("workspace.json"),
        r#"{"folder": "file:///d:/work/Antigravity-Manager"}"#,
    )
    .expect("Failed to write workspace.json");

    // Global storage state.vscdb
    let src_gs = src_user.join("globalStorage");
    fs::create_dir_all(&src_gs).expect("Failed to create src_gs");
    let src_db = src_gs.join("state.vscdb");
    {
        let conn = Connection::open(&src_db).expect("Failed to open state.vscdb");
        conn.execute(
            "CREATE TABLE ItemTable (key TEXT UNIQUE ON CONFLICT REPLACE, value BLOB)",
            [],
        )
        .expect("Failed to create table");
        conn.execute(
            "INSERT INTO ItemTable (key, value) VALUES (?1, ?2)",
            ["workbench.colorTheme", "Default Dark Modern"],
        )
        .expect("Failed to insert theme");
    }

    // 2. Perform resilient recursive copy
    let copy_res = copy_dir_recursive(&src_data, &dst_data);
    assert!(
        copy_res.is_ok(),
        "copy_dir_recursive failed: {:?}",
        copy_res
    );

    // 3. Verify all critical files exist and match in destination
    let dst_user = dst_data.join("User");

    // Settings
    let settings_path = dst_user.join("settings.json");
    assert!(
        settings_path.exists(),
        "settings.json missing in destination"
    );
    let settings_content = fs::read_to_string(settings_path).expect("Failed to read dst settings");
    assert!(settings_content.contains("Default Dark Modern"));

    // Keybindings
    let keybindings_path = dst_user.join("keybindings.json");
    assert!(
        keybindings_path.exists(),
        "keybindings.json missing in destination"
    );
    let keybindings_content =
        fs::read_to_string(keybindings_path).expect("Failed to read dst keybindings");
    assert!(keybindings_content.contains("openGlobalKeybindings"));

    // Security presets
    let presets_path = dst_user.join("security_presets.json");
    assert!(
        presets_path.exists(),
        "security_presets.json missing in destination"
    );
    let presets_content = fs::read_to_string(presets_path).expect("Failed to read dst presets");
    assert!(presets_content.contains("strictMode"));

    // Antigravity policies
    let policies_path = dst_user.join("antigravity_policies.json");
    assert!(
        policies_path.exists(),
        "antigravity_policies.json missing in destination"
    );
    let policies_content = fs::read_to_string(policies_path).expect("Failed to read dst policies");
    assert!(policies_content.contains("enforceLocalExecution"));

    // Snippets
    let snippet_path = dst_user.join("snippets").join("rust.json");
    assert!(
        snippet_path.exists(),
        "snippets/rust.json missing in destination"
    );

    // Workspace storage
    let ws_path = dst_user
        .join("workspaceStorage")
        .join("ws_main_app")
        .join("workspace.json");
    assert!(ws_path.exists(), "workspace.json missing in destination");
    let ws_content = fs::read_to_string(ws_path).expect("Failed to read dst workspace.json");
    assert!(ws_content.contains("Antigravity-Manager"));

    // SQLite DB in destination
    let dst_db = dst_user.join("globalStorage").join("state.vscdb");
    assert!(dst_db.exists(), "state.vscdb missing in destination");
    let conn_dst = Connection::open(&dst_db).expect("Failed to open dst state.vscdb");
    let theme_val: String = conn_dst
        .query_row(
            "SELECT value FROM ItemTable WHERE key = 'workbench.colorTheme'",
            [],
            |row| row.get(0),
        )
        .expect("Theme query failed in dst DB");
    assert_eq!(theme_val, "Default Dark Modern");
}
