// JSON Schema tests (split from json_schema.rs).
// Test-only module, declared #[cfg(test)] in mod.rs.
use super::*;
use serde_json::json;

fn test_fix_tool_call_args() {
    let mut args = serde_json::json!({
        "port": "8080",
        "enabled": "true",
        "timeout": "5.5",
        "metadata": {
            "retry": "3"
        },
        "tags": ["1", "2"]
    });

    let schema = serde_json::json!({
        "properties": {
            "port": { "type": "integer" },
            "enabled": { "type": "boolean" },
            "timeout": { "type": "number" },
            "metadata": {
                "type": "object",
                "properties": {
                    "retry": { "type": "integer" }
                }
            },
            "tags": {
                "type": "array",
                "items": { "type": "integer" }
            }
        }
    });

    fix_tool_call_args(&mut args, &schema);

    assert_eq!(args["port"], 8080);
    assert_eq!(args["enabled"], true);
    assert_eq!(args["timeout"], 5.5);
    assert_eq!(args["metadata"]["retry"], 3);
    assert_eq!(args["tags"], serde_json::json!([1, 2]));
}

fn test_fix_tool_call_args_protection() {
    let mut args = serde_json::json!({
        "version": "01.0",
        "code": "007"
    });

    let schema = serde_json::json!({
        "properties": {
            "version": { "type": "number" },
            "code": { "type": "integer" }
        }
    });

    fix_tool_call_args(&mut args, &schema);

    // 应保留字符串以防破坏语义
    assert_eq!(args["version"], "01.0");
    assert_eq!(args["code"], "007");
}

fn test_nested_defs_flattening() {
    // MCP 工具常常将 $defs 嵌套在 properties 内部，而非根层级
    let mut schema = json!({
        "type": "object",
        "properties": {
            "config": {
                "$defs": {
                    "Address": {
                        "type": "object",
                        "properties": {
                            "city": { "type": "string" },
                            "zip": { "type": "string" }
                        }
                    }
                },
                "type": "object",
                "properties": {
                    "home": { "$ref": "#/$defs/Address" },
                    "work": { "$ref": "#/$defs/Address" }
                }
            }
        }
    });

    clean_json_schema(&mut schema);

    // 验证嵌套的 $ref 被正确解析
    let home = &schema["properties"]["config"]["properties"]["home"];
    assert_eq!(
        home["type"], "object",
        "home should have type 'object' from resolved $ref"
    );
    assert_eq!(
        home["properties"]["city"]["type"], "string",
        "home.properties.city should exist from resolved Address"
    );

    // 验证没有残留的 $ref
    assert!(
        home.get("$ref").is_none(),
        "home should not have orphan $ref"
    );

    // 验证 work 也被正确解析
    let work = &schema["properties"]["config"]["properties"]["work"];
    assert_eq!(work["type"], "object");
    assert!(work.get("$ref").is_none());
}

fn test_unresolved_ref_fallback() {
    let mut schema = json!({
        "type": "object",
        "properties": {
            "external": { "$ref": "https://example.com/schemas/External.json" },
            "missing": { "$ref": "#/$defs/NonExistent" }
        }
    });

    clean_json_schema(&mut schema);

    // 验证外部引用被降级为 string 类型
    let external = &schema["properties"]["external"];
    assert_eq!(
        external["type"], "string",
        "unresolved external $ref should fallback to string"
    );
    assert!(
        external["description"]
            .as_str()
            .unwrap()
            .contains("Unresolved $ref"),
        "description should contain unresolved $ref hint"
    );

    // 验证内部缺失引用也被降级
    let missing = &schema["properties"]["missing"];
    assert_eq!(missing["type"], "string");
    assert!(missing["description"]
        .as_str()
        .unwrap()
        .contains("NonExistent"));
}

fn test_deeply_nested_multi_level_defs() {
    let mut schema = json!({
        "type": "object",
        "$defs": {
            "RootDef": { "type": "integer" }
        },
        "properties": {
            "level1": {
                "type": "object",
                "$defs": {
                    "Level1Def": { "type": "boolean" }
                },
                "properties": {
                    "level2": {
                        "type": "object",
                        "$defs": {
                            "Level2Def": { "type": "number" }
                        },
                        "properties": {
                            "useRoot": { "$ref": "#/$defs/RootDef" },
                            "useLevel1": { "$ref": "#/$defs/Level1Def" },
                            "useLevel2": { "$ref": "#/$defs/Level2Def" }
                        }
                    }
                }
            }
        }
    });

    clean_json_schema(&mut schema);

    let level2_props = &schema["properties"]["level1"]["properties"]["level2"]["properties"];

    // 验证所有层级的 $defs 都被正确解析
    assert_eq!(
        level2_props["useRoot"]["type"], "integer",
        "RootDef should resolve"
    );
    assert_eq!(
        level2_props["useLevel1"]["type"], "boolean",
        "Level1Def should resolve"
    );
    assert_eq!(
        level2_props["useLevel2"]["type"], "number",
        "Level2Def should resolve"
    );

    // 验证没有残留 $ref
    assert!(level2_props["useRoot"].get("$ref").is_none());
    assert!(level2_props["useLevel1"].get("$ref").is_none());
    assert!(level2_props["useLevel2"].get("$ref").is_none());
}

fn test_non_standard_field_cleaning_and_healing() {
    let mut schema = json!({
        "type": "array",
        "items": {
            "cornerRadius": { "type": "number" },
            "fillColor": { "type": "string" }
        }
    });

    clean_json_schema(&mut schema);

    // 验证 items 中的非标准字段被移动到了 properties 内部，并增加了 type: object
    let items = &schema["items"];
    assert_eq!(
        items["type"], "object",
        "Malformed items should be healed to type object"
    );
    assert!(
        items.get("properties").is_some(),
        "Malformed items should have properties object"
    );
    assert_eq!(items["properties"]["cornerRadius"]["type"], "number");
    assert_eq!(items["properties"]["fillColor"]["type"], "string");

    // 验证原始字段已从 items 顶层移除（白名单过滤）
    assert!(items.get("cornerRadius").is_none());
    assert!(items.get("fillColor").is_none());
}

fn test_implicit_type_injection() {
    let mut schema = json!({
        "properties": {
            "values": {
                "items": {
                    "cornerRadius": { "type": "number" }
                }
            }
        }
    });

    clean_json_schema(&mut schema);

    // 验证 values 被注入了 type: array
    assert_eq!(schema["properties"]["values"]["type"], "array");

    // 验证 items 被启发式修复为 type: object 并包含 properties
    let items = &schema["properties"]["values"]["items"];
    assert_eq!(items["type"], "object");
    assert!(items["properties"].get("cornerRadius").is_some());
}

fn test_gemini_strict_validation_injection() {
    let mut schema = json!({
        "type": "object",
        "properties": {
            "patterns": {
                "items": {
                    "properties": {
                        "type": {
                            "enum": ["A", "B"]
                        }
                    }
                }
            },
            "nested_props": {
                "properties": {
                    "foo": { "type": "string" }
                }
            }
        }
    });

    clean_json_schema(&mut schema);

    // 验证 enum 自动补全了 type: string
    let type_node = &schema["properties"]["patterns"]["items"]["properties"]["type"];
    assert_eq!(type_node["type"], "string");
    assert!(type_node.get("enum").is_some());

    // 验证 嵌套 properties 自动补全了 type: object
    assert_eq!(schema["properties"]["nested_props"]["type"], "object");

    // 验证 patterns 自动补全了 type: array
    assert_eq!(schema["properties"]["patterns"]["type"], "array");
}

fn test_malformed_items_as_properties() {
    let mut schema = json!({
        "type": "object",
        "properties": {
            "config": {
                "type": "object",
                "items": {
                    "color": { "type": "string" },
                    "size": { "type": "number" }
                }
            }
        }
    });

    clean_json_schema(&mut schema);

    // 验证 items 被移除并转换为 properties
    let config = &schema["properties"]["config"];
    assert!(config.get("items").is_none());
    assert_eq!(config["properties"]["color"]["type"], "string");
    assert_eq!(config["properties"]["size"]["type"], "number");
    assert_eq!(config["type"], "object");
}

fn test_circular_ref_flattening() {
    // 模拟循环引用：A -> B, B -> A
    let mut schema = json!({
        "$defs": {
            "A": {
                "type": "object",
                "properties": {
                    "toB": { "$ref": "#/$defs/B" }
                }
            },
            "B": {
                "type": "object",
                "properties": {
                    "toA": { "$ref": "#/$defs/A" }
                }
            }
        },
        "properties": {
            "start": { "$ref": "#/$defs/A" }
        }
    });

    // 如果没有深度限制，这里会发生栈溢出
    // 有了深度限制，它应该能正常返回（尽管展开是不完整的）
    clean_json_schema(&mut schema);

    // 验证基本结构保留，没有崩溃
    assert_eq!(schema["properties"]["start"]["type"], "object");
    assert!(schema["properties"]["start"]["properties"]
        .get("toB")
        .is_some());
}

fn test_any_of_best_branch_selection() {
    let mut schema = json!({
        "anyOf": [
            { "type": "string" },
            { "type": "object", "properties": { "foo": { "type": "string" } } },
            { "type": "null" }
        ]
    });

    clean_json_schema(&mut schema);

    // 验证选择了分数最高的 Object 分支
    assert_eq!(schema["type"], "object");
    assert!(schema.get("properties").is_some());
    assert_eq!(schema["properties"]["foo"]["type"], "string");
}

fn test_issue_3327_const_normalization() {
    // 场景 1: 基础 const 转换
    let mut schema1 = json!({
        "type": "object",
        "properties": {
            "action_type": {
                "const": "element"
            },
            "count": {
                "const": 5
            },
            "enabled": {
                "const": true
            }
        }
    });

    clean_json_schema(&mut schema1);

    assert_eq!(schema1["properties"]["action_type"]["type"], "string");
    assert_eq!(
        schema1["properties"]["action_type"]["enum"],
        json!(["element"])
    );
    assert!(schema1["properties"]["action_type"].get("const").is_none());

    assert_eq!(schema1["properties"]["count"]["type"], "integer");
    assert_eq!(schema1["properties"]["count"]["enum"], json!(["5"]));
    assert!(schema1["properties"]["count"].get("const").is_none());

    assert_eq!(schema1["properties"]["enabled"]["type"], "boolean");
    assert_eq!(schema1["properties"]["enabled"]["enum"], json!(["true"]));
    assert!(schema1["properties"]["enabled"].get("const").is_none());

    // 场景 2: ZCode Computer Use MCP 的 anyOf 联合嵌套包含 const
    let mut schema2 = json!({
        "type": "object",
        "properties": {
            "target": {
                "anyOf": [
                    {
                        "type": "object",
                        "properties": {
                            "type": {
                                "const": "element"
                            },
                            "state_id": {
                                "type": "string"
                            },
                            "index": {
                                "type": "integer"
                            }
                        },
                        "required": ["type", "state_id", "index"],
                        "additionalProperties": false
                    },
                    {
                        "type": "object",
                        "properties": {
                            "type": {
                                "const": "coordinate"
                            },
                            "x": {
                                "type": "integer"
                            },
                            "y": {
                                "type": "integer"
                            }
                        },
                        "required": ["type", "x", "y"],
                        "additionalProperties": false
                    }
                ]
            }
        }
    });

    clean_json_schema(&mut schema2);

    let target_props = &schema2["properties"]["target"]["properties"];
    assert!(target_props.get("type").is_some());
    assert_eq!(target_props["type"]["type"], "string");
    assert_eq!(target_props["type"]["enum"], json!(["element"]));
    assert!(target_props["type"].get("const").is_none());

    // 验证没有非合法的 Schema 结构 (如 properties 嵌套了 "element" 标量字符串)
    assert!(target_props["type"].get("properties").is_none());
}

fn test_nested_array_without_items_gets_gemini_fallback() {
    let mut schema = json!({
        "type": "object",
        "properties": {
            "query": {
                "type": "object",
                "properties": {
                    "where": {
                        "type": "array",
                        "items": { "type": "array" }
                    }
                }
            }
        }
    });

    clean_json_schema(&mut schema);

    assert_eq!(
        schema["properties"]["query"]["properties"]["where"]["items"]["items"],
        json!({ "type": "string" })
    );
}
