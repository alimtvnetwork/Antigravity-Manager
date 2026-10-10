use super::*;

fn fragment(source: &str) -> Result<YamlFragment, String> {
    YamlFragment::parse(source).map_err(|error| format!("Failed to build YAML fragment: {error}"))
}

fn string_fragment(value: &str) -> Result<YamlFragment, String> {
    fragment(
        &serde_json::to_string(value)
            .map_err(|error| format!("Failed to encode YAML string: {error}"))?,
    )
}

fn bool_fragment(value: bool) -> Result<YamlFragment, String> {
    fragment(if value { "true" } else { "false" })
}

fn preferred_line_ending(source: &str) -> &'static str {
    if source.contains("\r\n") {
        "\r\n"
    } else if source.contains('\r') {
        "\r"
    } else {
        "\n"
    }
}

fn sequence_fragment(values: &[String], line_ending: &str) -> Result<YamlFragment, String> {
    if values.is_empty() {
        return fragment("[]");
    }
    let mut source = String::new();
    for value in values {
        source.push_str("- ");
        source.push_str(
            &serde_json::to_string(value)
                .map_err(|error| format!("Failed to encode model id: {error}"))?,
        );
        source.push_str(line_ending);
    }
    fragment(&source)
}

fn provider_fragment(
    base_url: &str,
    api_key: &str,
    discover_models: bool,
    models: &[String],
    line_ending: &str,
) -> Result<YamlFragment, String> {
    let encode = |value: &str| {
        serde_json::to_string(value).map_err(|error| format!("Failed to encode provider: {error}"))
    };
    let mut source = [
        format!("name: {}", encode(PROVIDER_DISPLAY_NAME)?),
        format!("api: {}", encode(base_url)?),
        format!("api_key: {}", encode(api_key)?),
        "transport: chat_completions".to_string(),
        format!("discover_models: {discover_models}"),
    ]
    .join(line_ending);
    source.push_str(line_ending);
    if !discover_models {
        source.push_str("models:");
        source.push_str(line_ending);
        for model in models {
            source.push_str("  - ");
            source.push_str(&encode(model)?);
            source.push_str(line_ending);
        }
    }
    fragment(&source)
}

fn commit(doc: &mut YamlDoc) -> Result<(), String> {
    doc.commit_edits()
        .map_err(|error| format!("Failed to apply Hermes YAML edit: {error}"))
}

fn upsert(doc: &mut YamlDoc, path: &str, value: &YamlFragment) -> Result<(), String> {
    let pointer = parse_pointer(path)?;
    if doc.resolve_pointer(0, &pointer).is_ok() {
        doc.replace_at(0, &pointer, value)
            .map_err(|error| format!("Failed to replace YAML path {path}: {error}"))?;
    } else {
        doc.add_at(0, &pointer, value)
            .map_err(|error| format!("Failed to add YAML path {path}: {error}"))?;
    }
    commit(doc)
}

fn remove(doc: &mut YamlDoc, path: &str) -> Result<bool, String> {
    let pointer = parse_pointer(path)?;
    if doc.resolve_pointer(0, &pointer).is_err() {
        return Ok(false);
    }
    doc.remove_at(0, &pointer)
        .map_err(|error| format!("Failed to remove YAML path {path}: {error}"))?;
    commit(doc)?;
    Ok(true)
}

fn ensure_root_mapping(doc: &mut YamlDoc) -> Result<(), String> {
    let root = doc
        .document_root(0)
        .map_err(|error| format!("Failed to inspect Hermes YAML root: {error}"))?;
    if root
        .is_some_and(|node| matches!(doc.semantic_kind(node), Some(SemanticKind::Mapping { .. })))
    {
        Ok(())
    } else {
        upsert(doc, "", &fragment("{}")?)
    }
}

fn ensure_mapping(doc: &mut YamlDoc, path: &str) -> Result<(), String> {
    if is_mapping_at(doc, path) {
        Ok(())
    } else {
        upsert(doc, path, &fragment("{}")?)
    }
}

fn sync_sequence(
    doc: &mut YamlDoc,
    path: &str,
    values: &[String],
    line_ending: &str,
) -> Result<(), String> {
    let Some(node) = resolve_optional(doc, path) else {
        return upsert(doc, path, &sequence_fragment(values, line_ending)?);
    };
    if !matches!(doc.semantic_kind(node), Some(SemanticKind::Sequence { .. })) {
        return upsert(doc, path, &sequence_fragment(values, line_ending)?);
    }
    let current_len = doc.sequence_items(node).count();
    for (index, value) in values.iter().take(current_len).enumerate() {
        upsert(doc, &format!("{path}/{index}"), &string_fragment(value)?)?;
    }
    for index in (values.len()..current_len).rev() {
        remove(doc, &format!("{path}/{index}"))?;
    }
    for value in values.iter().skip(current_len) {
        upsert(doc, &format!("{path}/-"), &string_fragment(value)?)?;
    }
    Ok(())
}

fn extract_fragment(doc: &YamlDoc, path: &str) -> Result<Option<YamlFragment>, String> {
    let Some(node) = resolve_optional(doc, path) else {
        return Ok(None);
    };
    fragment(
        &doc.extract_node(node)
            .map_err(|error| format!("Failed to extract backup path {path}: {error}"))?,
    )
    .map(Some)
}

fn escape_pointer_token(token: &str) -> String {
    token.replace('~', "~0").replace('/', "~1")
}

fn mapping_keys(doc: &YamlDoc, path: &str) -> Vec<String> {
    let Some(node) = resolve_optional(doc, path) else {
        return Vec::new();
    };
    doc.mapping_entries(node)
        .filter_map(|(key, _)| doc.scalar_value(key).ok().map(|value| value.into_owned()))
        .collect()
}

fn restore_mapping(current: &mut YamlDoc, backup: &YamlDoc, path: &str) -> Result<(), String> {
    ensure_mapping(current, path)?;
    let backup_keys = mapping_keys(backup, path);
    for key in mapping_keys(current, path) {
        if !backup_keys.iter().any(|backup_key| backup_key == &key) {
            remove(current, &format!("{path}/{}", escape_pointer_token(&key)))?;
        }
    }
    for key in backup_keys {
        let child = format!("{path}/{}", escape_pointer_token(&key));
        if let Some(value) = extract_fragment(backup, &child)? {
            upsert(current, &child, &value)?;
        }
    }
    Ok(())
}
