// Schema cleaning entry points (split from json_schema.rs).
use super::refs::{extract_best_schema_from_union, merge_all_of};
use super::{MAX_DESCRIPTION_LENGTH, MAX_RECURSION_DEPTH};
use crate::proxy::common::json_schema::recursive::clean_json_schema_recursive;
use serde_json::{json, Value};

pub fn sanitize_description(s: &str) -> String {
    if s.chars().count() <= MAX_DESCRIPTION_LENGTH {
        s.to_string()
    } else {
        let truncated: String = s
            .chars()
            .take(MAX_DESCRIPTION_LENGTH.saturating_sub(15))
            .collect();
        format!("{}... [truncated]", truncated)
    }
}

/// Recursively clean JSON Schema to comply with Gemini interface requirements
///
/// 1. [New] Expand $ref and $defs: Replace references with actual definitions to solve the problem of Gemini not supporting $ref
/// 2. Remove unsupported fields: $schema, additionalProperties, format, default, uniqueItems, validation fields
/// 3. Process union types: ["string", "null"] -> "string"
/// 4. [NEW] Process anyOf union type: anyOf: [ {"type": "string"} , {"type": "null"} ] -> "type": "string"
/// 5. Convert the value of the type field to lowercase (required by Gemini v1internal)
/// 6. Remove numeric check fields: multipleOf, exclusiveMinimum, exclusiveMaximum, etc.
/// Clean the JSON Schema used for responseSchema
pub fn clean_response_schema(value: &mut Value) {
    clean_json_schema(value);
}

pub fn clean_json_schema(value: &mut Value) {
    // 0. Preprocessing: Expand $ref (Schema Flattening)
    // [FIX #952] Recursively collect $defs/definitions from all levels instead of just extracting from the root level
    let mut all_defs = serde_json::Map::new();
    collect_all_defs(value, &mut all_defs);

    // Remove root-level $defs/definitions (to maintain backward compatibility)
    if let Value::Object(map) = value {
        map.remove("$defs");
        map.remove("definitions");
    }

    // [FIX #952] Always run flatten_refs even if defs is empty
    // This catches and handles unresolved $ref (downgraded to string type)
    if let Value::Object(map) = value {
        flatten_refs(map, &all_defs, 0);
    }

    // recursive cleanup
    clean_json_schema_recursive(value, true, 0);
}

/// Clean JSON Schema to comply with Gemini interface requirements
///
/// Follow the principle of pure protocol transparent transmission and do not execute the private adaptation logic of any specific tool.
pub fn clean_json_schema_for_tool(value: &mut Value, _tool_name: &str) {
    clean_json_schema(value);
}

/// [NEW #952] Recursively collect $defs and definitions at all levels
///
/// An MCP tool's schema may define $defs at any nesting level, not just at the root level.
/// This function deeply traverses the entire schema, collecting all definitions into a unified map.
fn collect_all_defs(value: &Value, defs: &mut serde_json::Map<String, Value>) {
    if let Value::Object(map) = value {
        // Collect $defs for the current level
        if let Some(Value::Object(d)) = map.get("$defs") {
            for (k, v) in d {
                // Avoid overwriting existing definitions (first defined takes precedence)
                defs.entry(k.clone()).or_insert_with(|| v.clone());
            }
        }
        // Collect definitions of the current level (Draft-07 style)
        if let Some(Value::Object(d)) = map.get("definitions") {
            for (k, v) in d {
                defs.entry(k.clone()).or_insert_with(|| v.clone());
            }
        }
        // Process all child nodes recursively
        for (key, v) in map {
            // Skip $defs/definitions themselves to avoid duplication of processing
            if key != "$defs" && key != "definitions" {
                collect_all_defs(v, defs);
            }
        }
    } else if let Value::Array(arr) = value {
        for item in arr {
            collect_all_defs(item, defs);
        }
    }
}

/// Expand $ref recursively
fn flatten_refs(
    map: &mut serde_json::Map<String, Value>,
    defs: &serde_json::Map<String, Value>,
    depth: usize,
) {
    if depth > MAX_RECURSION_DEPTH {
        tracing::warn!("[Schema-Flatten] Max recursion depth reached, stopping ref expansion.");
        return;
    }

    // Check and replace $ref
    if let Some(Value::String(ref_path)) = map.remove("$ref") {
        // Resolve reference names (e.g. #/$defs/MyType -> MyType)
        let ref_name = ref_path.split('/').last().unwrap_or(&ref_path);

        if let Some(def_schema) = defs.get(ref_name) {
            // Merge the defined content into the current map
            if let Value::Object(def_map) = def_schema {
                for (k, v) in def_map {
                    // Insert only if the current map does not have the key (avoid overwriting)
                    // But usually the $ref node should not have other attributes
                    map.entry(k.clone()).or_insert_with(|| v.clone());
                }

                // Recursively process any $refs that may be contained in the content just merged in
                // NOTE: Due to the introduction of the depth limit, circular references no longer cause stack overflow
                flatten_refs(map, defs, depth + 1);
            }
        } else {
            // [FIX #952] Unresolved $ref: converted to relaxed string type to avoid API 400 errors
            // This is better than having the request fail, at least the tool calls can still be made
            map.insert("type".to_string(), serde_json::json!("string"));
            let hint = format!("(Unresolved $ref: {})", ref_path);
            let desc_val = map
                .entry("description".to_string())
                .or_insert_with(|| Value::String(String::new()));
            if let Value::String(s) = desc_val {
                if !s.contains(&hint) {
                    if !s.is_empty() {
                        s.push(' ');
                    }
                    s.push_str(&hint);
                }
            }
        }
    }

    // Traverse child nodes
    for (_, v) in map.iter_mut() {
        if let Value::Object(child_map) = v {
            flatten_refs(child_map, defs, depth + 1);
        } else if let Value::Array(arr) = v {
            for item in arr {
                if let Value::Object(item_map) = item {
                    flatten_refs(item_map, defs, depth + 1);
                }
            }
        }
    }
}
