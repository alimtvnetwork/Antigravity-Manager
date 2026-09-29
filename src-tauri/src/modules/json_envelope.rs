use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

fn default_version() -> String {
    "1.0".to_string()
}

/// Metadata attributes header required for all standard AGM JSON payloads.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonAttributes {
    #[serde(rename = "type")]
    pub data_type: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_alias: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl JsonAttributes {
    pub fn new(data_type: impl Into<String>) -> Self {
        Self {
            data_type: data_type.into(),
            version: default_version(),
            source: Some("agm-cli".to_string()),
            created_at: Some(chrono::Utc::now().to_rfc3339()),
            node_id: None,
            node_alias: None,
            encoding: Some("utf-8".to_string()),
            description: None,
        }
    }
}

/// Strongly typed universal JSON envelope containing `attributes` and `data`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonEnvelope<T> {
    pub attributes: JsonAttributes,
    pub data: T,
}

impl<T> JsonEnvelope<T> {
    pub fn new(data_type: impl Into<String>, data: T) -> Self {
        Self {
            attributes: JsonAttributes::new(data_type),
            data,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.attributes.description = Some(desc.into());
        self
    }
}

/// Dynamically typed universal JSON envelope where `data` is arbitrary JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicJsonEnvelope {
    pub attributes: JsonAttributes,
    pub data: Value,
}

/// Unpacks a JSON value: if wrapped in `{ "attributes": ..., "data": ... }`, returns
/// `(Some(attributes), data)`. Otherwise returns `(None, original_value)`.
pub fn unpack_envelope(value: Value) -> (Option<JsonAttributes>, Value) {
    if let Value::Object(ref map) = value {
        if let (Some(attrs_val), Some(data_val)) = (map.get("attributes"), map.get("data")) {
            if let Ok(attrs) = serde_json::from_value::<JsonAttributes>(attrs_val.clone()) {
                return (Some(attrs), data_val.clone());
            }
        }
    }
    (None, value)
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
            "agm/config-backup" | "config-backup" | "gui-config" => Self::ConfigBackup,
            "agm/prompt-backups" | "prompt-backups" | "prompts" => Self::PromptBackups,
            "agm/proxy-bindings" | "proxy-bindings" => Self::ProxyBindings,
            "agm/email-credentials" | "email-credentials" | "email" => Self::EmailCredentials,
            "agm/telegram-config" | "telegram-config" | "telegram" => Self::TelegramConfig,
            other => Self::Unrecognized(other.to_string()),
        }
    }
}

/// Detailed inspection result for a single JSON file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatInspectionResult {
    pub file_path: String,
    pub file_name: String,
    pub is_valid_json: bool,
    pub is_envelope: bool,
    pub is_legacy: bool,
    pub detected_type: Option<String>,
    pub human_name: String,
    pub mutation_summary: String,
    pub recommended_command: String,
    pub item_count: usize,
    pub error_detail: Option<String>,
}

/// Inspect a JSON file path and determine its schema type and mutation impact.
pub fn inspect_json_file(path: &Path) -> FormatInspectionResult {
    let file_name = path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown.json".to_string());
    let path_str = path.to_string_lossy().to_string();

    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(e) => {
            return FormatInspectionResult {
                file_path: path_str,
                file_name,
                is_valid_json: false,
                is_envelope: false,
                is_legacy: false,
                detected_type: None,
                human_name: "Unreadable File".to_string(),
                mutation_summary: "File cannot be read from filesystem".to_string(),
                recommended_command: String::new(),
                item_count: 0,
                error_detail: Some(e.to_string()),
            };
        }
    };

    let parsed: Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            return FormatInspectionResult {
                file_path: path_str,
                file_name,
                is_valid_json: false,
                is_envelope: false,
                is_legacy: false,
                detected_type: None,
                human_name: "Invalid JSON".to_string(),
                mutation_summary: "Content is not valid JSON syntax".to_string(),
                recommended_command: String::new(),
                item_count: 0,
                error_detail: Some(e.to_string()),
            };
        }
    };

    inspect_json_value(&parsed, path)
}

/// Inspect an already-parsed JSON value against both envelope schema and legacy signatures.
pub fn inspect_json_value(root: &Value, path: &Path) -> FormatInspectionResult {
    let file_name = path
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "memory.json".to_string());
    let path_str = path.to_string_lossy().to_string();

    // 1. Check for standard envelope { "attributes": { "type": "..." }, "data": ... }
    if let (Some(attrs), data_val) = unpack_envelope(root.clone()) {
        let known = KnownDataType::from_type_str(&attrs.data_type);
        let (human_name, mutation, cmd, count) =
            describe_format(&known, &data_val, &path_str, false);
        return FormatInspectionResult {
            file_path: path_str,
            file_name,
            is_valid_json: true,
            is_envelope: true,
            is_legacy: false,
            detected_type: Some(attrs.data_type),
            human_name,
            mutation_summary: mutation,
            recommended_command: cmd,
            item_count: count,
            error_detail: None,
        };
    }

    // 2. Fallback to signature detection for legacy flat JSONs
    if let Some((known, data_val, count)) = detect_legacy_signature(root) {
        let (human_name, mutation, cmd, _) = describe_format(&known, &data_val, &path_str, true);
        return FormatInspectionResult {
            file_path: path_str,
            file_name,
            is_valid_json: true,
            is_envelope: false,
            is_legacy: true,
            detected_type: Some(known.as_type_str().to_string()),
            human_name: format!("{} (Legacy)", human_name),
            mutation_summary: mutation,
            recommended_command: cmd,
            item_count: count,
            error_detail: None,
        };
    }

    // 3. Unrecognized JSON schema
    FormatInspectionResult {
        file_path: path_str,
        file_name,
        is_valid_json: true,
        is_envelope: false,
        is_legacy: false,
        detected_type: None,
        human_name: "Unrecognized Schema".to_string(),
        mutation_summary: "Does not match any recognized AGM envelope or schema signature"
            .to_string(),
        recommended_command: String::new(),
        item_count: 0,
        error_detail: Some(
            "Missing 'attributes.type' and no matching legacy signatures detected".to_string(),
        ),
    }
}

/// Fallback heuristics to recognize flat legacy JSONs
fn detect_legacy_signature(root: &Value) -> Option<(KnownDataType, Value, usize)> {
    if let Value::Object(ref map) = root {
        // Supabase Endpoints: contains "endpoints" array
        if let Some(Value::Array(arr)) = map.get("endpoints") {
            let matches = arr
                .iter()
                .any(|item| item.get("url").is_some() && item.get("api_key").is_some());
            if matches || !arr.is_empty() {
                return Some((KnownDataType::SupabaseEndpoints, root.clone(), arr.len()));
            }
        }

        // Supabase Credentials: contains "credentials" object or "endpoint" + "token" / "service"
        if let Some(Value::Object(creds)) = map.get("credentials") {
            if creds.get("endpoint").is_some() || creds.get("service").is_some() {
                return Some((KnownDataType::SupabaseCredentials, root.clone(), 1));
            }
        }
        if map.get("endpoint").is_some()
            && (map.get("token").is_some() || map.get("service").is_some())
        {
            return Some((KnownDataType::SupabaseCredentials, root.clone(), 1));
        }

        // Accounts Export: contains "accounts" array
        if let Some(Value::Array(arr)) = map.get("accounts") {
            return Some((KnownDataType::AccountsExport, root.clone(), arr.len()));
        }

        // Instances Export: contains "instances" array
        if let Some(Value::Array(arr)) = map.get("instances") {
            return Some((KnownDataType::InstancesExport, root.clone(), arr.len()));
        }

        // Config Backup: contains "quota_protection" or "proxy_listen_port" or "proxy_pass"
        if map.get("quota_protection").is_some()
            || map.get("proxy_listen_port").is_some()
            || map.get("theme").is_some()
        {
            return Some((KnownDataType::ConfigBackup, root.clone(), 1));
        }

        // Prompts Backup: contains "prompts" array
        if let Some(Value::Array(arr)) = map.get("prompts") {
            return Some((KnownDataType::PromptBackups, root.clone(), arr.len()));
        }
    } else if let Value::Array(arr) = root {
        // Array of Supabase endpoints
        if arr
            .iter()
            .any(|item| item.get("url").is_some() && item.get("api_key").is_some())
        {
            return Some((KnownDataType::SupabaseEndpoints, root.clone(), arr.len()));
        }
        // Array of accounts
        if arr.iter().any(|item| item.get("email").is_some()) {
            return Some((KnownDataType::AccountsExport, root.clone(), arr.len()));
        }
        // Array of instances
        if arr.iter().any(|item| item.get("bound_email").is_some()) {
            return Some((KnownDataType::InstancesExport, root.clone(), arr.len()));
        }
    }

    None
}

/// Generates human-friendly descriptions, mutation impact, and import commands
fn describe_format(
    dtype: &KnownDataType,
    data: &Value,
    path_str: &str,
    is_legacy: bool,
) -> (String, String, String, usize) {
    let clean_path = path_str.replace('\\', "/");
    match dtype {
        KnownDataType::SupabaseEndpoints => {
            let count = if let Some(arr) = data.get("endpoints").and_then(|e| e.as_array()) {
                arr.len()
            } else if let Some(arr) = data.as_array() {
                arr.len()
            } else {
                1
            };
            (
                "Supabase Fleet Endpoints".to_string(),
                format!(
                    "Registers/updates {} remote Supabase endpoint(s) in local 'supabase_config.json' and syncs node status",
                    count
                ),
                format!("agm supabase load-json \"{}\" -y", clean_path),
                count,
            )
        }
        KnownDataType::SupabaseCredentials => (
            "Supabase Fleet Credentials".to_string(),
            "Decodes and imports fleet REST authentication service tokens into local database vault".to_string(),
            format!("agm supabase load-json \"{}\" -y", clean_path),
            1,
        ),
        KnownDataType::AccountsExport => {
            let count = if let Some(arr) = data.get("accounts").and_then(|a| a.as_array()) {
                arr.len()
            } else if let Some(arr) = data.as_array() {
                arr.len()
            } else {
                1
            };
            (
                "Google Accounts Vault".to_string(),
                format!(
                    "Merges {} OAuth account token(s) into 'accounts.json' and local OS Keyring",
                    count
                ),
                format!("agm accounts import \"{}\" -y", clean_path),
                count,
            )
        }
        KnownDataType::InstancesExport => {
            let count = if let Some(arr) = data.get("instances").and_then(|i| i.as_array()) {
                arr.len()
            } else if let Some(arr) = data.as_array() {
                arr.len()
            } else {
                1
            };
            (
                "Sandbox Instance Profiles".to_string(),
                format!(
                    "Registers {} isolated IDE sandbox profile(s) and workspace directory bindings in 'instances.json'",
                    count
                ),
                format!("agm instances import \"{}\" -y", clean_path),
                count,
            )
        }
        KnownDataType::ConfigBackup => (
            "System Configuration Backup".to_string(),
            "Updates system configuration, proxy ports, and rotation thresholds in 'gui_config.json'".to_string(),
            format!("agm config restore \"{}\" -y", clean_path),
            1,
        ),
        KnownDataType::PromptBackups => {
            let count = if let Some(arr) = data.get("prompts").and_then(|p| p.as_array()) {
                arr.len()
            } else if let Some(arr) = data.as_array() {
                arr.len()
            } else {
                1
            };
            (
                "In-Flight Prompts Archive".to_string(),
                format!(
                    "Restores {} in-flight conversation prompt(s) and task states into 'backup-prompts.db'",
                    count
                ),
                format!("agm prompts restore \"{}\" -y", clean_path),
                count,
            )
        }
        KnownDataType::ProxyBindings => (
            "Proxy Route Bindings".to_string(),
            "Applies model routing, port redirection, and quota protection bindings".to_string(),
            format!("agm proxy bindings import \"{}\" -y", clean_path),
            1,
        ),
        KnownDataType::EmailCredentials => (
            "Fleet Email Credentials".to_string(),
            "Configures inbound IMAP & outbound SMTP notification accounts in email vault".to_string(),
            format!("agm email config import \"{}\" -y", clean_path),
            1,
        ),
        KnownDataType::TelegramConfig => (
            "Telegram Bot Configuration".to_string(),
            "Updates remote Telegram bot token and authorized chat ID in local vault".to_string(),
            format!("agm telegram config import \"{}\" -y", clean_path),
            1,
        ),
        KnownDataType::Unrecognized(other) => {
            let note = if is_legacy { "Legacy" } else { "Custom" };
            (
                format!("{} ({})", other, note),
                format!("Unrecognized AGM schema type '{}'", other),
                String::new(),
                0,
            )
        }
    }
}

fn collect_json_recursive(dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if path.extension().map_or(false, |ext| ext == "json") {
                    out.push(path);
                }
            } else if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !name.starts_with('.') && name != "node_modules" && name != "target" {
                    collect_json_recursive(&path, out);
                }
            }
        }
    }
}

/// Recursively collect all `.json` files if a directory is given, or return the files.
pub fn resolve_json_targets(inputs: &[PathBuf]) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let targets = if inputs.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        inputs.to_vec()
    };

    for target in targets {
        if target.is_file() {
            if target.extension().map_or(false, |ext| ext == "json") {
                files.push(target);
            }
        } else if target.is_dir() {
            collect_json_recursive(&target, &mut files);
        }
    }

    files.sort();
    files
}

/// Builds a single-line command to import all matched JSON files in one run.
pub fn build_bulk_import_command(results: &[FormatInspectionResult]) -> Option<String> {
    let matched_results: Vec<&FormatInspectionResult> = results
        .iter()
        .filter(|r| r.detected_type.is_some() && !r.recommended_command.is_empty())
        .collect();

    if matched_results.is_empty() {
        return None;
    }

    // Separate supabase loads vs accounts vs others
    let mut supabase_files = Vec::new();
    let mut other_cmds = Vec::new();

    for r in &matched_results {
        if let Some(ref dtype) = r.detected_type {
            if dtype.contains("supabase") {
                supabase_files.push(r.file_path.replace('\\', "/"));
            } else {
                other_cmds.push(r.recommended_command.clone());
            }
        }
    }

    let mut parts = Vec::new();
    if !supabase_files.is_empty() {
        parts.push(format!(
            "agm supabase load-json {} -y",
            supabase_files
                .iter()
                .map(|f| format!("\"{}\"", f))
                .collect::<Vec<_>>()
                .join(" ")
        ));
    }
    for cmd in other_cmds {
        parts.push(cmd);
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" && "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_envelope_serialization_and_unpacking() {
        let envelope = JsonEnvelope::new("agm/test-type", json!({ "key": "value", "count": 42 }))
            .with_description("Unit test payload");

        let serialized = serde_json::to_string(&envelope).expect("serialize envelope");
        let parsed: Value = serde_json::from_str(&serialized).expect("deserialize Value");

        let (attrs_opt, data) = unpack_envelope(parsed);
        assert!(attrs_opt.is_some());
        let attrs = attrs_opt.unwrap();
        assert_eq!(attrs.data_type, "agm/test-type");
        assert_eq!(attrs.version, "1.0");
        assert_eq!(attrs.description.as_deref(), Some("Unit test payload"));
        assert_eq!(data["key"], "value");
        assert_eq!(data["count"], 42);
    }

    #[test]
    fn test_legacy_format_detection() {
        let legacy_sb = json!({
            "endpoints": [
                { "id": "ep-1", "url": "https://test.supabase.co", "api_key": "sb_123", "role": "root" }
            ]
        });
        let res = inspect_json_value(&legacy_sb, Path::new("legacy_endpoints.json"));
        assert!(res.is_valid_json);
        assert!(!res.is_envelope);
        assert!(res.is_legacy);
        assert_eq!(res.detected_type.as_deref(), Some("agm/supabase-endpoints"));
        assert!(res.recommended_command.contains("agm supabase load-json"));
    }

    #[test]
    fn test_unrecognized_schema_rejection() {
        let foreign = json!({
            "name": "random-package",
            "version": "1.0.0",
            "scripts": { "start": "node index.js" }
        });
        let res = inspect_json_value(&foreign, Path::new("package.json"));
        assert!(res.is_valid_json);
        assert!(!res.is_envelope);
        assert!(!res.is_legacy);
        assert!(res.detected_type.is_none());
        assert!(res.recommended_command.is_empty());
    }

    #[test]
    fn test_bulk_command_generation() {
        let res1 = FormatInspectionResult {
            file_path: "ep1.json".to_string(),
            file_name: "ep1.json".to_string(),
            is_valid_json: true,
            is_envelope: true,
            is_legacy: false,
            detected_type: Some("agm/supabase-endpoints".to_string()),
            human_name: "Supabase Endpoints".to_string(),
            mutation_summary: "Registers endpoints".to_string(),
            recommended_command: "agm supabase load-json \"ep1.json\" -y".to_string(),
            item_count: 1,
            error_detail: None,
        };
        let res2 = FormatInspectionResult {
            file_path: "acc.json".to_string(),
            file_name: "acc.json".to_string(),
            is_valid_json: true,
            is_envelope: true,
            is_legacy: false,
            detected_type: Some("agm/accounts-export".to_string()),
            human_name: "Accounts".to_string(),
            mutation_summary: "Merges accounts".to_string(),
            recommended_command: "agm accounts import \"acc.json\" -y".to_string(),
            item_count: 2,
            error_detail: None,
        };
        let bulk = build_bulk_import_command(&[res1, res2]);
        assert!(bulk.is_some());
        let cmd = bulk.unwrap();
        assert!(cmd.contains("agm supabase load-json \"ep1.json\" -y"));
        assert!(cmd.contains("agm accounts import \"acc.json\" -y"));
    }
}
