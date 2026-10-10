use crate::models::AppConfig;
use tracing::{info, warn};

use super::*;

/// Export AppConfig wrapped in standard JSON envelope with variable section
pub fn export_app_config_envelope(config: &AppConfig) -> Result<String, String> {
    let envelope =
        crate::modules::json_envelope::JsonEnvelope::new("agm/config-backup", config.clone());
    serde_json::to_string_pretty(&envelope)
        .map_err(|e| format!("failed_to_serialize_config_envelope: {}", e))
}

/// Migrate configuration JSON values across versions.
/// Returns true if any value was modified and needs persistence.
pub fn migrate_config_value(v: &mut serde_json::Value) -> bool {
    let mut modified = false;

    // [MIGRATION] Default auto_sync to true for all existing users upgrading from legacy versions.
    // Legacy configs created before v4.35.0 had auto_sync: false by default.
    // When auto_sync_migrated is missing, we migrate auto_sync to true and stamp auto_sync_migrated: true.
    if v.get("auto_sync_migrated").is_none() {
        v["auto_sync"] = serde_json::Value::Bool(true);
        v["auto_sync_migrated"] = serde_json::Value::Bool(true);
        modified = true;
    }

    if let Some(switcher) = v
        .get_mut("auto_profile_switcher")
        .and_then(|s| s.as_object_mut())
    {
        let cur_model = switcher
            .get("target_model")
            .and_then(|m| m.as_str())
            .unwrap_or("");
        if cur_model.is_empty()
            || cur_model.eq_ignore_ascii_case("gemini-pro")
            || cur_model.eq_ignore_ascii_case("gemini pro")
        {
            switcher.insert(
                "target_model".to_string(),
                serde_json::Value::String("gemini-3.8-flash-high".to_string()),
            );
            modified = true;
        }
        if switcher
            .get("check_interval_seconds")
            .and_then(|n| n.as_u64())
            == Some(480)
        {
            switcher.insert("check_interval_seconds".to_string(), serde_json::json!(300));
            modified = true;
        }
        if switcher
            .get("low_quota_threshold_percent")
            .and_then(|n| n.as_f64())
            .map(|f| f == 25.0 || f == 12.0)
            .unwrap_or(false)
        {
            switcher.insert(
                "low_quota_threshold_percent".to_string(),
                serde_json::json!(15.0),
            );
            modified = true;
        }
        if switcher
            .get("caution_interval_seconds")
            .and_then(|n| n.as_u64())
            == Some(180)
        {
            switcher.insert(
                "caution_interval_seconds".to_string(),
                serde_json::json!(60),
            );
            modified = true;
        }
        if switcher
            .get("critical_interval_seconds")
            .and_then(|n| n.as_u64())
            == Some(60)
        {
            switcher.insert(
                "critical_interval_seconds".to_string(),
                serde_json::json!(40),
            );
            modified = true;
        }
        if switcher
            .get("critical_threshold_percent")
            .and_then(|n| n.as_f64())
            .map(|f| f == 10.0 || f == 12.0)
            .unwrap_or(false)
        {
            switcher.insert(
                "critical_threshold_percent".to_string(),
                serde_json::json!(15.0),
            );
            modified = true;
        }
        if switcher.get("auto_focus_window").and_then(|b| b.as_bool()) == Some(true) {
            switcher.insert(
                "auto_focus_window".to_string(),
                serde_json::Value::Bool(false),
            );
            modified = true;
        }
    }

    if let Some(quota) = v
        .get_mut("quota_protection")
        .and_then(|q| q.as_object_mut())
    {
        let thresh = quota.get("threshold_percentage").and_then(|n| n.as_u64());
        if thresh == Some(10) || thresh == Some(12) {
            quota.insert("threshold_percentage".to_string(), serde_json::json!(15));
            modified = true;
        }
    }

    // Migration logic
    if let Some(proxy) = v.get_mut("proxy") {
        // [FIX #1738] Enhanced type checking for custom_mapping
        // Ensures the field is always parsed as an object, preventing type mismatch errors
        let mut custom_mapping = match proxy.get("custom_mapping") {
            Some(m) if m.is_object() => m.as_object().unwrap().clone(),
            Some(m) => {
                // If custom_mapping is not an object type (e.g., string), log warning and reset to empty
                tracing::warn!(
                    "Invalid custom_mapping type (expected object, got {:?}), resetting to empty",
                    m
                );
                serde_json::Map::new()
            }
            None => serde_json::Map::new(),
        };

        // Migrate Anthropic mapping
        if let Some(anthropic) = proxy
            .get_mut("anthropic_mapping")
            .and_then(|m| m.as_object_mut())
        {
            for (k, v) in anthropic.iter() {
                // Only move non-series fields, as series fields are now handled by Preset logic or builtin tables
                if !k.ends_with("-series") {
                    if !custom_mapping.contains_key(k) {
                        custom_mapping.insert(k.clone(), v.clone());
                    }
                }
            }
            // Remove old field
            proxy.as_object_mut().unwrap().remove("anthropic_mapping");
            modified = true;
        }

        // Migrate OpenAI mapping
        if let Some(openai) = proxy
            .get_mut("openai_mapping")
            .and_then(|m| m.as_object_mut())
        {
            for (k, v) in openai.iter() {
                if !k.ends_with("-series") {
                    if !custom_mapping.contains_key(k) {
                        custom_mapping.insert(k.clone(), v.clone());
                    }
                }
            }
            // Remove old field
            proxy.as_object_mut().unwrap().remove("openai_mapping");
            modified = true;
        }

        // 预设无后缀 3.6+ Flash 模型到 Tiered 自适应模型的默认映射规则
        // 自动注入到用户的自定义模型列表中，用户可在 UI 界面查阅、删除或自定义修改保存；默认按此预设执行
        for (k, v) in [
            ("gemini-3.6-flash", "gemini-3.6-flash-tiered"),
            ("gemini-3.7-flash", "gemini-3.7-flash-tiered"),
            ("gemini-3.8-flash", "gemini-3.8-flash-tiered"),
            ("gemini-3.x-flash", "3.x-flash-tiered"),
        ] {
            if !custom_mapping.contains_key(k) {
                custom_mapping.insert(k.to_string(), serde_json::Value::String(v.to_string()));
                modified = true;
            }
        }

        // Migrate log retention max_disk_mb: cap at 450 MiB boundary
        if let Some(log_retention) = proxy
            .get_mut("log_retention")
            .and_then(|m| m.as_object_mut())
        {
            if let Some(max_disk_mb) = log_retention.get("max_disk_mb").and_then(|v| v.as_u64()) {
                if max_disk_mb == 0 || max_disk_mb > 450 {
                    log_retention.insert("max_disk_mb".to_string(), serde_json::Value::from(450));
                    modified = true;
                }
            }
        }

        if modified {
            proxy.as_object_mut().unwrap().insert(
                "custom_mapping".to_string(),
                serde_json::Value::Object(custom_mapping),
            );
        }
    }

    modified
}
