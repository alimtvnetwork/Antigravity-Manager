//! Unit Tests for Settings Deep Merge Validator (`Imp-251`).

use super::settings_deep_merge_validator::*;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_nested_object_deep_merge_preserving_orthogonal_keys() {
        let original = json!({
            "editor": {
                "tabSize": 4,
                "minimap": {
                    "enabled": true
                }
            },
            "workbench": {
                "colorTheme": "Default Dark+"
            }
        });

        let patch = json!({
            "editor": {
                "fontSize": 14,
                "minimap": {
                    "maxColumn": 80
                }
            }
        });

        let report = merge_settings(&original, &patch).expect("merge should succeed");
        assert!(report.is_success);
        assert!(report.has_modifications());

        let merged = report.merged_json;
        assert_eq!(merged["editor"]["tabSize"], 4);
        assert_eq!(merged["editor"]["fontSize"], 14);
        assert_eq!(merged["editor"]["minimap"]["enabled"], true);
        assert_eq!(merged["editor"]["minimap"]["maxColumn"], 80);
        assert_eq!(merged["workbench"]["colorTheme"], "Default Dark+");

        assert!(report.modified_keys_count >= 2);
    }

    #[test]
    fn test_shallow_overwrite_attempt_rejected_with_shallow_clobber() {
        let original = json!({
            "editor": {
                "tabSize": 4
            }
        });
        let patch = json!({
            "editor": "clobbered_string"
        });

        let err = merge_settings(&original, &patch).expect_err("should reject shallow clobber");
        assert!(err.is_shallow_clobber());
        match err {
            DeepMergeViolation::ShallowClobberDetected { key, detail } => {
                assert_eq!(key, "editor");
                assert!(detail.contains("Cannot clobber nested object"));
            }
            _ => panic!("Expected ShallowClobberDetected"),
        }
    }

    #[test]
    fn test_primitive_clobbered_with_object_rejected() {
        let original = json!({
            "zoomLevel": 0
        });
        let patch = json!({
            "zoomLevel": {
                "nested": 1
            }
        });

        let err = merge_settings(&original, &patch).expect_err("should reject object clobber");
        assert!(err.is_shallow_clobber());
        match err {
            DeepMergeViolation::ShallowClobberDetected { key, detail } => {
                assert_eq!(key, "zoomLevel");
                assert!(detail.contains("Cannot clobber primitive value"));
            }
            _ => panic!("Expected ShallowClobberDetected"),
        }
    }

    #[test]
    fn test_array_element_merging_deduplicates() {
        let original = json!({
            "extensions.ignored": ["ext.a", "ext.b"]
        });
        let patch = json!({
            "extensions.ignored": ["ext.b", "ext.c"]
        });

        let report = merge_settings(&original, &patch).expect("array merge should succeed");
        let merged_arr = report.merged_json["extensions.ignored"]
            .as_array()
            .expect("should be array");

        assert_eq!(merged_arr.len(), 3);
        assert!(merged_arr.contains(&json!("ext.a")));
        assert!(merged_arr.contains(&json!("ext.b")));
        assert!(merged_arr.contains(&json!("ext.c")));

        let arr_record = report
            .records
            .iter()
            .find(|r| r.field_path == "extensions.ignored")
            .expect("record must exist");
        assert!(arr_record.resolution.is_concatenated());
    }

    #[test]
    fn test_protected_keys_validation_detects_erasure_and_degradation() {
        let original = json!({
            "workspaceStorage": {
                "recent": ["workspace/alpha"]
            },
            "unprotected": "val"
        });

        let merged_missing = json!({
            "unprotected": "val"
        });
        let protected = vec!["workspaceStorage".to_string()];

        let err = validate_no_shallow_clobber(&original, &merged_missing, &protected)
            .expect_err("should detect erased protected key");
        assert!(err.is_shallow_clobber());
        assert!(err.to_string().contains("erased"));

        let merged_degraded = json!({
            "workspaceStorage": "invalid_primitive_string"
        });
        let err2 = validate_no_shallow_clobber(&original, &merged_degraded, &protected)
            .expect_err("should detect degraded protected key");
        assert!(err2.is_shallow_clobber());
        assert!(err2.to_string().contains("degraded"));

        let merged_valid = json!({
            "workspaceStorage": {
                "recent": ["workspace/alpha", "workspace/beta"]
            }
        });
        assert!(validate_no_shallow_clobber(&original, &merged_valid, &protected).is_ok());
    }

    #[test]
    fn test_protected_keys_validation_nested_dot_path() {
        let original = json!({
            "window": {
                "titleBarStyle": "custom"
            }
        });
        let merged_valid = json!({
            "window": {
                "titleBarStyle": "custom",
                "zoom": 1
            }
        });
        let protected = vec!["window.titleBarStyle".to_string()];
        assert!(validate_no_shallow_clobber(&original, &merged_valid, &protected).is_ok());

        let merged_missing = json!({
            "window": {
                "zoom": 1
            }
        });
        let err = validate_no_shallow_clobber(&original, &merged_missing, &protected)
            .expect_err("should detect missing nested protected key");
        assert!(err.is_shallow_clobber());
    }

    #[test]
    fn test_invalid_json_object_returns_error() {
        let non_obj = json!(["item1", "item2"]);
        let valid_obj = json!({ "key": "val" });

        let err1 = merge_settings(&non_obj, &valid_obj).expect_err("target must be object");
        assert!(err1.is_invalid_json());

        let err2 = merge_settings(&valid_obj, &non_obj).expect_err("patch must be object");
        assert!(err2.is_invalid_json());

        let protected = vec!["key".to_string()];
        let err3 = validate_no_shallow_clobber(&non_obj, &valid_obj, &protected)
            .expect_err("validate requires objects");
        assert!(err3.is_invalid_json());
    }

    #[test]
    fn test_identical_values_marked_as_target_value_retained() {
        let original = json!({
            "telemetry.enabled": false,
            "theme": "dark"
        });
        let patch = json!({
            "theme": "dark",
            "telemetry.enabled": true
        });

        let report = merge_settings(&original, &patch).expect("merge should succeed");
        assert_eq!(report.preserved_keys_count, 1);
        assert_eq!(report.modified_keys_count, 1);

        let theme_rec = report
            .records
            .iter()
            .find(|r| r.field_path == "theme")
            .unwrap();
        assert!(theme_rec.resolution.is_retained());

        let telem_rec = report
            .records
            .iter()
            .find(|r| r.field_path == "telemetry.enabled")
            .unwrap();
        assert!(telem_rec.resolution.is_applied());
    }

    #[test]
    fn test_display_and_error_formatting() {
        assert_eq!(
            MergeConflictResolution::TargetValueRetained.to_string(),
            "TargetValueRetained"
        );
        assert_eq!(
            MergeConflictResolution::SourceValueApplied.to_string(),
            "SourceValueApplied"
        );
        assert_eq!(
            MergeConflictResolution::ArrayConcatenated.to_string(),
            "ArrayConcatenated"
        );

        let clobber = DeepMergeViolation::ShallowClobberDetected {
            key: "test.key".to_string(),
            detail: "clobber detail".to_string(),
        };
        assert!(clobber.to_string().contains("test.key"));
        assert!(clobber.to_string().contains("clobber detail"));

        let invalid = DeepMergeViolation::InvalidJsonObject {
            detail: "not an obj".to_string(),
        };
        assert!(invalid.to_string().contains("not an obj"));

        // Verify std::error::Error trait
        let std_err: &dyn std::error::Error = &clobber;
        assert!(!std_err.to_string().is_empty());
    }

    #[test]
    fn test_deeply_nested_three_levels_merge() {
        let original = json!({
            "a": {
                "b": {
                    "c": {
                        "d1": 10
                    }
                }
            }
        });
        let patch = json!({
            "a": {
                "b": {
                    "c": {
                        "d2": 20
                    }
                }
            }
        });

        let report = merge_settings(&original, &patch).expect("three level merge should succeed");
        assert_eq!(report.merged_json["a"]["b"]["c"]["d1"], 10);
        assert_eq!(report.merged_json["a"]["b"]["c"]["d2"], 20);
        assert_eq!(report.total_records_count(), 1);
        assert!(report.records[0].is_nested);
        assert_eq!(report.records[0].field_path, "a.b.c.d2");
    }
}
