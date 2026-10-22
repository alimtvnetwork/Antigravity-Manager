// Unit Test Suite for Multi-Instance Sandboxed Runtime Environment (Imp-529, Imp-533, Imp-534)
// Validates environment token scrubbing, PATH construction & binary probing,
// remote debugging port allocation, directory permissions, and keychain bypass isolation.

#[cfg(test)]
mod tests {
    use super::directory_permission_guard::DirectoryPermissionGuard;
    use super::env_allowlist_filter::EnvAllowlistFilter;
    use super::env_path_normalizer::EnvPathNormalizer;
    use super::env_token_scrubber::{EnvTokenScrubber, SENSITIVE_TOKEN_KEYS};
    use super::env_whitelist_filter::{filter_child_environment, is_safe_instance_env_var};
    use super::keychain_file_guard::{
        check_keychain_markers_status, ensure_keychain_markers_present, KEYRING_BYPASS_MARKERS,
    };
    use super::keyring_flags::inject_keyring_isolation_flags;
    use super::keyring_mock_writer::{KeyringMockWriter, KEYRING_MARKER_FILENAME};
    use super::launch_env_sanitizer::LaunchEnvSanitizer;
    use super::port_allocation_guard::PortAllocationGuard;
    use super::port_collision_evictor::{
        is_port_available, resolve_collision_free_port, CDP_PORT_MAX, CDP_PORT_MIN,
    };
    use super::profile_dir_validator::{validate_profile_directory, ProfileDirValidationError};
    use super::profile_telemetry_isolation::ProfileTelemetryIsolation;
    use super::remote_debugging_port_allocator::RemoteDebuggingPortAllocator;
    use super::remote_port_guard::{RemotePortGuard, CDP_PORT_END, CDP_PORT_START};
    use super::storage_scrubber::{scrub_storage_json, SCRUBBED_TELEMETRY_KEYS};
    use super::workspace_directory_guard::{WorkspaceDirectoryGuard, REQUIRED_INSTANCE_SUBDIRS};
    use std::collections::HashMap;
    use std::fs;
    use std::net::{SocketAddr, TcpListener};
    use std::path::{Path, PathBuf};

    #[test]
    fn test_env_token_scrubber_filters_all_sensitive_keys_case_insensitively() {
        let raw_envs = vec![
            ("PATH".to_string(), "/usr/bin:/bin".to_string()),
            ("GITHUB_TOKEN".to_string(), "ghp_secret_token_12345".to_string()),
            ("github_token".to_string(), "ghp_lowercase_token_67890".to_string()),
            ("OpenAI_API_Key".to_string(), "sk-live-0987654321".to_string()),
            ("ANTHROPIC_API_KEY".to_string(), "sk-ant-live-abc123".to_string()),
            ("aws_secret_access_key".to_string(), "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string()),
            ("GEMINI_API_KEY".to_string(), "AIzaSySecretGeminiKey".to_string()),
            ("Claude_Api_Key".to_string(), "claude-test-secret".to_string()),
            ("SUPABASE_SERVICE_ROLE_KEY".to_string(), "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9".to_string()),
            ("STRIPE_API_KEY".to_string(), "rk_live_mock_key".to_string()),
            ("GH_TOKEN".to_string(), "gho_another_token".to_string()),
            ("USER".to_string(), "developer".to_string()),
            ("AGM_PROXY_PORT".to_string(), "8085".to_string()),
        ];

        let (scrubbed, removed_count) = EnvTokenScrubber::scrub_environment(raw_envs);

        assert_eq!(removed_count, 10, "All 10 sensitive tokens must be identified and removed");
        assert_eq!(scrubbed.len(), 3, "Only non-sensitive variables must remain");

        let remaining_keys: Vec<String> = scrubbed.into_iter().map(|(k, _)| k).collect();
        assert!(remaining_keys.contains(&"PATH".to_string()));
        assert!(remaining_keys.contains(&"USER".to_string()));
        assert!(remaining_keys.contains(&"AGM_PROXY_PORT".to_string()));

        for key in &remaining_keys {
            assert!(!EnvTokenScrubber::is_sensitive_key(key));
        }

        for &key in SENSITIVE_TOKEN_KEYS {
            assert!(EnvTokenScrubber::is_sensitive_key(key));
            assert!(EnvTokenScrubber::is_sensitive_key(&key.to_lowercase()));
        }
    }

    #[test]
    fn test_launch_env_sanitizer_removes_electron_and_vscode_hooks() {
        let mut env_map = HashMap::new();
        env_map.insert("NODE_OPTIONS".to_string(), "--inspect=9229".to_string());
        env_map.insert("ELECTRON_RUN_AS_NODE".to_string(), "1".to_string());
        env_map.insert("VSCODE_CWD".to_string(), "/home/user/project".to_string());
        env_map.insert("VSCODE_IPC_HOOK".to_string(), "/tmp/vscode-ipc.sock".to_string());
        env_map.insert("VSCODE_NLS_CONFIG".to_string(), "{\"locale\":\"en\"}".to_string());
        env_map.insert("VSCODE_PORTABLE".to_string(), "1".to_string());
        env_map.insert("ELECTRON_NO_ASAR".to_string(), "1".to_string());
        env_map.insert("SAFE_INSTANCE_ID".to_string(), "inst-alpha-001".to_string());
        env_map.insert("LANG".to_string(), "en_US.UTF-8".to_string());

        LaunchEnvSanitizer::sanitize_env_map(&mut env_map);

        assert_eq!(env_map.len(), 2, "Only non-contaminating environment variables must survive");
        assert_eq!(env_map.get("SAFE_INSTANCE_ID").map(|s| s.as_str()), Some("inst-alpha-001"));
        assert_eq!(env_map.get("LANG").map(|s| s.as_str()), Some("en_US.UTF-8"));

        for &stripped in LaunchEnvSanitizer::STRIPPED_VARS {
            assert!(!env_map.contains_key(stripped));
        }
    }

    #[test]
    fn test_env_path_normalizer_prepends_priority_and_deduplicates() {
        let temp_root = std::env::temp_dir().join(format!(
            "agm_path_norm_{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let sandboxed_bin = temp_root.join("sandboxed").join("bin");
        let gitmap_bin = temp_root.join("gitmap").join("bin");
        // Justification: Test fixture directory creation in temporary folder
        crate::error::record_ignored(std::fs::create_dir_all(&sandboxed_bin), "create test bin dir");
        // Justification: Test fixture directory creation in temporary folder
        crate::error::record_ignored(std::fs::create_dir_all(&gitmap_bin), "create test gitmap dir");

        let priority_paths = vec![
            sandboxed_bin.clone(),
            gitmap_bin.clone(),
            sandboxed_bin.clone(),
        ];

        let constructed_path = EnvPathNormalizer::build_sandboxed_path(&priority_paths);

        let separator = if cfg!(windows) { ';' } else { ':' };
        let entries: Vec<&str> = constructed_path.split(separator).collect();

        assert!(entries.len() >= 2);
        assert_eq!(entries[0], sandboxed_bin.to_string_lossy().as_ref());
        assert_eq!(entries[1], gitmap_bin.to_string_lossy().as_ref());

        let match_count = entries.iter().filter(|&&e| e == sandboxed_bin.to_string_lossy()).count();
        assert_eq!(match_count, 1, "Duplicate priority path must be deduplicated");

        // Justification: Best-effort test cleanup
        crate::error::record_ignored(std::fs::remove_dir_all(&temp_root), "clean test dir");
    }

    #[test]
    fn test_remote_port_guard_allocates_within_cdp_range_and_guarantees_idempotency() {
        let guard = RemotePortGuard::default();
        let inst_id = "inst-primary-cdp";

        let port1 = guard.allocate_port(inst_id);
        assert!(port1.is_some(), "Port allocation within CDP range must succeed");
        let allocated_port = port1.unwrap();

        assert!(allocated_port >= CDP_PORT_START && allocated_port <= CDP_PORT_END);

        let port2 = guard.allocate_port(inst_id);
        assert_eq!(port2, Some(allocated_port), "Re-allocation for same instance ID must return identical leased port");
    }

    #[test]
    fn test_remote_port_guard_prevents_port_collision_between_distinct_instances() {
        let guard = RemotePortGuard::default();

        let port_a = guard.allocate_port("instance-node-alpha");
        let port_b = guard.allocate_port("instance-node-beta");
        let port_c = guard.allocate_port("instance-node-gamma");

        assert!(port_a.is_some());
        assert!(port_b.is_some());
        assert!(port_c.is_some());

        let p_a = port_a.unwrap();
        let p_b = port_b.unwrap();
        let p_c = port_c.unwrap();

        assert_ne!(p_a, p_b, "Instance Alpha and Beta must receive distinct ports");
        assert_ne!(p_b, p_c, "Instance Beta and Gamma must receive distinct ports");
        assert_ne!(p_a, p_c, "Instance Alpha and Gamma must receive distinct ports");

        guard.release_port("instance-node-beta");
        let port_d = guard.allocate_port("instance-node-delta");
        assert!(port_d.is_some());
    }

    #[test]
    fn test_directory_permission_guard_validates_writable_and_creates_missing_hierarchy() {
        let temp_root = std::env::temp_dir().join(format!(
            "agm_perm_guard_{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        let data_dir = temp_root.join("instance_data");
        let home_dir = temp_root.join("instance_home");

        assert!(!data_dir.exists());
        assert!(!home_dir.exists());

        let result = DirectoryPermissionGuard::verify_instance_directories(&data_dir, &home_dir);
        assert!(result.is_ok(), "Directory permission validation must succeed for writable temp paths");

        assert!(data_dir.is_dir());
        assert!(home_dir.is_dir());
        assert!(!data_dir.join(".agm_write_probe.tmp").exists());
        assert!(!home_dir.join(".agm_write_probe.tmp").exists());

        // Justification: Best-effort test cleanup
        crate::error::record_ignored(fs::remove_dir_all(&temp_root), "clean test dir");
    }

    #[test]
    fn test_keychain_file_guard_creates_and_verifies_all_bypass_markers() {
        let temp_dir = std::env::temp_dir().join(format!(
            "agm_keychain_guard_{}",
            chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
        ));
        // Justification: Test fixture directory creation in temporary folder
        crate::error::record_ignored(fs::create_dir_all(&temp_dir), "create test dir");

        assert!(!check_keychain_markers_status(&temp_dir));

        let written = ensure_keychain_markers_present(&temp_dir)
            .expect("Keyring markers creation must succeed");
        assert_eq!(written, KEYRING_BYPASS_MARKERS.len());

        for &marker in KEYRING_BYPASS_MARKERS {
            let path = temp_dir.join(marker);
            assert!(path.is_file());
            let content = fs::read_to_string(&path).unwrap();
            assert_eq!(content, "1\n");
        }

        assert!(check_keychain_markers_status(&temp_dir));

        let second_written = ensure_keychain_markers_present(&temp_dir).unwrap();
        assert_eq!(second_written, 0);

        // Justification: Best-effort test cleanup
        crate::error::record_ignored(fs::remove_dir_all(&temp_dir), "clean test dir");
    }

    #[test]
    fn test_storage_scrubber_regenerates_telemetry_guids_and_scrubs_auth_tokens() {
        let source_storage = r#"{
            "telemetry.machineId": "host-machine-guid-1111-2222",
            "telemetry.macMachineId": "host-mac-guid-3333-4444",
            "telemetry.sqmId": "{AAAA-BBBB-CCCC}",
            "telemetry.devDeviceId": "host-dev-device-5555",
            "storage.serviceMachineId": "host-svc-machine-6666",
            "crashReporter.id": "host-crash-guid-7777",
            "github.auth.token": "gho_oauth_leaked_secret",
            "custom.oauth.secret": "client_secret_xyz",
            "editor.fontSize": 14,
            "workbench.colorTheme": "Default Dark+"
        }"#;

        let synthetic_id = "synthetic-machine-guid-9999-8888";
        let scrubbed_result = scrub_storage_json(source_storage, Some(synthetic_id))
            .expect("JSON scrubbing must succeed");

        let parsed: serde_json::Value = serde_json::from_str(&scrubbed_result).unwrap();

        assert_eq!(
            parsed.get("telemetry.machineId").and_then(|v| v.as_str()),
            Some(synthetic_id)
        );

        let new_dev_id = parsed.get("telemetry.devDeviceId").and_then(|v| v.as_str()).unwrap();
        assert_ne!(new_dev_id, "host-dev-device-5555");
        assert!(!new_dev_id.is_empty());

        for &key in SCRUBBED_TELEMETRY_KEYS {
            if key != "telemetry.machineId" && key != "telemetry.devDeviceId" {
                assert!(parsed.get(key).is_none(), "Telemetry key {:?} must be scrubbed", key);
            }
        }

        assert!(parsed.get("github.auth.token").is_none());
        assert!(parsed.get("custom.oauth.secret").is_none());
        assert_eq!(parsed.get("editor.fontSize").and_then(|v| v.as_u64()), Some(14));
        assert_eq!(parsed.get("workbench.colorTheme").and_then(|v| v.as_str()), Some("Default Dark+"));
    }
}
