use serde_json::Value;
use std::collections::HashMap;

use super::*;

/// Unpacks a JSON value: if wrapped in `{ "attributes": ..., "data": ... }`, returns
/// `(Some(attributes), expanded_data)`. If wrapped with top-level `variables`, expands them.
pub fn unpack_envelope(value: Value) -> (Option<JsonAttributes>, Value) {
    if let Value::Object(ref map) = value {
        if let (Some(attrs_val), Some(data_val)) = (map.get("attributes"), map.get("data")) {
            if let Ok(mut attrs) = serde_json::from_value::<JsonAttributes>(attrs_val.clone()) {
                let top_vars = map
                    .get("variables")
                    .and_then(|v| serde_json::from_value::<HashMap<String, Value>>(v.clone()).ok());
                let merged = merge_variables(top_vars.as_ref(), None);

                if let Some(ref mut wcfg) = attrs.work_directory {
                    if wcfg.path.contains('$') {
                        wcfg.path = resolve_string_variable(&wcfg.path, &merged);
                    }
                }
                if let Some(ref mut def_w) = attrs.default_work_directory {
                    if def_w.contains('$') {
                        *def_w = resolve_string_variable(def_w, &merged);
                    }
                }

                let mut expanded_data = data_val.clone();
                expand_variables_in_value(&mut expanded_data, &merged);
                return (Some(attrs), expanded_data);
            }
        } else if let Some(top_vars_val) = map.get("variables") {
            if let Ok(top_vars) =
                serde_json::from_value::<HashMap<String, Value>>(top_vars_val.clone())
            {
                let merged = merge_variables(Some(&top_vars), None);
                let mut expanded_val = value.clone();
                expand_variables_in_value(&mut expanded_val, &merged);
                return (None, expanded_val);
            }
        }
    }
    (None, value)
}

/// High-level typed extraction helper: unmarshals an envelope or flat JSON payload
/// with automatic variable interpolation into target type `T`.
pub fn extract_payload<T: serde::de::DeserializeOwned>(
    raw_json: &str,
) -> Result<(T, JsonAttributes), String> {
    let parsed: Value =
        serde_json::from_str(raw_json).map_err(|e| format!("Failed to parse JSON: {}", e))?;
    let (attrs_opt, data_val) = unpack_envelope(parsed);
    if let Some(attrs) = attrs_opt {
        let payload = serde_json::from_value::<T>(data_val)
            .map_err(|e| format!("Failed to deserialize payload into target type: {}", e))?;
        return Ok((payload, attrs));
    }
    let payload = serde_json::from_value::<T>(data_val)
        .map_err(|e| format!("Failed to deserialize flat payload: {}", e))?;
    let mut default_attrs = JsonAttributes::new("legacy");
    default_attrs.version = "legacy".to_string();
    Ok((payload, default_attrs))
}

/// Known standard data types across Antigravity-Manager.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KnownDataType {
    SupabaseEndpoints,
    SupabaseCredentials,
    AccountsExport,
    InstancesExport,
    ConfigBackup,
    PromptBackups,
    ProxyBindings,
    EmailCredentials,
    TelegramConfig,
    SshNodes,
    Unrecognized(String),
}

impl KnownDataType {
    pub fn as_type_str(&self) -> &str {
        match self {
            Self::SupabaseEndpoints => "agm/supabase-endpoints",
            Self::SupabaseCredentials => "agm/supabase-credentials",
            Self::AccountsExport => "agm/accounts-export",
            Self::InstancesExport => "agm/instances-export",
            Self::ConfigBackup => "agm/config-backup",
            Self::PromptBackups => "agm/prompt-backups",
            Self::ProxyBindings => "agm/proxy-bindings",
            Self::EmailCredentials => "agm/email-credentials",
            Self::TelegramConfig => "agm/telegram-config",
            Self::SshNodes => "agm/ssh-nodes",
            Self::Unrecognized(s) => s.as_str(),
        }
    }

    pub fn from_type_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "agm/supabase-endpoints" | "supabase-endpoints" | "supabase_endpoints" => {
                Self::SupabaseEndpoints
            }
            "agm/supabase-credentials" | "supabase-credentials" | "supabase_credentials" => {
                Self::SupabaseCredentials
            }
            "agm/accounts-export" | "accounts-export" | "accounts" => Self::AccountsExport,
            "agm/instances-export" | "instances-export" | "instances" => Self::InstancesExport,
            "agm/config-backup" | "config-backup" | "gui-config" | "gui_config" => {
                Self::ConfigBackup
            }
            "agm/prompt-backups" | "prompt-backups" | "prompts" => Self::PromptBackups,
            "agm/proxy-bindings" | "proxy-bindings" => Self::ProxyBindings,
            "agm/email-credentials" | "email-credentials" | "email" => Self::EmailCredentials,
            "agm/telegram-config" | "telegram-config" | "telegram" => Self::TelegramConfig,
            "agm/ssh-nodes" | "ssh-nodes" | "ssh_nodes" | "nodes" => Self::SshNodes,
            other => Self::Unrecognized(other.to_string()),
        }
    }
}
