use super::*;

/// Helper: build a ModelInput with just an id (no display name).
fn minput(id: &str) -> ModelInput {
    ModelInput {
        id: id.to_string(),
        name: None,
    }
}

#[test]
fn test_extract_providers_from_config_sorts_and_skips_invalid() {
    let config = serde_json::json!({
        "provider": {
            "zeta": {
                "name": "Zeta",
                "models": { "b-model": {}, "a-model": {} }
            },
            "alpha": {
                "name": "Alpha",
                "npm": "@ai-sdk/openai-compatible",
                "options": {
                    "baseURL": "https://api.example.com/v1",
                    "apiKey": "sk-test"
                },
                "models": { "m1": {} }
            },
            "invalid": "not-an-object"
        }
    });

    let providers = extract_providers_from_config(&config);
    assert_eq!(providers.len(), 2);
    assert_eq!(providers[0].id, "alpha");
    assert_eq!(providers[0].name.as_deref(), Some("Alpha"));
    assert_eq!(
        providers[0].npm.as_deref(),
        Some("@ai-sdk/openai-compatible")
    );
    assert_eq!(
        providers[0].base_url.as_deref(),
        Some("https://api.example.com/v1")
    );
    assert_eq!(providers[0].api_key.as_deref(), Some("sk-test"));
    assert_eq!(providers[0].models, vec!["m1".to_string()]);

    assert_eq!(providers[1].id, "zeta");
    assert_eq!(
        providers[1].models,
        vec!["a-model".to_string(), "b-model".to_string()]
    );
}

#[test]
fn test_openai_compatible_sync_empty_models_keeps_existing() {
    let config = serde_json::json!({
        "provider": {
            APIKEY_FUN_PROVIDER_ID: {
                "models": {
                    "gpt-4o": { "name": "GPT-4o", "custom": true }
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
        Some(&[]),
    );

    let models = result["provider"][APIKEY_FUN_PROVIDER_ID]["models"]
        .as_object()
        .unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models.get("gpt-4o").unwrap().get("name").unwrap(), "GPT-4o");
    assert_eq!(
        models.get("gpt-4o").unwrap().get("custom").unwrap(),
        &Value::Bool(true)
    );
}

#[test]
fn test_provider_id_validation() {
    assert!(validate_provider_id("apikey-fun").is_ok());
    assert!(validate_provider_id("My_Provider2").is_ok());
    assert!(validate_provider_id("").is_err());
    assert!(validate_provider_id("  ").is_err());
    assert!(validate_provider_id("bad/id").is_err());
    assert!(validate_provider_id("bad id").is_err());
}

#[test]
fn test_openai_sync_preserves_unknown_model_settings() {
    let existing = serde_json::json!({
        "name": "My GPT",
        "limit": { "context": 128_000, "output": 16_384 },
        "options": { "reasoningEffort": "high" },
        "tool_call": true
    });
    let result = apply_openai_compatible_provider_sync(
        serde_json::json!({ "provider": { APIKEY_FUN_PROVIDER_ID: {
            "models": { "gpt-5.5": existing.clone() }
        }}}),
        APIKEY_FUN_PROVIDER_ID,
        "APIKEY.FUN",
        "https://api.apikey.fun/v1/",
        "fun-key",
        Some(&[minput("gpt-5.5")]),
    );
    assert_eq!(
        result["provider"][APIKEY_FUN_PROVIDER_ID]["models"]["gpt-5.5"],
        existing
    );
    assert_eq!(
        result["provider"][APIKEY_FUN_PROVIDER_ID]["options"]["baseURL"],
        "https://api.apikey.fun/v1"
    );
}

#[test]
fn test_openai_sync_repairs_non_object_options() {
    for options in [
        Value::Null,
        serde_json::json!([]),
        serde_json::json!("invalid"),
    ] {
        let result = apply_openai_compatible_provider_sync(
            serde_json::json!({ "provider": { APIKEY_FUN_PROVIDER_ID: { "options": options }}}),
            APIKEY_FUN_PROVIDER_ID,
            "APIKEY.FUN",
            "https://api.apikey.fun",
            "fun-key",
            None,
        );
        assert_eq!(
            result["provider"][APIKEY_FUN_PROVIDER_ID]["options"]["apiKey"],
            "fun-key"
        );
    }
}

#[test]
fn test_openai_sync_leaves_invalid_config_untouched() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join(OPENCODE_CONFIG_FILE_JSONC);
    for content in ["{ broken", "[]", "null", ""] {
        fs::write(&path, content).unwrap();
        let result = sync_openai_provider_to_path(
            &path,
            APIKEY_FUN_PROVIDER_ID,
            "APIKEY.FUN",
            "https://api.apikey.fun",
            "fun-key",
            None,
        );
        assert!(result.is_err(), "must reject invalid config: {content}");
        assert_eq!(fs::read_to_string(&path).unwrap(), content);
        assert!(!path.with_extension("tmp").exists());
        assert!(!tmp
            .path()
            .join(format!("{OPENCODE_CONFIG_FILE_JSONC}{BACKUP_SUFFIX}"))
            .exists());
    }
    // A read error must also propagate instead of replacing the config.
    fs::remove_file(&path).unwrap();
    fs::create_dir(&path).unwrap();
    assert!(sync_openai_provider_to_path(
        &path,
        APIKEY_FUN_PROVIDER_ID,
        "APIKEY.FUN",
        "https://api.apikey.fun",
        "fun-key",
        None,
    )
    .is_err());
    assert!(path.is_dir());
}

#[test]
fn test_openai_sync_preserves_jsonc_unicode_and_backup() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join(OPENCODE_CONFIG_FILE_JSONC);
    let content = r#"{
            // User settings must survive syncing another provider.
            "instructions": ["инструкции.md", "日本語.md", "🚀.md",],
            "provider": {"custom": {"name": "Мой провайдер",},},
        }"#;
    fs::write(&path, content).unwrap();
    sync_openai_provider_to_path(
        &path,
        APIKEY_FUN_PROVIDER_ID,
        "APIKEY.FUN",
        "https://api.apikey.fun/v1/",
        "fun-key",
        Some(&[minput("gpt-5.5")]),
    )
    .unwrap();
    let config: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(
        config["instructions"],
        serde_json::json!(["инструкции.md", "日本語.md", "🚀.md"])
    );
    assert_eq!(config["provider"]["custom"]["name"], "Мой провайдер");
    assert_eq!(
        config["provider"][APIKEY_FUN_PROVIDER_ID]["options"]["baseURL"],
        "https://api.apikey.fun/v1"
    );
    let backup = tmp
        .path()
        .join(format!("{OPENCODE_CONFIG_FILE_JSONC}{BACKUP_SUFFIX}"));
    assert_eq!(fs::read_to_string(&backup).unwrap(), content);
    sync_openai_provider_to_path(
        &path,
        APIKEY_FUN_PROVIDER_ID,
        "APIKEY.FUN",
        "https://api.apikey.fun/v1",
        "next-key",
        None,
    )
    .unwrap();
    assert_eq!(fs::read_to_string(&backup).unwrap(), content);
}

#[test]
fn test_openai_sync_creates_missing_config_directory() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("new").join(OPENCODE_CONFIG_FILE);
    sync_openai_provider_to_path(
        &path,
        APIKEY_FUN_PROVIDER_ID,
        "APIKEY.FUN",
        "https://api.apikey.fun",
        "fun-key",
        None,
    )
    .unwrap();
    let config = parse_config_file(&path).unwrap();
    assert_eq!(
        config["provider"][APIKEY_FUN_PROVIDER_ID]["options"]["apiKey"],
        "fun-key"
    );
}

#[test]
fn test_humanize_joins_hyphenated_version() {
    assert_eq!(humanize_model_id("claude-sonnet-4-6"), "Claude Sonnet 4.6");
    assert_eq!(
        humanize_model_id("claude-sonnet-4-6-thinking"),
        "Claude Sonnet 4.6 Thinking"
    );
    assert_eq!(
        humanize_model_id("gemini-3.5-flash-low"),
        "Gemini 3.5 Flash Low"
    );
    assert_eq!(
        humanize_model_id("anthropic/claude-opus-4-6"),
        "Claude Opus 4.6"
    );
    assert_eq!(humanize_model_id("grok-4.20-0309"), "Grok 4.20 0309");
}

#[test]
fn test_openai_compatible_sync_matches_dotted_and_prefixed_ids() {
    let result = apply_openai_compatible_provider_sync(
        serde_json::json!({}),
        APIKEY_FUN_PROVIDER_ID,
        "APIKEY.FUN",
        "https://api.apikey.fun/v1",
        "fun-key",
        Some(&[
            minput("claude-sonnet-4.6"),
            minput("anthropic/claude-opus-4-6"),
        ]),
    );
    let models = result["provider"][APIKEY_FUN_PROVIDER_ID]["models"]
        .as_object()
        .unwrap();
    assert_eq!(
        models
            .get("claude-sonnet-4.6")
            .unwrap()
            .get("name")
            .unwrap(),
        "Claude Sonnet 4.6"
    );
    assert_eq!(
        models
            .get("anthropic/claude-opus-4-6")
            .unwrap()
            .get("name")
            .unwrap(),
        "Claude Opus 4.6"
    );
    assert_eq!(models["claude-sonnet-4.6"]["limit"]["context"], 200_000);
}

// Tests for apply_clear_to_config

#[test]
fn test_clear_removes_antigravity_provider() {
    let config = serde_json::json!({
        "provider": {
            "antigravity-manager": {
                "options": { "baseURL": "http://localhost:3000/v1" }
            },
            "google": { "options": { "apiKey": "key" } }
        }
    });

    let result = apply_clear_to_config(config, None, false);

    let provider = result.get("provider").unwrap();
    assert!(
        provider.get(ANTIGRAVITY_PROVIDER_ID).is_none(),
        "antigravity-manager should be removed"
    );
    assert!(
        provider.get("google").is_some(),
        "google should be preserved"
    );
}

#[test]
fn test_clear_legacy_removes_antigravity_models() {
    let config = serde_json::json!({
        "provider": {
            "anthropic": {
                "options": { "baseURL": "http://localhost:3000/v1", "apiKey": "key" },
                "models": {
                    "claude-sonnet-4-5": { "name": "Claude" },
                    "claude-3": { "name": "Claude 3" }
                }
            }
        }
    });

    let result = apply_clear_to_config(config, Some("http://localhost:3000"), true);

    let provider = result.get("provider").unwrap();
    let anthropic = provider.get("anthropic").unwrap();
    let models = anthropic.get("models").unwrap().as_object().unwrap();

    // Antigravity model IDs should be removed
    assert!(
        !models.contains_key("claude-sonnet-4-5"),
        "antigravity model should be removed"
    );
    // Non-antigravity models should be preserved
    assert!(
        models.contains_key("claude-3"),
        "non-antigravity model should be preserved"
    );
}

#[test]
fn test_clear_legacy_removes_options_when_baseurl_matches() {
    let config = serde_json::json!({
        "provider": {
            "anthropic": {
                "options": { "baseURL": "http://localhost:3000/v1", "apiKey": "key" }
            }
        }
    });

    let result = apply_clear_to_config(config, Some("http://localhost:3000"), true);

    let provider = result.get("provider").unwrap();
    let anthropic = provider.get("anthropic").unwrap();

    // Options should be removed when baseURL matches
    assert!(
        anthropic.get("options").is_none(),
        "options should be removed when baseURL matches"
    );
}

#[test]
fn test_clear_legacy_preserves_options_when_baseurl_different() {
    let config = serde_json::json!({
        "provider": {
            "anthropic": {
                "options": { "baseURL": "http://other-proxy.com/v1", "apiKey": "key" }
            }
        }
    });

    let result = apply_clear_to_config(config, Some("http://localhost:3000"), true);

    let provider = result.get("provider").unwrap();
    let anthropic = provider.get("anthropic").unwrap();
    let options = anthropic.get("options").unwrap();

    // Options should be preserved when baseURL doesn't match
    assert_eq!(options.get("baseURL").unwrap(), "http://other-proxy.com/v1");
    assert_eq!(options.get("apiKey").unwrap(), "key");
}

#[test]
fn test_clear_legacy_without_proxy_url_skips_cleanup() {
    let config = serde_json::json!({
        "provider": {
            "anthropic": {
                "options": { "baseURL": "http://localhost:3000/v1", "apiKey": "key" },
                "models": { "claude-sonnet-4-5": { "name": "Claude" } }
            }
        }
    });

    // clear_legacy=true but no proxy_url provided
    let result = apply_clear_to_config(config, None, true);

    let provider = result.get("provider").unwrap();
    let anthropic = provider.get("anthropic").unwrap();

    // Legacy cleanup should be skipped when proxy_url is None
    assert!(
        anthropic.get("options").is_some(),
        "options should be preserved when no proxy_url"
    );
    assert!(
        anthropic.get("models").is_some(),
        "models should be preserved when no proxy_url"
    );
}

// Tests for base_url_matches

#[test]
fn test_base_url_matches_with_v1() {
    assert!(base_url_matches(
        "http://localhost:3000/v1",
        "http://localhost:3000"
    ));
    assert!(base_url_matches(
        "http://localhost:3000",
        "http://localhost:3000/v1"
    ));
    assert!(base_url_matches(
        "http://localhost:3000/v1/",
        "http://localhost:3000"
    ));
}

#[test]
fn test_base_url_matches_without_v1() {
    assert!(base_url_matches(
        "http://localhost:3000",
        "http://localhost:3000"
    ));
    assert!(base_url_matches(
        "http://localhost:3000/",
        "http://localhost:3000/"
    ));
}

#[test]
fn test_base_url_matches_different_urls() {
    assert!(!base_url_matches(
        "http://localhost:3000",
        "http://other-host:3000"
    ));
    assert!(!base_url_matches(
        "http://localhost:3000/v1",
        "http://localhost:4000/v1"
    ));
}
