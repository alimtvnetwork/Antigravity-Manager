// Tool-call argument fixing (split from json_schema.rs).
use serde_json::Value;

/// 修正工具调用参数的类型，使其符合 schema 定义
///
/// 根据 schema 中的 type 定义，自动转换参数值的类型：
/// - "123" → 123 (string → number/integer)
/// - "true" → true (string → boolean)
/// - 123 → "123" (number → string)
///
/// # Arguments
/// * `args` - 工具调用的参数对象 (会被原地修改)
/// * `schema` - 工具的参数 schema 定义 (通常是 parameters 对象)
pub fn fix_tool_call_args(args: &mut Value, schema: &Value) {
    if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
        if let Some(args_obj) = args.as_object_mut() {
            for (key, value) in args_obj.iter_mut() {
                if let Some(prop_schema) = properties.get(key) {
                    fix_single_arg_recursive(value, prop_schema);
                }
            }
        }
    }
}

/// 递归修正单个参数的类型
pub fn fix_single_arg_recursive(value: &mut Value, schema: &Value) {
    // 1. 处理嵌套对象 (properties)
    if let Some(nested_props) = schema.get("properties").and_then(|p| p.as_object()) {
        if let Some(value_obj) = value.as_object_mut() {
            for (key, nested_value) in value_obj.iter_mut() {
                if let Some(nested_schema) = nested_props.get(key) {
                    fix_single_arg_recursive(nested_value, nested_schema);
                }
            }
        }
        return;
    }

    // 2. 处理数组 (items)
    let schema_type = schema
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_lowercase();
    if schema_type == "array" {
        if let Some(items_schema) = schema.get("items") {
            if let Some(arr) = value.as_array_mut() {
                for item in arr {
                    fix_single_arg_recursive(item, items_schema);
                }
            }
        }
        return;
    }

    // 3. 处理基础类型修正
    match schema_type.as_str() {
        "number" | "integer" => {
            // 字符串 → 数字
            if let Some(s) = value.as_str() {
                // [SAFETY] 保护具有前导零的版本号或代码 (如 "01", "007")，不应转为数字
                if s.starts_with('0') && s.len() > 1 && !s.starts_with("0.") {
                    return;
                }

                // 优先尝试解析为整数
                if let Ok(i) = s.parse::<i64>() {
                    *value = Value::Number(serde_json::Number::from(i));
                } else if let Ok(f) = s.parse::<f64>() {
                    if let Some(n) = serde_json::Number::from_f64(f) {
                        *value = Value::Number(n);
                    }
                }
            }
        }
        "boolean" => {
            // 字符串 → 布尔
            if let Some(s) = value.as_str() {
                match s.to_lowercase().as_str() {
                    "true" | "1" | "yes" | "on" => *value = Value::Bool(true),
                    "false" | "0" | "no" | "off" => *value = Value::Bool(false),
                    _ => {}
                }
            } else if let Some(n) = value.as_i64() {
                // 数字 1/0 -> 布尔
                if n == 1 {
                    *value = Value::Bool(true);
                } else if n == 0 {
                    *value = Value::Bool(false);
                }
            }
        }
        "string" => {
            // 非字符串 → 字符串 (防止客户端误传数字给文本字段)
            if !value.is_string() && !value.is_null() && !value.is_object() && !value.is_array() {
                *value = Value::String(value.to_string());
            }
        }
        _ => {}
    }
}
