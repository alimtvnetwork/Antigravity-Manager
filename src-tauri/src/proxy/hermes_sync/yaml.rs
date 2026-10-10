use super::*;

fn parse_pointer(path: &str) -> Result<JsonPointer, String> {
    JsonPointer::parse(path).map_err(|error| format!("Invalid YAML path {path:?}: {error}"))
}

#[derive(Debug, Clone)]
struct IndentlessSequenceStyle {
    parent_line: String,
    parent_occurrence: usize,
    parent_indent: usize,
}

fn line_without_ending(line: &str) -> &str {
    line.strip_suffix("\r\n")
        .or_else(|| line.strip_suffix('\n'))
        .unwrap_or(line)
}

fn leading_spaces(line: &str) -> usize {
    line.bytes().take_while(|byte| *byte == b' ').count()
}

fn is_sequence_entry(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed == "-" || trimmed.starts_with("- ")
}

fn is_empty_mapping_value(line: &str) -> bool {
    let trimmed = line.trim_start();
    if trimmed.starts_with('-') {
        return false;
    }
    let Some((key, value)) = trimmed.split_once(':') else {
        return false;
    };
    !key.trim().is_empty() && (value.trim().is_empty() || value.trim_start().starts_with('#'))
}

fn normalize_indentless_sequences(source: &str) -> (String, Vec<IndentlessSequenceStyle>) {
    let lines = source.split_inclusive('\n').collect::<Vec<_>>();
    let mut extra_indent = vec![0_usize; lines.len()];
    let mut styles = Vec::new();

    for (index, line) in lines.iter().enumerate() {
        let parent = line_without_ending(line);
        if !is_empty_mapping_value(parent) {
            continue;
        }
        let parent_indent = leading_spaces(parent);
        let Some(first_content) = ((index + 1)..lines.len()).find(|candidate| {
            let body = line_without_ending(lines[*candidate]);
            let trimmed = body.trim();
            !trimmed.is_empty() && !trimmed.starts_with('#')
        }) else {
            continue;
        };
        let first = line_without_ending(lines[first_content]);
        if leading_spaces(first) != parent_indent || !is_sequence_entry(first) {
            continue;
        }

        let mut end = first_content;
        while end < lines.len() {
            let body = line_without_ending(lines[end]);
            let trimmed = body.trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                let indent = leading_spaces(body);
                if indent < parent_indent || (indent == parent_indent && !is_sequence_entry(body)) {
                    break;
                }
            }
            end += 1;
        }
        for indent in &mut extra_indent[index + 1..end] {
            *indent += 2;
        }
        let parent_occurrence = lines[..index]
            .iter()
            .filter(|candidate| line_without_ending(candidate) == parent)
            .count();
        styles.push(IndentlessSequenceStyle {
            parent_line: parent.to_string(),
            parent_occurrence,
            parent_indent,
        });
    }

    let mut normalized = String::with_capacity(source.len() + extra_indent.iter().sum::<usize>());
    for (line, indent) in lines.into_iter().zip(extra_indent) {
        normalized.push_str(&" ".repeat(indent));
        normalized.push_str(line);
    }
    (normalized, styles)
}

fn restore_indentless_sequences(source: &str, styles: &[IndentlessSequenceStyle]) -> String {
    let mut lines = source
        .split_inclusive('\n')
        .map(str::to_string)
        .collect::<Vec<_>>();
    for style in styles {
        let Some(parent_index) = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line_without_ending(line) == style.parent_line)
            .nth(style.parent_occurrence)
            .map(|(index, _)| index)
        else {
            continue;
        };
        let mut index = parent_index + 1;
        while index < lines.len() {
            let body = line_without_ending(&lines[index]);
            let trimmed = body.trim();
            if !trimmed.is_empty()
                && !trimmed.starts_with('#')
                && leading_spaces(body) <= style.parent_indent
            {
                break;
            }
            if lines[index].starts_with("  ") {
                lines[index].drain(..2);
            }
            index += 1;
        }
    }
    lines.concat()
}

fn parse_doc(source: &str) -> Result<YamlDoc, String> {
    let source = if source.trim().is_empty() {
        EMPTY_CONFIG
    } else {
        source
    };
    let (normalized, _) = normalize_indentless_sequences(source);
    YamlDoc::parse(&normalized)
        .map_err(|error| format!("Hermes config.yaml is not valid YAML: {error}"))
}

fn render_doc(doc: &YamlDoc, original_source: &str) -> String {
    let (_, styles) = normalize_indentless_sequences(original_source);
    restore_indentless_sequences(doc.as_source(), &styles)
}

fn read_hermes_source(path: &PathBuf) -> Result<String, String> {
    match fs::read_to_string(path) {
        Ok(source) if source.trim().is_empty() => Ok(EMPTY_CONFIG.to_string()),
        Ok(source) => {
            parse_doc(&source)?;
            Ok(source)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(EMPTY_CONFIG.to_string()),
        Err(error) => Err(format!("Failed to read Hermes config: {error}")),
    }
}

fn atomically_write_source(path: &PathBuf, source: &str) -> Result<(), String> {
    parse_doc(source)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create directory: {error}"))?;
    }
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(HERMES_CONFIG_FILE);
    let temp = path.with_file_name(format!("{file_name}.tmp.{}", uuid::Uuid::new_v4()));
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options
        .open(&temp)
        .map_err(|error| format!("Failed to create temp file: {error}"))?;
    let result = (|| -> std::io::Result<()> {
        use std::io::Write;
        #[cfg(unix)]
        if let Ok(metadata) = fs::metadata(path) {
            file.set_permissions(metadata.permissions())?;
        }
        file.write_all(source.as_bytes())?;
        file.sync_all()
    })();
    drop(file);
    if let Err(error) = result {
        // Justification: best-effort cleanup; a leftover file is harmless
        crate::error::record_ignored(fs::remove_file(&temp), "remove_file");
        return Err(format!("Failed to write temp file: {error}"));
    }
    fs::rename(&temp, path).map_err(|error| {
        // Justification: best-effort cleanup; a leftover file is harmless
        crate::error::record_ignored(fs::remove_file(&temp), "remove_file");
        format!("Failed to rename config file: {error}")
    })
}

fn create_backup(path: &PathBuf) -> Result<(), String> {
    let backup = path.with_file_name(format!(
        "{}{}",
        path.file_name().unwrap_or_default().to_string_lossy(),
        BACKUP_SUFFIX
    ));
    if !backup.exists() {
        // Preserve an empty baseline too, so the first sync can be undone.
        atomically_write_source(&backup, &read_hermes_source(path)?)?;
    }
    Ok(())
}

fn resolve_optional(doc: &YamlDoc, path: &str) -> Option<NodeId> {
    doc.resolve_pointer(0, &parse_pointer(path).ok()?).ok()
}

fn scalar_at(doc: &YamlDoc, path: &str) -> Option<String> {
    doc.scalar_value(resolve_optional(doc, path)?)
        .ok()
        .map(|value| value.into_owned())
}

fn bool_at(doc: &YamlDoc, path: &str) -> Option<bool> {
    match scalar_at(doc, path)?.to_ascii_lowercase().as_str() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn is_mapping_at(doc: &YamlDoc, path: &str) -> bool {
    resolve_optional(doc, path)
        .is_some_and(|node| matches!(doc.semantic_kind(node), Some(SemanticKind::Mapping { .. })))
}

fn mapping_len_at(doc: &YamlDoc, path: &str) -> Option<usize> {
    let node = resolve_optional(doc, path)?;
    matches!(doc.semantic_kind(node), Some(SemanticKind::Mapping { .. }))
        .then(|| doc.mapping_entries(node).count())
}

fn string_list_at(doc: &YamlDoc, path: &str) -> Vec<String> {
    let Some(node) = resolve_optional(doc, path) else {
        return Vec::new();
    };
    match doc.semantic_kind(node) {
        Some(SemanticKind::Sequence { .. }) => doc
            .sequence_items(node)
            .filter_map(|item| doc.scalar_value(item).ok().map(|value| value.into_owned()))
            .collect(),
        Some(SemanticKind::Mapping { .. }) => doc
            .mapping_entries(node)
            .filter_map(|(key, _)| doc.scalar_value(key).ok().map(|value| value.into_owned()))
            .collect(),
        _ => Vec::new(),
    }
}
