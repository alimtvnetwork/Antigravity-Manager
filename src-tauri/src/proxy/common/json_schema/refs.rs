// $ref / allOf / union helpers (split from json_schema.rs).
use serde_json::Value;

/// [NEW] Merge all sub-Schemas in allOf array
pub(crate) fn merge_all_of(map: &mut serde_json::Map<String, Value>) {
    if let Some(Value::Array(all_of)) = map.remove("allOf") {
        let mut merged_properties = serde_json::Map::new();
        let mut merged_required = std::collections::HashSet::new();
        let mut other_fields = serde_json::Map::new();

        for sub_schema in all_of {
            if let Value::Object(sub_map) = sub_schema {
                // Merge properties
                if let Some(Value::Object(props)) = sub_map.get("properties") {
                    for (k, v) in props {
                        merged_properties.insert(k.clone(), v.clone());
                    }
                }

                // Merge required
                if let Some(Value::Array(reqs)) = sub_map.get("required") {
                    for req in reqs {
                        if let Some(s) = req.as_str() {
                            merged_required.insert(s.to_string());
                        }
                    }
                }

                // Merge remaining fields (first occurrence wins)
                for (k, v) in sub_map {
                    if k != "properties"
                        && k != "required"
                        && k != "allOf"
                        && !other_fields.contains_key(&k)
                    {
                        other_fields.insert(k, v);
                    }
                }
            }
        }

        // Apply merged fields
        for (k, v) in other_fields {
            if !map.contains_key(&k) {
                map.insert(k, v);
            }
        }

        if !merged_properties.is_empty() {
            let existing_props = map
                .entry("properties".to_string())
                .or_insert_with(|| Value::Object(serde_json::Map::new()));
            if let Value::Object(existing_map) = existing_props {
                for (k, v) in merged_properties {
                    existing_map.entry(k).or_insert(v);
                }
            }
        }

        if !merged_required.is_empty() {
            let existing_reqs = map
                .entry("required".to_string())
                .or_insert_with(|| Value::Array(Vec::new()));
            if let Value::Array(req_arr) = existing_reqs {
                let mut current_reqs: std::collections::HashSet<String> = req_arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();
                for req in merged_required {
                    if current_reqs.insert(req.clone()) {
                        req_arr.push(Value::String(req));
                    }
                }
            }
        }
    }
}

/// [NEW] Calculate the complexity score of Schema branches (for use in anyOf/oneOf selection)
/// Scoring criteria: Object (3) > Array (2) > Scalar (1) > Null (0)
fn score_schema_option(val: &Value) -> i32 {
    if let Value::Object(obj) = val {
        if obj.contains_key("properties")
            || obj.get("type").and_then(|t| t.as_str()) == Some("object")
        {
            return 3;
        }
        if obj.contains_key("items") || obj.get("type").and_then(|t| t.as_str()) == Some("array") {
            return 2;
        }
        if let Some(type_str) = obj.get("type").and_then(|t| t.as_str()) {
            if type_str != "null" {
                return 1;
            }
        }
    }
    0
}

/// Select the best non-null Schema branch from anyOf/oneOf union type array
pub(crate) fn extract_best_schema_from_union(union_array: &[Value]) -> Option<Value> {
    let mut best_option: Option<&Value> = None;
    let mut best_score = -1;

    for item in union_array {
        let score = score_schema_option(item);
        if score > best_score {
            best_score = score;
            best_option = Some(item);
        }
    }

    best_option.cloned()
}
