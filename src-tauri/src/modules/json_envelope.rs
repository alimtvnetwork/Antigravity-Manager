use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

fn default_version() -> String {
    "2.0".to_string()
}

/// Structured work directory configuration using root variables.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkDirectoryConfig {
    pub path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_path: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_applied: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_enforced: bool,
}

fn deserialize_work_directory<'de, D>(
    deserializer: D,
) -> Result<Option<WorkDirectoryConfig>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let val: Option<Value> = Option::deserialize(deserializer)?;
    match val {
        None => Ok(None),
        Some(Value::Null) => Ok(None),
        Some(Value::String(s)) => Ok(Some(WorkDirectoryConfig {
            path: s,
            default_path: None,
            is_applied: true,
            is_enforced: false,
        })),
        Some(Value::Object(_)) => serde_json::from_value::<WorkDirectoryConfig>(val.unwrap())
            .map(Some)
            .map_err(serde::de::Error::custom),
        _ => Ok(None),
    }
}

/// Metadata attributes header required for all standard AGM JSON envelopes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct JsonAttributes {
    #[serde(rename = "type")]
    pub data_type: String,
    #[serde(default = "default_version")]
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub how: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "agmVersion",
        alias = "gitmapVersion",
        alias = "gitmap_version"
    )]
    pub agm_version: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "importCommand"
    )]
    pub import_command: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "exportCommand"
    )]
    pub export_command: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "helpCommand"
    )]
    pub help_command: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "workDirectory",
        deserialize_with = "deserialize_work_directory"
    )]
    pub work_directory: Option<WorkDirectoryConfig>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        alias = "defaultWorkDirectory"
    )]
    pub default_work_directory: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "std::ops::Not::not",
        alias = "isWorkDirectoryApplied"
    )]
    pub is_work_directory_applied: bool,
    #[serde(
        default,
        skip_serializing_if = "std::ops::Not::not",
        alias = "isWorkDirectoryEnforced"
    )]
    pub is_work_directory_enforced: bool,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "createdAt")]
    pub created_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "nodeId")]
    pub node_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none", alias = "nodeAlias")]
    pub node_alias: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encoding: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Resolves a relative JSON filename against CWD and common relative subdirectories.
pub fn resolve_relative_json_path(input: &str) -> PathBuf {
    let trimmed = input.trim();
    let direct = PathBuf::from(trimmed);
    if direct.exists() {
        return direct;
    }
    let base_name = direct
        .file_name()
        .and_then(|f| f.to_str())
        .unwrap_or(trimmed);
    let search_dirs = [
        ".",
        "vault",
        "instances",
        "vault/instances",
        "02-antigravity-manager/vault",
        "02-antigravity-and-event-manager/vault",
        "02-antigravity-and-event-manager/vault/instances",
        "01-gitmap",
    ];
    for dir in &search_dirs {
        let candidate = Path::new(dir).join(base_name);
        if candidate.exists() {
            return candidate;
        }
    }
    direct
}

pub fn default_commands_for_type(
    dtype: &str,
) -> (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
) {
    let lower = dtype.to_lowercase();
    if lower.contains("supabase-endpoints") || lower.contains("supabase_endpoints") {
        (
            Some("agm supabase load-json supabase_config.json -y".to_string()),
            Some("agm supabase export --file supabase_config.json".to_string()),
            Some("agm which-format supabase_config.json".to_string()),
            Some("Supabase multi-node endpoints configuration for AGM fleet. Cross-platform compatible across Windows, Linux, Ubuntu, and macOS.".to_string()),
        )
    } else if lower.contains("supabase-credentials") || lower.contains("supabase_credentials") {
        (
            Some("agm supabase load-json supabase_config.json -y".to_string()),
            Some("agm supabase export --file supabase_config.json".to_string()),
            Some("agm which-format supabase_config.json".to_string()),
            Some("Supabase fleet REST authentication credentials and service tokens. Cross-platform compatible across Windows, Linux, Ubuntu, and macOS.".to_string()),
        )
    } else if lower.contains("accounts") {
        (
            Some("agm accounts import accounts.json".to_string()),
            Some("agm accounts export --file accounts.json".to_string()),
            Some("agm which-format accounts.json".to_string()),
            Some("Google Gemini accounts and quota matrix. Cross-platform compatible across Windows, Linux, Ubuntu, and macOS.".to_string()),
        )
    } else if lower.contains("instances") {
        (
            Some("agm instances import instances.json".to_string()),
            Some("agm instances export --file instances.json".to_string()),
            Some("agm which-format instances.json".to_string()),
            Some("Sandbox instance profiles and directory bindings for AGM fleet. Cross-platform compatible across Windows, Linux, Ubuntu, and macOS.".to_string()),
        )
    } else if lower.contains("email") {
        (
            Some("agm email import email-credentials.json".to_string()),
            Some("agm email export --file email-credentials.json".to_string()),
            Some("agm which-format email-credentials.json".to_string()),
            Some("Inbound IMAP & outbound SMTP fleet email credentials configuration. Cross-platform compatible across Windows, Linux, Ubuntu, and macOS.".to_string()),
        )
    } else if lower.contains("telegram") {
        (
            Some("agm telegram import telegram_config.json".to_string()),
            Some("agm telegram export --file telegram_config.json".to_string()),
            Some("agm which-format telegram_config.json".to_string()),
            Some("Telegram bot configuration for remote AGM commands and alerts. Cross-platform compatible across Windows, Linux, Ubuntu, and macOS.".to_string()),
        )
    } else if lower.contains("config") {
        (
            Some("agm config restore gui_config.json -y".to_string()),
            Some("agm config export --file gui_config.json".to_string()),
            Some("agm which-format gui_config.json".to_string()),
            Some("Antigravity Manager system configuration backup. Cross-platform compatible across Windows, Linux, Ubuntu, and macOS.".to_string()),
        )
    } else if lower.contains("ssh") || lower.contains("nodes") {
        (
            Some("agm ssh nodes import-json gitmap-ssh-nodes.json -y".to_string()),
            Some("agm ssh nodes export-json gitmap-ssh-nodes.json".to_string()),
            Some("agm which-format gitmap-ssh-nodes.json".to_string()),
            Some("SSH cluster fleet nodes configuration. Cross-platform compatible across Windows, Linux, Ubuntu, and macOS.".to_string()),
        )
    } else {
        (None, None, Some("agm which-format".to_string()), None)
    }
}

impl JsonAttributes {
    pub fn new(data_type: impl Into<String>) -> Self {
        let dtype = data_type.into();
        let cur_time = chrono::Utc::now().to_rfc3339();

        let work_cfg = WorkDirectoryConfig {
            path: "${workDir}".to_string(),
            default_path: Some("D:\\work".to_string()),
            is_applied: true,
            is_enforced: false,
        };

        let (import_cmd, export_cmd, help_cmd, notes) = default_commands_for_type(&dtype);

        Self {
            data_type: dtype,
            version: default_version(),
            source: Some("agm-cli".to_string()),
            how: export_cmd.clone(),
            agm_version: Some("4.103.0".to_string()),
            import_command: import_cmd,
            export_command: export_cmd,
            help_command: help_cmd,
            notes,
            timestamp: Some(cur_time.clone()),
            work_directory: Some(work_cfg),
            default_work_directory: Some("D:\\work".to_string()),
            is_work_directory_applied: true,
            is_work_directory_enforced: false,
            created_at: Some(cur_time),
            node_id: None,
            node_alias: None,
            encoding: Some("utf-8".to_string()),
            description: None,
        }
    }
}

/// Strongly typed universal JSON envelope containing `attributes`, optional `variables`, and `data`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct JsonEnvelope<T> {
    pub attributes: JsonAttributes,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<String, Value>>,
    pub data: T,
}

impl<T> JsonEnvelope<T> {
    pub fn new(data_type: impl Into<String>, data: T) -> Self {
        let mut vars = HashMap::new();
        vars.insert("workDir".to_string(), Value::String("D:\\work".to_string()));
        vars.insert(
            "repoDir".to_string(),
            Value::String("${workDir}\\antigravity-manager".to_string()),
        );
        vars.insert("secretsDir".to_string(), Value::String(".".to_string()));
        Self {
            attributes: JsonAttributes::new(data_type),
            variables: Some(vars),
            data,
        }
    }

    pub fn with_variables(mut self, vars: HashMap<String, Value>) -> Self {
        self.variables = Some(vars);
        self
    }

    pub fn with_work_directory(mut self, work_dir: WorkDirectoryConfig) -> Self {
        self.attributes.work_directory = Some(work_dir);
        self
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.attributes.description = Some(desc.into());
        self
    }

    pub fn with_source(mut self, src: impl Into<String>) -> Self {
        let raw = src.into();
        let clean = std::path::Path::new(&raw)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(&raw)
            .to_string();
        self.attributes.source = Some(clean);
        self
    }
}

/// Dynamically typed universal JSON envelope where `data` is arbitrary JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicJsonEnvelope {
    pub attributes: JsonAttributes,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variables: Option<HashMap<String, Value>>,
    pub data: Value,
}

fn value_to_string(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        other => other.to_string(),
    }
}

/// Merges top-level and workDirectory variable dictionaries into a single lookup table.
pub fn merge_variables(
    top_vars: Option<&HashMap<String, Value>>,
    work_dir_vars: Option<&HashMap<String, Value>>,
) -> HashMap<String, String> {
    let mut out = HashMap::new();
    if let Some(w_vars) = work_dir_vars {
        for (k, v) in w_vars {
            out.insert(k.clone(), value_to_string(v));
        }
    }
    if let Some(t_vars) = top_vars {
        for (k, v) in t_vars {
            out.insert(k.clone(), value_to_string(v));
        }
    }
    resolve_chained_variables(&mut out);
    out
}

fn resolve_chained_variables(vars: &mut HashMap<String, String>) {
    for _ in 0..2 {
        let snapshot = vars.clone();
        for val in vars.values_mut() {
            if val.contains('$') {
                *val = resolve_string_variable(val, &snapshot);
            }
        }
    }
}

/// Replaces `${varName}`, `${variables.varName}`, and `$variables.varName` in input string.
pub fn resolve_string_variable(input: &str, vars: &HashMap<String, String>) -> String {
    if !input.contains('$') || vars.is_empty() {
        return input.to_string();
    }
    let mut res = input.to_string();
    for _ in 0..2 {
        let before = res.clone();
        for (k, v) in vars {
            let pat1 = format!("${{{}}}", k);
            let pat2 = format!("${{variables.{}}}", k);
            let pat3 = format!("$variables.{}", k);
            if res.contains(&pat1) {
                res = res.replace(&pat1, v);
            }
            if res.contains(&pat2) {
                res = res.replace(&pat2, v);
            }
            if res.contains(&pat3) {
                res = res.replace(&pat3, v);
            }
        }
        if res == before {
            break;
        }
    }
    res
}

/// Recursively expands variables across all string values in a Serde JSON Value.
pub fn expand_variables_in_value(val: &mut Value, vars: &HashMap<String, String>) {
    if vars.is_empty() {
        return;
    }
    match val {
        Value::String(s) => {
            if s.contains('$') {
                *s = resolve_string_variable(s, vars);
            }
        }
        Value::Array(arr) => {
            for item in arr {
                expand_variables_in_value(item, vars);
            }
        }
        Value::Object(map) => {
            for (_k, v) in map.iter_mut() {
                expand_variables_in_value(v, vars);
            }
        }
        _ => {}
    }
}

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

/// Detailed inspection result for a single JSON file.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FormatInspectionResult {
    pub file_path: String,
    pub file_name: String,
    pub is_valid_json: bool,
    pub is_envelope: bool,
    pub is_legacy: bool,
    pub detected_type: Option<String>,
    pub version: Option<String>,
    pub human_name: String,
    pub mutation_summary: String,
    pub recommended_command: String,
    pub export_command: Option<String>,
    pub help_command: Option<String>,
    pub notes: Option<String>,
    pub work_directory: Option<String>,
    pub variables_count: usize,
    pub item_count: usize,
    pub error_detail: Option<String>,
}

/// Inspect a JSON file path and determine its schema type and mutation impact.
pub fn inspect_json_file(path: &Path) -> FormatInspectionResult {
    let resolved = resolve_relative_json_path(&path.to_string_lossy());
    let file_name = resolved
        .file_name()
        .map(|f| f.to_string_lossy().to_string())
        .unwrap_or_else(|| "unknown.json".to_string());
    let path_str = resolved.to_string_lossy().to_string();

    let content = match fs::read_to_string(&resolved) {
        Ok(c) => c,
        Err(e) => {
            return FormatInspectionResult {
                file_path: path_str,
                file_name,
                is_valid_json: false,
                is_envelope: false,
                is_legacy: false,
                detected_type: None,
                version: None,
                human_name: "Unreadable File".to_string(),
                mutation_summary: "File cannot be read from filesystem".to_string(),
                recommended_command: String::new(),
                export_command: None,
                help_command: None,
                notes: None,
                work_directory: None,
                variables_count: 0,
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
                version: None,
                human_name: "Invalid JSON".to_string(),
                mutation_summary: "Content is not valid JSON syntax".to_string(),
                recommended_command: String::new(),
                export_command: None,
                help_command: None,
                notes: None,
                work_directory: None,
                variables_count: 0,
                item_count: 0,
                error_detail: Some(e.to_string()),
            };
        }
    };

    inspect_json_value(&parsed, &resolved)
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
        let (human_name, mutation, default_cmd, count) =
            describe_format(&known, &data_val, &file_name, false);

        let final_cmd = attrs.import_command.clone().unwrap_or_else(|| default_cmd);

        let vars_count = if let Value::Object(ref m) = root {
            m.get("variables")
                .and_then(|v| v.as_object())
                .map(|o| o.len())
                .unwrap_or(0)
        } else {
            0
        };

        let work_dir_path = attrs.work_directory.as_ref().map(|w| w.path.clone());

        return FormatInspectionResult {
            file_path: path_str,
            file_name,
            is_valid_json: true,
            is_envelope: true,
            is_legacy: false,
            detected_type: Some(attrs.data_type),
            version: Some(attrs.version),
            human_name,
            mutation_summary: mutation,
            recommended_command: final_cmd,
            export_command: attrs.export_command,
            help_command: attrs.help_command,
            notes: attrs.notes,
            work_directory: work_dir_path,
            variables_count: vars_count,
            item_count: count,
            error_detail: None,
        };
    }

    // 2. Fallback to signature detection for legacy flat JSONs
    if let Some((known, data_val, count)) = detect_legacy_signature(root) {
        let (human_name, mutation, cmd, _) = describe_format(&known, &data_val, &file_name, true);
        return FormatInspectionResult {
            file_path: path_str,
            file_name,
            is_valid_json: true,
            is_envelope: false,
            is_legacy: true,
            detected_type: Some(known.as_type_str().to_string()),
            version: Some("legacy".to_string()),
            human_name: format!("{} (Legacy)", human_name),
            mutation_summary: mutation,
            recommended_command: cmd,
            export_command: None,
            help_command: None,
            notes: None,
            work_directory: None,
            variables_count: 0,
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
        version: None,
        human_name: "Unrecognized Schema".to_string(),
        mutation_summary: "Does not match any recognized AGM envelope or schema signature"
            .to_string(),
        recommended_command: String::new(),
        export_command: None,
        help_command: None,
        notes: None,
        work_directory: None,
        variables_count: 0,
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
            || map.get("language").is_some()
        {
            return Some((KnownDataType::ConfigBackup, root.clone(), 1));
        }

        // Prompts Backup: contains "prompts" array
        if let Some(Value::Array(arr)) = map.get("prompts") {
            return Some((KnownDataType::PromptBackups, root.clone(), arr.len()));
        }

        // SSH Nodes
        if map.get("connections").is_some() || map.get("nodes").is_some() {
            return Some((KnownDataType::SshNodes, root.clone(), 1));
        }
    } else if let Value::Array(arr) = root {
        if arr
            .iter()
            .any(|item| item.get("url").is_some() && item.get("api_key").is_some())
        {
            return Some((KnownDataType::SupabaseEndpoints, root.clone(), arr.len()));
        }
        if arr.iter().any(|item| item.get("email").is_some()) {
            return Some((KnownDataType::AccountsExport, root.clone(), arr.len()));
        }
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
    _is_legacy: bool,
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
        KnownDataType::SshNodes => (
            "SSH Cluster Fleet Nodes".to_string(),
            "Imports and synchronizes remote SSH machine nodes and key paths into local vault".to_string(),
            format!("agm ssh nodes import-json \"{}\" -y", clean_path),
            1,
        ),
        KnownDataType::Unrecognized(other) => (
            format!("{} (Custom)", other),
            format!("Unrecognized AGM schema type '{}'", other),
            String::new(),
            0,
        ),
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

    let mut supabase_files = Vec::new();
    let mut other_cmds = Vec::new();

    for r in &matched_results {
        if let Some(ref dtype) = r.detected_type {
            if dtype.contains("supabase") {
                supabase_files.push(r.file_name.clone());
            } else {
                other_cmds.push(r.recommended_command.clone());
            }
        }
    }

    let mut parts = Vec::new();
    if !supabase_files.is_empty() {
        parts.push(format!(
            "agm supabase load-json {} -y",
            supabase_files.join(" ")
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
        assert_eq!(attrs.version, "2.0");
        assert_eq!(attrs.description.as_deref(), Some("Unit test payload"));
        assert_eq!(data["key"], "value");
        assert_eq!(data["count"], 42);
    }

    #[test]
    fn test_variable_interpolation_and_chaining() {
        let raw = r#"{
            "attributes": {
                "type": "agm/supabase-endpoints",
                "version": "2.0",
                "workDirectory": {
                    "path": "${workDir}",
                    "defaultPath": "D:\\work",
                    "isApplied": true,
                    "isEnforced": false
                }
            },
            "variables": {
                "workDir": "D:\\work",
                "repoDir": "${workDir}\\antigravity-manager",
                "baseUrl": "https://pezjuuddecbyfmqxytrv.supabase.co",
                "restUrl": "${baseUrl}/rest/v1/"
            },
            "data": {
                "endpoint": "${restUrl}",
                "dir": "${repoDir}"
            }
        }"#;

        let (data, attrs) = extract_payload::<Value>(raw).expect("extract_payload");
        assert_eq!(attrs.version, "2.0");
        assert_eq!(
            attrs.work_directory.map(|w| w.path),
            Some("D:\\work".to_string())
        );
        assert_eq!(
            data["endpoint"].as_str().unwrap(),
            "https://pezjuuddecbyfmqxytrv.supabase.co/rest/v1/"
        );
        assert_eq!(
            data["dir"].as_str().unwrap(),
            "D:\\work\\antigravity-manager"
        );
    }

    #[test]
    fn test_work_directory_polymorphism() {
        let raw_str = r#"{
            "attributes": {
                "type": "agm/test",
                "workDirectory": "D:\\custom\\path"
            },
            "data": { "ok": true }
        }"#;
        let (_, attrs) =
            extract_payload::<Value>(raw_str).expect("extract flat workDirectory string");
        assert_eq!(
            attrs.work_directory.map(|w| w.path),
            Some("D:\\custom\\path".to_string())
        );

        let raw_obj = r#"{
            "attributes": {
                "type": "agm/test",
                "workDirectory": {
                    "path": "${workDir}",
                    "defaultPath": "D:\\work",
                    "isApplied": true,
                    "isEnforced": false
                }
            },
            "variables": {
                "workDir": "D:\\work"
            },
            "data": { "ok": true }
        }"#;
        let (_, attrs2) = extract_payload::<Value>(raw_obj).expect("extract obj workDirectory");
        assert_eq!(
            attrs2.work_directory.map(|w| w.path),
            Some("D:\\work".to_string())
        );
    }
}
