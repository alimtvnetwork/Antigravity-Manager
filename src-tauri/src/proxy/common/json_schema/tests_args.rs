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

    // Strings should be preserved to avoid breaking semantics
    assert_eq!(args["version"], "01.0");
    assert_eq!(args["code"], "007");
}

fn test_nested_defs_flattening() {
    // MCP tools often nest $defs inside properties rather than at the root level
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

    // Verify that nested $refs are parsed correctly
    let home = &schema["properties"]["config"]["properties"]["home"];
    assert_eq!(
        home["type"], "object",
        "home should have type 'object' from resolved $ref"
    );
    assert_eq!(
        home["properties"]["city"]["type"], "string",
        "home.properties.city should exist from resolved Address"
    );

    // Verify there are no leftover $refs
    assert!(
        home.get("$ref").is_none(),
        "home should not have orphan $ref"
    );

    // Verify that work is also parsed correctly
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

    // Verify that external references are downgraded to type string
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

    // Validate internal missing references are also downgraded
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

    // Verify that $defs at all levels are parsed correctly
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

    // Verify there is no residual $ref
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

    // Verify that the non-standard fields in items have been moved to properties and type: object has been added
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

    // Verify that the original field has been removed from the items top level (whitelist filtering)
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

    // Verify that values are injected with type: array
    assert_eq!(schema["properties"]["values"]["type"], "array");

    // Verify that items is heuristically fixed to type: object and contains properties
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

    // Verify that enum automatically completes type: string
    let type_node = &schema["properties"]["patterns"]["items"]["properties"]["type"];
    assert_eq!(type_node["type"], "string");
    assert!(type_node.get("enum").is_some());

    // Verify that nested properties automatically complete type: object
    assert_eq!(schema["properties"]["nested_props"]["type"], "object");

    // Verify that patterns automatically complete type: array
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

    // Verify items are removed and converted to properties
    let config = &schema["properties"]["config"];
    assert!(config.get("items").is_none());
    assert_eq!(config["properties"]["color"]["type"], "string");
    assert_eq!(config["properties"]["size"]["type"], "number");
    assert_eq!(config["type"], "object");
}

fn test_circular_ref_flattening() {
    // Simulate circular reference: A -> B, B -> A
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

    // If there is no depth limit, a stack overflow will occur here
    // With the depth limit, it should return normally (although the expansion is incomplete)
    clean_json_schema(&mut schema);

    // Verify that the basic structure is preserved and there are no crashes
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

    // Verify that the Object branch with the highest score is selected
    assert_eq!(schema["type"], "object");
    assert!(schema.get("properties").is_some());
    assert_eq!(schema["properties"]["foo"]["type"], "string");
}

fn test_issue_3327_const_normalization() {
    // Scenario 1: Basic const conversion
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

    // Scenario 2: AnyOf union nesting of ZCode Computer Use MCP contains const
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

    // Verify that there are no illegal Schema structures (such as properties nested with "element" scalar strings)
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
