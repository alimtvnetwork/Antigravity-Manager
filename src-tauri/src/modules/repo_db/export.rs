//! Repo DB: Scoped Prompt Tree Export Engine (JSON, SQLite DB, YAML)
//!
//! Supports 4 granular scopes (Full Tree, Project, Conversation, Prompt Subtree)
//! across 3 standard formats (JSON, SQLite DB, YAML).
//! Guarantees the Base64 Image Embedding Invariant: local images are scanned,
//! Base64-encoded, indexed into `embedded_images`, and replaced with `agm-asset://...` URIs.

use crate::error::AppError;
use crate::modules::repo_db::models::{ActivePrompt, RunningProject};
use base64::Engine;
use chrono::Utc;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub const EXPORT_SCHEMA_VERSION_2: &str = "2.0";
pub const EXPORT_KIND_SECTION: &str = "agm/prompt-tree-section-export";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSequenceRow {
    pub project_key: String,
    pub seq_id: i64,
    pub project_id: String,
    pub repo_name: String,
    pub repo_path: String,
    pub instance_id: String,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationSequenceRow {
    pub conversation_id: String,
    pub seq_id: i64,
    pub project_key: String,
    pub title: String,
    pub instance_id: String,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportScope {
    pub target_type: String, // "full_tree" | "project" | "conversation" | "prompt_subtree"
    pub project_id: Option<String>,
    pub project_name: Option<String>,
    pub conversation_id: Option<String>,
    pub prompt_ids: Option<Vec<String>>,
}

impl Default for ExportScope {
    fn default() -> Self {
        Self {
            target_type: "full_tree".to_string(),
            project_id: None,
            project_name: None,
            conversation_id: None,
            prompt_ids: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    #[serde(default = "default_true")]
    pub embed_images: bool,
    #[serde(default = "default_format")]
    pub format: String, // "json" | "sqlite" | "yaml"
}

fn default_true() -> bool {
    true
}
fn default_format() -> String {
    "json".to_string()
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            embed_images: true,
            format: "json".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportStats {
    pub path: String,
    pub project_count: usize,
    pub conversation_count: usize,
    pub prompt_count: usize,
    pub image_count: usize,
    pub total_image_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub success: bool,
    pub path: String,
    pub project_count: usize,
    pub conversation_count: usize,
    pub prompt_count: usize,
    pub image_count: usize,
    pub total_image_bytes: usize,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub projects: Vec<RunningProject>,
    pub active_prompts: Vec<ActivePrompt>,
    pub project_sequences: Vec<ProjectSequenceRow>,
    pub conversation_sequences: Vec<ConversationSequenceRow>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportEnvelope {
    pub schema_version: String,
    pub kind: String,
    pub exported_at: String,
    pub agm_version: String,
    pub scope: ExportScope,
    pub stats: ExportStats,
    pub data: ExportData,
    pub embedded_images: HashMap<String, String>, // image_key -> data:image/png;base64,...
}

/// Detects the MIME type of image data by inspecting magic header bytes or file extension.
pub fn detect_mime_type(path: &Path, bytes: &[u8]) -> &'static str {
    if bytes.len() >= 4 {
        if bytes.starts_with(&[0x89, b'P', b'N', b'G']) {
            return "image/png";
        }
        if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
            return "image/jpeg";
        }
        if bytes.starts_with(&[b'G', b'I', b'F']) {
            return "image/gif";
        }
        if bytes.starts_with(b"RIFF") && bytes.len() >= 12 && &bytes[8..12] == b"WEBP" {
            return "image/webp";
        }
        if bytes.starts_with(b"<?xml") || bytes.starts_with(b"<svg") {
            return "image/svg+xml";
        }
    }
    match path.extension().and_then(|s| s.to_str()).map(|s| s.to_lowercase()).as_deref() {
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("bmp") => "image/bmp",
        _ => "image/png",
    }
}

/// Scans all prompts in scope, locates local image paths and data URIs,
/// encodes file bytes as Base64 data URIs, stores them in `embedded_images`,
/// and normalizes prompt content references to portable `agm-asset://<key>`.
pub fn harvest_and_embed_images(
    prompts: &mut [ActivePrompt],
    embedded_images: &mut HashMap<String, String>,
) -> (usize, usize) {
    let mut total_count = 0;
    let mut total_bytes = 0;

    let img_exts = [".png", ".jpg", ".jpeg", ".webp", ".gif", ".svg", ".bmp"];

    for prompt in prompts.iter_mut() {
        let mut candidates = Vec::new();

        if let Some(ref payload) = prompt.image_payload {
            for part in payload.split(';') {
                let trimmed = part.trim();
                if !trimmed.is_empty() {
                    candidates.push(trimmed.to_string());
                }
            }
        }

        let content = &prompt.prompt_content;
        for token in content.split_whitespace() {
            let clean = token.trim_matches(|c| {
                c == '"'
                    || c == '\''
                    || c == '('
                    || c == ')'
                    || c == '['
                    || c == ']'
                    || c == '<'
                    || c == '>'
                    || c == '!'
            });
            let lower = clean.to_lowercase();
            if img_exts.iter().any(|ext| lower.ends_with(ext))
                || clean.starts_with("data:image/")
                || clean.starts_with("file://")
                || clean.contains(".antigravity/attachments")
                || clean.contains(".user_uploaded")
            {
                let normalized = clean
                    .strip_prefix("file:///")
                    .or_else(|| clean.strip_prefix("file://"))
                    .unwrap_or(clean);
                if !candidates.iter().any(|c| c == normalized || c == clean) {
                    candidates.push(normalized.to_string());
                }
            }
        }

        for candidate in candidates {
            if candidate.starts_with("agm-asset://") {
                continue;
            }

            if candidate.starts_with("data:image/") {
                let mut hasher = Sha256::new();
                hasher.update(candidate.as_bytes());
                let hash = format!("{:x}", hasher.finalize());
                let key = format!("img_{}", &hash[..12]);
                if !embedded_images.contains_key(&key) {
                    total_count += 1;
                    total_bytes += candidate.len();
                    embedded_images.insert(key.clone(), candidate.clone());
                }
                let asset_ref = format!("agm-asset://{key}");
                prompt.prompt_content = prompt.prompt_content.replace(&candidate, &asset_ref);
                if let Some(ref mut payload) = prompt.image_payload {
                    *payload = payload.replace(&candidate, &asset_ref);
                }
                continue;
            }

            let file_path = PathBuf::from(&candidate);
            let resolved_path = if file_path.is_absolute() && file_path.exists() {
                Some(file_path)
            } else {
                let in_repo = Path::new(&prompt.repo_path).join(&candidate);
                if in_repo.exists() {
                    Some(in_repo)
                } else {
                    None
                }
            };

            if let Some(p) = resolved_path {
                if let Ok(bytes) = std::fs::read(&p) {
                    let mut hasher = Sha256::new();
                    hasher.update(&bytes);
                    let hash = format!("{:x}", hasher.finalize());
                    let key = format!("img_{}", &hash[..12]);
                    let mime = detect_mime_type(&p, &bytes);
                    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                    let data_uri = format!("data:{mime};base64,{b64}");

                    if !embedded_images.contains_key(&key) {
                        total_count += 1;
                        total_bytes += bytes.len();
                        embedded_images.insert(key.clone(), data_uri);
                    }

                    let asset_uri = format!("agm-asset://{key}");
                    prompt.prompt_content = prompt.prompt_content.replace(&candidate, &asset_uri);
                    let file_uri = format!("file:///{}", candidate.replace('\\', "/"));
                    prompt.prompt_content = prompt.prompt_content.replace(&file_uri, &asset_uri);

                    if let Some(ref mut payload) = prompt.image_payload {
                        *payload = payload.replace(&candidate, &asset_uri);
                        *payload = payload.replace(&file_uri, &asset_uri);
                    } else {
                        prompt.image_payload = Some(asset_uri);
                    }
                }
            }
        }
    }

    (total_count, total_bytes)
}

/// Executes scoped SQLite queries against `repo_prompts.db` enforcing strict FIFO ordering:
/// `ORDER BY created_at ASC, id ASC`.
pub fn query_scoped_export_data(scope: &ExportScope) -> Result<ExportData, AppError> {
    let conn = super::schema::connect_db().map_err(AppError::Config)?;

    let mut projects = Vec::new();
    let mut prompts = Vec::new();
    let mut project_sequences = Vec::new();
    let mut conversation_sequences = Vec::new();

    match scope.target_type.as_str() {
        "project" => {
            let pid = scope.project_id.as_deref().unwrap_or("");
            let pname = scope.project_name.as_deref().unwrap_or("");
            let mut stmt = conn
                .prepare(
                    "SELECT id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at
                     FROM running_projects
                     WHERE id = ?1 OR repo_name = ?2 OR repo_path = ?1",
                )
                .map_err(AppError::from)?;
            let p_rows = stmt
                .query_map(params![pid, pname], |r| {
                    Ok(RunningProject {
                        id: r.get(0)?,
                        instance_id: r.get(1)?,
                        repo_name: r.get(2)?,
                        repo_path: r.get(3)?,
                        workspace_storage_path: r.get(4)?,
                        is_running: r.get::<_, i64>(5)? != 0,
                        last_detected_at: r.get(6)?,
                    })
                })
                .map_err(AppError::from)?;
            for p in p_rows.flatten() {
                projects.push(p);
            }

            let mut p_stmt = conn
                .prepare(
                    "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload
                     FROM active_prompts
                     WHERE project_id = ?1 OR project_id IN (SELECT id FROM running_projects WHERE repo_name = ?2)
                     ORDER BY created_at ASC, id ASC",
                )
                .map_err(AppError::from)?;
            let prompt_rows = p_stmt
                .query_map(params![pid, pname], |r| {
                    Ok(ActivePrompt {
                        id: r.get(0)?,
                        project_id: r.get(1)?,
                        instance_id: r.get(2)?,
                        repo_path: r.get(3)?,
                        prompt_content: r.get(4)?,
                        model: r.get(5)?,
                        session_id: r.get(6)?,
                        status: r.get(7)?,
                        created_at: r.get(8)?,
                        updated_at: r.get(9)?,
                        image_payload: r.get(10)?,
                    })
                })
                .map_err(AppError::from)?;
            for pr in prompt_rows.flatten() {
                prompts.push(pr);
            }

            let mut ps_stmt = conn
                .prepare(
                    "SELECT project_key, seq_id, project_id, repo_name, repo_path, instance_id, updated_at
                     FROM agm_project_sequences
                     WHERE project_id = ?1 OR repo_name = ?2",
                )
                .map_err(AppError::from)?;
            let ps_rows = ps_stmt
                .query_map(params![pid, pname], |r| {
                    Ok(ProjectSequenceRow {
                        project_key: r.get(0)?,
                        seq_id: r.get(1)?,
                        project_id: r.get(2)?,
                        repo_name: r.get(3)?,
                        repo_path: r.get(4)?,
                        instance_id: r.get(5)?,
                        updated_at: r.get(6)?,
                    })
                })
                .map_err(AppError::from)?;
            for ps in ps_rows.flatten() {
                project_sequences.push(ps);
            }

            let mut cs_stmt = conn
                .prepare(
                    "SELECT conversation_id, seq_id, project_key, title, instance_id, updated_at
                     FROM agm_conversation_sequences
                     WHERE project_key IN (SELECT project_key FROM agm_project_sequences WHERE project_id = ?1 OR repo_name = ?2)",
                )
                .map_err(AppError::from)?;
            let cs_rows = cs_stmt
                .query_map(params![pid, pname], |r| {
                    Ok(ConversationSequenceRow {
                        conversation_id: r.get(0)?,
                        seq_id: r.get(1)?,
                        project_key: r.get(2)?,
                        title: r.get(3)?,
                        instance_id: r.get(4)?,
                        updated_at: r.get(5)?,
                    })
                })
                .map_err(AppError::from)?;
            for cs in cs_rows.flatten() {
                conversation_sequences.push(cs);
            }
        }
        "conversation" => {
            let cid = scope.conversation_id.as_deref().unwrap_or("");
            let mut p_stmt = conn
                .prepare(
                    "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload
                     FROM active_prompts
                     WHERE session_id = ?1
                     ORDER BY created_at ASC, id ASC",
                )
                .map_err(AppError::from)?;
            let prompt_rows = p_stmt
                .query_map(params![cid], |r| {
                    Ok(ActivePrompt {
                        id: r.get(0)?,
                        project_id: r.get(1)?,
                        instance_id: r.get(2)?,
                        repo_path: r.get(3)?,
                        prompt_content: r.get(4)?,
                        model: r.get(5)?,
                        session_id: r.get(6)?,
                        status: r.get(7)?,
                        created_at: r.get(8)?,
                        updated_at: r.get(9)?,
                        image_payload: r.get(10)?,
                    })
                })
                .map_err(AppError::from)?;
            for pr in prompt_rows.flatten() {
                prompts.push(pr);
            }

            if let Some(first_prompt) = prompts.first() {
                let mut stmt = conn
                    .prepare(
                        "SELECT id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at
                         FROM running_projects
                         WHERE id = ?1 OR repo_path = ?2",
                    )
                    .map_err(AppError::from)?;
                let p_rows = stmt
                    .query_map(params![first_prompt.project_id, first_prompt.repo_path], |r| {
                        Ok(RunningProject {
                            id: r.get(0)?,
                            instance_id: r.get(1)?,
                            repo_name: r.get(2)?,
                            repo_path: r.get(3)?,
                            workspace_storage_path: r.get(4)?,
                            is_running: r.get::<_, i64>(5)? != 0,
                            last_detected_at: r.get(6)?,
                        })
                    })
                    .map_err(AppError::from)?;
                for p in p_rows.flatten() {
                    projects.push(p);
                }
            }

            let mut cs_stmt = conn
                .prepare(
                    "SELECT conversation_id, seq_id, project_key, title, instance_id, updated_at
                     FROM agm_conversation_sequences
                     WHERE conversation_id = ?1",
                )
                .map_err(AppError::from)?;
            let cs_rows = cs_stmt
                .query_map(params![cid], |r| {
                    Ok(ConversationSequenceRow {
                        conversation_id: r.get(0)?,
                        seq_id: r.get(1)?,
                        project_key: r.get(2)?,
                        title: r.get(3)?,
                        instance_id: r.get(4)?,
                        updated_at: r.get(5)?,
                    })
                })
                .map_err(AppError::from)?;
            for cs in cs_rows.flatten() {
                conversation_sequences.push(cs);
            }

            if let Some(first_cs) = conversation_sequences.first() {
                let mut ps_stmt = conn
                    .prepare(
                        "SELECT project_key, seq_id, project_id, repo_name, repo_path, instance_id, updated_at
                         FROM agm_project_sequences
                         WHERE project_key = ?1",
                    )
                    .map_err(AppError::from)?;
                let ps_rows = ps_stmt
                    .query_map(params![first_cs.project_key], |r| {
                        Ok(ProjectSequenceRow {
                            project_key: r.get(0)?,
                            seq_id: r.get(1)?,
                            project_id: r.get(2)?,
                            repo_name: r.get(3)?,
                            repo_path: r.get(4)?,
                            instance_id: r.get(5)?,
                            updated_at: r.get(6)?,
                        })
                    })
                    .map_err(AppError::from)?;
                for ps in ps_rows.flatten() {
                    project_sequences.push(ps);
                }
            }
        }
        "prompt_subtree" => {
            let pids = scope.prompt_ids.as_deref().unwrap_or(&[]);
            for pid in pids {
                let mut p_stmt = conn
                    .prepare(
                        "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload
                         FROM active_prompts
                         WHERE id = ?1
                         ORDER BY created_at ASC, id ASC",
                    )
                    .map_err(AppError::from)?;
                let prompt_rows = p_stmt
                    .query_map(params![pid], |r| {
                        Ok(ActivePrompt {
                            id: r.get(0)?,
                            project_id: r.get(1)?,
                            instance_id: r.get(2)?,
                            repo_path: r.get(3)?,
                            prompt_content: r.get(4)?,
                            model: r.get(5)?,
                            session_id: r.get(6)?,
                            status: r.get(7)?,
                            created_at: r.get(8)?,
                            updated_at: r.get(9)?,
                            image_payload: r.get(10)?,
                        })
                    })
                    .map_err(AppError::from)?;
                for pr in prompt_rows.flatten() {
                    prompts.push(pr);
                }
            }
            if let Some(first_prompt) = prompts.first() {
                let mut stmt = conn
                    .prepare(
                        "SELECT id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at
                         FROM running_projects
                         WHERE id = ?1 OR repo_path = ?2",
                    )
                    .map_err(AppError::from)?;
                let p_rows = stmt
                    .query_map(params![first_prompt.project_id, first_prompt.repo_path], |r| {
                        Ok(RunningProject {
                            id: r.get(0)?,
                            instance_id: r.get(1)?,
                            repo_name: r.get(2)?,
                            repo_path: r.get(3)?,
                            workspace_storage_path: r.get(4)?,
                            is_running: r.get::<_, i64>(5)? != 0,
                            last_detected_at: r.get(6)?,
                        })
                    })
                    .map_err(AppError::from)?;
                for p in p_rows.flatten() {
                    projects.push(p);
                }
            }
        }
        _ /* full_tree */ => {
            let mut stmt = conn
                .prepare(
                    "SELECT id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at
                     FROM running_projects",
                )
                .map_err(AppError::from)?;
            let p_rows = stmt
                .query_map([], |r| {
                    Ok(RunningProject {
                        id: r.get(0)?,
                        instance_id: r.get(1)?,
                        repo_name: r.get(2)?,
                        repo_path: r.get(3)?,
                        workspace_storage_path: r.get(4)?,
                        is_running: r.get::<_, i64>(5)? != 0,
                        last_detected_at: r.get(6)?,
                    })
                })
                .map_err(AppError::from)?;
            for p in p_rows.flatten() {
                projects.push(p);
            }

            let mut p_stmt = conn
                .prepare(
                    "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload
                     FROM active_prompts
                     ORDER BY created_at ASC, id ASC",
                )
                .map_err(AppError::from)?;
            let prompt_rows = p_stmt
                .query_map([], |r| {
                    Ok(ActivePrompt {
                        id: r.get(0)?,
                        project_id: r.get(1)?,
                        instance_id: r.get(2)?,
                        repo_path: r.get(3)?,
                        prompt_content: r.get(4)?,
                        model: r.get(5)?,
                        session_id: r.get(6)?,
                        status: r.get(7)?,
                        created_at: r.get(8)?,
                        updated_at: r.get(9)?,
                        image_payload: r.get(10)?,
                    })
                })
                .map_err(AppError::from)?;
            for pr in prompt_rows.flatten() {
                prompts.push(pr);
            }

            let mut ps_stmt = conn
                .prepare(
                    "SELECT project_key, seq_id, project_id, repo_name, repo_path, instance_id, updated_at
                     FROM agm_project_sequences",
                )
                .map_err(AppError::from)?;
            let ps_rows = ps_stmt
                .query_map([], |r| {
                    Ok(ProjectSequenceRow {
                        project_key: r.get(0)?,
                        seq_id: r.get(1)?,
                        project_id: r.get(2)?,
                        repo_name: r.get(3)?,
                        repo_path: r.get(4)?,
                        instance_id: r.get(5)?,
                        updated_at: r.get(6)?,
                    })
                })
                .map_err(AppError::from)?;
            for ps in ps_rows.flatten() {
                project_sequences.push(ps);
            }

            let mut cs_stmt = conn
                .prepare(
                    "SELECT conversation_id, seq_id, project_key, title, instance_id, updated_at
                     FROM agm_conversation_sequences",
                )
                .map_err(AppError::from)?;
            let cs_rows = cs_stmt
                .query_map([], |r| {
                    Ok(ConversationSequenceRow {
                        conversation_id: r.get(0)?,
                        seq_id: r.get(1)?,
                        project_key: r.get(2)?,
                        title: r.get(3)?,
                        instance_id: r.get(4)?,
                        updated_at: r.get(5)?,
                    })
                })
                .map_err(AppError::from)?;
            for cs in cs_rows.flatten() {
                conversation_sequences.push(cs);
            }
        }
    }

    Ok(ExportData {
        projects,
        active_prompts: prompts,
        project_sequences,
        conversation_sequences,
    })
}

/// Builds a structured two-tier export envelope (v2.0) with embedded Base64 images.
pub fn build_scoped_export_envelope(
    scope: &ExportScope,
    options: &ExportOptions,
    dest_path_label: &str,
) -> Result<ExportEnvelope, AppError> {
    let mut data = query_scoped_export_data(scope)?;

    let mut embedded_images = HashMap::new();
    let (image_count, total_image_bytes) = if options.embed_images {
        harvest_and_embed_images(&mut data.active_prompts, &mut embedded_images)
    } else {
        (0, 0)
    };

    let mut unique_convs = std::collections::HashSet::new();
    for p in &data.active_prompts {
        if let Some(ref sid) = p.session_id {
            unique_convs.insert(sid.clone());
        }
    }
    let conversation_count = unique_convs.len().max(data.conversation_sequences.len());

    let stats = ExportStats {
        path: dest_path_label.to_string(),
        project_count: data.projects.len(),
        conversation_count,
        prompt_count: data.active_prompts.len(),
        image_count,
        total_image_bytes,
    };

    Ok(ExportEnvelope {
        schema_version: EXPORT_SCHEMA_VERSION_2.to_string(),
        kind: EXPORT_KIND_SECTION.to_string(),
        exported_at: Utc::now().to_rfc3339(),
        agm_version: env!("CARGO_PKG_VERSION").to_string(),
        scope: scope.clone(),
        stats,
        data,
        embedded_images,
    })
}

fn format_yaml_scalar(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::Null => "null".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => {
            if s.contains('\n') {
                let lines: Vec<&str> = s.lines().collect();
                let mut out = "|\n".to_string();
                for line in lines {
                    out.push_str("    ");
                    out.push_str(line);
                    out.push('\n');
                }
                out
            } else if s.contains(':') || s.contains('#') || s.contains('{') || s.contains('[') || s.starts_with('"') || s.starts_with('\'') || s.is_empty() {
                format!("{:?}", s)
            } else {
                s.clone()
            }
        }
        _ => "".to_string(),
    }
}

fn format_yaml_recursive(val: &serde_json::Value, indent: usize, out: &mut String) {
    let spaces = " ".repeat(indent);
    match val {
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                if v.is_object() {
                    out.push_str(&format!("{spaces}{k}:\n"));
                    format_yaml_recursive(v, indent + 2, out);
                } else if v.is_array() {
                    out.push_str(&format!("{spaces}{k}:\n"));
                    format_yaml_recursive(v, indent + 2, out);
                } else {
                    let scalar = format_yaml_scalar(v);
                    if scalar.starts_with("|\n") {
                        out.push_str(&format!("{spaces}{k}: |\n"));
                        for line in scalar.lines().skip(1) {
                            out.push_str(&format!("{spaces}  {line}\n"));
                        }
                    } else {
                        out.push_str(&format!("{spaces}{k}: {scalar}\n"));
                    }
                }
            }
        }
        serde_json::Value::Array(arr) => {
            for item in arr {
                if item.is_object() {
                    let map = item.as_object().unwrap();
                    let mut first = true;
                    for (k, v) in map {
                        if first {
                            if v.is_object() || v.is_array() {
                                out.push_str(&format!("{spaces}- {k}:\n"));
                                format_yaml_recursive(v, indent + 4, out);
                            } else {
                                out.push_str(&format!("{spaces}- {k}: {}\n", format_yaml_scalar(v)));
                            }
                            first = false;
                        } else if v.is_object() || v.is_array() {
                            out.push_str(&format!("{spaces}  {k}:\n"));
                            format_yaml_recursive(v, indent + 4, out);
                        } else {
                            out.push_str(&format!("{spaces}  {k}: {}\n", format_yaml_scalar(v)));
                        }
                    }
                } else {
                    out.push_str(&format!("{spaces}- {}\n", format_yaml_scalar(item)));
                }
            }
        }
        _ => {
            out.push_str(&format!("{spaces}{}\n", format_yaml_scalar(val)));
        }
    }
}

pub fn serialize_to_yaml(env: &ExportEnvelope) -> Result<String, AppError> {
    let val = serde_json::to_value(env).map_err(|e| AppError::Config(format!("JSON serialization error: {e}")))?;
    let mut out = String::new();
    format_yaml_recursive(&val, 0, &mut out);
    Ok(out)
}

/// Exports prompt tree data to a file at `dest_path` in JSON, YAML, or SQLite format.
pub fn export_prompt_tree_to_file(
    scope: &ExportScope,
    options: &ExportOptions,
    dest_path: &Path,
) -> Result<ExportResult, AppError> {
    if let Some(parent) = dest_path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(AppError::from)?;
        }
    }

    let is_sqlite = options.format.eq_ignore_ascii_case("sqlite")
        || dest_path.extension().map_or(false, |ext| ext.eq_ignore_ascii_case("db") || ext.eq_ignore_ascii_case("sqlite"));
    let is_yaml = options.format.eq_ignore_ascii_case("yaml")
        || options.format.eq_ignore_ascii_case("yml")
        || dest_path.extension().map_or(false, |ext| ext.eq_ignore_ascii_case("yaml") || ext.eq_ignore_ascii_case("yml"));

    if is_sqlite {
        super::sqlite_export::write_scoped_prompt_tree_db(dest_path, scope, options)
    } else {
        let envelope = build_scoped_export_envelope(scope, options, &dest_path.to_string_lossy())?;
        if is_yaml {
            let yaml_str = serialize_to_yaml(&envelope)?;
            std::fs::write(dest_path, yaml_str.as_bytes()).map_err(AppError::from)?;
        } else {
            let json_str = serde_json::to_string_pretty(&envelope).map_err(|e| AppError::Config(e.to_string()))?;
            std::fs::write(dest_path, json_str.as_bytes()).map_err(AppError::from)?;
        }

        Ok(ExportResult {
            success: true,
            path: dest_path.to_string_lossy().to_string(),
            project_count: envelope.stats.project_count,
            conversation_count: envelope.stats.conversation_count,
            prompt_count: envelope.stats.prompt_count,
            image_count: envelope.stats.image_count,
            total_image_bytes: envelope.stats.total_image_bytes,
            message: format!(
                "Successfully exported {} project(s), {} prompt(s), {} embedded image(s) to {}",
                envelope.stats.project_count,
                envelope.stats.prompt_count,
                envelope.stats.image_count,
                dest_path.display()
            ),
        })
    }
}

/// Exports prompt tree data to in-memory bytes and Base64 string for Web downloads.
pub fn export_prompt_tree_to_bytes(
    scope: &ExportScope,
    options: &ExportOptions,
) -> Result<(String, String, ExportResult), AppError> {
    let stamp = Utc::now().timestamp_millis();
    let ext = match options.format.to_lowercase().as_str() {
        "sqlite" | "db" => "db",
        "yaml" | "yml" => "yaml",
        _ => "json",
    };
    let file_name = format!("prompt-tree-{}-{stamp}.{ext}", scope.target_type);
    let tmp = std::env::temp_dir().join(&file_name);

    let res = export_prompt_tree_to_file(scope, options, &tmp)?;
    let bytes = std::fs::read(&tmp).map_err(AppError::from)?;
    let data_b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);

    // Justification: Best-effort temp cleanup
    crate::error::record_ignored(std::fs::remove_file(&tmp), "remove temp export file");

    Ok((data_b64, file_name, res))
}
