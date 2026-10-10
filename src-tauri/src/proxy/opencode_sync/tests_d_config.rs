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
fn test_clear_removes_empty_provider() {
    let config = serde_json::json!({
        "provider": {
            "antigravity-manager": {
                "options": { "baseURL": "http://localhost:3000/v1" }
            }
        }
    });

    let result = apply_clear_to_config(config, None, false);

    // Provider object should be removed when empty
    assert!(
        result.get("provider").is_none(),
        "empty provider object should be removed"
    );
}

/// Regression: model ids not present in the hardcoded catalog must still be
/// written to the config (previously silently dropped). A minimal `{ "name": ... }`
/// entry is valid per the OpenCode schema.
#[test]
fn test_sync_creates_fallback_model_for_unknown_id() {
    let config = serde_json::json!({});
    // "some-new-model" is deliberately not in build_model_catalog()
    let models_to_sync = [minput("some-new-model")];

    let result = apply_sync_to_config(
        config,
        "http://localhost:3000",
        "test-api-key",
        Some(&models_to_sync),
    );

    let models = result
        .get("provider")
        .unwrap()
        .get(ANTIGRAVITY_PROVIDER_ID)
        .unwrap()
        .get("models")
        .unwrap()
        .as_object()
        .unwrap();

    assert!(
        models.contains_key("some-new-model"),
        "unknown model id must still be written, not dropped"
    );
    let entry = models.get("some-new-model").unwrap();
    assert!(
        entry.get("name").is_some(),
        "fallback entry must at least have a name"
    );
}

/// Mixing known catalog ids with unknown ids should write all of them.
#[test]
fn test_sync_filtered_models_with_known_and_unknown_ids() {
    let config = serde_json::json!({});
    let models_to_sync = [
        minput("claude-sonnet-4-6"),
        minput("gemini-3.1-pro"),
        minput("custom-future-model"),
    ];

    let result = apply_sync_to_config(
        config,
        "http://localhost:3000",
        "test-api-key",
        Some(&models_to_sync),
    );

    let models = result
        .get("provider")
        .unwrap()
        .get(ANTIGRAVITY_PROVIDER_ID)
        .unwrap()
        .get("models")
        .unwrap()
        .as_object()
        .unwrap();

    // Known ids get full catalog metadata
    let claude = models.get("claude-sonnet-4-6").unwrap();
    assert_eq!(claude.get("name").unwrap(), "Claude Sonnet 4.6");
    assert!(claude.get("limit").is_some());

    // Unknown id gets a minimal fallback entry
    let custom = models.get("custom-future-model").unwrap();
    assert!(custom.get("name").is_some());
    assert_eq!(models["gemini-3.1-pro"]["limit"]["output"], 65_535);
}

/// Fallback entries should not clobber a model object the user already defined.
#[test]
fn test_sync_fallback_preserves_user_defined_model() {
    let config = serde_json::json!({
        "provider": {
            "antigravity-manager": {
                "models": {
                    "custom-future-model": {
                        "name": "My Custom Name",
                        "limit": { "context": 128000, "output": 4096 }
                    }
                }
            }
        }
    });
    let models_to_sync = [minput("custom-future-model")];

    let result = apply_sync_to_config(
        config,
        "http://localhost:3000",
        "test-api-key",
        Some(&models_to_sync),
    );

    let entry = result
        .get("provider")
        .unwrap()
        .get(ANTIGRAVITY_PROVIDER_ID)
        .unwrap()
        .get("models")
        .unwrap()
        .get("custom-future-model")
        .unwrap();

    // User's own name/limit must be preserved, not overwritten by the fallback.
    assert_eq!(entry.get("name").unwrap(), "My Custom Name");
    assert_eq!(
        entry.get("limit").unwrap().get("context").unwrap(),
        &serde_json::json!(128000)
    );
}

/// The fallback name derivation should produce human-readable names.
/// The fallback name derivation should preserve version dots (e.g. "3.5").
#[test]
fn test_build_fallback_model_json_readable_name() {
    // No display name: derived from id, dots preserved.
    let entry = build_fallback_model_json("gemini-2.5-pro", None);
    assert_eq!(entry.get("name").unwrap(), "Gemini 2.5 Pro");

    // 3.5 should stay together, not split into "3 5".
    let entry = build_fallback_model_json("gemini-3.5-flash-low", None);
    assert_eq!(entry.get("name").unwrap(), "Gemini 3.5 Flash Low");
}

/// When the frontend provides a display name, it must be used verbatim so the
/// config shows the same name the user saw (e.g. "Gemini 3.5 Flash (High)").
#[test]
fn test_build_fallback_model_json_uses_display_name() {
    let entry = build_fallback_model_json("gemini-3.5-flash-high", Some("Gemini 3.5 Flash (High)"));
    assert_eq!(entry.get("name").unwrap(), "Gemini 3.5 Flash (High)");
}

/// Fallback entries for known families should pick up sensible limit/modalities.
#[test]
fn test_build_fallback_model_json_series_defaults() {
    // A gemini-3.x id gets 1M context + multimodal input.
    let entry = build_fallback_model_json("gemini-3.5-flash-low", None);
    let limit = entry.get("limit").unwrap();
    assert_eq!(limit.get("context").unwrap(), &serde_json::json!(1_048_576));
    assert!(entry.get("modalities").is_some());

    // A claude id gets 200k context.
    let entry = build_fallback_model_json("claude-sonnet-4-9", None);
    let limit = entry.get("limit").unwrap();
    assert_eq!(limit.get("context").unwrap(), &serde_json::json!(200_000));

    // An unknown family (not gemini/claude) only gets a name.
    let entry = build_fallback_model_json("acme-model-1", None);
    assert!(entry.get("limit").is_none());
    assert!(entry.get("name").is_some());
}

/// resolve_active_config_file_name should prefer .jsonc when None is passed
/// (pure helper path), and the default file should be opencode.json.
#[test]
fn test_resolve_active_config_file_name_default() {
    // Without probing a directory, the default is opencode.json.
    assert_eq!(resolve_active_config_file_name(None), OPENCODE_CONFIG_FILE);
}

#[test]
fn test_resolve_active_config_file_name_prefers_existing_jsonc() {
    // Create a temp dir that has only opencode.jsonc and confirm it is preferred.
    let tmp = tempfile::tempdir().expect("create temp dir");
    let dir = tmp.path().to_path_buf();
    std::fs::write(dir.join(OPENCODE_CONFIG_FILE_JSONC), "{}").expect("write jsonc");
    assert_eq!(
        resolve_active_config_file_name(Some(&dir)),
        OPENCODE_CONFIG_FILE_JSONC
    );
}

#[test]
fn test_resolve_active_config_file_name_falls_back_to_json() {
    // A temp dir with no config files at all defaults to opencode.json.
    let tmp = tempfile::tempdir().expect("create temp dir");
    let dir = tmp.path().to_path_buf();
    assert_eq!(
        resolve_active_config_file_name(Some(&dir)),
        OPENCODE_CONFIG_FILE
    );
}

/// strip_jsonc_comments must remove line and block comments while preserving
/// `//` and `/* */` that appear inside string values.
#[test]
fn test_strip_jsonc_comments_line_and_block() {
    let input = r#"{
  // a line comment
  "key": "value", /* trailing block comment */
  "url": "https://example.com/path" // not a comment inside string
}"#;
    let stripped = strip_jsonc_comments(input);
    // The resulting string must be valid JSON.
    let parsed: Value = serde_json::from_str(&stripped).expect("stripped jsonc must be valid JSON");
    assert_eq!(parsed.get("key").unwrap(), "value");
    // The URL's // must be preserved (it is inside a string).
    assert_eq!(parsed.get("url").unwrap(), "https://example.com/path");
}

#[test]
fn test_strip_jsonc_comments_preserves_escaped_quotes() {
    // A string containing an escaped quote followed by // must not confuse the scanner.
    let input = r##"{"msg": "a \"quoted//\" end", "n": 1 // comment
}"##;
    let stripped = strip_jsonc_comments(input);
    let parsed: Value = serde_json::from_str(&stripped).expect("stripped jsonc must be valid JSON");
    assert_eq!(parsed.get("msg").unwrap(), "a \"quoted//\" end");
    assert_eq!(parsed.get("n").unwrap(), 1);
}

/// strip_jsonc_trailing_commas must drop a comma right before `}` or `]` (with
/// optional whitespace between), while keeping commas that separate real elements.
#[test]
fn test_strip_jsonc_trailing_commas() {
    let input = "{\n  \"a\": 1,\n  \"b\": 2,\n}"; // trailing comma before }
    let stripped = strip_jsonc_trailing_commas(input);
    let parsed: Value = serde_json::from_str(&stripped).expect("stripped must be valid JSON");
    assert_eq!(parsed.get("a").unwrap(), 1);
    assert_eq!(parsed.get("b").unwrap(), 2);
}

#[test]
fn test_strip_jsonc_trailing_commas_in_nested_array() {
    // trailing comma before ] and before }, with whitespace/newlines in between.
    let input = "{\n  \"arr\": [1, 2, 3,],\n  \"obj\": { \"k\": \"v\", },\n}";
    let stripped = strip_jsonc_trailing_commas(input);
    let parsed: Value = serde_json::from_str(&stripped).expect("stripped must be valid JSON");
    assert_eq!(parsed.get("arr").unwrap().as_array().unwrap().len(), 3);
    assert_eq!(parsed.get("obj").unwrap().get("k").unwrap(), "v");
}

#[test]
fn test_strip_jsonc_trailing_commas_keeps_real_commas_in_strings() {
    // A comma inside a string must NOT be removed even if followed by }.
    let input = "{\"msg\": \"hello, }\",}";
    let stripped = strip_jsonc_trailing_commas(input);
    let parsed: Value = serde_json::from_str(&stripped).expect("stripped must be valid JSON");
    assert_eq!(parsed.get("msg").unwrap(), "hello, }");
}

/// parse_config_file must read a jsonc file with BOTH comments and trailing commas
/// — this is the regression for the user-reported "config destroyed" case where a
/// trailing comma made serde_json fail and the whole config was replaced with {}.
#[test]
fn test_parse_config_file_handles_jsonc_comments_and_trailing_commas() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let path = tmp.path().join("opencode.jsonc");
    std::fs::write(
        &path,
        r#"{
  // user's config with a trailing comma (valid JSONC, invalid strict JSON)
  "$schema": "https://opencode.ai/config.json",
  "provider": {
    "myown": { "name": "My Own Provider" },
  },
}"#,
    )
    .unwrap();

    let parsed = parse_config_file(&path).expect("jsonc with trailing comma must parse");
    assert_eq!(
        parsed
            .get("provider")
            .unwrap()
            .get("myown")
            .unwrap()
            .get("name")
            .unwrap(),
        "My Own Provider"
    );
}

/// parse_config_file must read a jsonc file (with comments) into a Value.
#[test]
fn test_parse_config_file_handles_jsonc_comments() {
    let tmp = tempfile::tempdir().expect("create temp dir");
    let path = tmp.path().join("opencode.jsonc");
    std::fs::write(
        &path,
        r#"{
  // my opencode config
  "$schema": "https://opencode.ai/config.json",
  "provider": {
    "myown": { "name": "My Own Provider" }
  }
}"#,
    )
    .unwrap();

    let parsed = parse_config_file(&path).expect("jsonc file must parse");
    assert_eq!(
        parsed
            .get("provider")
            .unwrap()
            .get("myown")
            .unwrap()
            .get("name")
            .unwrap(),
        "My Own Provider"
    );
}

/// The frontend-provided display name must flow through to the config for models
/// that aren't in the catalog. This is the regression for the user-reported case
/// where "Gemini 3.5 Flash (High)" became a stripped "Gemini 3 5 Flash Low".
#[test]
fn test_sync_uses_frontend_display_name_for_unknown_model() {
    let config = serde_json::json!({});
    let models_to_sync = [
        minput_named("custom-unknown-flash", "Custom Unknown Flash (High)"),
        minput_named("custom-unknown-agent", "Custom Unknown Agent"),
    ];

    let result = apply_sync_to_config(
        config,
        "http://localhost:3000",
        "test-api-key",
        Some(&models_to_sync),
    );

    let models = result
        .get("provider")
        .unwrap()
        .get(ANTIGRAVITY_PROVIDER_ID)
        .unwrap()
        .get("models")
        .unwrap()
        .as_object()
        .unwrap();

    // The display name must be used as-is, preserving parentheses/variant info.
    assert_eq!(
        models
            .get("custom-unknown-flash")
            .unwrap()
            .get("name")
            .unwrap(),
        "Custom Unknown Flash (High)"
    );
    assert_eq!(
        models
            .get("custom-unknown-agent")
            .unwrap()
            .get("name")
            .unwrap(),
        "Custom Unknown Agent"
    );
}
