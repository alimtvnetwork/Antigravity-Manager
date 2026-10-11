#![cfg(test)]

use super::*;
use std::collections::HashSet;
use std::sync::Mutex as StdMutex;

use super::*;

pub(crate) static TEST_MUTEX: Lazy<StdMutex<()>> = Lazy::new(|| StdMutex::new(()));

pub(crate) static TEST_COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

pub(crate) struct TestDataDir {
    pub(crate) path: PathBuf,
}

impl TestDataDir {
    pub(crate) fn new() -> Self {
        let count = TEST_COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let temp_path = std::env::temp_dir().join(format!(
            "antigravity_test_{}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            count
        ));
        fs::create_dir_all(&temp_path).expect("Failed to create temp dir");

        Self { path: temp_path }
    }

    pub(crate) fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl Drop for TestDataDir {
    fn drop(&mut self) {
        // Justification: best-effort cleanup; a leftover directory is harmless
        crate::error::record_ignored(fs::remove_dir_all(&self.path), "remove_dir_all");
    }
}

pub(crate) fn write_corrupted_index(path: &PathBuf, content: &[u8]) {
    let index_path = path.join("accounts.json");
    fs::write(&index_path, content).expect("Failed to write corrupted index");
}

pub(crate) fn create_account_file(path: &PathBuf, account_id: &str, email: &str) {
    let accounts_dir = path.join("accounts");
    fs::create_dir_all(&accounts_dir).expect("Failed to create accounts dir");

    let account = Account::new(
        account_id.to_string(),
        email.to_string(),
        TokenData::new(
            "test_access_token".to_string(),
            "test_refresh_token".to_string(),
            3600,
            Some(email.to_string()),
            None,
            None,
            true,
            None,
        ),
    );

    let content = serde_json::to_string_pretty(&account).expect("Failed to serialize account");
    let account_path = accounts_dir.join(format!("{}.json", account_id));
    fs::write(&account_path, content).expect("Failed to write account file");
}

#[test]
pub(crate) fn test_normalize_data_dir_path_strips_windows_prefix() {
    assert_eq!(
        format_data_dir_path(Path::new(r"\\?\F:\antigravity-tools-data")),
        r"F:\antigravity-tools-data"
    );
    assert_eq!(
        format_data_dir_path(Path::new(r"\\?\UNC\server\share\data")),
        r"\\server\share\data"
    );
    assert_eq!(format_data_dir_path(Path::new("//?/C:/data")), "C:/data");
    assert_eq!(format_data_dir_path(Path::new("/app/data")), "/app/data");
    assert_eq!(
        format_data_dir_path(Path::new(r"F:\antigravity-tools-data")),
        r"F:\antigravity-tools-data"
    );
}

#[test]
pub(crate) fn test_migrate_data_dir_rename_and_copy() {
    let _data_dir_guard = TEST_DATA_DIR_MUTEX
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    let _guard = TEST_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    let pointer_path = dirs::home_dir()
        .expect("home")
        .join(".antigravity_tools_location");
    let previous_env = std::env::var("ABV_DATA_DIR").ok();
    let previous_pointer = fs::read_to_string(&pointer_path).ok();

    let restore = || {
        match &previous_env {
            Some(value) => std::env::set_var("ABV_DATA_DIR", value),
            None => std::env::remove_var("ABV_DATA_DIR"),
        }
        match &previous_pointer {
            Some(value) => {
                // Justification: best-effort file write; failure is logged and surfaces on the next read
                crate::error::record_ignored(fs::write(&pointer_path, value), "fs::write");
            }
            None => {
                // Justification: best-effort cleanup; a leftover file is harmless
                crate::error::record_ignored(fs::remove_file(&pointer_path), "remove_file");
            }
        }
        if let Ok(mut guard) = data_dir_override_slot().write() {
            *guard = None;
        }
    };

    let src = TestDataDir::new();
    fs::write(src.path().join("marker.txt"), "hello").unwrap();
    std::env::set_var("ABV_DATA_DIR", src.path());

    let dest_parent = TestDataDir::new();
    let dest = dest_parent.path().join("moved_data");
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let resolved = migrate_data_dir(dest.clone()).unwrap();
        assert!(resolved.join("marker.txt").exists());
        assert!(!src.path().join("marker.txt").exists());
        assert_eq!(
            fs::read_to_string(resolved.join("marker.txt")).unwrap(),
            "hello"
        );
        let shown = format_data_dir_path(&resolved);
        assert!(
            !shown.contains(r"\\?\"),
            "migrated path must not keep Windows verbatim prefix: {shown}"
        );
    }));
    restore();
    if let Err(panic) = result {
        std::panic::resume_unwind(panic);
    }
}

#[test]
pub(crate) fn test_load_account_index_with_bom_prefix() {
    let _guard = TEST_MUTEX.lock().unwrap();
    let dir = TestDataDir::new();

    // UTF-8 BOM followed by valid JSON
    let bom = [0xEF, 0xBB, 0xBF];
    let json = r#"{"version":"2.0","accounts":[],"current_account_id":null}"#;
    let mut content = Vec::new();
    content.extend_from_slice(&bom);
    content.extend_from_slice(json.as_bytes());

    write_corrupted_index(dir.path(), &content);

    let result = load_account_index_in_dir(dir.path());

    // New behavior: BOM is stripped and JSON parses successfully
    assert!(
        result.is_ok(),
        "BOM should be stripped and JSON should parse: {:?}",
        result
    );
    let index = result.unwrap();
    assert!(index.accounts.is_empty());
    println!("BOM case: successfully loaded index after sanitization");
}
