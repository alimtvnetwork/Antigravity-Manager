use super::catalog::{normalize_opencode_base_url, ModelDef, VariantType};
use super::config_paths::{get_config_paths, parse_jsonc};
use super::lock::{
    acquire_opencode_config_lock, ANTIGRAVITY_PROVIDER_ID, BACKUP_SUFFIX, OLD_BACKUP_SUFFIX,
    OPENCODE_CONFIG_FILE,
};
use crate::proxy::common::variant_mapping::VariantTier;
use serde_json::Value;
use std::fs;
use std::path::PathBuf;

fn get_provider_options<'a>(value: &'a Value, provider_name: &str) -> Option<&'a Value> {
    value
        .get("provider")
        .and_then(|p| p.get(provider_name))
        .and_then(|prov| prov.get("options"))
}

pub fn get_sync_status(proxy_url: &str) -> (bool, bool, Option<String>) {
    let _lock = acquire_opencode_config_lock();

    let Some((config_path, _, _)) = get_config_paths() else {
        return (false, false, None);
    };

    let mut is_synced = true;
    let mut has_backup = false;
    let mut current_base_url = None;

    // Backups may have been created against either opencode.json or opencode.jsonc.
    // Check both the active file name's backup and the canonical json backup so a
    // previously-synced state is still detected after switching file types.
    let active_file_name = config_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(OPENCODE_CONFIG_FILE);
    let backup_candidates = [
        format!("{}{}", active_file_name, BACKUP_SUFFIX),
        format!("{}{}", active_file_name, OLD_BACKUP_SUFFIX),
        format!("{}{}", OPENCODE_CONFIG_FILE, BACKUP_SUFFIX),
        format!("{}{}", OPENCODE_CONFIG_FILE, OLD_BACKUP_SUFFIX),
    ];
    for name in &backup_candidates {
        if config_path.with_file_name(name).exists() {
            has_backup = true;
            break;
        }
    }

    if !config_path.exists() {
        return (false, has_backup, None);
    }

    let content = match fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return (false, has_backup, None),
    };

    let json: Value = parse_jsonc(&content).unwrap_or_default();

    // Normalize proxy URL for comparison
    let normalized_proxy = normalize_opencode_base_url(proxy_url);

    // Only check antigravity-manager provider
    let ag_opts = get_provider_options(&json, ANTIGRAVITY_PROVIDER_ID);
    let ag_url = ag_opts
        .and_then(|o| o.get("baseURL"))
        .and_then(|v| v.as_str());
    let ag_key = ag_opts
        .and_then(|o| o.get("apiKey"))
        .and_then(|v| v.as_str());

    if let (Some(url), Some(_key)) = (ag_url, ag_key) {
        current_base_url = Some(url.to_string());
        // Normalize config URL before comparison
        let normalized_config_url = normalize_opencode_base_url(url);
        if normalized_config_url != normalized_proxy {
            is_synced = false;
        }
    } else {
        is_synced = false;
    }

    (is_synced, has_backup, current_base_url)
}

pub(crate) fn create_backup(path: &PathBuf) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }

    let backup_path = path.with_file_name(format!(
        "{}{}",
        path.file_name().unwrap_or_default().to_string_lossy(),
        BACKUP_SUFFIX
    ));

    if backup_path.exists() {
        return Ok(());
    }

    fs::copy(path, &backup_path).map_err(|e| format!("Failed to create backup: {}", e))?;

    Ok(())
}

pub(crate) fn restore_backup_to_target(
    backup_path: &PathBuf,
    target_path: &PathBuf,
    label: &str,
) -> Result<(), String> {
    if target_path.exists() {
        fs::remove_file(target_path)
            .map_err(|e| format!("Failed to remove existing {}: {}", label, e))?;
    }

    fs::rename(backup_path, target_path).map_err(|e| format!("Failed to restore {}: {}", label, e))
}

pub(crate) fn ensure_object(value: &mut Value, key: &str) {
    let needs_reset = match value.get(key) {
        None => true,
        Some(v) if !v.is_object() => true,
        _ => false,
    };
    if needs_reset {
        value[key] = serde_json::json!({});
    }
}

pub(crate) fn ensure_provider_object(provider: &mut serde_json::Map<String, Value>, name: &str) {
    let needs_reset = match provider.get(name) {
        None => true,
        Some(v) if !v.is_object() => true,
        _ => false,
    };
    if needs_reset {
        provider.insert(name.to_string(), serde_json::json!({}));
    }
}

pub(crate) fn merge_provider_options(provider: &mut Value, base_url: &str, api_key: &str) {
    if provider.get("options").is_none() {
        provider["options"] = serde_json::json!({});
    }

    if let Some(options) = provider.get_mut("options").and_then(|o| o.as_object_mut()) {
        options.insert("baseURL".to_string(), Value::String(base_url.to_string()));
        options.insert("apiKey".to_string(), Value::String(api_key.to_string()));
    }
}

pub(crate) fn ensure_provider_string_field(provider: &mut Value, key: &str, value: &str) {
    if let Some(obj) = provider.as_object_mut() {
        obj.insert(key.to_string(), Value::String(value.to_string()));
    }
}

/// Build Claude-style thinking variant with thinkingConfig and thinking
fn build_claude_thinking_variant(budget: u32) -> Value {
    serde_json::json!({
        "thinkingConfig": {
            "thinkingBudget": budget
        },
        "thinking": {
            "type": "enabled",
            "budget_tokens": budget,
            "budgetTokens": budget
        }
    })
}

/// Build Gemini 3 effort-based variant for the @ai-sdk/anthropic SDK.
///
/// The provider serializes `{ "effort": "<tier>" }` as `output_config.effort`
/// on the outgoing Anthropic request. No thinking-budget fields are used.
fn build_gemini3_effort_variant(tier: VariantTier) -> Value {
    let effort = match tier {
        VariantTier::Low => "low",
        VariantTier::Medium => "medium",
        VariantTier::High => "high",
    };
    serde_json::json!({ "effort": effort })
}

/// Build Gemini 2.5 thinking variant with thinkingConfig and thinking
fn build_gemini25_thinking_variant(budget: u32) -> Value {
    serde_json::json!({
        "thinkingConfig": {
            "thinkingBudget": budget
        },
        "thinking": {
            "type": "enabled",
            "budget_tokens": budget,
            "budgetTokens": budget
        }
    })
}

/// Build variants object based on variant type
pub(crate) fn build_variants_object(variant_type: Option<VariantType>) -> Option<Value> {
    match variant_type {
        Some(VariantType::ClaudeThinking) => {
            let mut variants = serde_json::Map::new();
            variants.insert("low".to_string(), build_claude_thinking_variant(8192));
            variants.insert("medium".to_string(), build_claude_thinking_variant(16384));
            variants.insert("high".to_string(), build_claude_thinking_variant(24576));
            variants.insert("max".to_string(), build_claude_thinking_variant(32768));
            Some(Value::Object(variants))
        }
        Some(VariantType::Gemini3Pro) => {
            let mut variants = serde_json::Map::new();
            variants.insert(
                "low".to_string(),
                build_gemini3_effort_variant(VariantTier::Low),
            );
            variants.insert(
                "medium".to_string(),
                serde_json::json!({ "disabled": true }),
            );
            variants.insert(
                "high".to_string(),
                build_gemini3_effort_variant(VariantTier::High),
            );
            variants.insert("max".to_string(), serde_json::json!({ "disabled": true }));
            Some(Value::Object(variants))
        }
        Some(VariantType::Gemini3Flash) => {
            let mut variants = serde_json::Map::new();
            variants.insert(
                "low".to_string(),
                build_gemini3_effort_variant(VariantTier::Low),
            );
            variants.insert(
                "medium".to_string(),
                build_gemini3_effort_variant(VariantTier::Medium),
            );
            variants.insert(
                "high".to_string(),
                build_gemini3_effort_variant(VariantTier::High),
            );
            variants.insert("max".to_string(), serde_json::json!({ "disabled": true }));
            Some(Value::Object(variants))
        }
        Some(VariantType::Gemini25Thinking) => {
            let mut variants = serde_json::Map::new();
            variants.insert("low".to_string(), build_gemini25_thinking_variant(8192));
            variants.insert("medium".to_string(), build_gemini25_thinking_variant(12288));
            variants.insert("high".to_string(), build_gemini25_thinking_variant(16384));
            variants.insert("max".to_string(), build_gemini25_thinking_variant(24576));
            Some(Value::Object(variants))
        }
        None => None,
    }
}

/// Build model JSON object with full metadata
pub(crate) fn build_model_json(model_def: &ModelDef) -> Value {
    let mut model_obj = serde_json::Map::new();

    model_obj.insert(
        "name".to_string(),
        Value::String(model_def.name.to_string()),
    );

    let limits = serde_json::json!({
        "context": model_def.context_limit,
        "output": model_def.output_limit,
    });
    model_obj.insert("limit".to_string(), limits);

    let modalities = serde_json::json!({
        "input": model_def.input_modalities,
        "output": model_def.output_modalities,
    });
    model_obj.insert("modalities".to_string(), modalities);

    if model_def.reasoning {
        model_obj.insert("reasoning".to_string(), Value::Bool(true));
    }

    // Build variants as object map instead of array
    if let Some(variants) = build_variants_object(model_def.variant_type) {
        model_obj.insert("variants".to_string(), variants);
    }

    Value::Object(model_obj)
}
