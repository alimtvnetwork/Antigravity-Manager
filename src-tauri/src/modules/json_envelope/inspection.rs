use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

use super::*;

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
pub(crate) fn detect_legacy_signature(root: &Value) -> Option<(KnownDataType, Value, usize)> {
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
pub(crate) fn describe_format(
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

pub(crate) fn collect_json_recursive(dir: &Path, out: &mut Vec<PathBuf>) {
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
