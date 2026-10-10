// Recursive schema cleaner (split from json_schema.rs).
use super::refs::{extract_best_schema_from_union, merge_all_of};
use super::MAX_RECURSION_DEPTH;
use serde_json::{json, Value};

fn clean_json_schema_recursive(value: &mut Value, is_schema_node: bool, depth: usize) -> bool {
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
            // 0. [NEW] 合并 allOf
            merge_all_of(map);

            // 0.1 [NEW #3327] 规范化 const 关键字 (转换为 enum 与对应 type)
            // Gemini/Vertex 的 Schema proto 不支持 const，直接传入会导致 400 INVALID_ARGUMENT
            // 例如 {"const": "element"} -> {"type": "string", "enum": ["element"]}
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

            // 0.5 [NEW] 结构归一化 (Normalization)
            // 针对某些 MCP 工具（如 pencil）误用 items 定义对象属性的情况进行修复。
            // 如果 type=object 或包含 properties，但又定义了 items，Gemini 会因为 items 只能出现在 array 中而报错。
            // 我们将 items 的内容“对齐”到 properties 中。
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

            // 1. [CRITICAL] 深度递归处理子项
            // 处理 properties (对象)
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
                    // properties 的每一个值都必须是一个独立的 Schema 节点
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

                // [NEW] 隐式类型注入：如果有 properties 但没 type，补全为 object
                if !map.contains_key("type") {
                    map.insert("type".to_string(), Value::String("object".to_string()));
                }
            }

            // 处理 items (数组)
            // [FIX] items must be a Schema object; drop bare boolean / invalid items
            // (JSON Schema allows boolean `items`, Gemini's Schema proto rejects it).
            if map.get("items").map(|i| !i.is_object()).unwrap_or(false) {
                map.remove("items");
            }
            if let Some(items) = map.get_mut("items") {
                // items 的内容必须是一个独立的 Schema 节点
                clean_json_schema_recursive(items, true, depth + 1);

                // [NEW] 隐式类型注入：如果有 items 但没 type，补全为 array
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

            // Fallback: 对既没有 properties 也没有 items 的常规对象进行清理
            if !map.contains_key("properties") && !map.contains_key("items") {
                for (k, v) in map.iter_mut() {
                    // 排除掉关键字
                    if k != "anyOf" && k != "oneOf" && k != "allOf" && k != "enum" && k != "type" {
                        clean_json_schema_recursive(v, false, depth + 1);
                    }
                }
            }

            // 1.5. [FIX] 递归清理 anyOf/oneOf 数组中的每个分支
            // 必须在合并逻辑之前执行，确保合并的分支已经被清洗
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

            // 2. [FIX #815] 处理 anyOf/oneOf 联合类型: 合并属性或择优选择分支
            let mut union_to_merge = None;
            if let Some(Value::Array(any_of)) = map.get("anyOf") {
                union_to_merge = Some(any_of.clone());
            } else if let Some(Value::Array(one_of)) = map.get("oneOf") {
                union_to_merge = Some(one_of.clone());
            }

            if let Some(union_array) = union_to_merge {
                if let Some(best_branch) = extract_best_schema_from_union(&union_array) {
                    if let Value::Object(branch_obj) = best_branch {
                        // 合并分支属性到当前 map
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

            // 3. [SAFETY] 检查当前对象是否为 JSON Schema 节点
            // 只有当对象看起来像 Schema (包含 type, properties, items, enum, anyOf 等) 时，才执行白名单过滤。
            // 否则，如果它是一个普通的 Value (如 request.rs 中的 functionCall 对象)，直接应用激进过滤会破坏结构。
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

            // [NEW] 启发式修复：如果明确是 Schema 节点，但没有标准关键字，却有其他 Key
            // 我们推测这是一个“简写”的对象定义，尝试将其内部 Key 移动到 properties 中。
            // 补充：必须确保它不是工具调用或结果 (含有 functionCall/functionResponse/args 等)，防止运行时参数结构被破坏。
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

                // 递归清理刚刚移动进去的属性
                if let Some(Value::Object(props_map)) = map.get_mut("properties") {
                    for v in props_map.values_mut() {
                        clean_json_schema_recursive(v, true, depth + 1);
                    }
                }
            }

            let looks_like_schema = is_schema_node && !is_not_schema_payload;

            if looks_like_schema {
                // 4. [CRITICAL] 白名单过滤：彻底物理移除 Gemini 不支持的内容，防止 400 错误
                let keys_to_remove: Vec<String> = map
                    .keys()
                    .filter(|k| !allowed_fields.contains(&k.as_str()))
                    .cloned()
                    .collect();
                for k in keys_to_remove {
                    map.remove(&k);
                }

                // 5. 保持客户端原始 description 文本，不折叠换行，不截断内容，保证完全保真透传

                // 6. [SAFETY] 处理空 Object
                // [FIX] 移除 reason 字段注入逻辑
                // 之前的实现会为空 Object 注入 reason 字段，导致 Gemini CLI 等工具报 "malformed function call"
                // 因为模型会生成包含 reason 参数的调用，但工具定义中并没有这个参数
                // 现在改为：空 Object 保持空的 properties，让 Gemini 模型自行决定是否需要参数
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

                // 7. [SAFETY] Required 字段对齐
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

                // [IMPROVED] 提前计算回退类型以避免借用冲突
                let fallback = if map.contains_key("properties") {
                    "object"
                } else if map.contains_key("items") {
                    "array"
                } else {
                    "string"
                };

                // 8. 处理 type 字段
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

                // 9. Enum 值强制转字符串
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
            // [FIX] 递归清理数组中的每个元素
            // 这确保了所有数组类型的值（包括但不限于 anyOf、oneOf、items、enum 等）都会被递归处理
            for item in arr.iter_mut() {
                clean_json_schema_recursive(item, is_schema_node, depth + 1);
            }
        }
        _ => {}
    }

    is_effectively_nullable
}
