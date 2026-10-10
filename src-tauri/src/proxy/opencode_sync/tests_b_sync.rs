use super::*;

/// Helper: build a ModelInput with just an id (no display name).
fn minput(id: &str) -> ModelInput {
    ModelInput {
        id: id.to_string(),
        name: None,
    }
}

/// Helper: build a ModelInput carrying a display name.
fn minput_named(id: &str, name: &str) -> ModelInput {
    ModelInput {
        id: id.to_string(),
        name: Some(name.to_string()),
    }
}

#[test]
fn test_extract_version_opencode_format() {
    let input = "opencode/1.2.3";
    assert_eq!(extract_version(input), "1.2.3");
}

#[test]
fn test_extract_version_codex_cli_format() {
    let input = "codex-cli 0.86.0\n";
    assert_eq!(extract_version(input), "0.86.0");
}

#[test]
fn test_extract_version_simple() {
    let input = "v2.0.1";
    assert_eq!(extract_version(input), "2.0.1");
}

#[test]
fn test_extract_version_unknown() {
    let input = "some random text without version";
    assert_eq!(extract_version(input), "unknown");
}

#[test]
fn test_normalize_opencode_base_url_without_v1() {
    assert_eq!(
        normalize_opencode_base_url("http://localhost:3000"),
        "http://localhost:3000/v1"
    );
    assert_eq!(
        normalize_opencode_base_url("http://localhost:3000/"),
        "http://localhost:3000/v1"
    );
}

#[test]
fn test_normalize_opencode_base_url_with_v1() {
    assert_eq!(
        normalize_opencode_base_url("http://localhost:3000/v1"),
        "http://localhost:3000/v1"
    );
    assert_eq!(
        normalize_opencode_base_url("http://localhost:3000/v1/"),
        "http://localhost:3000/v1"
    );
}

#[test]
fn test_normalize_opencode_base_url_with_whitespace() {
    assert_eq!(
        normalize_opencode_base_url("  http://localhost:3000  "),
        "http://localhost:3000/v1"
    );
    assert_eq!(
        normalize_opencode_base_url("  http://localhost:3000/v1  "),
        "http://localhost:3000/v1"
    );
}

#[test]
fn test_normalize_opencode_base_url_no_double_v1() {
    // Ensure we don't create double /v1/v1
    assert_eq!(
        normalize_opencode_base_url("http://localhost:3000/v1"),
        "http://localhost:3000/v1"
    );
    assert_eq!(
        normalize_opencode_base_url("http://localhost:3000/v1/"),
        "http://localhost:3000/v1"
    );
}

// Tests for apply_sync_to_config

#[test]
fn test_sync_preserves_existing_providers() {
    // Config with existing google and anthropic providers
    let config = serde_json::json!({
        "provider": {
            "google": {
                "options": { "apiKey": "google-key" },
                "models": { "gemini-pro": { "name": "Gemini Pro" } }
            },
            "anthropic": {
                "options": { "apiKey": "anthropic-key" },
                "models": { "claude-3": { "name": "Claude 3" } }
            }
        }
    });

    let result = apply_sync_to_config(config, "http://localhost:3000", "test-api-key", None);

    // Existing providers should be preserved
    let provider = result.get("provider").unwrap();
    assert!(
        provider.get("google").is_some(),
        "google provider should be preserved"
    );
    assert!(
        provider.get("anthropic").is_some(),
        "anthropic provider should be preserved"
    );
    assert_eq!(
        provider
            .get("google")
            .unwrap()
            .get("options")
            .unwrap()
            .get("apiKey")
            .unwrap(),
        "google-key"
    );
    assert_eq!(
        provider
            .get("anthropic")
            .unwrap()
            .get("options")
            .unwrap()
            .get("apiKey")
            .unwrap(),
        "anthropic-key"
    );
}

#[test]
fn test_sync_creates_antigravity_provider() {
    let config = serde_json::json!({});

    let result = apply_sync_to_config(config, "http://localhost:3000", "test-api-key", None);

    // antigravity-manager provider should be created
    let provider = result.get("provider").unwrap();
    let ag = provider.get(ANTIGRAVITY_PROVIDER_ID).unwrap();

    // Check npm and name
    assert_eq!(ag.get("npm").unwrap(), "@ai-sdk/anthropic");
    assert_eq!(ag.get("name").unwrap(), "Antigravity Manager");

    // Check options
    let options = ag.get("options").unwrap();
    assert_eq!(options.get("baseURL").unwrap(), "http://localhost:3000/v1");
    assert_eq!(options.get("apiKey").unwrap(), "test-api-key");
}

#[test]
fn test_sync_creates_models() {
    let config = serde_json::json!({});

    let result = apply_sync_to_config(config, "http://localhost:3000", "test-api-key", None);

    let provider = result.get("provider").unwrap();
    let ag = provider.get(ANTIGRAVITY_PROVIDER_ID).unwrap();
    let models = ag.get("models").unwrap().as_object().unwrap();

    // Should have all catalog models
    assert!(
        models.contains_key("claude-sonnet-4-6"),
        "should have claude-sonnet-4-6"
    );
    assert!(
        models.contains_key("gemini-3.1-pro"),
        "should have gemini-3.1-pro"
    );
    assert!(
        models.contains_key("gemini-2.5-pro"),
        "should have gemini-2.5-pro"
    );

    // Check model structure
    let claude_model = models.get("claude-sonnet-4-6").unwrap();
    assert_eq!(claude_model.get("name").unwrap(), "Claude Sonnet 4.6");
    assert!(claude_model.get("limit").is_some());
    assert!(claude_model.get("modalities").is_some());
}

#[test]
fn test_sync_with_filtered_models() {
    let config = serde_json::json!({});
    let models_to_sync = [minput("claude-sonnet-4-6"), minput("gemini-3.1-pro")];

    let result = apply_sync_to_config(
        config,
        "http://localhost:3000",
        "test-api-key",
        Some(&models_to_sync),
    );

    let provider = result.get("provider").unwrap();
    let ag = provider.get(ANTIGRAVITY_PROVIDER_ID).unwrap();
    let models = ag.get("models").unwrap().as_object().unwrap();

    assert!(models.contains_key("claude-sonnet-4-6"));
    let pro_model = models.get("gemini-3.1-pro").unwrap();
    assert_eq!(pro_model.get("name").unwrap(), "Gemini 3.1 Pro");
    assert_eq!(pro_model["limit"]["output"], 65_535);
    assert!(
        !models.contains_key("gemini-2.5-pro"),
        "should not have unselected models"
    );
}

#[test]
fn test_openai_compatible_sync_creates_apikey_fun_provider() {
    let config = serde_json::json!({
        "provider": {
            ANTIGRAVITY_PROVIDER_ID: {
                "npm": "@ai-sdk/anthropic",
                "name": "Antigravity Manager",
                "options": { "apiKey": "ag-key" }
            }
        }
    });
    let models_to_sync = [
        minput("gpt-5.5"),
        minput_named("claude-sonnet-4-6", "Claude Sonnet 4.6"),
    ];

    let result = apply_openai_compatible_provider_sync(
        config,
        APIKEY_FUN_PROVIDER_ID,
        "APIKEY.FUN",
        "https://api.apikey.fun",
        "fun-key",
        Some(&models_to_sync),
    );

    let provider = result.get("provider").unwrap();
    assert!(
        provider.get(ANTIGRAVITY_PROVIDER_ID).is_some(),
        "antigravity-manager provider should be preserved"
    );
    let fun = provider.get(APIKEY_FUN_PROVIDER_ID).unwrap();
    assert_eq!(fun.get("npm").unwrap(), OPENAI_COMPATIBLE_NPM);
    assert_eq!(fun.get("name").unwrap(), "APIKEY.FUN");
    assert_eq!(
        fun.get("options").unwrap().get("baseURL").unwrap(),
        "https://api.apikey.fun/v1"
    );
    assert_eq!(
        fun.get("options").unwrap().get("apiKey").unwrap(),
        "fun-key"
    );

    let models = fun.get("models").unwrap().as_object().unwrap();
    assert_eq!(models.len(), 2);
    assert_eq!(
        models.get("gpt-5.5").unwrap().get("name").unwrap(),
        "Gpt 5.5"
    );

    // claude-sonnet-4-6 is in the catalog: full metadata, dotted display name.
    let claude = models.get("claude-sonnet-4-6").unwrap();
    assert_eq!(claude.get("name").unwrap(), "Claude Sonnet 4.6");
    assert_eq!(claude["limit"]["context"], 200_000);
    assert_eq!(claude["limit"]["output"], 64_000);
    assert!(claude.get("modalities").is_some());
}

#[test]
fn test_openai_compatible_sync_replaces_models() {
    let config = serde_json::json!({
        "provider": {
            APIKEY_FUN_PROVIDER_ID: {
                "models": {
                    "old-model": { "name": "Old Model" }
                }
            }
        }
    });

    let result = apply_openai_compatible_provider_sync(
        config,
        APIKEY_FUN_PROVIDER_ID,
        "APIKEY.FUN",
        "https://api.apikey.fun/v1",
        "fun-key",
        Some(&[minput("gpt-5.5")]),
    );

    let models = result["provider"][APIKEY_FUN_PROVIDER_ID]["models"]
        .as_object()
        .unwrap();
    assert!(models.contains_key("gpt-5.5"));
    assert!(!models.contains_key("old-model"));
}

#[test]
fn test_profile_collision_leaves_config_unchanged() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join(OPENCODE_CONFIG_FILE);
    let original = r#"{"provider":{"apikey-fun-abcdef":{"options":{"apiKey":"first-key"}}}}"#;
    fs::write(&path, original).unwrap();
    let error = sync_openai_provider_to_path(
        &path,
        "apikey-fun-abcdef",
        "APIKEY.FUN",
        "https://api.example.com",
        "second-key",
        None,
    )
    .unwrap_err();
    assert!(is_provider_validation_error(&error));
    assert_eq!(fs::read_to_string(&path).unwrap(), original);
    assert_eq!(fs::read_dir(tmp.path()).unwrap().count(), 1);
}

#[test]
fn test_concurrent_profile_sync_preserves_both_keys() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join(OPENCODE_CONFIG_FILE);
    std::thread::scope(|scope| {
        for id in ["apikey-fun-111111", "apikey-fun-222222"] {
            let path = &path;
            scope.spawn(move || {
                sync_openai_provider_to_path(path, id, id, "https://api.example.com", id, None)
                    .unwrap();
            });
        }
    });
    let config = parse_config_file(&path).unwrap();
    assert_eq!(config["provider"].as_object().unwrap().len(), 2);
    for id in ["apikey-fun-111111", "apikey-fun-222222"] {
        assert_eq!(config["provider"][id]["options"]["apiKey"], id);
    }
}

#[test]
fn test_atomic_write_cleans_temp_after_rename_failure() {
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("directory");
    fs::create_dir(&target).unwrap();
    assert!(atomically_write_config(&target, &serde_json::json!({})).is_err());
    assert!(target.is_dir());
    assert_eq!(fs::read_dir(tmp.path()).unwrap().count(), 1);
}

#[cfg(unix)]
#[test]
fn test_atomic_write_private_permissions_and_replacement() {
    use std::os::unix::fs::PermissionsExt;
    let tmp = tempfile::tempdir().unwrap();
    let target = tmp.path().join("nested").join(OPENCODE_CONFIG_FILE);
    atomically_write_config(&target, &serde_json::json!({"key": "first"})).unwrap();
    assert_eq!(
        fs::metadata(&target).unwrap().permissions().mode() & 0o777,
        0o600
    );
    fs::set_permissions(&target, fs::Permissions::from_mode(0o640)).unwrap();
    atomically_write_config(&target, &serde_json::json!({"key": "second"})).unwrap();
    assert_eq!(
        fs::metadata(&target).unwrap().permissions().mode() & 0o777,
        0o640
    );
    assert_eq!(parse_config_file(&target).unwrap()["key"], "second");
    assert_eq!(fs::read_dir(target.parent().unwrap()).unwrap().count(), 1);
}

#[test]
fn test_apply_remove_provider_removes_correct_provider() {
    let config = serde_json::json!({
        "provider": {
            "apikey-fun-123456": { "name": "APIKEY.FUN (123456)" },
            "apikey-fun-abcdef": { "name": "APIKEY.FUN (abcdef)" },
            "antigravity-manager": { "name": "Antigravity" }
        }
    });

    let (result, changed) = apply_remove_provider(config, "apikey-fun-123456");
    assert!(changed);
    let providers = result["provider"].as_object().unwrap();
    assert_eq!(providers.len(), 2);
    assert!(!providers.contains_key("apikey-fun-123456"));
    assert!(providers.contains_key("apikey-fun-abcdef"));
    assert!(providers.contains_key("antigravity-manager"));
}

#[test]
fn test_apply_remove_last_provider_removes_provider_key() {
    let config = serde_json::json!({
        "$schema": "https://opencode.ai/config.json",
        "provider": {
            "apikey-fun-123456": { "name": "APIKEY.FUN (123456)" }
        }
    });

    let (result, changed) = apply_remove_provider(config, "apikey-fun-123456");
    assert!(changed);
    assert!(result.get("provider").is_none());
    assert_eq!(result["$schema"], "https://opencode.ai/config.json");
}

#[test]
fn test_apply_remove_nonexistent_provider_returns_false() {
    let config = serde_json::json!({
        "provider": {
            "antigravity-manager": { "name": "Antigravity" }
        }
    });

    let (result, changed) = apply_remove_provider(config, "nonexistent");
    assert!(!changed);
    assert!(result["provider"]
        .as_object()
        .unwrap()
        .contains_key("antigravity-manager"));
}

#[test]
fn test_apply_remove_on_empty_provider_map_does_not_mutate() {
    let config = serde_json::json!({ "provider": {} });

    let (result, changed) = apply_remove_provider(config, "apikey-fun-abc123");
    assert!(!changed);
    assert!(
        result.get("provider").is_some(),
        "provider key must survive when nothing was removed"
    );
}

#[test]
fn test_remove_opencode_provider_rejects_invalid_and_unmanaged_ids() {
    // Empty / whitespace-only
    assert!(remove_opencode_provider("   ").is_err());
    // Path traversal characters are rejected by the charset check
    assert!(remove_opencode_provider("../../etc/passwd").is_err());
    // Reserved provider cannot be removed through this path
    assert!(remove_opencode_provider(ANTIGRAVITY_PROVIDER_ID).is_err());
    // Providers not owned by this feature are refused
    for id in ["anthropic", "openai", "github-copilot", "openrouter"] {
        let err = remove_opencode_provider(id).expect_err("unmanaged provider must be refused");
        assert!(
            err.contains("is not managed by Antigravity-Manager"),
            "unexpected error for {}: {}",
            id,
            err
        );
    }
}

#[test]
fn test_is_provider_validation_error_classification() {
    assert!(is_provider_validation_error(
        "OpenCode provider id is required"
    ));
    assert!(is_provider_validation_error(
        "Invalid OpenCode provider id 'a/b': only letters, digits, '-' and '_' are allowed"
    ));
    assert!(is_provider_validation_error(
        "Provider 'antigravity-manager' is reserved and cannot be removed; use clear instead"
    ));
    assert!(is_provider_validation_error(
        "Provider 'openai' is not managed by Antigravity-Manager and cannot be removed"
    ));
    // Generic OS errors must stay 5xx
    assert!(!is_provider_validation_error(
        "Failed to write temp file: Invalid argument (os error 22)"
    ));
    assert!(!is_provider_validation_error(
        "Failed to read OpenCode config: Permission denied"
    ));
}
