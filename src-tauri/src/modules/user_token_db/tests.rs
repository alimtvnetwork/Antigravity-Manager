use uuid::Uuid;

use super::*;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    pub(crate) struct EnvDataDirGuard {
        _temp_dir: tempfile::TempDir,
        previous: Option<std::ffi::OsString>,
    }

    impl EnvDataDirGuard {
        fn new() -> Self {
            let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
            let previous = std::env::var_os("ABV_DATA_DIR");
            std::env::set_var("ABV_DATA_DIR", temp_dir.path());
            Self {
                _temp_dir: temp_dir,
                previous,
            }
        }
    }

    impl Drop for EnvDataDirGuard {
        fn drop(&mut self) {
            if let Some(previous) = &self.previous {
                std::env::set_var("ABV_DATA_DIR", previous);
            } else {
                std::env::remove_var("ABV_DATA_DIR");
            }
        }
    }

    #[test]
    pub(crate) fn test_create_and_query_token() {
        let _lock = crate::modules::account::TEST_DATA_DIR_MUTEX
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _env_guard = EnvDataDirGuard::new();

        let init_res = init_db();
        assert!(init_res.is_ok(), "init_db failed: {:?}", init_res.err());

        // Use a random username to avoid collisions in existing DB runs during dev
        let username = format!("TestUser_{}", Uuid::new_v4());
        let token_res = create_token(
            username.clone(),
            "day".to_string(),
            Some("Test token".to_string()),
            0,
            None,
            None,
            None,
        );
        assert!(
            token_res.is_ok(),
            "create_token failed: {:?}",
            token_res.err()
        );

        let token = token_res.unwrap();
        assert_eq!(token.username, username);
        assert!(token.token.starts_with("sk-"));

        let fetched = get_token_by_id(&token.id);
        assert!(fetched.is_ok());
        assert_eq!(fetched.unwrap().unwrap().username, username);
    }

    #[test]
    pub(crate) fn test_never_expire_token_validation() {
        let _lock = crate::modules::account::TEST_DATA_DIR_MUTEX
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let _env_guard = EnvDataDirGuard::new();

        let init_res = init_db();
        assert!(init_res.is_ok(), "init_db failed: {:?}", init_res.err());

        let username = format!("NeverExpireUser_{}", Uuid::new_v4());
        let token_res = create_token(
            username.clone(),
            "never".to_string(),
            Some("Never expire test token".to_string()),
            0,
            None,
            None,
            None,
        );
        assert!(
            token_res.is_ok(),
            "create_token failed: {:?}",
            token_res.err()
        );
        let token = token_res.unwrap();
        let (valid, reason) =
            validate_token(&token.token, "127.0.0.1").expect("validation must succeed");
        assert!(
            valid,
            "Token with expires_type never must be valid, reason: {:?}",
            reason
        );
    }
}
