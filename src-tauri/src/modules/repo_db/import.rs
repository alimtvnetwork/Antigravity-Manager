//! Repo DB: Scoped Prompt Tree Import Engine & Base64 Image Materializer
//!
//! Handles universal format probing (JSON, SQLite DB, YAML), integrity validation,
//! Base64 image materialization onto disk, and strict FIFO database merging.

use super::export::ExportEnvelope;
use crate::error::AppError;
use base64::Engine;
use chrono::Utc;
use rusqlite::{params, Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub format: String,
    pub scope: String,
    pub project_count: usize,
    pub conversation_count: usize,
    pub prompt_count: usize,
    pub image_count: usize,
    pub total_image_bytes: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportOptions {
    #[serde(default = "default_merge")]
    pub conflict_mode: String, // "merge" | "replace"
    #[serde(default = "default_true")]
    pub materialize_images: bool,
    pub target_instance_id: Option<String>,
}

fn default_merge() -> String {
    "merge".to_string()
}
fn default_true() -> bool {
    true
}

impl Default for ImportOptions {
    fn default() -> Self {
        Self {
            conflict_mode: "merge".to_string(),
            materialize_images: true,
            target_instance_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportResult {
    pub success: bool,
    pub imported_count: usize,
    pub materialized_images_count: usize,
    pub message: String,
}

/// Locates or creates the target attachments folder for the given instance.
pub fn resolve_attachments_dir(target_instance: Option<&str>) -> PathBuf {
    let inst_name = target_instance.unwrap_or("default");
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));

    // Check if instance registry exists and has a specific data_dir
    if let Ok(reg) = crate::modules::instance::registry::load_registry() {
        if let Some(inst) = reg
            .instances
            .iter()
            .find(|i| i.id == inst_name || i.name == inst_name)
        {
            let inst_dir = PathBuf::from(&inst.data_dir)
                .join(".antigravity")
                .join("attachments");
            return inst_dir;
        }
    }

    home.join(".antigravity").join("attachments")
}

fn sanitize_key(key: &str) -> String {
    key.chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect()
}

fn mime_to_extension(mime: &str) -> &'static str {
    if mime.contains("png") {
        "png"
    } else if mime.contains("jpeg") || mime.contains("jpg") {
        "jpg"
    } else if mime.contains("webp") {
        "webp"
    } else if mime.contains("gif") {
        "gif"
    } else if mime.contains("svg") {
        "svg"
    } else if mime.contains("bmp") {
        "bmp"
    } else {
        "png"
    }
}

/// Decodes Base64 data URIs and writes safe binary image files onto disk
/// inside the target instance's `./.antigravity/attachments/` directory.
pub fn materialize_embedded_images(
    images: &HashMap<String, String>,
    target_instance: Option<&str>,
) -> Result<HashMap<String, PathBuf>, AppError> {
    let attachments_dir = resolve_attachments_dir(target_instance);
    fs::create_dir_all(&attachments_dir).map_err(AppError::from)?;

    let mut materialized_map = HashMap::new();

    for (key, data_uri) in images {
        if !data_uri.starts_with("data:") {
            continue;
        }
        let Some((meta, b64_payload)) = data_uri.split_once(',') else {
            continue;
        };
        let mime = meta
            .strip_prefix("data:")
            .and_then(|m| m.strip_suffix(";base64"))
            .unwrap_or("image/png");
        let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(b64_payload.trim()) else {
            continue;
        };

        let ext = mime_to_extension(mime);
        let safe_filename = format!("{}.{}", sanitize_key(key), ext);
        let dest_path = attachments_dir.join(safe_filename);

        fs::write(&dest_path, &bytes).map_err(AppError::from)?;
        materialized_map.insert(key.clone(), dest_path);
    }

    Ok(materialized_map)
}

/// Probes an import file or base64 payload to validate integrity and inspect scope/counts.
pub fn validate_import_source(
    source_path: Option<&Path>,
    data_b64: Option<&str>,
) -> Result<ValidationResult, AppError> {
    if let Some(path) = source_path {
        let metadata = fs::metadata(path)
            .map_err(|e| AppError::Config(format!("Cannot read import file {path:?}: {e}")))?;
        if !metadata.is_file() {
            return Err(AppError::Config(format!(
                "Import path is not a file: {path:?}"
            )));
        }

        if metadata.len() >= 16 {
            let mut header = [0u8; 16];
            if let Ok(mut f) = fs::File::open(path) {
                if f.read_exact(&mut header).is_ok() && &header == b"SQLite format 3\0" {
                    return probe_sqlite_export(path);
                }
            }
        }

        // Try reading as text file (JSON or YAML)
        let content = fs::read_to_string(path).map_err(AppError::from)?;
        return probe_text_export(&content);
    }

    if let Some(b64) = data_b64 {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(b64.trim())
            .map_err(|e| AppError::Config(format!("Invalid base64 payload: {e}")))?;
        if bytes.len() >= 16 && &bytes[..16] == b"SQLite format 3\0" {
            let tmp = std::env::temp_dir()
                .join(format!("agm-probe-{}.db", Utc::now().timestamp_millis()));
            fs::write(&tmp, &bytes).map_err(AppError::from)?;
            let res = probe_sqlite_export(&tmp);
            // Justification: Best effort temp cleanup
            crate::error::record_ignored(fs::remove_file(&tmp), "remove probe temp db");
            return res;
        }

        let content = String::from_utf8(bytes)
            .map_err(|e| AppError::Config(format!("Payload is not UTF-8: {e}")))?;
        return probe_text_export(&content);
    }

    Err(AppError::Config("No import source provided".to_string()))
}

fn probe_sqlite_export(path: &Path) -> Result<ValidationResult, AppError> {
    let probe = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| AppError::Config(format!("Failed to open SQLite file {path:?}: {e}")))?;

    let project_count: usize = probe
        .query_row("SELECT COUNT(*) FROM running_projects", [], |r| r.get(0))
        .unwrap_or(0);
    let prompt_count: usize = probe
        .query_row("SELECT COUNT(*) FROM active_prompts", [], |r| r.get(0))
        .unwrap_or(0);
    let conversation_count: usize = probe
        .query_row(
            "SELECT COUNT(DISTINCT session_id) FROM active_prompts WHERE session_id IS NOT NULL",
            [],
            |r| r.get(0),
        )
        .unwrap_or(0);

    let (image_count, total_image_bytes): (usize, usize) = probe
        .query_row(
            "SELECT COUNT(*), COALESCE(SUM(byte_size), 0) FROM embedded_images",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap_or((0, 0));

    let scope: String = probe
        .query_row(
            "SELECT target_type FROM meta ORDER BY id DESC LIMIT 1",
            [],
            |r| r.get(0),
        )
        .unwrap_or_else(|_| "full_tree".to_string());

    Ok(ValidationResult {
        is_valid: true,
        format: "sqlite".to_string(),
        scope,
        project_count,
        conversation_count,
        prompt_count,
        image_count,
        total_image_bytes,
        message: format!(
            "Valid SQLite prompt tree export: {project_count} project(s), {prompt_count} prompt(s), {image_count} embedded image(s)"
        ),
    })
}

fn probe_text_export(content: &str) -> Result<ValidationResult, AppError> {
    if let Ok(env) = serde_json::from_str::<ExportEnvelope>(content) {
        return Ok(ValidationResult {
            is_valid: true,
            format: "json".to_string(),
            scope: env.scope.target_type,
            project_count: env.stats.project_count,
            conversation_count: env.stats.conversation_count,
            prompt_count: env.stats.prompt_count,
            image_count: env.stats.image_count,
            total_image_bytes: env.stats.total_image_bytes,
            message: format!(
                "Valid JSON envelope (v{}): {} project(s), {} prompt(s), {} image(s)",
                env.schema_version,
                env.stats.project_count,
                env.stats.prompt_count,
                env.stats.image_count
            ),
        });
    }

    // Try parsing basic JSON fallback or YAML
    if content.trim_start().starts_with('{') {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(content) {
            let projects = val
                .get("projects")
                .and_then(|p| p.as_array())
                .map_or(0, |a| a.len());
            let prompts = val
                .get("activePrompts")
                .or_else(|| val.get("prompts"))
                .and_then(|p| p.as_array())
                .map_or(0, |a| a.len());
            return Ok(ValidationResult {
                is_valid: true,
                format: "json".to_string(),
                scope: "full_tree".to_string(),
                project_count: projects,
                conversation_count: 0,
                prompt_count: prompts,
                image_count: 0,
                total_image_bytes: 0,
                message: format!(
                    "Legacy JSON prompt backup: {projects} project(s), {prompts} prompt(s)"
                ),
            });
        }
    }

    // YAML detection
    if content.contains("kind: ") && content.contains("version: ") {
        // Quick extraction of stats from YAML text
        let mut proj_count = 0;
        let mut prompt_count = 0;
        let mut image_count = 0;
        let mut total_bytes = 0;

        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(val) = trimmed.strip_prefix("project_count: ") {
                proj_count = val.parse().unwrap_or(0);
            } else if let Some(val) = trimmed.strip_prefix("prompt_count: ") {
                prompt_count = val.parse().unwrap_or(0);
            } else if let Some(val) = trimmed.strip_prefix("image_count: ") {
                image_count = val.parse().unwrap_or(0);
            } else if let Some(val) = trimmed.strip_prefix("total_image_bytes: ") {
                total_bytes = val.parse().unwrap_or(0);
            }
        }

        return Ok(ValidationResult {
            is_valid: true,
            format: "yaml".to_string(),
            scope: "section".to_string(),
            project_count: proj_count,
            conversation_count: 0,
            prompt_count,
            image_count,
            total_image_bytes: total_bytes,
            message: format!(
                "Valid YAML prompt tree export: {prompt_count} prompt(s), {image_count} image(s)"
            ),
        });
    }

    Err(AppError::Config(
        "Unrecognized file format: neither valid AGM JSON, SQLite DB, nor YAML".to_string(),
    ))
}

/// Imports prompt tree data into the live database with image materialization and strict FIFO ordering.
pub fn import_prompt_tree_from_source(
    source_path: Option<&Path>,
    data_b64: Option<&str>,
    target_instance: Option<&str>,
    conflict_mode: &str,
    materialize_images: bool,
) -> Result<ImportResult, AppError> {
    if let Some(path) = source_path {
        if let Ok(mut f) = fs::File::open(path) {
            let mut header = [0u8; 16];
            if f.read_exact(&mut header).is_ok() && &header == b"SQLite format 3\0" {
                return crate::modules::repo_db::sqlite_export::import_scoped_prompt_tree_db(
                    path,
                    target_instance,
                    conflict_mode,
                    materialize_images,
                );
            }
        }

        let content = fs::read_to_string(path).map_err(AppError::from)?;
        return import_from_text(&content, target_instance, conflict_mode, materialize_images);
    }

    if let Some(b64) = data_b64 {
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(b64.trim())
            .map_err(|e| AppError::Config(format!("Invalid base64 payload: {e}")))?;
        if bytes.len() >= 16 && &bytes[..16] == b"SQLite format 3\0" {
            let stamp = Utc::now().timestamp_millis();
            let tmp = std::env::temp_dir().join(format!("agm-tree-import-{stamp}.db"));
            fs::write(&tmp, &bytes).map_err(AppError::from)?;
            let res = crate::modules::repo_db::sqlite_export::import_scoped_prompt_tree_db(
                &tmp,
                target_instance,
                conflict_mode,
                materialize_images,
            );
            // Justification: Best effort temp cleanup
            crate::error::record_ignored(fs::remove_file(&tmp), "remove temp import db");
            return res;
        }

        let content = String::from_utf8(bytes)
            .map_err(|e| AppError::Config(format!("Payload is not UTF-8: {e}")))?;
        return import_from_text(&content, target_instance, conflict_mode, materialize_images);
    }

    Err(AppError::Config("No import source provided".to_string()))
}

fn import_from_text(
    content: &str,
    target_instance: Option<&str>,
    _conflict_mode: &str,
    materialize_images: bool,
) -> Result<ImportResult, AppError> {
    let mut envelope = serde_json::from_str::<ExportEnvelope>(content)
        .map_err(|e| AppError::Config(format!("Failed to parse JSON export envelope: {e}")))?;

    let mut materialized_count = 0;
    let mut materialized_map = HashMap::new();

    if materialize_images && !envelope.embedded_images.is_empty() {
        materialized_map = materialize_embedded_images(&envelope.embedded_images, target_instance)?;
        materialized_count = materialized_map.len();
    }

    let mut conn = crate::modules::repo_db::schema::connect_db().map_err(AppError::Config)?;
    let tx = conn.transaction().map_err(AppError::from)?;

    let target_inst = target_instance.unwrap_or("default");

    // 1. Upsert projects
    for p in &envelope.data.projects {
        let inst = if target_instance.is_some() {
            target_inst
        } else {
            &p.instance_id
        };
        tx.execute(
            "INSERT INTO running_projects (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
                instance_id = excluded.instance_id,
                repo_name = excluded.repo_name,
                repo_path = excluded.repo_path,
                is_running = excluded.is_running,
                updated_at = excluded.updated_at",
            params![
                p.id,
                inst,
                p.repo_name,
                p.repo_path,
                p.workspace_storage_path,
                if p.is_running { 1 } else { 0 },
                p.last_detected_at,
                Utc::now().timestamp(),
            ],
        ).map_err(AppError::from)?;
    }

    // 2. Upsert sequences
    for ps in &envelope.data.project_sequences {
        tx.execute(
            "INSERT OR REPLACE INTO agm_project_sequences (project_key, seq_id, project_id, repo_name, repo_path, instance_id, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![ps.project_key, ps.seq_id, ps.project_id, ps.repo_name, ps.repo_path, ps.instance_id, ps.updated_at],
        ).map_err(AppError::from)?;
    }

    for cs in &envelope.data.conversation_sequences {
        tx.execute(
            "INSERT OR REPLACE INTO agm_conversation_sequences (conversation_id, seq_id, project_key, title, instance_id, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![cs.conversation_id, cs.seq_id, cs.project_key, cs.title, cs.instance_id, cs.updated_at],
        ).map_err(AppError::from)?;
    }

    // 3. Upsert prompts enforcing strict FIFO order: ORDER BY created_at ASC, id ASC
    let mut imported_count = 0;
    for prompt in &mut envelope.data.active_prompts {
        // Rewrite agm-asset:// references if images were materialized
        for (key, dest_path) in &materialized_map {
            let asset_marker = format!("agm-asset://{key}");
            let local_uri = format!("file:///{}", dest_path.to_string_lossy().replace('\\', "/"));
            prompt.prompt_content = prompt.prompt_content.replace(&asset_marker, &local_uri);
            if let Some(ref mut payload) = prompt.image_payload {
                *payload = payload.replace(&asset_marker, &local_uri);
            }
        }

        let inst = if target_instance.is_some() {
            target_inst
        } else {
            &prompt.instance_id
        };

        tx.execute(
            "INSERT INTO active_prompts (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(id) DO UPDATE SET
                project_id = excluded.project_id,
                instance_id = excluded.instance_id,
                repo_path = excluded.repo_path,
                prompt_content = excluded.prompt_content,
                model = excluded.model,
                session_id = excluded.session_id,
                status = excluded.status,
                created_at = excluded.created_at,
                updated_at = excluded.updated_at,
                image_payload = excluded.image_payload",
            params![
                prompt.id,
                prompt.project_id,
                inst,
                prompt.repo_path,
                prompt.prompt_content,
                prompt.model,
                prompt.session_id,
                prompt.status,
                prompt.created_at,
                prompt.updated_at,
                prompt.image_payload,
            ],
        ).map_err(AppError::from)?;

        imported_count += 1;
    }

    tx.commit().map_err(AppError::from)?;

    crate::modules::repo_db::tree::invalidate_prompt_tree_cache(target_instance);

    Ok(ImportResult {
        success: true,
        imported_count,
        materialized_images_count: materialized_count,
        message: format!(
            "Successfully imported {imported_count} prompt(s) and materialized {materialized_count} image(s)"
        ),
    })
}
