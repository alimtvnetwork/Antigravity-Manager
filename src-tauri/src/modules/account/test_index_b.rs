#![cfg(test)]

use super::*;
use std::collections::HashSet;
use std::fs;
use std::sync::Mutex as StdMutex;

use super::*;

#[test]
pub(crate) fn test_load_account_index_with_nul_prefix() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let dir = TestDataDir::new();

    // NUL byte prefix followed by valid JSON
    let nul = [0x00];
    let json = r#"{"version":"2.0","accounts":[],"current_account_id":null}"#;
    let mut content = Vec::new();
    content.extend_from_slice(&nul);
    content.extend_from_slice(json.as_bytes());

    write_corrupted_index(dir.path(), &content);

    let result = load_account_index_in_dir(dir.path());

    // New behavior: NUL bytes are stripped and JSON parses successfully
    assert!(
        result.is_ok(),
        "NUL prefix should be stripped and JSON should parse: {:?}",
        result
    );
    let index = result.unwrap();
    assert!(index.accounts.is_empty());
    println!("NUL prefix case: successfully loaded index after sanitization");
}

#[test]
pub(crate) fn test_load_account_index_with_garbage_content() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let dir = TestDataDir::new();

    // Non-JSON garbage content - should trigger recovery
    write_corrupted_index(dir.path(), b"\0\0not json");

    let result = load_account_index_in_dir(dir.path());

    // New behavior: garbage content triggers recovery, returns empty index
    assert!(
        result.is_ok(),
        "Garbage content should trigger recovery and return Ok: {:?}",
        result
    );
    let index = result.unwrap();
    assert!(
        index.accounts.is_empty(),
        "Recovered index should be empty when no account files exist"
    );
    println!("Garbage content case: successfully recovered to empty index");
}

#[test]
pub(crate) fn test_load_account_index_with_empty_file() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let dir = TestDataDir::new();

    // Empty file
    write_corrupted_index(dir.path(), b"");

    let result = load_account_index_in_dir(dir.path());

    // Current behavior: empty file returns new empty index
    assert!(result.is_ok());
    let index = result.unwrap();
    assert!(index.accounts.is_empty());
}

#[test]
pub(crate) fn test_load_account_index_with_whitespace_only() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let dir = TestDataDir::new();

    // Whitespace-only file
    write_corrupted_index(dir.path(), b"   \n\t  ");

    let result = load_account_index_in_dir(dir.path());

    // Current behavior: whitespace-only file returns new empty index
    assert!(result.is_ok());
    let index = result.unwrap();
    assert!(index.accounts.is_empty());
}

#[test]
pub(crate) fn test_missing_index_with_existing_accounts() {
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let dir = TestDataDir::new();

    // Create accounts directory with account files but NO accounts.json index
    create_account_file(dir.path(), "test-id-1", "user1@example.com");
    create_account_file(dir.path(), "test-id-2", "user2@example.com");

    // accounts.json does not exist
    let index_path = dir.path().join("accounts.json");
    assert!(!index_path.exists());

    // Load account index - should recover from accounts directory
    let result = load_account_index_in_dir(dir.path());
    assert!(result.is_ok(), "Should recover from accounts directory");
    let index = result.unwrap();
    assert_eq!(
        index.accounts.len(),
        2,
        "Index should have 2 accounts recovered from accounts directory"
    );

    // Verify recovered accounts have correct data
    let emails: Vec<_> = index.accounts.iter().map(|s| s.email.clone()).collect();
    assert!(emails.contains(&"user1@example.com".to_string()));
    assert!(emails.contains(&"user2@example.com".to_string()));

    // Verify account files still exist
    let accounts_dir = dir.path().join("accounts");
    let account_files: Vec<_> = fs::read_dir(&accounts_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
        .collect();
    assert_eq!(
        account_files.len(),
        2,
        "Account files should still exist on disk"
    );

    println!(
        "Missing index with existing accounts: successfully recovered {} accounts",
        index.accounts.len()
    );
}

#[test]
pub(crate) fn test_save_account_index_roundtrip() {
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let dir = TestDataDir::new();

    // Build an AccountIndex with 2 accounts
    let now = chrono::Utc::now().timestamp();
    let index = AccountIndex {
        version: "2.0".to_string(),
        accounts: vec![
            AccountSummary {
                id: "acc-1".to_string(),
                email: "user1@example.com".to_string(),
                name: Some("User One".to_string()),
                disabled: false,
                proxy_disabled: false,
                protected_models: HashSet::new(),
                created_at: now,
                last_used: now,
            },
            AccountSummary {
                id: "acc-2".to_string(),
                email: "user2@example.com".to_string(),
                name: None,
                disabled: true,
                proxy_disabled: true,
                protected_models: HashSet::new(),
                created_at: now - 100,
                last_used: now - 50,
            },
        ],
        current_account_id: Some("acc-1".to_string()),
        current_target_ide: None,
    };

    // Save the index
    save_account_index_in_dir(dir.path(), &index).expect("Failed to save account index");

    // Load it back
    let loaded = load_account_index_in_dir(dir.path()).expect("Failed to load account index");

    // Assert it matches
    assert_eq!(loaded.accounts.len(), 2, "Should have 2 accounts");
    assert_eq!(
        loaded.current_account_id,
        Some("acc-1".to_string()),
        "current_account_id should match"
    );

    // Check first account
    let acc1 = loaded
        .accounts
        .iter()
        .find(|a| a.id == "acc-1")
        .expect("acc-1 should exist");
    assert_eq!(acc1.email, "user1@example.com");
    assert_eq!(acc1.name, Some("User One".to_string()));
    assert!(!acc1.disabled);
    assert!(!acc1.proxy_disabled);

    // Check second account
    let acc2 = loaded
        .accounts
        .iter()
        .find(|a| a.id == "acc-2")
        .expect("acc-2 should exist");
    assert_eq!(acc2.email, "user2@example.com");
    assert_eq!(acc2.name, None);
    assert!(acc2.disabled);
    assert!(acc2.proxy_disabled);

    println!(
        "save_account_index roundtrip: successfully saved and loaded index with {} accounts",
        loaded.accounts.len()
    );
}

#[test]
pub(crate) fn test_set_current_account_id_with_target() {
    let _env_guard = TEST_DATA_DIR_MUTEX
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let dir = TestDataDir::new();
    let previous = std::env::var_os("ABV_DATA_DIR");
    std::env::set_var("ABV_DATA_DIR", dir.path());
    if let Ok(mut guard) = data_dir_override_slot().write() {
        *guard = None;
    }

    // Create a dummy account index with some accounts
    let now = chrono::Utc::now().timestamp();
    let index = AccountIndex {
        version: "2.0".to_string(),
        accounts: vec![AccountSummary {
            id: "acc-1".to_string(),
            email: "user1@example.com".to_string(),
            name: Some("User One".to_string()),
            disabled: false,
            proxy_disabled: false,
            protected_models: HashSet::new(),
            created_at: now,
            last_used: now,
        }],
        current_account_id: None,
        current_target_ide: None,
    };
    save_account_index_in_dir(dir.path(), &index).unwrap();

    // 1. Call set_current_account_id_with_target with Some("agy")
    set_current_account_id_with_target("acc-1", Some("agy")).unwrap();

    // Load back and verify
    let index = load_account_index_in_dir(dir.path()).unwrap();
    assert_eq!(index.current_account_id, Some("acc-1".to_string()));
    assert_eq!(index.current_target_ide, Some("agy".to_string()));

    // 2. Call set_current_account_id (which sets target to None)
    set_current_account_id("acc-1").unwrap();

    // Load back and verify target is None
    let index = load_account_index_in_dir(dir.path()).unwrap();
    assert_eq!(index.current_account_id, Some("acc-1".to_string()));
    assert_eq!(index.current_target_ide, None);

    // Clean up environment variable
    if let Some(previous) = previous {
        std::env::set_var("ABV_DATA_DIR", previous);
    } else {
        std::env::remove_var("ABV_DATA_DIR");
    }
}

#[test]
pub(crate) fn test_backup_created_on_parse_failure() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let dir = TestDataDir::new();

    // Create a valid account file
    create_account_file(dir.path(), "recovered-acc", "recovered@example.com");

    // Create corrupt accounts.json with garbage (non-empty)
    let garbage_content = b"this is not valid json { broken";
    write_corrupted_index(dir.path(), garbage_content);

    // Verify accounts.json exists and is corrupt
    let index_path = dir.path().join("accounts.json");
    assert!(index_path.exists(), "accounts.json should exist");

    // Call load_account_index to trigger recovery and backup creation
    let recovered = load_account_index_in_dir(dir.path()).expect("Should recover from accounts");
    assert_eq!(recovered.accounts.len(), 1, "Should recover 1 account");
    assert_eq!(recovered.accounts[0].email, "recovered@example.com");
    assert_eq!(
        recovered.current_account_id,
        Some("recovered-acc".to_string())
    );

    // Assert a backup file exists with prefix "accounts.json.corrupt-"
    let data_dir = dir.path();
    let backup_files: Vec<_> = fs::read_dir(data_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_str()
                .map_or(false, |name| name.starts_with("accounts.json.corrupt-"))
        })
        .collect();

    assert_eq!(backup_files.len(), 1, "Should have exactly one backup file");

    // Verify backup contains the original garbage content
    let backup_content =
        fs::read(&backup_files[0].path()).expect("Should be able to read backup file");
    assert_eq!(
        backup_content, garbage_content,
        "Backup should contain original corrupt content"
    );

    println!("Backup creation on parse failure: successfully created backup");
}

#[test]
pub(crate) fn test_load_account_with_trailing_characters() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let dir = TestDataDir::new();

    create_account_file(dir.path(), "corrupt-tail-acc", "tail@example.com");
    let account_path = dir.path().join("accounts").join("corrupt-tail-acc.json");

    // Append trailing '}' to simulate Issue #3345
    let mut raw = fs::read_to_string(&account_path).unwrap();
    raw.push('}');
    fs::write(&account_path, &raw).unwrap();

    // Load account should successfully self-heal and return valid Account
    let loaded = load_account_at_path(&account_path).expect("Should self-heal trailing characters");
    assert_eq!(loaded.id, "corrupt-tail-acc");
    assert_eq!(loaded.email, "tail@example.com");

    // Verify the file was cleaned and re-written as valid JSON
    let healed_raw = fs::read_to_string(&account_path).unwrap();
    let regular_parse: Result<Account, _> = serde_json::from_str(&healed_raw);
    assert!(
        regular_parse.is_ok(),
        "Healed file should be standard valid JSON"
    );
}
