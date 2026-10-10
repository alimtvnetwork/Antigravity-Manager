// JSON Schema tests (split from json_schema.rs).
// Test-only module, declared #[cfg(test)] in mod.rs.
use super::*;
use serde_json::json;

fn test_drops_boolean_subschemas() {
    // JSON Schema permits boolean sub-schemas (`prop: true|false`), but Gemini's
    // Schema proto rejects a non-object property value with HTTP 400. They must be
    // stripped at every depth (including inside `items`).
    let mut schema = json!({
        "type": "object",
        "properties": {
            "outer": {
                "type": "object",
                "properties": {
                    "forbidden": false,
                    "allowed": { "type": "string" }
                },
                "required": ["forbidden", "allowed"]
            },
            "list": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "nope": false,
                        "ok": { "type": "number" }
                    }
                }
            }
        }
    });
    clean_json_schema(&mut schema);

    let outer_props = &schema["properties"]["outer"]["properties"];
    assert!(
        outer_props.get("forbidden").is_none(),
        "boolean sub-schema must be dropped"
    );
    assert!(
        outer_props["allowed"].is_object(),
        "valid sibling must survive"
    );

    let item_props = &schema["properties"]["list"]["items"]["properties"];
    assert!(
        item_props.get("nope").is_none(),
        "nested boolean sub-schema must be dropped"
    );
    assert!(item_props["ok"].is_object());

    let req = schema["properties"]["outer"]["required"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    assert!(req.iter().all(|r| r.as_str() != Some("forbidden")));
}

fn test_clean_json_schema_draft_2020_12() {
    let mut schema = json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "properties": {
            "location": {
                "type": "string",
                "description": "The city and state, e.g. San Francisco, CA",
                "minLength": 1,
                "format": "city"
            },
            // 模拟属性名冲突：pattern 是一个 Object 属性，不应被移除
            "pattern": {
                "type": "object",
                "properties": {
                    "regex": {
                        "type": "string",
                        "description": "Regex pattern",
                        "pattern": "^[a-z]+$"
                    }
                }
            },
            "unit": {
                "type": ["string", "null"],
                "default": "celsius"
            }
        },
        "required": ["location"]
    });

    clean_json_schema(&mut schema);

    // 1. 验证类型保持小写
    assert_eq!(schema["type"], "object");
    assert_eq!(schema["properties"]["location"]["type"], "string");

    // 2. 验证非标准约束字段被安全移除，且描述保持纯透传不被篡改
    assert!(schema["properties"]["location"].get("minLength").is_none());
    assert!(schema["properties"]["location"].get("format").is_none());
    assert_eq!(
        schema["properties"]["location"]["description"],
        "The city and state, e.g. San Francisco, CA"
    );

    // 3. 验证名为 "pattern" 的属性未被误删
    assert!(schema["properties"].get("pattern").is_some());
    assert_eq!(schema["properties"]["pattern"]["type"], "object");

    // 4. 验证内部的 pattern 校验字段被移除，且描述保持原样
    assert!(schema["properties"]["pattern"]["properties"]["regex"]
        .get("pattern")
        .is_none());
    assert_eq!(
        schema["properties"]["pattern"]["properties"]["regex"]["description"],
        "Regex pattern"
    );

    // 5. 验证联合类型被降级为单一类型 (Protobuf 兼容性)
    assert_eq!(schema["properties"]["unit"]["type"], "string");

    // 6. 验证元数据字段被移除
    assert!(schema.get("$schema").is_none());
}

fn test_type_fallback() {
    // Test ["string", "null"] -> "string"
    let mut s1 = json!({"type": ["string", "null"]});
    clean_json_schema(&mut s1);
    assert_eq!(s1["type"], "string");

    // Test ["integer", "null"] -> "integer" (and lowercase check if needed, though usually integer)
    let mut s2 = json!({"type": ["integer", "null"]});
    clean_json_schema(&mut s2);
    assert_eq!(s2["type"], "integer");
}

fn test_flatten_refs() {
    let mut schema = json!({
        "$defs": {
            "Address": {
                "type": "object",
                "properties": {
                    "city": { "type": "string" }
                }
            }
        },
        "properties": {
            "home": { "$ref": "#/$defs/Address" }
        }
    });

    clean_json_schema(&mut schema);

    // 验证引用被展开且类型转为小写
    assert_eq!(schema["properties"]["home"]["type"], "object");
    assert_eq!(
        schema["properties"]["home"]["properties"]["city"]["type"],
        "string"
    );
}

fn test_clean_json_schema_missing_required() {
    let mut schema = json!({
        "type": "object",
        "properties": {
            "existing_prop": { "type": "string" }
        },
        "required": ["existing_prop", "missing_prop"]
    });

    clean_json_schema(&mut schema);

    // 验证 missing_prop 被从 required 中移除
    let required = schema["required"].as_array().unwrap();
    assert_eq!(required.len(), 1);
    assert_eq!(required[0].as_str().unwrap(), "existing_prop");
}

fn test_anyof_type_extraction() {
    // 测试 FastMCP 风格的 Optional[str] schema
    let mut schema = json!({
        "type": "object",
        "properties": {
            "testo": {
                "anyOf": [
                    {"type": "string"},
                    {"type": "null"}
                ],
                "default": null,
                "title": "Testo"
            },
            "importo": {
                "anyOf": [
                    {"type": "number"},
                    {"type": "null"}
                ],
                "default": null,
                "title": "Importo"
            },
            "attivo": {
                "type": "boolean",
                "title": "Attivo"
            }
        }
    });

    clean_json_schema(&mut schema);

    // 验证 anyOf 被移除
    assert!(schema["properties"]["testo"].get("anyOf").is_none());
    assert!(schema["properties"]["importo"].get("anyOf").is_none());

    // 验证 type 被正确提取
    assert_eq!(schema["properties"]["testo"]["type"], "string");
    assert_eq!(schema["properties"]["importo"]["type"], "number");
    assert_eq!(schema["properties"]["attivo"]["type"], "boolean");

    // 验证 default 被移除 (白名单之外)
    assert!(schema["properties"]["testo"].get("default").is_none());
}

fn test_oneof_type_extraction() {
    let mut schema = json!({
        "properties": {
            "value": {
                "oneOf": [
                    {"type": "integer"},
                    {"type": "null"}
                ]
            }
        }
    });

    clean_json_schema(&mut schema);

    assert!(schema["properties"]["value"].get("oneOf").is_none());
    assert_eq!(schema["properties"]["value"]["type"], "integer");
}

fn test_existing_type_preserved() {
    let mut schema = json!({
        "properties": {
            "name": {
                "type": "string",
                "anyOf": [
                    {"type": "number"}
                ]
            }
        }
    });

    clean_json_schema(&mut schema);

    // type 已存在，不应被 anyOf 中的类型覆盖
    assert_eq!(schema["properties"]["name"]["type"], "string");
    assert!(schema["properties"]["name"].get("anyOf").is_none());
}

fn test_issue_815_anyof_properties_preserved() {
    let mut schema = json!({
        "type": "object",
        "properties": {
            "config": {
                "anyOf": [
                    {
                        "type": "object",
                        "properties": {
                            "path": { "type": "string" },
                            "recursive": { "type": "boolean" }
                        },
                        "required": ["path"]
                    },
                    { "type": "null" }
                ]
            }
        }
    });

    clean_json_schema(&mut schema);

    let config = &schema["properties"]["config"];

    // 1. 验证类型被提取
    assert_eq!(config["type"], "object");

    // 2. 验证 anyOf 内部的 properties 被合并上来了
    assert!(config.get("properties").is_some());
    assert_eq!(config["properties"]["path"]["type"], "string");
    assert_eq!(config["properties"]["recursive"]["type"], "boolean");

    // 3. 验证 required 被合并上来了
    let req = config["required"].as_array().unwrap();
    assert!(req.iter().any(|v| v == "path"));

    // 4. 验证 anyOf 字段本身被移除
    assert!(config.get("anyOf").is_none());

    // 5. 验证没有因为“空”而注入 reason (因为我们保留了属性)
    assert!(config["properties"].get("reason").is_none());
}

fn test_clean_json_schema_on_non_schema_object() {
    // 模拟包含 description、command、type 等常规业务字段的运行时工具调用对象
    let mut tool_call = json!({
        "functionCall": {
            "name": "run_command",
            "args": {
                "description": "Run: git status",
                "command": "git status",
                "shell": "default"
            },
            "id": "call_123"
        }
    });

    // 调用清洗逻辑
    clean_json_schema(&mut tool_call);

    // 验证：非 Schema 字段与运行时实参绝对不应被剥离，command 与 description 必须同时完好保留
    let fc = &tool_call["functionCall"];
    assert_eq!(fc["name"], "run_command");
    assert_eq!(fc["args"]["command"], "git status");
    assert_eq!(fc["args"]["description"], "Run: git status");
    assert_eq!(fc["args"]["shell"], "default");
    assert_eq!(fc["id"], "call_123");
}

fn test_nullable_handling_with_description() {
    let mut schema = json!({
        "type": ["string", "null"],
        "description": "User name"
    });

    clean_json_schema(&mut schema);

    // 验证 type 被降级，且描述被追加 (nullable)
    assert_eq!(schema["type"], "string");
    assert!(schema["description"]
        .as_str()
        .unwrap()
        .contains("User name"));
    assert!(schema["description"]
        .as_str()
        .unwrap()
        .contains("(nullable)"));
}

fn test_clean_anyof_with_propertynames() {
    let mut schema = json!({
        "properties": {
            "config": {
                "anyOf": [
                    {
                        "type": "object",
                        "propertyNames": {"pattern": "^[a-z]+$"},
                        "properties": {
                            "key": {"type": "string"}
                        }
                    },
                    {"type": "null"}
                ]
            }
        }
    });

    clean_json_schema(&mut schema);

    // 验证 anyOf 被移除（已被合并）
    let config = &schema["properties"]["config"];
    assert!(config.get("anyOf").is_none());

    // 验证 propertyNames 被移除
    assert!(config.get("propertyNames").is_none());

    // 验证合并后的 properties 存在且没有 propertyNames
    assert!(config.get("properties").is_some());
    assert_eq!(config["properties"]["key"]["type"], "string");
}

fn test_clean_items_array_with_const() {
    let mut schema = json!({
        "type": "array",
        "items": {
            "type": "object",
            "properties": {
                "status": {
                    "const": "active",
                    "type": "string"
                }
            }
        }
    });

    clean_json_schema(&mut schema);

    // 验证 const 被移除
    let status = &schema["items"]["properties"]["status"];
    assert!(status.get("const").is_none());

    // 验证 type 仍然存在
    assert_eq!(status["type"], "string");
}

fn test_deep_nested_array_cleaning() {
    let mut schema = json!({
        "properties": {
            "data": {
                "anyOf": [
                    {
                        "type": "array",
                        "items": {
                            "anyOf": [
                                {
                                    "type": "object",
                                    "propertyNames": {"maxLength": 10},
                                    "const": "test",
                                    "properties": {
                                        "name": {"type": "string"}
                                    }
                                },
                                {"type": "null"}
                            ]
                        }
                    }
                ]
            }
        }
    });

    clean_json_schema(&mut schema);

    // 验证深层嵌套的非法字段都被移除
    let data = &schema["properties"]["data"];

    // anyOf 应该被合并移除
    assert!(data.get("anyOf").is_none());

    // 验证没有 propertyNames 和 const 逃逸到顶层
    assert!(data.get("propertyNames").is_none());
    assert!(data.get("const").is_none());

    // 验证结构被正确保留
    assert_eq!(data["type"], "array");
    if let Some(items) = data.get("items") {
        // items 内部的 anyOf 也应该被合并
        assert!(items.get("anyOf").is_none());
        assert!(items.get("propertyNames").is_none());
        assert!(items.get("const").is_none());
    }
}

fn test_sanitize_description() {
    let multi_line = "This is a tool description\nwith multiple lines\r\nand   extra   spaces.";
    assert_eq!(sanitize_description(multi_line), multi_line);

    let overlong = "a".repeat(10000);
    let sanitized = sanitize_description(&overlong);
    assert!(sanitized.len() <= MAX_DESCRIPTION_LENGTH);
    assert!(sanitized.ends_with("... [truncated]"));
}

fn test_clean_json_schema_ensures_properties_on_object() {
    let mut schema = json!({
        "type": "OBJECT",
        "description": "Some description\nwith newlines"
    });

    clean_json_schema(&mut schema);
    assert_eq!(schema["type"], "object");
    assert_eq!(schema["properties"], json!({}));
    assert_eq!(schema["description"], "Some description\nwith newlines");
}
