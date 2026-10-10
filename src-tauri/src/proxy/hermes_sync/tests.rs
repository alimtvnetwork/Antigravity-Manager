use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    fn version_probe_test_command(mode: &str) -> Command {
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args([
                "--exact",
                "proxy::hermes_sync::tests::version_probe_subprocess_helper",
                "--nocapture",
            ])
            .env("ANTIGRAVITY_HERMES_VERSION_PROBE_TEST", mode);
        command
    }

    #[test]
    fn version_probe_subprocess_helper() {
        match std::env::var("ANTIGRAVITY_HERMES_VERSION_PROBE_TEST").as_deref() {
            Ok("success") => println!("Hermes Agent v0.21.3 (2026.9.14)"),
            Ok("timeout") => std::thread::sleep(Duration::from_secs(60)),
            _ => {}
        }
    }

    #[tokio::test]
    async fn version_probe_extracts_version_from_successful_subprocess() {
        assert_eq!(
            run_version_command(
                version_probe_test_command("success"),
                Duration::from_secs(10)
            )
            .await
            .as_deref(),
            Some("0.21.3")
        );
    }

    #[tokio::test]
    async fn version_probe_times_out_hung_subprocess() {
        assert!(run_version_command(
            version_probe_test_command("timeout"),
            Duration::from_millis(100)
        )
        .await
        .is_none());
    }

    fn doc(source: &str) -> YamlDoc {
        parse_doc(source).expect("valid yaml")
    }

    #[test]
    fn extract_version_accepts_v_prefixed_hermes_output() {
        assert_eq!(
            extract_version("Hermes Agent v0.21.3 (2026.9.14)"),
            "0.21.3"
        );
        assert_eq!(extract_version("hermes/V1.2.0"), "1.2.0");
    }

    #[test]
    fn sync_creates_provider_with_selected_models() {
        let models = vec!["gemini-2.5-pro".into(), "claude-sonnet-4-6".into()];
        let updated = apply_sync_losslessly(
            EMPTY_CONFIG,
            "http://127.0.0.1:8045/v1",
            "sk-test",
            false,
            &models,
            false,
            None,
            None,
        )
        .unwrap();
        let parsed = doc(&updated);
        assert_eq!(
            scalar_at(&parsed, "/providers/antigravity-manager/api").as_deref(),
            Some("http://127.0.0.1:8045/v1")
        );
        assert_eq!(
            string_list_at(&parsed, "/providers/antigravity-manager/models"),
            models
        );
    }

    #[test]
    fn sync_preserves_comments_formatting_and_unrelated_providers() {
        let source = "# user header\nproviders: # provider registry\n  other:\n    api: 'https://example.test/v1' # untouched\n  antigravity-manager:\n    name: Old Name # managed comment\n    api: http://old.test/v1 # endpoint comment\n    api_key: old-key # credential comment\n    transport: chat_completions\n    discover_models: false\n    models:\n      - old-model # model comment\ndisplay: {theme: custom} # flow stays flow\n";
        let updated = apply_sync_losslessly(
            source,
            "http://127.0.0.1:8045/v1",
            "sk-new",
            false,
            &["new-model".into()],
            false,
            None,
            None,
        )
        .unwrap();
        for expected in [
            "# user header\nproviders: # provider registry",
            "api: 'https://example.test/v1' # untouched",
            "name: Antigravity Manager # managed comment",
            "api: http://127.0.0.1:8045/v1 # endpoint comment",
            "api_key: sk-new # credential comment",
            "- new-model # model comment",
            "display: {theme: custom} # flow stays flow",
        ] {
            assert!(
                updated.contains(expected),
                "missing {expected:?}\n{updated}"
            );
        }
    }

    #[test]
    fn identical_sync_is_byte_for_byte_unchanged() {
        let source = "# exact bytes\nproviders:\n  antigravity-manager:\n    name: 'Antigravity Manager'\n    api: http://127.0.0.1:8045/v1\n    api_key: sk-test\n    transport: chat_completions\n    discover_models: true\nother: 0x10 # spelling\n";
        assert_eq!(
            apply_sync_losslessly(
                source,
                "http://127.0.0.1:8045/v1",
                "sk-test",
                true,
                &[],
                false,
                None,
                None
            )
            .unwrap(),
            source
        );
    }

    #[test]
    fn sync_accepts_and_preserves_hermes_indentless_toolset_sequences() {
        let toolsets = "platform_toolsets:\n  cli:\n  - hermes-cli\n  telegram:\n  - hermes-telegram\n  discord:\n  - hermes-discord\n";
        let source =
            format!("{toolsets}providers:\n  existing:\n    api: https://example.test/v1\n");
        let updated = apply_sync_losslessly(
            &source,
            "http://127.0.0.1:8045/v1",
            "sk-test",
            true,
            &[],
            false,
            None,
            None,
        )
        .unwrap();

        assert!(updated.starts_with(toolsets));
        assert!(updated.contains("  antigravity-manager:\n"));
        assert!(parse_doc(&updated).is_ok());
    }

    #[test]
    fn activation_preserves_model_comments_and_unknown_settings() {
        let source = "model:\n  provider: openrouter # selected provider\n  default: old-model # selected model\n  fallback: keep-me\n";
        let updated = apply_sync_losslessly(
            source,
            "http://127.0.0.1:8045/v1",
            "sk-test",
            true,
            &[],
            true,
            Some("gemini-2.5-pro"),
            None,
        )
        .unwrap();
        let parsed = doc(&updated);
        assert_eq!(
            scalar_at(&parsed, "/model/provider").as_deref(),
            Some(PROVIDER_REF)
        );
        assert_eq!(
            scalar_at(&parsed, "/model/fallback").as_deref(),
            Some("keep-me")
        );
        assert!(updated.contains("provider: custom:antigravity-manager # selected provider"));
        assert!(updated.contains("default: gemini-2.5-pro # selected model"));
    }

    #[test]
    fn deactivation_restores_previous_provider() {
        let backup = "model:\n  provider: openai-codex # original provider\n  default: gpt-5-codex # original model\n";
        let current = "model:\n  provider: custom:antigravity-manager # original provider\n  default: gemini-3.8-flash-high # original model\n  fallback: keep-me\n";
        let updated = apply_sync_losslessly(
            current,
            "http://127.0.0.1:8045/v1",
            "sk-test",
            true,
            &[],
            false,
            None,
            Some(backup),
        )
        .unwrap();
        let parsed = doc(&updated);
        assert_eq!(
            scalar_at(&parsed, "/model/provider").as_deref(),
            Some("openai-codex")
        );
        assert_eq!(
            scalar_at(&parsed, "/model/default").as_deref(),
            Some("gpt-5-codex")
        );
        assert!(updated.contains("provider: openai-codex # original provider"));
    }

    #[test]
    fn deactivation_without_safe_backup_removes_only_managed_selection() {
        let current = "model:\n  provider: custom:antigravity-manager\n  default: managed-model\n  fallback: keep-me\n";
        let updated = apply_sync_losslessly(
            current,
            "http://127.0.0.1:8045/v1",
            "sk-test",
            true,
            &[],
            false,
            None,
            None,
        )
        .unwrap();
        let parsed = doc(&updated);
        assert!(scalar_at(&parsed, "/model/provider").is_none());
        assert!(scalar_at(&parsed, "/model/default").is_none());
        assert_eq!(
            scalar_at(&parsed, "/model/fallback").as_deref(),
            Some("keep-me")
        );
    }

    #[test]
    fn deactivation_restores_model_with_implicit_provider() {
        let backup = "model:\n  default: original-model\n";
        let current = "model:\n  provider: custom:antigravity-manager\n  default: managed-model\n  fallback: keep-me\n";
        let mut parsed = doc(current);
        deactivate(&mut parsed, Some(&doc(backup))).unwrap();
        assert!(scalar_at(&parsed, "/model/provider").is_none());
        assert_eq!(
            scalar_at(&parsed, "/model/default").as_deref(),
            Some("original-model")
        );
        assert_eq!(
            scalar_at(&parsed, "/model/fallback").as_deref(),
            Some("keep-me")
        );
        let restored = doc(&apply_restore_losslessly(
            "model:\n  provider: custom:antigravity-manager\n  default: managed-model\n",
            backup,
        )
        .unwrap());
        assert!(scalar_at(&restored, "/model/provider").is_none());
        assert_eq!(
            scalar_at(&restored, "/model/default").as_deref(),
            Some("original-model")
        );
    }

    #[test]
    fn first_sync_backup_preserves_empty_baseline() {
        let directory = std::env::temp_dir().join(format!("hermes-test-{}", uuid::Uuid::new_v4()));
        let path = directory.join(HERMES_CONFIG_FILE);
        let backup = directory.join(format!("{HERMES_CONFIG_FILE}{BACKUP_SUFFIX}"));
        create_backup(&path).unwrap();
        assert_eq!(fs::read_to_string(&backup).unwrap(), EMPTY_CONFIG);
        let current = apply_sync_losslessly(
            EMPTY_CONFIG,
            "http://127.0.0.1:8045/v1",
            "sk-test",
            true,
            &[],
            true,
            Some("test-model"),
            Some(EMPTY_CONFIG),
        )
        .unwrap();
        atomically_write_source(&path, &current).unwrap();
        create_backup(&path).unwrap();
        let baseline = fs::read_to_string(&backup).unwrap();
        assert_eq!(baseline, EMPTY_CONFIG);
        let restored = doc(&apply_restore_losslessly(&current, &baseline).unwrap());
        assert!(resolve_optional(&restored, "/providers").is_none());
        assert!(resolve_optional(&restored, "/model").is_none());
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn clear_preserves_unrelated_bytes() {
        let source = "# top\nproviders:\n  antigravity-manager:\n    api: http://local/v1\n  other: {api: https://example.test/v1} # keep\ndisplay:\n  theme: custom # keep comment\n";
        let (updated, changed) = apply_clear_losslessly(source, None).unwrap();
        assert!(changed);
        assert_eq!(
            updated,
            "# top\nproviders:\n  other: {api: https://example.test/v1} # keep\ndisplay:\n  theme: custom # keep comment\n"
        );
    }

    #[test]
    fn clear_automatically_deactivates_managed_provider_with_backup() {
        let backup = "model:\n  provider: openrouter\n  default: anthropic/claude-3.5-sonnet\nproviders:\n  openrouter:\n    api_key: test\n";
        let source = "model:\n  provider: custom:antigravity-manager\n  default: gemini-2.5-flash\nproviders:\n  antigravity-manager:\n    name: Antigravity Manager\n  openrouter:\n    api_key: test\n";
        let (updated, changed) = apply_clear_losslessly(source, Some(backup)).unwrap();
        assert!(changed);
        let parsed = doc(&updated);
        assert_eq!(
            scalar_at(&parsed, "/model/provider").as_deref(),
            Some("openrouter")
        );
        assert_eq!(
            scalar_at(&parsed, "/model/default").as_deref(),
            Some("anthropic/claude-3.5-sonnet")
        );
        assert!(resolve_optional(&parsed, "/providers/antigravity-manager").is_none());
        assert!(resolve_optional(&parsed, "/providers/openrouter").is_some());
    }

    #[test]
    fn clear_automatically_deactivates_managed_provider_without_backup() {
        let source = "model:\n  provider: custom:antigravity-manager\n  default: gemini-2.5-flash\nproviders:\n  antigravity-manager:\n    name: Antigravity Manager\n";
        let (updated, changed) = apply_clear_losslessly(source, None).unwrap();
        assert!(changed);
        let parsed = doc(&updated);
        assert!(resolve_optional(&parsed, "/model").is_none());
        assert!(resolve_optional(&parsed, "/providers").is_none());
    }

    #[test]
    fn clear_deactivates_managed_provider_retaining_other_model_fields_when_no_backup() {
        let source = "model:\n  provider: custom:antigravity-manager\n  default: gemini-2.5-flash\n  temperature: 0.7\nproviders:\n  antigravity-manager:\n    name: Antigravity Manager\n";
        let (updated, changed) = apply_clear_losslessly(source, None).unwrap();
        assert!(changed);
        let parsed = doc(&updated);
        assert!(resolve_optional(&parsed, "/model/provider").is_none());
        assert!(resolve_optional(&parsed, "/model/default").is_none());
        assert_eq!(
            scalar_at(&parsed, "/model/temperature").as_deref(),
            Some("0.7")
        );
        assert!(resolve_optional(&parsed, "/providers").is_none());
    }

    #[test]
    fn restore_preserves_unrelated_current_changes() {
        let backup = "providers:\n  antigravity-manager:\n    # original provider comment\n    api: https://old.example/v1\n    custom: keep-original\nmodel:\n  provider: openrouter\n  default: old-model\n";
        let current = "providers:\n  antigravity-manager:\n    # original provider comment\n    api: http://127.0.0.1:8045/v1\n    api_key: sk-test\n  other:\n    api: https://new.example/v1 # changed later\n  added-later:\n    api: https://added.example/v1\nmodel:\n  provider: custom:antigravity-manager\n  default: gemini-2.5-pro\n  fallback: keep-me\ndisplay:\n  theme: custom-theme\n";
        let restored = apply_restore_losslessly(current, backup).unwrap();
        let parsed = doc(&restored);
        assert_eq!(
            scalar_at(&parsed, "/providers/antigravity-manager/api").as_deref(),
            Some("https://old.example/v1")
        );
        assert_eq!(
            scalar_at(&parsed, "/providers/other/api").as_deref(),
            Some("https://new.example/v1")
        );
        assert_eq!(
            scalar_at(&parsed, "/model/provider").as_deref(),
            Some("openrouter")
        );
        assert_eq!(
            scalar_at(&parsed, "/model/fallback").as_deref(),
            Some("keep-me")
        );
        assert!(
            restored.contains("# original provider comment"),
            "{restored}"
        );
        assert!(restored.contains("added-later:"));
    }

    #[test]
    fn redaction_preserves_comments_and_formatting() {
        let source = "# credentials\nproviders:\n  antigravity-manager:\n    api_key: 'secret-value' # never expose\n    max_tokens: 4096\ngateway: {telegram_bot_token: bot-secret, enabled: true} # flow\n";
        let redacted = redact_sensitive_source(source).unwrap();
        assert!(redacted.contains("api_key: '[REDACTED]' # never expose"));
        assert!(redacted.contains("max_tokens: 4096"));
        assert!(redacted.contains("telegram_bot_token: [REDACTED]"));
        assert!(redacted.contains("enabled: true} # flow"));
        assert!(!redacted.contains("secret-value"));
        assert!(!redacted.contains("bot-secret"));
    }

    #[test]
    fn sync_preserves_crlf_line_endings() {
        let source = "# windows\r\nproviders:\r\n  other:\r\n    api: https://example.test/v1\r\n";
        let updated = apply_sync_losslessly(
            source,
            "http://127.0.0.1:8045/v1",
            "sk-test",
            true,
            &[],
            false,
            None,
            None,
        )
        .unwrap();
        assert!(!updated.replace("\r\n", "").contains('\n'), "{updated:?}");
        assert!(updated.contains("\r\n  antigravity-manager:\r\n"));
    }

    #[test]
    fn sync_repairs_malformed_managed_shapes() {
        let updated = apply_sync_losslessly(
            "providers: broken\nmodel: keep-me\n",
            "http://127.0.0.1:8045/v1",
            "sk-test",
            true,
            &[],
            true,
            Some("gemini-2.5-pro"),
            None,
        )
        .unwrap();
        let parsed = doc(&updated);
        assert_eq!(
            scalar_at(&parsed, "/providers/antigravity-manager/api").as_deref(),
            Some("http://127.0.0.1:8045/v1")
        );
        assert_eq!(
            scalar_at(&parsed, "/model/provider").as_deref(),
            Some(PROVIDER_REF)
        );
    }

    #[test]
    fn malformed_yaml_is_rejected_before_editing() {
        assert!(apply_sync_losslessly(
            "providers: [broken\n",
            "http://127.0.0.1:8045/v1",
            "sk-test",
            true,
            &[],
            false,
            None,
            None,
        )
        .is_err());
    }

    #[test]
    fn normalize_base_url_appends_v1_once() {
        assert_eq!(
            normalize_base_url("http://127.0.0.1:8045"),
            "http://127.0.0.1:8045/v1"
        );
        assert_eq!(
            normalize_base_url("http://127.0.0.1:8045/v1/"),
            "http://127.0.0.1:8045/v1"
        );
    }
}
