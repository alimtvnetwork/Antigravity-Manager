// Schema cleaning entry points (split from json_schema.rs).
use super::refs::{extract_best_schema_from_union, merge_all_of};
use super::{MAX_DESCRIPTION_LENGTH, MAX_RECURSION_DEPTH};
use serde_json::{json, Value};
use crate::proxy::common::json_schema::recursive::clean_json_schema_recursive;

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

/// 递归清理 JSON Schema 以符合 Gemini 接口要求
///
/// 1. [New] 展开 $ref 和 $defs: 将引用替换为实际定义，解决 Gemini 不支持 $ref 的问题
/// 2. 移除不支持的字段: $schema, additionalProperties, format, default, uniqueItems, validation fields
/// 3. 处理联合类型: ["string", "null"] -> "string"
/// 4. [NEW] 处理 anyOf 联合类型: anyOf: [{"type": "string"}, {"type": "null"}] -> "type": "string"
/// 5. 将 type 字段的值转换为小写 (Gemini v1internal 要求)
/// 6. 移除数字校验字段: multipleOf, exclusiveMinimum, exclusiveMaximum 等
/// 清洗用于 responseSchema 的 JSON Schema
pub fn clean_response_schema(value: &mut Value) {
    clean_json_schema(value);
}

pub fn clean_json_schema(value: &mut Value) {
    // 0. 预处理：展开 $ref (Schema Flattening)
    // [FIX #952] 递归收集所有层级的 $defs/definitions，而非仅从根层级提取
    let mut all_defs = serde_json::Map::new();
    collect_all_defs(value, &mut all_defs);

    // 移除根层级的 $defs/definitions (保持向后兼容)
    if let Value::Object(map) = value {
        map.remove("$defs");
        map.remove("definitions");
    }

    // [FIX #952] 始终运行 flatten_refs，即使 defs 为空
    // 这样可以捕获并处理无法解析的 $ref (降级为 string 类型)
    if let Value::Object(map) = value {
        flatten_refs(map, &all_defs, 0);
    }

    // 递归清理
    clean_json_schema_recursive(value, true, 0);
}

/// 清洗 JSON Schema 以符合 Gemini 接口要求
///
/// 遵循纯粹协议透传原则，不执行任何特定工具的私有适配逻辑
pub fn clean_json_schema_for_tool(value: &mut Value, _tool_name: &str) {
    clean_json_schema(value);
}

/// [NEW #952] 递归收集所有层级的 $defs 和 definitions
///
/// MCP 工具的 schema 可能在任意嵌套层级定义 $defs，而非仅在根层级。
/// 此函数深度遍历整个 schema，收集所有定义到统一的 map 中。
fn collect_all_defs(value: &Value, defs: &mut serde_json::Map<String, Value>) {
    if let Value::Object(map) = value {
        // 收集当前层级的 $defs
        if let Some(Value::Object(d)) = map.get("$defs") {
            for (k, v) in d {
                // 避免覆盖已存在的定义（先定义的优先）
                defs.entry(k.clone()).or_insert_with(|| v.clone());
            }
        }
        // 收集当前层级的 definitions (Draft-07 风格)
        if let Some(Value::Object(d)) = map.get("definitions") {
            for (k, v) in d {
                defs.entry(k.clone()).or_insert_with(|| v.clone());
            }
        }
        // 递归处理所有子节点
        for (key, v) in map {
            // 跳过 $defs/definitions 本身，避免重复处理
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

/// 递归展开 $ref
fn flatten_refs(
    map: &mut serde_json::Map<String, Value>,
    defs: &serde_json::Map<String, Value>,
    depth: usize,
) {
    if depth > MAX_RECURSION_DEPTH {
        tracing::warn!("[Schema-Flatten] Max recursion depth reached, stopping ref expansion.");
        return;
    }

    // 检查并替换 $ref
    if let Some(Value::String(ref_path)) = map.remove("$ref") {
        // 解析引用名 (例如 #/$defs/MyType -> MyType)
        let ref_name = ref_path.split('/').last().unwrap_or(&ref_path);

        if let Some(def_schema) = defs.get(ref_name) {
            // 将定义的内容合并到当前 map
            if let Value::Object(def_map) = def_schema {
                for (k, v) in def_map {
                    // 仅当当前 map 没有该 key 时才插入 (避免覆盖)
                    // 但通常 $ref 节点不应该有其他属性
                    map.entry(k.clone()).or_insert_with(|| v.clone());
                }

                // 递归处理刚刚合并进来的内容中可能包含的 $ref
                // 注意：由于引入了 depth 限制，循环引用不再会导致栈溢出
                flatten_refs(map, defs, depth + 1);
            }
        } else {
            // [FIX #952] 无法解析的 $ref: 转换为宽松的 string 类型，避免 API 400 错误
            // 这比让请求失败要好，至少工具调用仍可进行
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

    // 遍历子节点
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
