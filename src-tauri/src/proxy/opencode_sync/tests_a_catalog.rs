use super::*;
use crate::proxy::common::variant_mapping::{resolve_real_model, VariantTier, GEMINI_FAMILIES};

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
fn merge_catalog_models_normalizes_aliases_to_canonical() {
    let mut provider = serde_json::json!({ "models": {} });
    let inputs = [minput("gemini-3.1-pro-high"), minput("gemini-3.1-pro-low")];

    merge_catalog_models(&mut provider, Some(&inputs));

    let models = provider["models"].as_object().expect("models object");
    assert_eq!(models.len(), 1);
    assert!(models.contains_key("gemini-3.1-pro"));
    assert!(!models.contains_key("gemini-3.1-pro-high"));
    assert!(!models.contains_key("gemini-3.1-pro-low"));
}

#[test]
fn merge_catalog_models_keeps_one_pro_and_one_flash_for_legacy_inputs() {
    let mut provider = serde_json::json!({ "models": {} });
    let inputs = [
        minput("gemini-3.1-pro-high"),
        minput("gemini-3.1-pro-low"),
        minput("gemini-3-flash"),
    ];

    merge_catalog_models(&mut provider, Some(&inputs));

    let models = provider["models"].as_object().expect("models object");
    assert_eq!(models.len(), 2);
    assert!(models.contains_key("gemini-3.1-pro"));
    assert!(models.contains_key("gemini-3.5-flash"));
}

#[test]
fn apply_sync_migrates_old_alias_keys_to_canonical() {
    let config = serde_json::json!({
        "provider": {
            ANTIGRAVITY_PROVIDER_ID: {
                "models": {
                    "gemini-3.1-pro-high": { "from_high": true },
                    "gemini-3.1-pro-low": { "from_low": true },
                    "custom-model": { "preserved": true }
                }
            }
        }
    });

    let updated = apply_sync_to_config(config, "http://localhost:8045", "key", Some(&[]));
    let models = updated["provider"][ANTIGRAVITY_PROVIDER_ID]["models"]
        .as_object()
        .expect("models object");
    let canonical = models
        .get("gemini-3.1-pro")
        .and_then(Value::as_object)
        .expect("canonical model");

    assert_eq!(canonical.get("from_high"), Some(&Value::Bool(true)));
    assert_eq!(canonical.get("from_low"), Some(&Value::Bool(true)));
    assert!(!models.contains_key("gemini-3.1-pro-high"));
    assert!(!models.contains_key("gemini-3.1-pro-low"));
    assert_eq!(
        models.get("custom-model"),
        Some(&serde_json::json!({ "preserved": true }))
    );
}

#[test]
fn apply_sync_keeps_non_conflicting_user_fields() {
    let config = serde_json::json!({
        "provider": {
            ANTIGRAVITY_PROVIDER_ID: {
                "models": {
                    "gemini-3.1-pro": { "canonical_only": "keep" },
                    "gemini-3.1-pro-high": { "alias_only": "also keep" }
                }
            }
        }
    });

    let updated = apply_sync_to_config(config, "http://localhost:8045", "key", Some(&[]));
    let canonical = updated["provider"][ANTIGRAVITY_PROVIDER_ID]["models"]["gemini-3.1-pro"]
        .as_object()
        .expect("canonical model");

    assert_eq!(
        canonical.get("canonical_only"),
        Some(&Value::String("keep".to_string()))
    );
    assert_eq!(
        canonical.get("alias_only"),
        Some(&Value::String("also keep".to_string()))
    );
}

#[test]
fn apply_sync_warns_on_conflicts_and_canonical_wins() {
    use std::sync::{Arc, Mutex};
    use tracing::{Event, Level, Subscriber};
    use tracing_subscriber::{
        layer::{Context, SubscriberExt},
        registry::LookupSpan,
        Layer,
    };

    #[derive(Clone)]
    struct WarnCapture(Arc<Mutex<usize>>);

    impl<S> Layer<S> for WarnCapture
    where
        S: Subscriber + for<'lookup> LookupSpan<'lookup>,
    {
        fn on_event(&self, event: &Event<'_>, _: Context<'_, S>) {
            if *event.metadata().level() == Level::WARN {
                *self.0.lock().expect("warning counter lock") += 1;
            }
        }
    }

    let config = serde_json::json!({
        "provider": {
            ANTIGRAVITY_PROVIDER_ID: {
                "models": {
                    "gemini-3.1-pro": { "custom": "canonical" },
                    "gemini-3.1-pro-high": { "custom": "alias" }
                }
            }
        }
    });
    let warnings = Arc::new(Mutex::new(0));
    let subscriber = tracing_subscriber::registry().with(WarnCapture(warnings.clone()));
    let updated = tracing::subscriber::with_default(subscriber, || {
        apply_sync_to_config(config, "http://localhost:8045", "key", Some(&[]))
    });

    assert_eq!(
        updated["provider"][ANTIGRAVITY_PROVIDER_ID]["models"]["gemini-3.1-pro"]["custom"],
        "canonical"
    );
    assert_eq!(*warnings.lock().expect("warning counter lock"), 1);
}

fn preserved_catalog_json_snapshot(models: &[ModelDef]) -> String {
    let entries = models
        .iter()
        .filter(|model| {
            model.id.starts_with("claude-")
                || model.id == "gemini-3-pro-image"
                || model.id.starts_with("gemini-2.5-")
        })
        .map(|model| serde_json::json!({ "id": model.id, "model": build_model_json(model) }))
        .collect::<Vec<_>>();

    serde_json::to_string(&entries).unwrap()
}

#[test]
fn catalog_preserves_non_gemini3_variant_json_snapshot() {
    let expected = vec![
        ModelDef {
            id: "claude-sonnet-4-6",
            name: "Claude Sonnet 4.6",
            context_limit: 200_000,
            output_limit: 64_000,
            input_modalities: &["text", "image", "pdf"],
            output_modalities: &["text"],
            reasoning: true,
            variant_type: Some(VariantType::ClaudeThinking),
        },
        ModelDef {
            id: "claude-sonnet-4-6-thinking",
            name: "Claude Sonnet 4.6 Thinking",
            context_limit: 200_000,
            output_limit: 64_000,
            input_modalities: &["text", "image", "pdf"],
            output_modalities: &["text"],
            reasoning: true,
            variant_type: Some(VariantType::ClaudeThinking),
        },
        ModelDef {
            id: "claude-sonnet-4-5",
            name: "Claude Sonnet 4.5",
            context_limit: 200_000,
            output_limit: 64_000,
            input_modalities: &["text", "image", "pdf"],
            output_modalities: &["text"],
            reasoning: true,
            variant_type: Some(VariantType::ClaudeThinking),
        },
        ModelDef {
            id: "claude-sonnet-4-5-thinking",
            name: "Claude Sonnet 4.5 Thinking",
            context_limit: 200_000,
            output_limit: 64_000,
            input_modalities: &["text", "image", "pdf"],
            output_modalities: &["text"],
            reasoning: true,
            variant_type: Some(VariantType::ClaudeThinking),
        },
        ModelDef {
            id: "claude-opus-4-5",
            name: "Claude Opus 4.5",
            context_limit: 200_000,
            output_limit: 64_000,
            input_modalities: &["text", "image", "pdf"],
            output_modalities: &["text"],
            reasoning: true,
            variant_type: Some(VariantType::ClaudeThinking),
        },
        ModelDef {
            id: "claude-opus-4-5-thinking",
            name: "Claude Opus 4.5 Thinking",
            context_limit: 200_000,
            output_limit: 64_000,
            input_modalities: &["text", "image", "pdf"],
            output_modalities: &["text"],
            reasoning: true,
            variant_type: Some(VariantType::ClaudeThinking),
        },
        ModelDef {
            id: "claude-opus-4-6",
            name: "Claude Opus 4.6",
            context_limit: 200_000,
            output_limit: 64_000,
            input_modalities: &["text", "image", "pdf"],
            output_modalities: &["text"],
            reasoning: true,
            variant_type: Some(VariantType::ClaudeThinking),
        },
        ModelDef {
            id: "claude-opus-4-6-thinking",
            name: "Claude Opus 4.6 Thinking",
            context_limit: 200_000,
            output_limit: 64_000,
            input_modalities: &["text", "image", "pdf"],
            output_modalities: &["text"],
            reasoning: true,
            variant_type: Some(VariantType::ClaudeThinking),
        },
        ModelDef {
            id: "gemini-3-pro-image",
            name: "Gemini 3 Pro Image",
            context_limit: 1_048_576,
            output_limit: 65_535,
            input_modalities: &["text", "image", "pdf"],
            output_modalities: &["text", "image"],
            reasoning: false,
            variant_type: None,
        },
        ModelDef {
            id: "gemini-2.5-flash",
            name: "Gemini 2.5 Flash",
            context_limit: 1_048_576,
            output_limit: 65_536,
            input_modalities: &["text", "image", "pdf"],
            output_modalities: &["text"],
            reasoning: false,
            variant_type: None,
        },
        ModelDef {
            id: "gemini-2.5-flash-lite",
            name: "Gemini 2.5 Flash Lite",
            context_limit: 1_048_576,
            output_limit: 65_536,
            input_modalities: &["text", "image", "pdf"],
            output_modalities: &["text"],
            reasoning: false,
            variant_type: None,
        },
        ModelDef {
            id: "gemini-2.5-flash-thinking",
            name: "Gemini 2.5 Flash Thinking",
            context_limit: 1_048_576,
            output_limit: 65_536,
            input_modalities: &["text", "image", "pdf"],
            output_modalities: &["text"],
            reasoning: true,
            variant_type: Some(VariantType::Gemini25Thinking),
        },
        ModelDef {
            id: "gemini-2.5-pro",
            name: "Gemini 2.5 Pro",
            context_limit: 1_048_576,
            output_limit: 65_536,
            input_modalities: &["text", "image", "pdf"],
            output_modalities: &["text"],
            reasoning: true,
            variant_type: None,
        },
    ];

    assert_eq!(
        preserved_catalog_json_snapshot(&build_model_catalog()),
        preserved_catalog_json_snapshot(&expected)
    );
}

#[test]
fn catalog_uses_canonical_gemini_family_entries() {
    let catalog = build_model_catalog();
    let variant_ids = catalog
        .iter()
        .filter(|model| {
            matches!(
                model.variant_type,
                Some(VariantType::Gemini3Pro) | Some(VariantType::Gemini3Flash)
            )
        })
        .map(|model| model.id)
        .collect::<std::collections::BTreeSet<_>>();
    let canonical_ids = GEMINI_FAMILIES
        .iter()
        .map(|family| family.canonical_id)
        .collect::<std::collections::BTreeSet<_>>();

    assert_eq!(variant_ids, canonical_ids);

    for family in GEMINI_FAMILIES {
        let model = catalog
            .iter()
            .find(|model| model.id == family.canonical_id)
            .unwrap();
        assert_eq!(model.name, family.display_name);
        assert_eq!(model.context_limit, family.context_limit);
        assert_eq!(model.output_limit, family.output_limit);
        assert_eq!(model.input_modalities, family.input_modalities);
        assert_eq!(model.output_modalities, family.output_modalities);
        assert_eq!(model.reasoning, family.reasoning);

        match family.canonical_id {
            "gemini-3.1-pro" => {
                assert!(matches!(model.variant_type, Some(VariantType::Gemini3Pro)));
            }
            "gemini-3.7-flash" | "gemini-3.5-flash" => {
                assert!(matches!(
                    model.variant_type,
                    Some(VariantType::Gemini3Flash)
                ));
            }
            _ => panic!("unexpected Gemini family: {}", family.canonical_id),
        }
    }
}

#[test]
fn catalog_marks_gemini_31_flash_lite_as_non_variant() {
    let catalog = build_model_catalog();
    let flash_lite = catalog
        .iter()
        .find(|model| model.id == "gemini-3.1-flash-lite")
        .unwrap();

    assert!(matches!(flash_lite.variant_type, None));
}

/// Map an Anthropic SDK effort string to the internal VariantTier.
fn effort_to_tier(effort: &str) -> VariantTier {
    match effort {
        "low" => VariantTier::Low,
        "medium" => VariantTier::Medium,
        "high" => VariantTier::High,
        _ => panic!("unrecognized effort variant: {effort}"),
    }
}

#[test]
fn build_variants_pro_resolve_to_real_ids() {
    let variants = build_variants_object(Some(VariantType::Gemini3Pro))
        .expect("Gemini 3.1 Pro variants must be configured");

    for (name, expected_id) in [("low", "gemini-3.1-pro-low"), ("high", "gemini-pro-agent")] {
        let variant = &variants[name];
        let effort = variant["effort"]
            .as_str()
            .expect("Gemini 3 variant must expose effort string");

        let tier = effort_to_tier(effort);
        assert_eq!(
            resolve_real_model("gemini-3.1-pro", tier)
                .expect("Gemini 3.1 Pro tier must resolve")
                .id,
            expected_id
        );
    }

    assert_eq!(
        variants
            .as_object()
            .expect("variants must be an object")
            .len(),
        4
    );
    assert_eq!(variants["medium"]["disabled"], true);
    assert_eq!(variants["max"]["disabled"], true);

    // Verify the JSON shape contains only `effort` — no budget fields
    let low = &variants["low"];
    assert_eq!(low.as_object().unwrap().len(), 1);
    assert!(low.get("effort").is_some());
    assert!(low.get("thinking").is_none());
}

#[test]
fn build_variants_flash_resolve_to_real_ids() {
    let variants = build_variants_object(Some(VariantType::Gemini3Flash))
        .expect("Gemini 3.5 Flash variants must be configured");

    for (name, expected_id) in [
        ("low", "gemini-3.5-flash-extra-low"),
        ("medium", "gemini-3.5-flash-low"),
        ("high", "gemini-3-flash-agent"),
    ] {
        let variant = &variants[name];
        let effort = variant["effort"]
            .as_str()
            .expect("Gemini 3 variant must expose effort string");

        let tier = effort_to_tier(effort);
        assert_eq!(
            resolve_real_model("gemini-3.5-flash", tier)
                .expect("Gemini 3.5 Flash tier must resolve")
                .id,
            expected_id
        );
    }

    assert_eq!(
        variants
            .as_object()
            .expect("variants must be an object")
            .len(),
        4
    );
    assert_eq!(variants["max"]["disabled"], true);

    // Verify the JSON shape contains only `effort` — no budget fields
    let low = &variants["low"];
    assert_eq!(low.as_object().unwrap().len(), 1);
    assert!(low.get("effort").is_some());
    assert!(low.get("thinking").is_none());
}
