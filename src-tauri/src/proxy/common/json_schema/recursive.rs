// Recursive schema cleaner (split from json_schema.rs).
use super::refs::{extract_best_schema_from_union, merge_all_of};
use super::MAX_RECURSION_DEPTH;
use serde_json::{json, Value};

pub fn clean_json_schema_recursive(value: &mut Value, is_schema_node: bool, depth: usize) -> bool {
    if depth > MAX_RECURSION_DEPTH {
        debug_assert!(
            false,
            "Max recursion depth reached in clean_json_schema_recursive"
        );
        return false;
    }
    let mut is_effectively_nullable = false;

    match value {
        Value::Object(map) => {
            // 0. [NEW] Merge allOf
            merge_all_of(map);

            // 0.1 [NEW #3327] Normalize const keywords (convert to enum and corresponding type)
            // The Schema proto of Gemini/Vertex does not support const, and passing it in directly will result in 400 INVALID_ARGUMENT.
            // For example {"const": "element"} -> {"type": "string", "enum": ["element"]}
            if let Some(const_val) = map.remove("const") {
                if !map.contains_key("type") {
                    let inferred_type = match &const_val {
                        Value::String(_) => Some("string"),
                        Value::Number(n) => {
                            if n.is_i64() || n.is_u64() {
                                Some("integer")
                            } else {
                                Some("number")
                            }
                        }
                        Value::Bool(_) => Some("boolean"),
                        Value::Array(_) => Some("array"),
                        Value::Object(_) => Some("object"),
                        Value::Null => None,
                    };
                    if let Some(t) = inferred_type {
                        map.insert("type".to_string(), Value::String(t.to_string()));
                    }
                }
                let enum_entry = map
                    .entry("enum".to_string())
                    .or_insert_with(|| Value::Array(Vec::new()));
                if let Value::Array(enum_arr) = enum_entry {
                    if !enum_arr.contains(&const_val) {
                        enum_arr.push(const_val);
                    }
                }
            }

            // 0.5 [NEW] Structural normalization (Normalization)
            // Fix for some MCP tools (such as pencil) incorrectly using items to define object properties.
            // If type=object or contains properties but items is defined, Gemini will report an error because items can only appear in array.
            // We "align" the contents of items into properties.
            if map.get("type").and_then(|t| t.as_str()) == Some("object")
                || map.contains_key("properties")
            {
                if let Some(items) = map.remove("items") {
                    tracing::warn!("[Schema-Normalization] Found 'items' in an Object-like node. Moving content to 'properties'.");
                    let target_props = map
                        .entry("properties".to_string())
                        .or_insert_with(|| json!({}));
                    if let Some(target_map) = target_props.as_object_mut() {
                        if let Some(source_map) = items.as_object() {
                            for (k, v) in source_map {
                                target_map.entry(k.clone()).or_insert_with(|| v.clone());
                            }
                        }
                    }
                }
            }

            // 1. [CRITICAL] Deep recursive processing of sub-items
            // Handle properties (object)
            if let Some(Value::Object(props)) = map.get_mut("properties") {
                // [FIX] Drop boolean / non-object sub-schemas. JSON Schema allows
                // `prop: true|false`, but Gemini's Schema proto requires every property
                // value to be an object; a bare boolean triggers an upstream 400
                // ("Invalid value at '...properties[N].value' ... false").
                let dropped_keys: Vec<String> = props
                    .iter()
                    .filter(|(_, v)| !v.is_object())
                    .map(|(k, _)| k.clone())
                    .collect();
                for k in &dropped_keys {
                    props.remove(k);
                }

                let mut nullable_keys = std::collections::HashSet::new();
                for (k, v) in props.iter_mut() {
                    // Each value of properties must be an independent Schema node
                    if clean_json_schema_recursive(v, true, depth + 1) {
                        nullable_keys.insert(k.clone());
                    }
                }

                if !nullable_keys.is_empty() || !dropped_keys.is_empty() {
                    if let Some(Value::Array(req_arr)) = map.get_mut("required") {
                        req_arr.retain(|r| {
                            r.as_str()
                                .map(|s| {
                                    !nullable_keys.contains(s)
                                        && !dropped_keys.iter().any(|d| d == s)
                                })
                                .unwrap_or(true)
                        });
                        if req_arr.is_empty() {
                            map.remove("required");
                        }
                    }
                }

                // [NEW] Implicit type injection: if there are properties but no type, the completion is object
                if !map.contains_key("type") {
                    map.insert("type".to_string(), Value::String("object".to_string()));
                }
            }

            // Process items (array)
            // [FIX] items must be a Schema object; drop bare boolean / invalid items
            // (JSON Schema allows boolean `items`, Gemini's Schema proto rejects it).
            if map.get("items").map(|i| !i.is_object()).unwrap_or(false) {
                map.remove("items");
            }
            if let Some(items) = map.get_mut("items") {
                // The content of items must be an independent Schema node
                clean_json_schema_recursive(items, true, depth + 1);

                // [NEW] Implicit type injection: if there are items but no type, completion is array
                if !map.contains_key("type") {
                    map.insert("type".to_string(), Value::String("array".to_string()));
                }
            }

            // Gemini's Schema proto requires every ARRAY node to declare `items`,
            // including nested arrays. JSON Schema permits an itemless array, so
            // clients such as Claude Code may legitimately emit {"type":"array"}.
            // Use a string item schema as a Gemini-compatible fallback for these
            // otherwise unconstrained arrays.
            let is_array = map
                .get("type")
                .and_then(Value::as_str)
                .map(|t| t.eq_ignore_ascii_case("array"))
                .unwrap_or(false);
            if is_array && !map.contains_key("items") {
                map.insert("items".to_string(), json!({ "type": "string" }));
            }

            // Fallback: Cleanup of regular objects with neither properties nor items
            if !map.contains_key("properties") && !map.contains_key("items") {
                for (k, v) in map.iter_mut() {
                    // exclude keywords
                    if k != "anyOf" && k != "oneOf" && k != "allOf" && k != "enum" && k != "type" {
                        clean_json_schema_recursive(v, false, depth + 1);
                    }
                }
            }

            // 1.5. [FIX] Recursively clean every branch in anyOf/oneOf array
            // Must be executed before merging logic to ensure that the merged branch has been cleaned
            if let Some(Value::Array(any_of)) = map.get_mut("anyOf") {
                for branch in any_of.iter_mut() {
                    clean_json_schema_recursive(branch, true, depth + 1);
                }
            }
            if let Some(Value::Array(one_of)) = map.get_mut("oneOf") {
                for branch in one_of.iter_mut() {
                    clean_json_schema_recursive(branch, true, depth + 1);
                }
            }

            // 2. [FIX #815] Handling anyOf/oneOf union types: merge attributes or cherry-pick branches
            let mut union_to_merge = None;
            if let Some(Value::Array(any_of)) = map.get("anyOf") {
                union_to_merge = Some(any_of.clone());
            } else if let Some(Value::Array(one_of)) = map.get("oneOf") {
                union_to_merge = Some(one_of.clone());
            }

            if let Some(union_array) = union_to_merge {
                if let Some(best_branch) = extract_best_schema_from_union(&union_array) {
                    if let Value::Object(branch_obj) = best_branch {
                        // Merge branch attributes into the current map
                        for (k, v) in branch_obj {
                            if k == "properties" {
                                if let Some(target_props) = map
                                    .entry("properties".to_string())
                                    .or_insert_with(|| Value::Object(serde_json::Map::new()))
                                    .as_object_mut()
                                {
                                    if let Some(source_props) = v.as_object() {
                                        for (pk, pv) in source_props {
                                            target_props
                                                .entry(pk.clone())
                                                .or_insert_with(|| pv.clone());
                                        }
                                    }
                                }
                            } else if k == "required" {
                                if let Some(target_req) = map
                                    .entry("required".to_string())
                                    .or_insert_with(|| Value::Array(Vec::new()))
                                    .as_array_mut()
                                {
                                    if let Some(source_req) = v.as_array() {
                                        for rv in source_req {
                                            if !target_req.contains(rv) {
                                                target_req.push(rv.clone());
                                            }
                                        }
                                    }
                                }
                            } else if !map.contains_key(&k) {
                                map.insert(k, v);
                            }
                        }
                    }
                }
            }

            // 3. [SAFETY] Check whether the current object is a JSON Schema node
            // Whitelist filtering is performed only if the object looks like the Schema (contains type, properties, items, enum, anyOf, etc.).
            // Otherwise, if it is a normal Value (such as a functionCall object in request.rs), applying aggressive filtering directly would break the structure.
            let allowed_fields = [
                "type",
                "description",
                "properties",
                "required",
                "items",
                "enum",
                "title",
            ];

            let has_standard_keyword = map.keys().any(|k| allowed_fields.contains(&k.as_str()));

            // [NEW] Heuristic fix: If it is clearly a Schema node, but there is no standard key, there is other Key
            // We speculate that this is a "shorthand" object definition and try to move its internal Key into properties.
            // Supplement: You must ensure that it is not a tool call or result (containing functionCall/functionResponse/args, etc.) to prevent the runtime parameter structure from being destroyed.
            let is_not_schema_payload = map.contains_key("functionCall")
                || map.contains_key("functionResponse")
                || map.contains_key("args")
                || map.contains_key("tool_calls")
                || map.contains_key("tool_call_id");
            if is_schema_node && !has_standard_keyword && !map.is_empty() && !is_not_schema_payload
            {
                let mut properties = serde_json::Map::new();
                let keys: Vec<String> = map.keys().cloned().collect();
                for k in keys {
                    if let Some(v) = map.remove(&k) {
                        properties.insert(k, v);
                    }
                }
                map.insert("type".to_string(), Value::String("object".to_string()));
                map.insert("properties".to_string(), Value::Object(properties));

                // Recursively clean up the properties just moved in
                if let Some(Value::Object(props_map)) = map.get_mut("properties") {
                    for v in props_map.values_mut() {
                        clean_json_schema_recursive(v, true, depth + 1);
                    }
                }
            }

            let looks_like_schema = is_schema_node && !is_not_schema_payload;

            if looks_like_schema {
                // 4. [CRITICAL] Whitelist filtering: completely physically remove content not supported by Gemini to prevent 400 errors
                let keys_to_remove: Vec<String> = map
                    .keys()
                    .filter(|k| !allowed_fields.contains(&k.as_str()))
                    .cloned()
                    .collect();
                for k in keys_to_remove {
                    map.remove(&k);
                }

                // 5. Keep the client’s original description text, without folding or line breaks, or truncating the content, ensuring complete fidelity and transparent transmission.

                // 6. [SAFETY] Handling empty Object
                // [FIX] Remove reason field injection logic
                // The previous implementation would inject the reason field into an empty Object, causing tools such as Gemini CLI to report "malformed function call"
                // Because the model will generate a call containing the reason parameter, but this parameter is not included in the tool definition.
                // Now change to: Empty Object Keep empty properties and let the Gemini model decide whether it needs parameters.
                let is_object_type = map
                    .get("type")
                    .and_then(|t| t.as_str())
                    .map(|t| t.eq_ignore_ascii_case("object"))
                    .unwrap_or(false);
                if is_object_type {
                    if !map.contains_key("properties") {
                        map.insert("properties".to_string(), serde_json::json!({}));
                    }
                }

                // 7. [SAFETY] Required field alignment
                let valid_prop_keys: Option<std::collections::HashSet<String>> = map
                    .get("properties")
                    .and_then(|p| p.as_object())
                    .map(|obj| obj.keys().cloned().collect());

                if let Some(required_val) = map.get_mut("required") {
                    if let Some(req_arr) = required_val.as_array_mut() {
                        if let Some(keys) = &valid_prop_keys {
                            req_arr
                                .retain(|k| k.as_str().map(|s| keys.contains(s)).unwrap_or(false));
                        } else {
                            req_arr.clear();
                        }
                    }
                }

                if !map.contains_key("type") {
                    if map.contains_key("enum") {
                        map.insert("type".to_string(), Value::String("string".to_string()));
                    } else if map.contains_key("properties") {
                        map.insert("type".to_string(), Value::String("object".to_string()));
                    } else if map.contains_key("items") {
                        map.insert("type".to_string(), Value::String("array".to_string()));
                    }
                }

                // [IMPROVED] Calculate fallback types ahead of time to avoid borrow conflicts
                let fallback = if map.contains_key("properties") {
                    "object"
                } else if map.contains_key("items") {
                    "array"
                } else {
                    "string"
                };

                // 8. Process the type field
                if let Some(type_val) = map.get_mut("type") {
                    let mut selected_type = None;
                    match type_val {
                        Value::String(s) => {
                            let lower = s.to_lowercase();
                            if lower == "null" {
                                is_effectively_nullable = true;
                            } else {
                                selected_type = Some(lower);
                            }
                        }
                        Value::Array(arr) => {
                            for item in arr {
                                if let Value::String(s) = item {
                                    let lower = s.to_lowercase();
                                    if lower == "null" {
                                        is_effectively_nullable = true;
                                    } else if selected_type.is_none() {
                                        selected_type = Some(lower);
                                    }
                                }
                            }
                        }
                        _ => {}
                    }

                    *type_val =
                        Value::String(selected_type.unwrap_or_else(|| fallback.to_string()));
                }

                if is_effectively_nullable {
                    let desc_val = map
                        .entry("description".to_string())
                        .or_insert_with(|| Value::String("".to_string()));
                    if let Value::String(s) = desc_val {
                        if !s.contains("nullable") {
                            if !s.is_empty() {
                                s.push(' ');
                            }
                            s.push_str("(nullable)");
                        }
                    }
                }

                // 9. Enum value forced conversion to string
                if let Some(Value::Array(arr)) = map.get_mut("enum") {
                    for item in arr {
                        if !item.is_string() {
                            *item = Value::String(if item.is_null() {
                                "null".to_string()
                            } else {
                                item.to_string()
                            });
                        }
                    }
                }
            }
        }
        Value::Array(arr) => {
            // [FIX] Clean each element in an array recursively
            // This ensures that all array type values (including but not limited to anyOf, oneOf, items, enum, etc.) will be processed recursively
            for item in arr.iter_mut() {
                clean_json_schema_recursive(item, is_schema_node, depth + 1);
            }
        }
        _ => {}
    }

    is_effectively_nullable
}
