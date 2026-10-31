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
            // Simulated property name conflict: pattern is an Object property and should not be removed
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

    // 1. Keep validation types in lowercase
    assert_eq!(schema["type"], "object");
    assert_eq!(schema["properties"]["location"]["type"], "string");

    // 2. Verify that non-standard constraint fields are safely removed and the description remains transparent and cannot be tampered with.
    assert!(schema["properties"]["location"].get("minLength").is_none());
    assert!(schema["properties"]["location"].get("format").is_none());
    assert_eq!(
        schema["properties"]["location"]["description"],
        "The city and state, e.g. San Francisco, CA"
    );

    // 3. Verify that the attribute named "pattern" has not been deleted by mistake
    assert!(schema["properties"].get("pattern").is_some());
    assert_eq!(schema["properties"]["pattern"]["type"], "object");

    // 4. Verify that the internal pattern check field is removed and the description remains the same
    assert!(schema["properties"]["pattern"]["properties"]["regex"]
        .get("pattern")
        .is_none());
    assert_eq!(
        schema["properties"]["pattern"]["properties"]["regex"]["description"],
        "Regex pattern"
    );

    // 5. Verify that union types are downgraded to single types (Protobuf compatibility)
    assert_eq!(schema["properties"]["unit"]["type"], "string");

    // 6. Verify metadata fields are removed
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

    // Verify that the reference is expanded and the type is converted to lowercase
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

    // Verify missing_prop is removed from required
    let required = schema["required"].as_array().unwrap();
    assert_eq!(required.len(), 1);
    assert_eq!(required[0].as_str().unwrap(), "existing_prop");
}

fn test_anyof_type_extraction() {
    // Test FastMCP style Optional[str] schema
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

    // Verify anyOf is removed
    assert!(schema["properties"]["testo"].get("anyOf").is_none());
    assert!(schema["properties"]["importo"].get("anyOf").is_none());

    // Verify type is extracted correctly
    assert_eq!(schema["properties"]["testo"]["type"], "string");
    assert_eq!(schema["properties"]["importo"]["type"], "number");
    assert_eq!(schema["properties"]["attivo"]["type"], "boolean");

    // Verify default is removed (out of whitelist)
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

    // type already exists and should not be overridden by the type in anyOf
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

    // 1. Verification type is extracted
    assert_eq!(config["type"], "object");

    // 2. Verify that the properties inside anyOf are merged
    assert!(config.get("properties").is_some());
    assert_eq!(config["properties"]["path"]["type"], "string");
    assert_eq!(config["properties"]["recursive"]["type"], "boolean");

    // 3. Verify that required is merged
    let req = config["required"].as_array().unwrap();
    assert!(req.iter().any(|v| v == "path"));

    // 4. Verify that the anyOf field itself is removed
    assert!(config.get("anyOf").is_none());

    // 5. Verify that reason is not injected because of "null" (because we retain the attribute)
    assert!(config["properties"].get("reason").is_none());
}

fn test_clean_json_schema_on_non_schema_object() {
    // Simulate runtime tool call objects containing general business fields such as description, command, type, etc.
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

    // Call cleaning logic
    clean_json_schema(&mut tool_call);

    // Verification: Non-Schema fields and runtime parameters should never be stripped, and command and description must be kept intact at the same time.
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

    // Verify that type is downgraded and description is appended (nullable)
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

    // Verify anyOf is removed (merged)
    let config = &schema["properties"]["config"];
    assert!(config.get("anyOf").is_none());

    // Verify propertyNames are removed
    assert!(config.get("propertyNames").is_none());

    // Verify that the merged properties exist and have no propertyNames
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

    // Verify const is removed
    let status = &schema["items"]["properties"]["status"];
    assert!(status.get("const").is_none());

    // Verify type still exists
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

    // Verify that deeply nested illegal fields are removed
    let data = &schema["properties"]["data"];

    // anyOf should be merged and removed
    assert!(data.get("anyOf").is_none());

    // Verify that no propertyNames and const escape to the top level
    assert!(data.get("propertyNames").is_none());
    assert!(data.get("const").is_none());

    // Verify that the structure is preserved correctly
    assert_eq!(data["type"], "array");
    if let Some(items) = data.get("items") {
        // anyOf inside items should also be merged
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
