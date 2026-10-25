//! IDE Profile Bundle Import Engine (`.agmide` / `.json`).
//!
//! Conforms to `02-spec/21-app/226-antigravity-manager-clone-and-ui-enhancements/03-migration-and-verification-spec.md`.

use super::bundle_types::*;
use crate::error::AppError;
use std::fs;
use std::path::{Path, PathBuf};

/// Validates, unpacks, and restores an `.agmide` or `.json` profile bundle.
pub fn import_ide_profile_bundle(
    bundle_path: &Path,
    options: ImportOptions,
) -> Result<ImportResult, AppError> {
    let path_str = bundle_path.to_string_lossy();
    let content = if bundle_path.is_file() {
        fs::read_to_string(bundle_path)
            .map_err(|e| AppError::Config(format!("Failed to read bundle file: {e}")))?
    } else if path_str.trim_start().starts_with('{') {
        path_str.to_string()
    } else {
        return Err(AppError::Config(format!(
            "Bundle file not found: {}",
            bundle_path.display()
        )));
    };

    let bundle: AgmIdeProfileBundle = serde_json::from_str(&content)
        .map_err(|e| AppError::Config(format!("Invalid bundle format: {e}")))?;

    if bundle.agm_bundle_version != AGM_IDE_BUNDLE_SCHEMA_VERSION {
        return Err(AppError::Config(format!(
            "Unsupported bundle schema version: '{}' (expected '{}')",
            bundle.agm_bundle_version, AGM_IDE_BUNDLE_SCHEMA_VERSION
        )));
    }

    // Determine target instance ID and display name
    let (target_id, display_name) = if let Some(ref new_name) = options.new_instance_name {
        let clean_name = new_name.trim();
        if clean_name.is_empty() {
            return Err(AppError::Config(
                "New instance name cannot be empty".to_string(),
            ));
        }
        let cfg = crate::modules::instance::create_instance(clean_name.to_string())
            .map_err(|e| AppError::Config(format!("Failed to create instance: {e}")))?;
        (cfg.id, cfg.name)
    } else if let Some(ref tid) = options.target_instance_id {
        let resolved = crate::modules::instance::resolve_instance_id(tid)
            .map_err(|e| AppError::Config(format!("Failed to resolve target instance: {e}")))?;
        let registry = crate::modules::instance::load_registry()
            .map_err(|e| AppError::Config(format!("Failed to load registry: {e}")))?;
        let inst = registry
            .instances
            .iter()
            .find(|i| i.id == resolved)
            .ok_or_else(|| AppError::Config(format!("Target instance '{resolved}' not found")))?;
        (inst.id.clone(), inst.name.clone())
    } else {
        return Err(AppError::Config(
            "Either target_instance_id or new_instance_name must be specified".to_string(),
        ));
    };

    // 1. Apply Settings, Keybindings, Snippets
    let settings_applied = apply_bundle_settings(&target_id, &bundle.settings)?;

    // 2. Restore Projects & Sequences
    let projects_restored = apply_bundle_projects(&target_id, &bundle.projects)?;

    // 3. Restore Prompts (strict FIFO, status: QUEUED)
    let prompts_queued = apply_bundle_prompts(&target_id, &bundle.prompts)?;

    Ok(ImportResult {
        instance_id: target_id,
        instance_name: display_name,
        settings_applied,
        projects_restored,
        prompts_queued,
    })
}

fn apply_bundle_settings(instance_id: &str, settings: &BundleSettings) -> Result<usize, AppError> {
    let registry = crate::modules::instance::load_registry()
        .map_err(|e| AppError::Config(format!("Failed to load registry: {e}")))?;
    let inst = registry
        .instances
        .iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| AppError::Config(format!("Instance '{instance_id}' not found")))?;

    let mut applied_count = 0usize;

    // Locate target settings.json path
    let settings_path =
        crate::modules::instance::find_instance_settings_path(inst).unwrap_or_else(|| {
            PathBuf::from(&inst.data_dir)
                .join("User")
                .join("settings.json")
        });

    if let Some(parent) = settings_path.parent() {
        fs::create_dir_all(parent).map_err(AppError::Io)?;

        // 1. Deep merge editor settings
        if settings.editor_settings.is_object() {
            let mut current: serde_json::Value = if settings_path.is_file() {
                fs::read_to_string(&settings_path)
                    .ok()
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_else(|| serde_json::json!({}))
            } else {
                serde_json::json!({})
            };

            if !current.is_object() {
                current = serde_json::json!({});
            }

            crate::modules::instance::deep_merge_json(&mut current, &settings.editor_settings);

            let pretty = serde_json::to_string_pretty(&current)
                .map_err(|e| AppError::Config(format!("Serialization error: {e}")))?;
            fs::write(&settings_path, pretty).map_err(AppError::Io)?;

            if let serde_json::Value::Object(ref m) = settings.editor_settings {
                applied_count += m.len();
            }
        }

        // 2. Merge keybindings
        if !settings.keybindings.is_empty() {
            let kb_path = parent.join("keybindings.json");
            let mut current_kb: Vec<serde_json::Value> = if kb_path.is_file() {
                fs::read_to_string(&kb_path)
                    .ok()
                    .and_then(|s| serde_json::from_str(&s).ok())
                    .unwrap_or_default()
            } else {
                Vec::new()
            };

            for new_kb in &settings.keybindings {
                let is_duplicate = current_kb.iter().any(|existing| {
                    existing.get("key") == new_kb.get("key")
                        && existing.get("command") == new_kb.get("command")
                });
                if !is_duplicate {
                    current_kb.push(new_kb.clone());
                    applied_count += 1;
                }
            }

            if let Ok(pretty) = serde_json::to_string_pretty(&current_kb) {
                // Justification: Best-effort write of auxiliary keybindings
                crate::error::record_ignored(
                    fs::write(&kb_path, pretty),
                    "write User/keybindings.json",
                );
            }
        }

        // 3. Write snippets
        if !settings.snippets.is_empty() {
            let snippets_dir = parent.join("snippets");
            let _ = fs::create_dir_all(&snippets_dir);
            for (filename, content) in &settings.snippets {
                let file_path = snippets_dir.join(filename);
                // Justification: Best-effort write of snippet file
                crate::error::record_ignored(
                    fs::write(&file_path, content),
                    "write User/snippets file",
                );
                applied_count += 1;
            }
        }
    }

    Ok(applied_count)
}

fn apply_bundle_projects(target_id: &str, projects: &BundleProjects) -> Result<usize, AppError> {
    let conn = crate::modules::repo_db::connect_db()
        .map_err(|e| AppError::Config(format!("Failed to connect to repo database: {e}")))?;

    let mut restored_count = 0usize;
    let now = chrono::Utc::now().timestamp();

    // 1. Restore running projects
    for rp in &projects.running_projects {
        let res = conn.execute(
            "INSERT OR REPLACE INTO running_projects 
             (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, ?7)",
            rusqlite::params![
                rp.id,
                target_id,
                rp.repo_name,
                rp.repo_path,
                rp.workspace_storage_path,
                now,
                now
            ],
        );
        if res.is_ok() {
            restored_count += 1;
        }
    }

    // 2. Restore pinned project sequences
    for pp in &projects.pinned_project_sequences {
        let res = conn.execute(
            "INSERT OR REPLACE INTO agm_project_sequences 
             (project_key, seq_id, project_id, repo_name, repo_path, instance_id, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![
                pp.project_key,
                pp.seq_id,
                pp.project_id,
                pp.repo_name,
                pp.repo_path,
                target_id,
                now
            ],
        );
        if res.is_ok() {
            restored_count += 1;
        }
    }

    Ok(restored_count)
}

fn apply_bundle_prompts(target_id: &str, prompts: &BundlePrompts) -> Result<usize, AppError> {
    let conn = crate::modules::repo_db::connect_db()
        .map_err(|e| AppError::Config(format!("Failed to connect to repo database: {e}")))?;

    let mut queued_count = 0usize;
    let now = chrono::Utc::now().timestamp();

    // Fallback project ID for prompt association
    let default_project_id = conn
        .query_row(
            "SELECT id FROM running_projects WHERE instance_id = ?1 LIMIT 1",
            rusqlite::params![target_id],
            |row| row.get::<_, String>(0),
        )
        .unwrap_or_else(|_| {
            let synth_id = format!("proj-{}", &uuid::Uuid::new_v4().to_string()[..8]);
            // Justification: Best-effort synthetic running project insertion for orphan prompt containment
            crate::error::record_ignored(
                conn.execute(
                    "INSERT OR IGNORE INTO running_projects 
                     (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
                     VALUES (?1, ?2, 'Default', '', NULL, 0, ?3, ?4)",
                    rusqlite::params![synth_id, target_id, now, now],
                ),
                "insert synthetic project for bundle prompts",
            );
            synth_id
        });

    // 1. Restore Active Prompts in strict FIFO order (status: 'QUEUED')
    for ap in &prompts.active_prompts {
        let prompt_id = format!("prompt-{}", uuid::Uuid::new_v4());
        let res = conn.execute(
            "INSERT INTO active_prompts 
             (id, project_id, instance_id, repo_path, prompt_content, model, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'QUEUED', ?7, ?8)",
            rusqlite::params![
                prompt_id,
                default_project_id,
                target_id,
                ap.repo_path,
                ap.prompt_content,
                ap.model,
                ap.created_at,
                now
            ],
        );
        if res.is_ok() {
            queued_count += 1;
        }
    }

    // 2. Restore Suffix Templates
    for st in &prompts.suffix_templates {
        let input = crate::modules::suffix_templates::CreateSuffixTemplateInput {
            id: st.id.clone(),
            name: st.name.clone(),
            description: st.description.clone(),
            category: Some(st.category.clone()),
            template: st.template.clone(),
            is_default: Some(st.is_default),
        };
        // Justification: Suffix template restore is best-effort idempotent persistence
        crate::error::record_ignored(
            crate::modules::suffix_templates::save_suffix_template(&input),
            "save suffix template during bundle import",
        );
    }

    // 3. Restore Resend-As Templates
    for rat in &prompts.resend_as_templates {
        let input = crate::modules::suffix_templates::CreateResendAsTemplateInput {
            id: rat.id.clone(),
            name: rat.name.clone(),
            title_pattern: rat.title_pattern.clone(),
            target_mode: Some(rat.target_mode.clone()),
            suffix_id: rat.suffix_id.clone(),
            is_default: Some(rat.is_default),
        };
        // Justification: Resend As template restore is best-effort idempotent persistence
        crate::error::record_ignored(
            crate::modules::suffix_templates::save_resend_as_template(&input),
            "save resend-as template during bundle import",
        );
    }

    // 4. Update global system prompt if provided
    if let Some(ref gsp) = prompts.global_system_prompt {
        let mut cfg = crate::proxy::config::get_global_system_prompt();
        cfg.content = gsp.clone();
        cfg.enabled = true;
        crate::proxy::config::update_global_system_prompt_config(cfg);
    }

    Ok(queued_count)
}
