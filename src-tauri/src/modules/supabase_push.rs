//! One-shot Supabase settings push from an explicit vault path (plan 88, subtask 005).
//!
//! `agm supabase push-settings --vault <path>` imports the Supabase config
//! (keys included) from the given vault file — or every known Supabase JSON
//! in the given folder — then syncs the local node and runs a connectivity
//! test against each enabled endpoint. The vault path is always explicit
//! (argument or `AGM_SUPABASE_VAULT`); nothing is auto-discovered outside
//! it. Secret values are never included in the report.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::modules::supabase_client::SupabaseClient;
use crate::modules::supabase_sync;

/// Result of one step of the push pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushStepResult {
    pub name: String,
    pub is_ok: bool,
    pub message: String,
}

/// Aggregate report returned by `push_settings`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PushReport {
    pub vault_path: String,
    pub files_loaded: usize,
    pub endpoints_imported: usize,
    pub credentials_imported: usize,
    pub endpoints_total: usize,
    pub steps: Vec<PushStepResult>,
    pub is_success: bool,
}

fn step(name: &str, is_ok: bool, message: impl Into<String>) -> PushStepResult {
    PushStepResult {
        name: name.to_string(),
        is_ok,
        message: message.into(),
    }
}

/// Collect the JSON files to import from an explicit vault path: the file
/// itself, or every `*.json` file in the folder whose name marks it as a
/// Supabase file. No discovery happens outside the given path.
pub fn collect_vault_files(vault_path: &Path) -> Result<Vec<PathBuf>, String> {
    if vault_path.is_file() {
        return Ok(vec![vault_path.to_path_buf()]);
    }
    if vault_path.is_dir() {
        let mut files: Vec<PathBuf> = Vec::new();
        let entries = std::fs::read_dir(vault_path)
            .map_err(|e| format!("Failed to read vault folder: {}", e))?;
        for entry in entries {
            let entry = entry
                .map_err(|e| format!("Failed to read an entry in the vault folder: {}", e))?;
            let path = entry.path();
            let is_json = path
                .extension()
                .map(|ext| ext.eq_ignore_ascii_case("json"))
                .unwrap_or(false);
            let name = entry.file_name().to_string_lossy().to_lowercase();
            if is_json && name.contains("supabase") {
                files.push(path);
            }
        }
        files.sort();
        if files.is_empty() {
            return Err(
                "No Supabase JSON files found in the given vault folder.".to_string(),
            );
        }
        return Ok(files);
    }
    Err(format!(
        "Vault path '{}' is neither a file nor a folder.",
        vault_path.display()
    ))
}

/// Import, sync, and verify Supabase settings from an explicit vault path.
pub async fn push_settings(vault_path: &Path) -> PushReport {
    let mut report = PushReport {
        vault_path: vault_path.display().to_string(),
        files_loaded: 0,
        endpoints_imported: 0,
        credentials_imported: 0,
        endpoints_total: 0,
        steps: Vec::new(),
        is_success: false,
    };

    // Step 1: import every vault file into the config.
    let files = match collect_vault_files(vault_path) {
        Ok(f) => f,
        Err(e) => {
            report.steps.push(step("import", false, e));
            return report;
        }
    };

    let mut cfg = match supabase_sync::load_config() {
        Ok(cfg) => cfg,
        Err(e) => {
            report.steps.push(step(
                "import",
                false,
                format!("Failed to load the existing Supabase config: {}", e),
            ));
            return report;
        }
    };
    let mut sync_flag: Option<bool> = None;
    let mut import_failed = false;
    for file in &files {
        let label = file.display().to_string();
        let content = match std::fs::read_to_string(file) {
            Ok(c) => c,
            Err(e) => {
                import_failed = true;
                report
                    .steps
                    .push(step("import", false, format!("Failed to read '{}': {}", label, e)));
                continue;
            }
        };
        let raw_val: serde_json::Value = match serde_json::from_str(&content) {
            Ok(v) => v,
            Err(e) => {
                import_failed = true;
                report.steps.push(step(
                    "import",
                    false,
                    format!("Failed to parse JSON in '{}': {}", label, e),
                ));
                continue;
            }
        };
        let outcome = supabase_sync::merge_supabase_json_into_config(&mut cfg, raw_val, &label);
        if outcome.sync_flag.is_some() {
            sync_flag = outcome.sync_flag;
        }
        report.files_loaded += 1;
        report.endpoints_imported += outcome.endpoints_added;
        report.credentials_imported += outcome.credentials_added;
        for msg in &outcome.messages {
            report.steps.push(step("import", true, msg.clone()));
        }
        if outcome.endpoints_added == 0 && outcome.credentials_added == 0 {
            import_failed = true;
            report.steps.push(step(
                "import",
                false,
                format!("No endpoints or credentials recognized in '{}'.", label),
            ));
        }
    }

    if report.files_loaded == 0 || import_failed {
        report.steps.push(step(
            "import",
            false,
            "Import step failed; sync and connectivity test skipped.",
        ));
        return report;
    }

    supabase_sync::finalize_imported_config(&mut cfg, sync_flag);
    report.endpoints_total = cfg.endpoints.len();
    match supabase_sync::save_config(&cfg) {
        Ok(()) => report.steps.push(step(
            "import",
            true,
            format!(
                "Saved {} active endpoint(s) to supabase_config.json.",
                cfg.endpoints.len()
            ),
        )),
        Err(e) => {
            report.steps.push(step(
                "import",
                false,
                format!("Failed to save Supabase config: {}", e),
            ));
            return report;
        }
    }

    // Step 2: sync the local node now.
    match supabase_sync::sync_local_node_now().await {
        Ok(()) => report.steps.push(step("sync", true, "Sync completed successfully.")),
        Err(e) => {
            report
                .steps
                .push(step("sync", false, format!("Sync failed: {}", e)));
            return report;
        }
    }

    // Step 3: connectivity test against each enabled endpoint.
    let enabled: Vec<_> = cfg.endpoints.iter().filter(|ep| ep.is_enabled).collect();
    if enabled.is_empty() {
        report.steps.push(step(
            "test",
            false,
            "No enabled endpoints to test after import.",
        ));
        return report;
    }
    let mut has_test_failure = false;
    for ep in enabled {
        match SupabaseClient::new(ep) {
            Ok(client) => match client.test_connection().await {
                Ok(res) => {
                    if !res.is_success {
                        has_test_failure = true;
                    }
                    report.steps.push(step(
                        "test",
                        res.is_success,
                        format!("[{}] {}", ep.id, res.message),
                    ));
                }
                Err(e) => {
                    has_test_failure = true;
                    report
                        .steps
                        .push(step("test", false, format!("[{}] {}", ep.id, e)));
                }
            },
            Err(e) => {
                has_test_failure = true;
                report.steps.push(step(
                    "test",
                    false,
                    format!("[{}] Client init error: {}", ep.id, e),
                ));
            }
        }
    }

    report.is_success = !has_test_failure;
    report
}
