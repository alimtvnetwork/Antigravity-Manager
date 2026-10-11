use super::*;
use crate::modules::supabase_sync::config_a::get_config_path;
use crate::proxy::hermes_sync::detect::get_backup_path;
use crate::proxy::hermes_sync::detect::normalize_base_url;
use crate::proxy::openclaw_sync::PROVIDER_ID;

pub fn is_managed_provider(provider: &str) -> bool {
    matches!(provider, PROVIDER_REF | PROVIDER_ID)
}

pub fn model_has_only_managed_fields(doc: &YamlDoc) -> bool {
    let Some(model) = resolve_optional(doc, "/model") else {
        return false;
    };
    matches!(doc.semantic_kind(model), Some(SemanticKind::Mapping { .. }))
        && doc.mapping_entries(model).all(|(key, _)| {
            doc.scalar_value(key)
                .is_ok_and(|value| matches!(value.as_ref(), "provider" | "default"))
        })
}

pub fn deactivate(doc: &mut YamlDoc, backup: Option<&YamlDoc>) -> Result<(), String> {
    let Some(current) = scalar_at(doc, "/model/provider") else {
        return Ok(());
    };
    if !is_managed_provider(&current) {
        return Ok(());
    }
    let safe_backup = backup.filter(|value| {
        !scalar_at(value, "/model/provider").is_some_and(|provider| is_managed_provider(&provider))
    });
    if let Some(backup) = safe_backup {
        if let Some(provider) = extract_fragment(backup, "/model/provider")? {
            upsert(doc, "/model/provider", &provider)?;
        } else {
            remove(doc, "/model/provider")?;
        }
        if let Some(default_model) = extract_fragment(backup, "/model/default")? {
            upsert(doc, "/model/default", &default_model)?;
        } else {
            remove(doc, "/model/default")?;
        }
    } else if model_has_only_managed_fields(doc) {
        remove(doc, "/model")?;
    } else {
        remove(doc, "/model/provider")?;
        remove(doc, "/model/default")?;
    }
    if mapping_len_at(doc, "/model") == Some(0) {
        remove(doc, "/model")?;
    }
    Ok(())
}

pub fn apply_sync_losslessly(
    source: &str,
    base_url: &str,
    api_key: &str,
    discover_models: bool,
    models: &[String],
    activate: bool,
    default_model: Option<&str>,
    backup: Option<&str>,
) -> Result<String, String> {
    let line_ending = preferred_line_ending(source);
    let mut doc = parse_doc(source)?;
    let backup = backup.map(parse_doc).transpose()?;
    ensure_root_mapping(&mut doc)?;
    ensure_mapping(&mut doc, "/providers")?;
    let provider = format!("/providers/{PROVIDER_ID}");
    if !is_mapping_at(&doc, &provider) {
        upsert(
            &mut doc,
            &provider,
            &provider_fragment(base_url, api_key, discover_models, models, line_ending)?,
        )?;
    } else {
        for key in [
            "key_cmd",
            "key_env",
            "api_key_env",
            "apiKey",
            "keyEnv",
            "apiKeyEnv",
            "base_url",
            "url",
            "api_mode",
        ] {
            remove(&mut doc, &format!("{provider}/{key}"))?;
        }
        for (key, value) in [
            ("name", PROVIDER_DISPLAY_NAME),
            ("api", base_url),
            ("api_key", api_key),
            ("transport", "chat_completions"),
        ] {
            upsert(
                &mut doc,
                &format!("{provider}/{key}"),
                &string_fragment(value)?,
            )?;
        }
        upsert(
            &mut doc,
            &format!("{provider}/discover_models"),
            &bool_fragment(discover_models)?,
        )?;
        let model_path = format!("{provider}/models");
        if discover_models {
            remove(&mut doc, &model_path)?;
        } else {
            sync_sequence(&mut doc, &model_path, models, line_ending)?;
        }
    }

    if activate {
        ensure_mapping(&mut doc, "/model")?;
        upsert(&mut doc, "/model/provider", &string_fragment(PROVIDER_REF)?)?;
        if let Some(model) = default_model.filter(|model| !model.trim().is_empty()) {
            upsert(&mut doc, "/model/default", &string_fragment(model)?)?;
        }
    } else {
        deactivate(&mut doc, backup.as_ref())?;
    }
    Ok(render_doc(&doc, source))
}

pub fn apply_clear_losslessly(
    source: &str,
    backup: Option<&str>,
) -> Result<(String, bool), String> {
    let mut doc = parse_doc(source)?;
    let backup_doc = backup.map(parse_doc).transpose()?;
    let mut changed = false;

    if scalar_at(&doc, "/model/provider").is_some_and(|provider| is_managed_provider(&provider)) {
        deactivate(&mut doc, backup_doc.as_ref())?;
        changed = true;
    }

    let provider = format!("/providers/{PROVIDER_ID}");
    if resolve_optional(&doc, &provider).is_some() {
        if mapping_len_at(&doc, "/providers") == Some(1) {
            remove(&mut doc, "/providers")?;
        } else {
            remove(&mut doc, &provider)?;
        }
        changed = true;
    }
    Ok((render_doc(&doc, source), changed))
}

pub fn apply_restore_losslessly(current: &str, backup: &str) -> Result<String, String> {
    let current_source = current;
    let mut current = parse_doc(current)?;
    let backup = parse_doc(backup)?;
    ensure_root_mapping(&mut current)?;
    let provider = format!("/providers/{PROVIDER_ID}");
    if is_mapping_at(&backup, &provider) {
        ensure_mapping(&mut current, "/providers")?;
        restore_mapping(&mut current, &backup, &provider)?;
    } else if resolve_optional(&current, &provider).is_some() {
        if mapping_len_at(&current, "/providers") == Some(1) {
            remove(&mut current, "/providers")?;
        } else {
            remove(&mut current, &provider)?;
        }
    }

    if scalar_at(&current, "/model/provider").is_some_and(|provider| is_managed_provider(&provider))
    {
        if let Some(value) = extract_fragment(&backup, "/model/provider")? {
            upsert(&mut current, "/model/provider", &value)?;
        } else {
            remove(&mut current, "/model/provider")?;
        }
        if resolve_optional(&current, "/model").is_some() {
            if let Some(value) = extract_fragment(&backup, "/model/default")? {
                upsert(&mut current, "/model/default", &value)?;
            } else {
                remove(&mut current, "/model/default")?;
            }
        }
        if mapping_len_at(&current, "/model") == Some(0) {
            remove(&mut current, "/model")?;
        }
    }
    Ok(render_doc(&current, current_source))
}

pub fn is_sensitive_key(key: &str) -> bool {
    let key = key.to_ascii_lowercase();
    key == "api_key"
        || key == "apikey"
        || key.ends_with("_api_key")
        || key == "password"
        || key.ends_with("_password")
        || key == "secret"
        || key.ends_with("_secret")
        || key == "token"
        || key.ends_with("_token")
        || key == "credential"
        || key.ends_with("_credential")
        || key == "private_key"
        || key.ends_with("_private_key")
}

pub fn collect_sensitive_paths(doc: &YamlDoc, node: NodeId, path: &str, output: &mut Vec<String>) {
    match doc.semantic_kind(node) {
        Some(SemanticKind::Mapping { .. }) => {
            for (key_node, value_node) in doc.mapping_entries(node).collect::<Vec<_>>() {
                let Ok(key) = doc.scalar_value(key_node) else {
                    continue;
                };
                let escaped = escape_pointer_token(&key);
                let child = format!("{path}/{escaped}");
                if is_sensitive_key(&key) {
                    output.push(child);
                } else {
                    collect_sensitive_paths(doc, value_node, &child, output);
                }
            }
        }
        Some(SemanticKind::Sequence { .. }) => {
            for (index, item) in doc
                .sequence_items(node)
                .collect::<Vec<_>>()
                .into_iter()
                .enumerate()
            {
                collect_sensitive_paths(doc, item, &format!("{path}/{index}"), output);
            }
        }
        _ => {}
    }
}

pub fn redact_sensitive_source(source: &str) -> Result<String, String> {
    let mut doc = parse_doc(source)?;
    let root = doc
        .document_root(0)
        .map_err(|error| format!("Failed to inspect Hermes config: {error}"))?;
    let Some(root) = root else {
        return Ok(render_doc(&doc, source));
    };
    let mut paths = Vec::new();
    collect_sensitive_paths(&doc, root, "", &mut paths);
    let redacted = string_fragment("[REDACTED]")?;
    for path in paths {
        upsert(&mut doc, &path, &redacted)?;
    }
    Ok(render_doc(&doc, source))
}

pub fn read_provider_entry(
    doc: &YamlDoc,
) -> Option<(Option<String>, Option<String>, Option<String>)> {
    let provider = format!("/providers/{PROVIDER_ID}");
    is_mapping_at(doc, &provider).then_some((
        scalar_at(doc, &format!("{provider}/api"))
            .or_else(|| scalar_at(doc, &format!("{provider}/base_url")))
            .or_else(|| scalar_at(doc, &format!("{provider}/url"))),
        scalar_at(doc, &format!("{provider}/api_key")),
        scalar_at(doc, &format!("{provider}/transport"))
            .or_else(|| scalar_at(doc, &format!("{provider}/api_mode"))),
    ))
}

#[derive(Default)]
struct HermesConfigState {
    pub(crate) is_synced: bool,
    pub(crate) has_backup: bool,
    pub(crate) current_base_url: Option<String>,
    pub(crate) discover_models: bool,
    pub(crate) configured_models: Vec<String>,
    pub(crate) is_active: bool,
    pub(crate) default_model: Option<String>,
}

pub fn read_config_state(proxy_url: Option<String>) -> HermesConfigState {
    let mut state = HermesConfigState {
        has_backup: get_backup_path().is_some_and(|path| path.exists()),
        discover_models: true,
        ..HermesConfigState::default()
    };
    let Some(path) = get_config_path() else {
        return state;
    };
    let Some(source) = read_hermes_source(&path) else {
        return state;
    };
    let Ok(doc) = parse_doc(&source) else {
        return state;
    };
    let provider = format!("/providers/{PROVIDER_ID}");
    if is_mapping_at(&doc, &provider) {
        state.discover_models =
            bool_at(&doc, &format!("{provider}/discover_models")).unwrap_or(true);
        state.configured_models = string_list_at(&doc, &format!("{provider}/models"));
    }
    let active = scalar_at(&doc, "/model/provider").unwrap_or_default();
    state.is_active = is_managed_provider(&active);
    if state.is_active {
        state.default_model =
            scalar_at(&doc, "/model/default").or_else(|| scalar_at(&doc, "/model/name"));
    }
    let Some((base_url, api_key, _)) = read_provider_entry(&doc) else {
        return state;
    };
    state.current_base_url = base_url.clone();
    let (Some(url), Some(key)) = (base_url, api_key) else {
        return state;
    };
    if url.trim().is_empty() || key.trim().is_empty() {
        return state;
    }
    state.is_synced = proxy_url
        .filter(|value| !value.trim().is_empty())
        .map(|expected| normalize_base_url(&url) == normalize_base_url(&expected))
        .unwrap_or(true);
    state
}
