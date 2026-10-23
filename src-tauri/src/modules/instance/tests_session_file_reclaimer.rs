// Unit Test Suite for Singleton Session Lockfile Reclaimer (Imp-534)
// Validates stale singleton lock removal, VS Code session marker sweeps,
// non-existent directory safety, and counting reclaimed locks.

#[cfg(test)]
mod tests {
    use crate::modules::instance::session_file_reclaimer::SessionFileReclaimer;
    use std::fs;

    #[test]
    fn test_session_file_reclaimer_constants() {
        assert!(SessionFileReclaimer::STALE_SESSION_FILES.contains(&"SingletonLock"));
        assert!(SessionFileReclaimer::STALE_SESSION_FILES.contains(&"SingletonCookie"));
        assert!(SessionFileReclaimer::STALE_SESSION_FILES.contains(&"SingletonSocket"));
        assert!(SessionFileReclaimer::STALE_SESSION_FILES.contains(&"lockfile"));
        assert!(SessionFileReclaimer::STALE_SESSION_FILES.contains(&"code.lock"));
        assert!(SessionFileReclaimer::STALE_SESSION_FILES.contains(&"DevToolsActivePort"));
    }

    #[test]
    fn test_session_file_reclaimer_missing_directory_returns_zero() {
        let nonexistent = std::path::PathBuf::from("nonexistent_session_dir_12345");
        let reclaimed = SessionFileReclaimer::reclaim_stale_files(&nonexistent).unwrap();
        assert_eq!(reclaimed, 0);
    }

    #[test]
    fn test_session_file_reclaimer_removes_stale_locks() {
        let temp_dir = std::env::temp_dir().join(format!(
            "agm_session_reclaim_test_{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        // Justification: Create test fixture directory
        crate::error::record_ignored(fs::create_dir_all(&temp_dir), "create test dir");

        // Create several stale lockfiles
        let lock1 = temp_dir.join("SingletonLock");
        let lock2 = temp_dir.join("code.lock");
        let lock3 = temp_dir.join("DevToolsActivePort");
        let non_lock = temp_dir.join("user_settings.json");

        // Justification: Write dummy test lockfiles
        crate::error::record_ignored(fs::write(&lock1, b"1234"), "write lock1");
        crate::error::record_ignored(fs::write(&lock2, b"5678"), "write lock2");
        crate::error::record_ignored(
            fs::write(&lock3, b"9222\n/devtools/browser/xyz"),
            "write lock3",
        );
        crate::error::record_ignored(
            fs::write(&non_lock, b"{\"theme\":\"dark\"}"),
            "write settings",
        );

        assert!(lock1.exists());
        assert!(lock2.exists());
        assert!(lock3.exists());
        assert!(non_lock.exists());

        let count = SessionFileReclaimer::reclaim_stale_files(&temp_dir).unwrap();
        assert_eq!(count, 3, "Must reclaim exactly the 3 stale lockfiles");

        assert!(!lock1.exists(), "SingletonLock must be deleted");
        assert!(!lock2.exists(), "code.lock must be deleted");
        assert!(!lock3.exists(), "DevToolsActivePort must be deleted");
        assert!(
            non_lock.exists(),
            "Non-stale user settings must remain untouched"
        );

        // Second run finds 0 stale files
        let count2 = SessionFileReclaimer::reclaim_stale_files(&temp_dir).unwrap();
        assert_eq!(count2, 0);

        // Justification: Clean up test directory
        crate::error::record_ignored(fs::remove_dir_all(&temp_dir), "clean test dir");
    }
}
