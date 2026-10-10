use super::catalog::{build_model_catalog, ModelDef};
use super::sync_helpers::build_model_json;
use crate::proxy::common::variant_mapping::GEMINI_FAMILIES;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

/// A model to sync, optionally carrying the display name from the frontend.
/// When `name` is provided (the common case from the OpenCode sync modal), it is used
/// verbatim so the config shows the same recognizable name the user saw in the UI.
/// When absent (e.g. a plain id list), a name is derived from the id.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelInput {
    pub id: String,
    pub name: Option<String>,
}

/// Sensible per-series defaults derived from the model id prefix. Real model metadata
/// lives in the catalog; this is only used for ids the catalog doesn't know (e.g. a
/// newly released model or an account-specific variant), so the entry still gets useful
/// limit/modalities instead of a bare `{ "name": ... }`. Returns None for unknown
/// families, in which case only the name is written.
fn series_defaults_for(model_id: &str) -> Option<Value> {
    let id = model_id.to_lowercase();
    // Gemini 3.x / 3.5 / 3.1 family: 1M context, 64k output, multimodal in, text out.
    if id.starts_with("gemini-3") || id.starts_with("gemini-3.1") || id.starts_with("gemini-3.5") {
        return Some(serde_json::json!({
            "limit": { "context": 1_048_576, "output": 65_536 },
            "modalities": { "input": ["text", "image", "pdf"], "output": ["text"] },
        }));
    }
    // Gemini 2.5 family: same shape.
    if id.starts_with("gemini-2.5") || id.starts_with("gemini-2.0") {
        return Some(serde_json::json!({
            "limit": { "context": 1_048_576, "output": 65_536 },
            "modalities": { "input": ["text", "image", "pdf"], "output": ["text"] },
        }));
    }
    // Claude family: 200k context, 64k output.
    if id.starts_with("claude") {
        return Some(serde_json::json!({
            "limit": { "context": 200_000, "output": 64_000 },
            "modalities": { "input": ["text", "image", "pdf"], "output": ["text"] },
        }));
    }
    None
}

/// Build a minimal but valid model entry for a model id that is not in the catalog.
///
/// OpenCode's schema only requires a `name` for a model entry, so a model the user
/// explicitly selected — even one we have no metadata for — should still be written
/// rather than silently dropped. If a display name was passed from the frontend we use
/// it verbatim (so the config matches what the user saw); otherwise we derive one.
/// When we can infer the model family from the id we also fill in limit/modalities.
pub(crate) fn build_fallback_model_json(model_id: &str, display_name: Option<&str>) -> Value {
    // Prefer the caller-provided display name; fall back to a cleaned-up id.
    let name = display_name
        .filter(|n| !n.trim().is_empty())
        .map(|n| n.to_string())
        .unwrap_or_else(|| humanize_model_id(model_id));

    let mut entry = serde_json::Map::new();
    entry.insert("name".to_string(), Value::String(name));
    if let Some(defaults) = series_defaults_for(model_id) {
        if let Some(obj) = defaults.as_object() {
            for (k, v) in obj.iter() {
                entry.insert(k.clone(), v.clone());
            }
        }
    }
    Value::Object(entry)
}

/// Bare model id without a `vendor/` prefix (`anthropic/claude-sonnet-4-6` -> `claude-sonnet-4-6`).
fn strip_model_vendor_prefix(model_id: &str) -> &str {
    model_id.rsplit('/').next().unwrap_or(model_id)
}

fn is_short_version_token(s: &str) -> bool {
    let len = s.len();
    (1..=2).contains(&len) && s.chars().all(|c| c.is_ascii_digit())
}

/// Join adjacent 1–2 digit version tokens with dots (`4-6` -> `4.6`).
fn merge_hyphenated_version_tokens(model_id: &str) -> String {
    let parts: Vec<&str> = model_id.split('-').filter(|s| !s.is_empty()).collect();
    let mut out: Vec<String> = Vec::with_capacity(parts.len());
    let mut i = 0;
    while i < parts.len() {
        if i + 1 < parts.len()
            && is_short_version_token(parts[i])
            && is_short_version_token(parts[i + 1])
        {
            out.push(format!("{}.{}", parts[i], parts[i + 1]));
            i += 2;
        } else {
            out.push(parts[i].to_string());
            i += 1;
        }
    }
    out.join("-")
}

/// IDs to try against the catalog: original, vendor-stripped, dotted/dashed version variants.
fn catalog_lookup_ids(model_id: &str) -> Vec<String> {
    let bare = strip_model_vendor_prefix(model_id.trim());
    let mut ids = Vec::new();
    let mut push = |id: String| {
        if !id.is_empty() && !ids.iter().any(|existing| existing == &id) {
            ids.push(id);
        }
    };
    push(model_id.trim().to_string());
    push(bare.to_string());
    push(bare.replace('.', "-"));
    push(merge_hyphenated_version_tokens(bare));
    ids
}

pub(crate) fn lookup_catalog_model<'a>(
    catalog: &HashMap<&str, &'a ModelDef>,
    model_id: &str,
) -> Option<&'a ModelDef> {
    for candidate in catalog_lookup_ids(model_id) {
        if let Some(model) = catalog.get(candidate.as_str()) {
            return Some(*model);
        }
    }
    None
}

/// Derive a readable name from a model id, preserving version dots
/// (e.g. "gemini-3.5-flash-low" -> "Gemini 3.5 Flash Low",
/// "claude-sonnet-4-6" -> "Claude Sonnet 4.6").
pub(crate) fn humanize_model_id(model_id: &str) -> String {
    merge_hyphenated_version_tokens(strip_model_vendor_prefix(model_id))
        .split('-')
        .filter(|s| !s.is_empty())
        .map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Return a Gemini canonical catalog id for a canonical id or one of its aliases.
fn canonical_gemini_model_id(model_id: &str) -> Option<&'static str> {
    GEMINI_FAMILIES.iter().find_map(|family| {
        (family.canonical_id == model_id
            || family
                .aliases
                .iter()
                .any(|(alias_id, _)| *alias_id == model_id))
        .then_some(family.canonical_id)
    })
}

/// Normalize Gemini aliases and preserve the first frontend entry for each model id.
fn normalize_model_inputs(model_inputs: &[ModelInput]) -> Vec<ModelInput> {
    let mut seen_model_ids = HashSet::new();

    model_inputs
        .iter()
        .filter_map(|input| {
            let model_id = canonical_gemini_model_id(&input.id).unwrap_or(&input.id);
            if !seen_model_ids.insert(model_id.to_string()) {
                return None;
            }

            let mut normalized = input.clone();
            normalized.id = model_id.to_string();
            Some(normalized)
        })
        .collect()
}

/// Migrate known Gemini alias keys without touching non-Gemini user models.
pub(crate) fn migrate_gemini_alias_models(provider: &mut Value) {
    let Some(models) = provider.get_mut("models").and_then(Value::as_object_mut) else {
        return;
    };

    for family in GEMINI_FAMILIES {
        for (alias_id, _) in family.aliases {
            let Some(alias_value) = models.remove(*alias_id) else {
                continue;
            };

            let merged = match models.remove(family.canonical_id) {
                Some(canonical_value) => {
                    match (canonical_value.as_object(), alias_value.as_object()) {
                        (Some(canonical), Some(alias)) => {
                            let mut merged = canonical.clone();
                            for (field, value) in alias {
                                match merged.get(field) {
                                    Some(canonical_value) if canonical_value != value => {
                                        tracing::warn!(
                                            canonical_model = family.canonical_id,
                                            alias_model = alias_id,
                                            field = field.as_str(),
                                            "OpenCode sync model field conflict; canonical value retained"
                                        );
                                    }
                                    Some(_) => {}
                                    None => {
                                        merged.insert(field.clone(), value.clone());
                                    }
                                }
                            }
                            Value::Object(merged)
                        }
                        _ => {
                            tracing::warn!(
                                canonical_model = family.canonical_id,
                                alias_model = alias_id,
                                "OpenCode sync could not merge a non-object alias model; canonical value retained"
                            );
                            canonical_value
                        }
                    }
                }
                None => alias_value,
            };
            models.insert(family.canonical_id.to_string(), merged);
        }
    }
}

/// Merge catalog models into provider.models without deleting user models.
/// Each entry carries an optional display name from the frontend so fallback entries
/// for non-catalog models get a recognizable name.
pub(crate) fn merge_catalog_models(provider: &mut Value, model_inputs: Option<&[ModelInput]>) {
    if provider.get("models").is_none() {
        provider["models"] = serde_json::json!({});
    }

    let catalog = build_model_catalog();
    let catalog_map: HashMap<&str, &ModelDef> = catalog.iter().map(|m| (m.id, m)).collect();

    if let Some(models) = provider.get_mut("models").and_then(|m| m.as_object_mut()) {
        // When no specific models are requested, sync the whole catalog.
        let catalog_ids: Vec<&str> = catalog_map.keys().copied().collect();

        let normalized_inputs = normalize_model_inputs(model_inputs.unwrap_or(&[]));
        for input in &normalized_inputs {
            let model_id = input.id.as_str();
            if let Some(model_def) = catalog_map.get(model_id) {
                let catalog_model = build_model_json(model_def);

                if let Some(existing) = models.get(model_id) {
                    // Merge: keep user-defined fields, update catalog fields
                    if let Some(existing_obj) = existing.as_object() {
                        let mut merged = existing_obj.clone();

                        // Update/insert catalog fields
                        if let Some(catalog_obj) = catalog_model.as_object() {
                            for (key, value) in catalog_obj.iter() {
                                merged.insert(key.clone(), value.clone());
                            }
                        }

                        models.insert(model_id.to_string(), Value::Object(merged));
                    } else {
                        // Existing is not an object, replace with catalog
                        models.insert(model_id.to_string(), catalog_model);
                    }
                } else {
                    // Model doesn't exist, insert full catalog entry
                    models.insert(model_id.to_string(), catalog_model);
                }
            } else {
                // Fallback: the id isn't in our catalog (e.g. a dynamically discovered
                // model from the user's account quota, or a new model not yet listed).
                // Previously these were silently dropped, which caused selected models to
                // never appear in the config. OpenCode only requires a `name`, so write a
                // minimal entry using the frontend-provided display name when available.
                // Preserve any user-defined object the user may already have.
                if !models.contains_key(model_id) {
                    models.insert(
                        model_id.to_string(),
                        build_fallback_model_json(model_id, input.name.as_deref()),
                    );
                }
            }
        }

        // No inputs at all => sync the entire catalog (None means "all").
        if model_inputs.is_none() {
            for model_id in &catalog_ids {
                if let Some(model_def) = catalog_map.get(model_id) {
                    if !models.contains_key(*model_id) {
                        models.insert((*model_id).to_string(), build_model_json(model_def));
                    }
                }
            }
        }
    }
}
