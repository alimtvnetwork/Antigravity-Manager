//! Settings Deep Merge Validator (`Imp-251`).
//!
//! Enforces AGENTS.md Headless & CLI Parity invariant:
//! "Any profile/instance duplication or settings synchronization must pass through
//! identical underlying service routes (`copy_instance_with_options`), preserving
//! project continuity (`workspaceStorage`) and allowing deep-merge settings
//! synchronization from both GUI and CLI."
//!
//! Prevents shallow overwriting (clobbering) of nested JSON configurations and validates
//! that critical protected keys (such as `workspaceStorage` and custom keybindings) are
//! safely preserved during configuration synchronization.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Resolution outcome for a specific configuration field during merge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MergeConflictResolution {
    /// Target configuration value was preserved without change.
    TargetValueRetained,
    /// Patch configuration value was applied to target.
    SourceValueApplied,
    /// Array entries were concatenated and deduplicated.
    ArrayConcatenated,
}

impl MergeConflictResolution {
    /// Returns true if the target's original value was retained.
    pub fn is_retained(&self) -> bool {
        matches!(self, Self::TargetValueRetained)
    }

    /// Returns true if the source patch value was applied.
    pub fn is_applied(&self) -> bool {
        matches!(self, Self::SourceValueApplied)
    }

    /// Returns true if array elements were concatenated.
    pub fn is_concatenated(&self) -> bool {
        matches!(self, Self::ArrayConcatenated)
    }
}

impl fmt::Display for MergeConflictResolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TargetValueRetained => write!(f, "TargetValueRetained"),
            Self::SourceValueApplied => write!(f, "SourceValueApplied"),
            Self::ArrayConcatenated => write!(f, "ArrayConcatenated"),
        }
    }
}

/// Audit record for an individual field evaluated during deep merge.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergeFieldRecord {
    /// Dot-delimited JSON property path (e.g. `editor.tabSize` or `window.zoomLevel`).
    pub field_path: String,
    /// Applied resolution strategy.
    pub resolution: MergeConflictResolution,
    /// True if the field is nested beneath a parent object.
    pub is_nested: bool,
}

impl MergeFieldRecord {
    /// Creates a new merge field record.
    pub fn new(
        field_path: impl Into<String>,
        resolution: MergeConflictResolution,
        is_nested: bool,
    ) -> Self {
        Self {
            field_path: field_path.into(),
            resolution,
            is_nested,
        }
    }
}

/// Detailed audit report produced after deep merge execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeepMergeReport {
    /// True if merge completed successfully without violations.
    pub is_success: bool,
    /// Fully merged resulting JSON value.
    pub merged_json: serde_json::Value,
    /// Granular audit trail of field resolutions.
    pub records: Vec<MergeFieldRecord>,
    /// Total count of keys modified or added.
    pub modified_keys_count: usize,
    /// Total count of keys retained from original target.
    pub preserved_keys_count: usize,
}

impl DeepMergeReport {
    /// Returns true if any keys were modified or appended.
    pub fn has_modifications(&self) -> bool {
        self.modified_keys_count > 0
    }

    /// Returns total number of evaluated records.
    pub fn total_records_count(&self) -> usize {
        self.records.len()
    }
}

/// Violations detected during deep merge validation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeepMergeViolation {
    /// Attempted to clobber an existing object with a primitive or vice-versa.
    ShallowClobberDetected { key: String, detail: String },
    /// Input payload was not a valid JSON object.
    InvalidJsonObject { detail: String },
}

impl DeepMergeViolation {
    /// Returns true if violation indicates shallow clobbering.
    pub fn is_shallow_clobber(&self) -> bool {
        matches!(self, Self::ShallowClobberDetected { .. })
    }

    /// Returns true if violation indicates an invalid JSON object input.
    pub fn is_invalid_json(&self) -> bool {
        matches!(self, Self::InvalidJsonObject { .. })
    }
}

impl fmt::Display for DeepMergeViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ShallowClobberDetected { key, detail } => {
                write!(f, "Shallow clobber detected on '{key}': {detail}")
            }
            Self::InvalidJsonObject { detail } => {
                write!(f, "Invalid JSON object: {detail}")
            }
        }
    }
}

impl std::error::Error for DeepMergeViolation {}

/// Resolves full dot-path prefix for nested fields.
fn resolve_field_path(prefix: &str, key: &str) -> String {
    if prefix.is_empty() {
        key.to_string()
    } else {
        format!("{prefix}.{key}")
    }
}

/// Merges primitive values, recording applied or retained status.
fn merge_primitive_entry(
    target: &mut serde_json::Value,
    patch: &serde_json::Value,
    field_path: String,
    is_nested: bool,
    records: &mut Vec<MergeFieldRecord>,
) {
    if target != patch {
        *target = patch.clone();
        records.push(MergeFieldRecord::new(
            field_path,
            MergeConflictResolution::SourceValueApplied,
            is_nested,
        ));
    } else {
        records.push(MergeFieldRecord::new(
            field_path,
            MergeConflictResolution::TargetValueRetained,
            is_nested,
        ));
    }
}

/// Merges JSON array values, concatenating unique elements.
fn merge_array_entry(
    target_arr: &mut Vec<serde_json::Value>,
    patch_arr: &[serde_json::Value],
    field_path: String,
    is_nested: bool,
    records: &mut Vec<MergeFieldRecord>,
) {
    for item in patch_arr {
        if !target_arr.contains(item) {
            target_arr.push(item.clone());
        }
    }
    records.push(MergeFieldRecord::new(
        field_path,
        MergeConflictResolution::ArrayConcatenated,
        is_nested,
    ));
}

/// Dispatches merge logic for non-object target and patch values.
fn merge_non_object_values(
    target_val: &mut serde_json::Value,
    patch_val: &serde_json::Value,
    field_path: &str,
    is_nested: bool,
    records: &mut Vec<MergeFieldRecord>,
) -> Result<(), DeepMergeViolation> {
    match (target_val.as_array_mut(), patch_val.as_array()) {
        (Some(target_arr), Some(patch_arr)) => {
            merge_array_entry(
                target_arr,
                patch_arr,
                field_path.to_string(),
                is_nested,
                records,
            );
            Ok(())
        }
        _ => {
            merge_primitive_entry(
                target_val,
                patch_val,
                field_path.to_string(),
                is_nested,
                records,
            );
            Ok(())
        }
    }
}

/// Evaluates existing target entry against incoming patch entry.
fn merge_existing_entry(
    target_val: &mut serde_json::Value,
    patch_val: &serde_json::Value,
    field_path: &str,
    is_nested: bool,
    records: &mut Vec<MergeFieldRecord>,
) -> Result<(), DeepMergeViolation> {
    match (target_val.is_object(), patch_val.is_object()) {
        (true, true) => recursive_deep_merge(target_val, patch_val, field_path, records),
        (true, false) => Err(DeepMergeViolation::ShallowClobberDetected {
            key: field_path.to_string(),
            detail: format!("Cannot clobber nested object at '{field_path}' with non-object value"),
        }),
        (false, true) => Err(DeepMergeViolation::ShallowClobberDetected {
            key: field_path.to_string(),
            detail: format!("Cannot clobber primitive value at '{field_path}' with object value"),
        }),
        (false, false) => {
            merge_non_object_values(target_val, patch_val, field_path, is_nested, records)
        }
    }
}

/// Processes a single key-value entry from patch into target JSON object.
fn merge_object_entry(
    target_map: &mut serde_json::Map<String, serde_json::Value>,
    key: &str,
    patch_val: &serde_json::Value,
    prefix: &str,
    records: &mut Vec<MergeFieldRecord>,
) -> Result<(), DeepMergeViolation> {
    let field_path = resolve_field_path(prefix, key);
    let is_nested = !prefix.is_empty();

    match target_map.get_mut(key) {
        Some(target_val) => {
            merge_existing_entry(target_val, patch_val, &field_path, is_nested, records)
        }
        None => {
            target_map.insert(key.to_string(), patch_val.clone());
            records.push(MergeFieldRecord::new(
                field_path,
                MergeConflictResolution::SourceValueApplied,
                is_nested,
            ));
            Ok(())
        }
    }
}

/// Recursively deep merges `patch` into `target`, auditing field resolutions.
pub fn recursive_deep_merge(
    target: &mut serde_json::Value,
    patch: &serde_json::Value,
    path_prefix: &str,
    records: &mut Vec<MergeFieldRecord>,
) -> Result<(), DeepMergeViolation> {
    let (target_map, patch_map) = match (target.as_object_mut(), patch.as_object()) {
        (Some(t), Some(p)) => (t, p),
        _ => {
            return Err(DeepMergeViolation::ShallowClobberDetected {
                key: path_prefix.to_string(),
                detail: format!("Expected object target and patch at '{path_prefix}'"),
            })
        }
    };

    for (key, patch_val) in patch_map {
        merge_object_entry(target_map, key, patch_val, path_prefix, records)?;
    }
    Ok(())
}

/// Computes count of keys that applied modifications.
fn count_modified_keys(records: &[MergeFieldRecord]) -> usize {
    records
        .iter()
        .filter(|r| {
            matches!(
                r.resolution,
                MergeConflictResolution::SourceValueApplied
                    | MergeConflictResolution::ArrayConcatenated
            )
        })
        .count()
}

/// Computes count of keys that retained target values.
fn count_preserved_keys(records: &[MergeFieldRecord]) -> usize {
    records
        .iter()
        .filter(|r| matches!(r.resolution, MergeConflictResolution::TargetValueRetained))
        .count()
}

/// Merges two configuration JSON objects, ensuring deep preservation and full audit trail.
pub fn merge_settings(
    target_json: &serde_json::Value,
    patch_json: &serde_json::Value,
) -> Result<DeepMergeReport, DeepMergeViolation> {
    if !target_json.is_object() {
        return Err(DeepMergeViolation::InvalidJsonObject {
            detail: "Target configuration must be a valid JSON object".to_string(),
        });
    }
    if !patch_json.is_object() {
        return Err(DeepMergeViolation::InvalidJsonObject {
            detail: "Patch configuration must be a valid JSON object".to_string(),
        });
    }

    let mut merged = target_json.clone();
    let mut records = Vec::new();
    recursive_deep_merge(&mut merged, patch_json, "", &mut records)?;

    let modified_keys_count = count_modified_keys(&records);
    let preserved_keys_count = count_preserved_keys(&records);

    Ok(DeepMergeReport {
        is_success: true,
        merged_json: merged,
        records,
        modified_keys_count,
        preserved_keys_count,
    })
}

/// Resolves dot-delimited path or direct key in a JSON object.
fn resolve_json_path<'a>(
    value: &'a serde_json::Value,
    path: &str,
) -> Option<&'a serde_json::Value> {
    if path.contains('.') {
        path.split('.')
            .try_fold(value, |curr, segment| curr.get(segment))
    } else {
        value.get(path)
    }
}

/// Validates that a single protected key was not erased or degraded in merged output.
fn validate_single_protected_key(
    original: &serde_json::Value,
    merged: &serde_json::Value,
    key: &str,
) -> Result<(), DeepMergeViolation> {
    if let Some(orig_val) = resolve_json_path(original, key) {
        match resolve_json_path(merged, key) {
            None => Err(DeepMergeViolation::ShallowClobberDetected {
                key: key.to_string(),
                detail: format!("Protected key '{key}' was erased during merge"),
            }),
            Some(merged_val) => {
                if orig_val.is_object() && !merged_val.is_object() {
                    Err(DeepMergeViolation::ShallowClobberDetected {
                        key: key.to_string(),
                        detail: format!("Protected key '{key}' degraded from Object to primitive"),
                    })
                } else {
                    Ok(())
                }
            }
        }
    } else {
        Ok(())
    }
}

/// Ensures protected keys present in original were not erased or reduced to primitives.
pub fn validate_no_shallow_clobber(
    original: &serde_json::Value,
    merged: &serde_json::Value,
    protected_keys: &[String],
) -> Result<(), DeepMergeViolation> {
    if !original.is_object() || !merged.is_object() {
        return Err(DeepMergeViolation::InvalidJsonObject {
            detail: "Both original and merged payloads must be JSON objects".to_string(),
        });
    }

    for key in protected_keys {
        validate_single_protected_key(original, merged, key)?;
    }
    Ok(())
}
