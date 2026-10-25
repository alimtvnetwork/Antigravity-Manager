//! IDE Profile Bundle Export Engine (`.agmide` / `.json`).
//!
//! Conforms to `02-spec/21-app/226-antigravity-manager-clone-and-ui-enhancements/03-migration-and-verification-spec.md`.

use super::bundle_types::*;
use crate::error::AppError;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Packages an instance's settings, projects, and prompts into an `AgmIdeProfileBundle`.
///
/// If `output_path` is provided, writes the serialized JSON document to disk.
pub fn export_ide_profile_bundle(
    instance_id: &str,
    output_path: Option<&Path>,
) -> Result<AgmIdeProfileBundle, AppError> {
    // 1. Resolve instance
    let resolved_id = crate::modules::instance::resolve_instance_id(instance_id)
        .map_err(|e| AppError::Config(format!("Failed to resolve instance: {e}")))?;
    let registry = crate::modules::instance::load_registry()
        .map_err(|e| AppError::Config(format!("Failed to load registry: {e}")))?;
    let inst = registry
        .instances
        .iter()
        .find(|i| i.id == resolved_id)
        .ok_or_else(|| AppError::Config(format!("Instance '{resolved_id}' not found")))?;

    // 2. Read Settings, Keybindings, Snippets
    let settings = package_bundle_settings(inst)?;

    // 3. Read Projects & Sequences
    let projects = package_bundle_projects(&resolved_id, inst.is_default)?;

    // 4. Read Prompts & Templates (strict FIFO)
    let prompts = package_bundle_prompts(&resolved_id, inst.is_default)?;

    // 5. Metadata & Summary Counts
    let mut item_counts = HashMap::new();
    let settings_keys = if let serde_json::Value::Object(ref map) = settings.editor_settings {
        map.len()
    } else {
        0
    };
    item_counts.insert("settings_keys".to_string(), settings_keys);
    item_counts.insert("keybindings".to_string(), settings.keybindings.len());
    item_counts.insert("snippets".to_string(), settings.snippets.len());
    item_counts.insert("workspaces".to_string(), projects.workspace_uris.len());
    item_counts.insert(
        "pinned_projects".to_string(),
        projects.pinned_project_sequences.len(),
    );
    item_counts.insert(
        "running_projects".to_string(),
        projects.running_projects.len(),
    );
    item_counts.insert("active_prompts".to_string(), prompts.active_prompts.len());
    item_counts.insert(
        "suffix_templates".to_string(),
        prompts.suffix_templates.len(),
    );
    item_counts.insert(
        "resend_as_templates".to_string(),
        prompts.resend_as_templates.len(),
    );

    let bundle = AgmIdeProfileBundle {
        agm_bundle_version: AGM_IDE_BUNDLE_SCHEMA_VERSION.to_string(),
        exported_at: chrono::Utc::now().to_rfc3339(),
        source_environment: SourceEnvironment {
            instance_id: inst.id.clone(),
            instance_name: inst.name.clone(),
            agm_version: crate::VERSION.to_string(),
            platform: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
        },
        settings,
        projects,
        prompts,
        metadata: BundleMetadata {
            bundle_checksum: String::new(),
            item_counts,
        },
    };

    // Calculate SHA-256 Checksum over pre-checksum serialized JSON
    let serialized = serde_json::to_string_pretty(&bundle)
        .map_err(|e| AppError::Config(format!("Failed to serialize bundle: {e}")))?;
    let mut hasher = Sha256::new();
    hasher.update(serialized.as_bytes());
    let checksum = format!("{:x}", hasher.finalize());

    let mut final_bundle = bundle;
    final_bundle.metadata.bundle_checksum = checksum;

    // Optional Disk Write
    if let Some(dest) = output_path {
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).map_err(AppError::Io)?;
        }
        let pretty = serde_json::to_string_pretty(&final_bundle)
            .map_err(|e| AppError::Config(format!("Serialization error: {e}")))?;
        fs::write(dest, pretty).map_err(AppError::Io)?;
    }

    Ok(final_bundle)
}

fn package_bundle_settings(
    inst: &crate::models::InstanceConfig,
) -> Result<BundleSettings, AppError> {
    let mut editor_settings = serde_json::json!({});
    let mut keybindings = Vec::new();
    let mut snippets = HashMap::new();

    if !inst.data_dir.is_empty() {
        let user_dir = Path::new(&inst.data_dir).join("User");
        if user_dir.is_dir() {
            // 1. settings.json
            let settings_file = user_dir.join("settings.json");
            if settings_file.is_file() {
                if let Ok(content) = fs::read_to_string(&settings_file) {
                    if let Ok(val) = serde_json::from_str(&content) {
                        editor_settings = val;
                    }
                }
            }

            // 2. keybindings.json
            let kb_file = user_dir.join("keybindings.json");
            if kb_file.is_file() {
                if let Ok(content) = fs::read_to_string(&kb_file) {
                    if let Ok(val) = serde_json::from_str(&content) {
                        keybindings = val;
                    }
                }
            }

            // 3. snippets/*.json
            let snippets_dir = user_dir.join("snippets");
            if snippets_dir.is_dir() {
                if let Ok(entries) = fs::read_dir(&snippets_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_file()
                            && path.extension().and_then(|s| s.to_str()) == Some("json")
                        {
                            if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                                if let Ok(content) = fs::read_to_string(&path) {
                                    snippets.insert(file_name.to_string(), content);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(BundleSettings {
        editor_settings,
        keybindings,
        snippets,
    })
}

fn package_bundle_projects(
    instance_id: &str,
    is_default: bool,
) -> Result<BundleProjects, AppError> {
    let workspace_uris = crate::modules::instance::get_instance_workspace_paths(instance_id);

    let mut pinned_project_sequences = Vec::new();
    let mut running_projects = Vec::new();

    if let Ok(conn) = crate::modules::repo_db::connect_db() {
        // Query pinned sequences
        let is_def = is_default || instance_id == "default";
        let seq_query = if is_def {
            "SELECT project_key, seq_id, project_id, repo_name, repo_path 
             FROM agm_project_sequences 
             WHERE instance_id = 'default' OR instance_id = '__default__' OR instance_id = '' OR instance_id IS NULL 
             ORDER BY seq_id ASC"
        } else {
            "SELECT project_key, seq_id, project_id, repo_name, repo_path 
             FROM agm_project_sequences 
             WHERE instance_id = ?1 
             ORDER BY seq_id ASC"
        };

        if let Ok(mut seq_stmt) = conn.prepare(seq_query) {
            let map_fn = |row: &rusqlite::Row| {
                Ok(BundlePinnedProject {
                    project_key: row.get(0)?,
                    seq_id: row.get(1)?,
                    project_id: row.get(2)?,
                    repo_name: row.get(3)?,
                    repo_path: row.get(4)?,
                })
            };

            if is_def {
                if let Ok(rows) = seq_stmt.query_map([], map_fn) {
                    pinned_project_sequences.extend(rows.flatten());
                }
            } else {
                if let Ok(rows) = seq_stmt.query_map(rusqlite::params![instance_id], map_fn) {
                    pinned_project_sequences.extend(rows.flatten());
                }
            }
        }

        // Query running projects
        let proj_query = if is_def {
            "SELECT id, repo_name, repo_path, workspace_storage_path 
             FROM running_projects 
             WHERE instance_id = 'default' OR instance_id = '__default__' OR instance_id = '' OR instance_id IS NULL"
        } else {
            "SELECT id, repo_name, repo_path, workspace_storage_path 
             FROM running_projects 
             WHERE instance_id = ?1"
        };

        if let Ok(mut proj_stmt) = conn.prepare(proj_query) {
            let map_fn = |row: &rusqlite::Row| {
                Ok(BundleRunningProject {
                    id: row.get(0)?,
                    repo_name: row.get(1)?,
                    repo_path: row.get(2)?,
                    workspace_storage_path: row.get(3)?,
                })
            };

            if is_def {
                if let Ok(rows) = proj_stmt.query_map([], map_fn) {
                    running_projects.extend(rows.flatten());
                }
            } else {
                if let Ok(rows) = proj_stmt.query_map(rusqlite::params![instance_id], map_fn) {
                    running_projects.extend(rows.flatten());
                }
            }
        }
    }

    Ok(BundleProjects {
        workspace_uris,
        pinned_project_sequences,
        running_projects,
    })
}

fn package_bundle_prompts(instance_id: &str, is_default: bool) -> Result<BundlePrompts, AppError> {
    let mut active_prompts = Vec::new();
    let mut suffix_templates = Vec::new();
    let mut resend_as_templates = Vec::new();

    if let Ok(conn) = crate::modules::repo_db::connect_db() {
        let is_def = is_default || instance_id == "default";
        // Enforce FIFO invariant: ORDER BY created_at ASC, id ASC
        let prompt_query = if is_def {
            "SELECT id, repo_path, prompt_content, model, status, created_at, updated_at 
             FROM active_prompts 
             WHERE instance_id = 'default' OR instance_id = '__default__' OR instance_id = '' OR instance_id IS NULL 
             ORDER BY created_at ASC, id ASC"
        } else {
            "SELECT id, repo_path, prompt_content, model, status, created_at, updated_at 
             FROM active_prompts 
             WHERE instance_id = ?1 
             ORDER BY created_at ASC, id ASC"
        };

        if let Ok(mut prompt_stmt) = conn.prepare(prompt_query) {
            let map_fn = |row: &rusqlite::Row| {
                Ok(BundleActivePrompt {
                    id: row.get(0)?,
                    repo_path: row.get(1)?,
                    prompt_content: row.get(2)?,
                    model: row.get(3)?,
                    status: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            };

            if is_def {
                if let Ok(rows) = prompt_stmt.query_map([], map_fn) {
                    active_prompts.extend(rows.flatten());
                }
            } else {
                if let Ok(rows) = prompt_stmt.query_map(rusqlite::params![instance_id], map_fn) {
                    active_prompts.extend(rows.flatten());
                }
            }
        }
    }

    // Query suffix templates from suffix_templates module
    if let Ok(st_rows) = crate::modules::suffix_templates::list_suffix_templates() {
        for row in st_rows {
            suffix_templates.push(BundleSuffixTemplate {
                id: row.id,
                name: row.name,
                description: row.description,
                category: row.category,
                template: row.template,
                is_default: row.is_default,
                created_at: row.created_at,
                updated_at: row.updated_at,
            });
        }
    }

    if let Ok(rat_rows) = crate::modules::suffix_templates::list_resend_as_templates() {
        for row in rat_rows {
            resend_as_templates.push(BundleResendAsTemplate {
                id: row.id,
                name: row.name,
                title_pattern: row.title_pattern,
                target_mode: row.target_mode,
                suffix_id: row.suffix_id,
                is_default: row.is_default,
                created_at: row.created_at,
                updated_at: row.updated_at,
            });
        }
    }

    // Global system prompt
    let global_prompt_cfg = crate::proxy::config::get_global_system_prompt();
    let global_system_prompt =
        if global_prompt_cfg.enabled && !global_prompt_cfg.content.trim().is_empty() {
            Some(global_prompt_cfg.content)
        } else {
            None
        };

    Ok(BundlePrompts {
        active_prompts,
        suffix_templates,
        resend_as_templates,
        global_system_prompt,
    })
}
