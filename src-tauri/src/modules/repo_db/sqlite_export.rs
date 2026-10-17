//! Repo DB: portable SQLite export/import of the full prompt tree.
//!
//! Unlike `compact_engine::export_to_sqlite_file` (which serializes a
//! `CompactExportData` compaction snapshot), this module snapshots the live
//! prompt-tree tables (`running_projects`, `active_prompts`,
//! `agm_project_sequences`, `agm_conversation_sequences`, `embedded_images`) into a single
//! portable `.db` file — prompts, tree structure, metadata, and Base64 images — and can
//! validate and merge such a file back into the live `repo_prompts.db`.

#[path = "export.rs"]
pub mod export;
#[path = "import.rs"]
pub mod import;

pub use export::{
    build_scoped_export_envelope, export_prompt_tree_to_bytes, export_prompt_tree_to_file,
    harvest_and_embed_images, ExportData, ExportEnvelope, ExportOptions, ExportResult, ExportScope,
    ExportStats, EXPORT_KIND_SECTION, EXPORT_SCHEMA_VERSION_2,
};
pub use import::{
    import_prompt_tree_from_source, materialize_embedded_images, validate_import_source,
    ImportOptions, ImportResult, ValidationResult,
};

use crate::error::AppError;
use base64::Engine;
use rusqlite::{params, Connection, OpenFlags, OptionalExtension};
use serde::Serialize;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

/// Exported database kind marker stored in the `meta` table.
pub const TREE_EXPORT_KIND: &str = "agm/prompt-tree-export";
/// Schema version of the exported tree database format.
pub const TREE_EXPORT_SCHEMA_VERSION: &str = "1.0";

/// Summary of a prompt-tree SQLite export, returned to the frontend so the
/// save confirmation can report real numbers instead of a bare path.
#[derive(Debug, Clone, Serialize)]
pub struct TreeExportStats {
    pub path: String,
    pub project_count: usize,
    pub prompt_count: usize,
    pub sequence_count: usize,
}

/// Payload for the web (non-Tauri) export path: base64-encoded `.db` bytes so
/// the browser can trigger a plain blob download.
#[derive(Debug, Clone, Serialize)]
pub struct TreeExportPayload {
    pub data_b64: String,
    pub file_name: String,
    pub stats: TreeExportStats,
}

const RP_COLS: &str =
    "id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at";
const AP_COLS: &str =
    "id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload";
const PS_COLS: &str =
    "project_key, seq_id, project_id, repo_name, repo_path, instance_id, updated_at";
const CS_COLS: &str = "conversation_id, seq_id, project_key, title, instance_id, updated_at";

const REQUIRED_TABLES: &[&str] = &[
    "running_projects",
    "active_prompts",
    "agm_project_sequences",
    "agm_conversation_sequences",
];

/// Creates the portable export schema on an already-open destination database.
fn init_tree_export_schema(conn: &Connection) -> Result<(), AppError> {
    // Single-file export: DELETE journal mode avoids stray `-wal`/`-shm` sidecars.
    conn.pragma_update(None, "journal_mode", "DELETE")?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS meta (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            schema_version TEXT NOT NULL,
            kind TEXT NOT NULL,
            target_type TEXT NOT NULL DEFAULT 'full_tree',
            target_id TEXT,
            created_at TEXT NOT NULL,
            agm_version TEXT NOT NULL,
            notes TEXT
        );
        CREATE TABLE IF NOT EXISTS running_projects (
            id TEXT PRIMARY KEY,
            instance_id TEXT NOT NULL,
            repo_name TEXT NOT NULL,
            repo_path TEXT NOT NULL,
            workspace_storage_path TEXT,
            is_running INTEGER NOT NULL DEFAULT 1,
            last_detected_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS active_prompts (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            instance_id TEXT NOT NULL,
            repo_path TEXT NOT NULL,
            prompt_content TEXT NOT NULL,
            model TEXT,
            session_id TEXT,
            status TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            image_payload TEXT
        );
        CREATE TABLE IF NOT EXISTS agm_project_sequences (
            project_key TEXT PRIMARY KEY,
            seq_id INTEGER NOT NULL UNIQUE,
            project_id TEXT NOT NULL,
            repo_name TEXT NOT NULL,
            repo_path TEXT NOT NULL,
            instance_id TEXT NOT NULL DEFAULT 'default',
            updated_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS agm_conversation_sequences (
            conversation_id TEXT PRIMARY KEY,
            seq_id INTEGER NOT NULL UNIQUE,
            project_key TEXT NOT NULL,
            title TEXT NOT NULL,
            instance_id TEXT NOT NULL DEFAULT 'default',
            updated_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS embedded_images (
            image_key TEXT PRIMARY KEY,
            mime_type TEXT NOT NULL,
            base64_data TEXT NOT NULL,
            byte_size INTEGER NOT NULL,
            original_rel_path TEXT,
            created_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS idx_export_prompts_instance_status
            ON active_prompts(instance_id, status);",
    )
    .map_err(AppError::from)?;
    Ok(())
}

fn now_iso() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{secs}")
}

/// Writes the full live prompt tree into a portable SQLite file at `path`.
pub fn write_prompt_tree_db(path: &Path) -> Result<TreeExportStats, AppError> {
    let scope = ExportScope::default();
    let options = ExportOptions::default();
    let res = write_scoped_prompt_tree_db(path, &scope, &options)?;
    Ok(TreeExportStats {
        path: res.path,
        project_count: res.project_count,
        prompt_count: res.prompt_count,
        sequence_count: res.conversation_count,
    })
}

/// Writes a scoped prompt tree into a portable SQLite file at `path`,
/// optionally harvesting local images and storing them in `embedded_images`.
pub fn write_scoped_prompt_tree_db(
    path: &Path,
    scope: &ExportScope,
    options: &ExportOptions,
) -> Result<ExportResult, AppError> {
    let live_guard = super::schema::connect_db().map_err(AppError::Config)?;
    drop(live_guard);
    let src_db_path = super::schema::get_repo_db_path().map_err(AppError::Config)?;

    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(AppError::from)?;
        }
    }
    if path.exists() {
        fs::remove_file(path).map_err(AppError::from)?;
    }

    let dst = Connection::open(path)
        .map_err(|e| AppError::Config(format!("Failed to create export database {path:?}: {e}")))?;
    init_tree_export_schema(&dst)?;

    let src_str = src_db_path.to_string_lossy().to_string();
    dst.execute("ATTACH DATABASE ?1 AS srcdb", params![src_str])
        .map_err(AppError::from)?;

    let tx = dst.unchecked_transaction().map_err(AppError::from)?;

    let (project_count, prompt_count, _ps_count, cs_count) = match scope.target_type.as_str() {
        "project" => {
            let pid = scope.project_id.as_deref().unwrap_or("");
            let pname = scope.project_name.as_deref().unwrap_or("");
            let prj = tx.execute(
                &format!("INSERT INTO running_projects ({RP_COLS}) SELECT {RP_COLS} FROM srcdb.running_projects WHERE id = ?1 OR repo_name = ?2 OR repo_path = ?1"),
                params![pid, pname],
            ).map_err(AppError::from)? as usize;
            let prm = tx.execute(
                &format!("INSERT INTO active_prompts ({AP_COLS}) SELECT {AP_COLS} FROM srcdb.active_prompts WHERE project_id = ?1 OR project_id IN (SELECT id FROM srcdb.running_projects WHERE repo_name = ?2) ORDER BY created_at ASC, id ASC"),
                params![pid, pname],
            ).map_err(AppError::from)? as usize;
            let ps = tx.execute(
                &format!("INSERT INTO agm_project_sequences ({PS_COLS}) SELECT {PS_COLS} FROM srcdb.agm_project_sequences WHERE project_id = ?1 OR repo_name = ?2"),
                params![pid, pname],
            ).map_err(AppError::from)? as usize;
            let cs = tx.execute(
                &format!("INSERT INTO agm_conversation_sequences ({CS_COLS}) SELECT {CS_COLS} FROM srcdb.agm_conversation_sequences WHERE project_key IN (SELECT project_key FROM srcdb.agm_project_sequences WHERE project_id = ?1 OR repo_name = ?2)"),
                params![pid, pname],
            ).map_err(AppError::from)? as usize;
            (prj, prm, ps, cs)
        }
        "conversation" => {
            let cid = scope.conversation_id.as_deref().unwrap_or("");
            let prm = tx.execute(
                &format!("INSERT INTO active_prompts ({AP_COLS}) SELECT {AP_COLS} FROM srcdb.active_prompts WHERE session_id = ?1 ORDER BY created_at ASC, id ASC"),
                params![cid],
            ).map_err(AppError::from)? as usize;
            let prj = tx.execute(
                &format!("INSERT INTO running_projects ({RP_COLS}) SELECT {RP_COLS} FROM srcdb.running_projects WHERE id IN (SELECT project_id FROM srcdb.active_prompts WHERE session_id = ?1)"),
                params![cid],
            ).map_err(AppError::from)? as usize;
            let cs = tx.execute(
                &format!("INSERT INTO agm_conversation_sequences ({CS_COLS}) SELECT {CS_COLS} FROM srcdb.agm_conversation_sequences WHERE conversation_id = ?1"),
                params![cid],
            ).map_err(AppError::from)? as usize;
            let ps = tx.execute(
                &format!("INSERT INTO agm_project_sequences ({PS_COLS}) SELECT {PS_COLS} FROM srcdb.agm_project_sequences WHERE project_key IN (SELECT project_key FROM srcdb.agm_conversation_sequences WHERE conversation_id = ?1)"),
                params![cid],
            ).map_err(AppError::from)? as usize;
            (prj, prm, ps, cs)
        }
        "prompt_subtree" => {
            let pids = scope.prompt_ids.as_deref().unwrap_or(&[]);
            let mut prm = 0;
            for pid in pids {
                prm += tx.execute(
                    &format!("INSERT INTO active_prompts ({AP_COLS}) SELECT {AP_COLS} FROM srcdb.active_prompts WHERE id = ?1 ORDER BY created_at ASC, id ASC"),
                    params![pid],
                ).map_err(AppError::from)? as usize;
            }
            let prj = tx.execute(
                &format!("INSERT INTO running_projects ({RP_COLS}) SELECT {RP_COLS} FROM srcdb.running_projects WHERE id IN (SELECT project_id FROM active_prompts)"),
                [],
            ).map_err(AppError::from)? as usize;
            (prj, prm, 0, 0)
        }
        _ /* full_tree */ => {
            let prj = tx.execute(
                &format!("INSERT INTO running_projects ({RP_COLS}) SELECT {RP_COLS} FROM srcdb.running_projects"),
                [],
            ).map_err(AppError::from)? as usize;
            let prm = tx.execute(
                &format!("INSERT INTO active_prompts ({AP_COLS}) SELECT {AP_COLS} FROM srcdb.active_prompts ORDER BY created_at ASC, id ASC"),
                [],
            ).map_err(AppError::from)? as usize;
            let ps = tx.execute(
                &format!("INSERT INTO agm_project_sequences ({PS_COLS}) SELECT {PS_COLS} FROM srcdb.agm_project_sequences"),
                [],
            ).map_err(AppError::from)? as usize;
            let cs = tx.execute(
                &format!("INSERT INTO agm_conversation_sequences ({CS_COLS}) SELECT {CS_COLS} FROM srcdb.agm_conversation_sequences"),
                [],
            ).map_err(AppError::from)? as usize;
            (prj, prm, ps, cs)
        }
    };

    // Base64 Image Embedding into embedded_images table
    let mut image_count = 0;
    let mut total_image_bytes = 0;
    if options.embed_images {
        let mut prompts_to_scan = Vec::new();
        {
            let mut stmt = tx.prepare("SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload FROM active_prompts").map_err(AppError::from)?;
            let rows = stmt
                .query_map([], |r| {
                    Ok(super::models::ActivePrompt {
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
            for p in rows.flatten() {
                prompts_to_scan.push(p);
            }
        }

        let mut embedded_map = std::collections::HashMap::new();
        let (cnt, bytes) = harvest_and_embed_images(&mut prompts_to_scan, &mut embedded_map);
        image_count = cnt;
        total_image_bytes = bytes;

        let now_sec = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);
        for (key, data_uri) in &embedded_map {
            let mime = data_uri
                .split(';')
                .next()
                .and_then(|s| s.strip_prefix("data:"))
                .unwrap_or("image/png");
            tx.execute(
                "INSERT OR REPLACE INTO embedded_images (image_key, mime_type, base64_data, byte_size, original_rel_path, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![key, mime, data_uri, data_uri.len() as i64, "", now_sec],
            ).map_err(AppError::from)?;
        }

        for p in prompts_to_scan {
            tx.execute(
                "UPDATE active_prompts SET prompt_content = ?1, image_payload = ?2 WHERE id = ?3",
                params![p.prompt_content, p.image_payload, p.id],
            )
            .map_err(AppError::from)?;
        }
    }

    let target_id = scope
        .project_id
        .as_deref()
        .or(scope.conversation_id.as_deref());
    tx.execute(
        "INSERT INTO meta (schema_version, kind, target_type, target_id, created_at, agm_version, notes)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            EXPORT_SCHEMA_VERSION_2,
            EXPORT_KIND_SECTION,
            scope.target_type,
            target_id,
            now_iso(),
            env!("CARGO_PKG_VERSION"),
            format!(
                "Scoped export ({}): {project_count} project(s), {prompt_count} prompt(s), {image_count} image(s)",
                scope.target_type
            ),
        ],
    ).map_err(AppError::from)?;

    tx.commit().map_err(AppError::from)?;
    dst.execute("DETACH DATABASE srcdb", [])
        .map_err(AppError::from)?;

    Ok(ExportResult {
        success: true,
        path: path.to_string_lossy().to_string(),
        project_count,
        conversation_count: cs_count,
        prompt_count,
        image_count,
        total_image_bytes,
        message: format!(
            "Exported {project_count} project(s), {prompt_count} prompt(s), {image_count} image(s) to SQLite"
        ),
    })
}

/// Validates that `path` points at a plausible SQLite database file and
/// returns its table names. Real errors (not a DB, wrong kind) are reported
/// to the caller — never silently ignored.
pub fn probe_export_file(path: &Path) -> Result<Vec<String>, AppError> {
    let metadata = fs::metadata(path)
        .map_err(|e| AppError::Config(format!("Cannot read import file {path:?}: {e}")))?;
    if !metadata.is_file() {
        return Err(AppError::Config(format!(
            "Import path is not a regular file: {path:?}"
        )));
    }
    if metadata.len() < 100 {
        return Err(AppError::Config(format!(
            "File {path:?} is too small ({} bytes) to be a SQLite database",
            metadata.len()
        )));
    }

    let mut header = [0u8; 16];
    fs::File::open(path)
        .map_err(AppError::from)?
        .read_exact(&mut header)
        .map_err(|e| AppError::Config(format!("Failed to read file header {path:?}: {e}")))?;
    if &header != b"SQLite format 3\0" {
        return Err(AppError::Config(format!(
            "File {path:?} is not a SQLite database (bad magic header)"
        )));
    }

    let probe = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| AppError::Config(format!("Failed to open SQLite file {path:?}: {e}")))?;
    let tables: Vec<String> = probe
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table'")
        .map_err(AppError::from)?
        .query_map([], |row| row.get(0))
        .map_err(AppError::from)?
        .collect::<Result<Vec<String>, _>>()
        .map_err(AppError::from)?;

    for required in REQUIRED_TABLES {
        if !tables.iter().any(|t| t == required) {
            return Err(AppError::Config(format!(
                "File {path:?} is not an AGM prompt-tree export: missing table '{required}'"
            )));
        }
    }

    // When a meta marker exists, enforce export kind so random databases are rejected.
    if tables.iter().any(|t| t == "meta") {
        let kind: Option<String> = probe
            .query_row(
                "SELECT kind FROM meta ORDER BY id DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()
            .map_err(AppError::from)?
            .flatten();
        if let Some(k) = kind {
            if k != TREE_EXPORT_KIND && k != EXPORT_KIND_SECTION {
                return Err(AppError::Config(format!(
                    "File {path:?} is a SQLite database but not an AGM prompt-tree export (kind '{k}')"
                )));
            }
        }
    }

    Ok(tables)
}

/// Imports a portable prompt-tree `.db` export into the live database.
pub fn import_prompt_tree_db(path: &Path) -> Result<usize, AppError> {
    let res = import_scoped_prompt_tree_db(path, None, "merge", true)?;
    Ok(res.imported_count)
}

/// Imports a portable SQLite `.db` export with Base64 image materialization into the live database.
pub fn import_scoped_prompt_tree_db(
    path: &Path,
    target_instance: Option<&str>,
    _conflict_mode: &str,
    materialize_images: bool,
) -> Result<ImportResult, AppError> {
    probe_export_file(path)?;

    let mut materialized_count = 0;
    let mut materialized_map = std::collections::HashMap::new();

    if materialize_images {
        let probe = Connection::open_with_flags(path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| AppError::Config(format!("Failed to open SQLite file {path:?}: {e}")))?;

        let has_images_table: bool = probe
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'embedded_images'",
                [],
                |r| r.get::<_, i64>(0),
            )
            .map(|c| c > 0)
            .unwrap_or(false);

        if has_images_table {
            let mut stmt = probe
                .prepare("SELECT image_key, base64_data FROM embedded_images")
                .map_err(AppError::from)?;
            let rows = stmt
                .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
                .map_err(AppError::from)?;
            let mut img_map = std::collections::HashMap::new();
            for item in rows.flatten() {
                img_map.insert(item.0, item.1);
            }
            if !img_map.is_empty() {
                materialized_map = import::materialize_embedded_images(&img_map, target_instance)?;
                materialized_count = materialized_map.len();
            }
        }
    }

    let mut live = super::schema::connect_db().map_err(AppError::Config)?;
    let src_str = path.to_string_lossy().to_string();
    live.execute("ATTACH DATABASE ?1 AS importdb", params![src_str])
        .map_err(AppError::from)?;

    let tx = live.transaction().map_err(AppError::from)?;

    tx.execute(
        &format!("INSERT OR REPLACE INTO running_projects ({RP_COLS}) SELECT {RP_COLS} FROM importdb.running_projects"),
        [],
    )
    .map_err(AppError::from)?;

    tx.execute(
        &format!("INSERT OR REPLACE INTO agm_project_sequences ({PS_COLS}) SELECT {PS_COLS} FROM importdb.agm_project_sequences"),
        [],
    )
    .map_err(AppError::from)?;

    tx.execute(
        &format!("INSERT OR REPLACE INTO agm_conversation_sequences ({CS_COLS}) SELECT {CS_COLS} FROM importdb.agm_conversation_sequences"),
        [],
    )
    .map_err(AppError::from)?;

    // FIFO merge order per house rule: ORDER BY created_at ASC, id ASC.
    let restored = tx
        .execute(
            &format!(
                "INSERT INTO active_prompts ({AP_COLS}) \
                 SELECT {AP_COLS} FROM importdb.active_prompts \
                 ORDER BY created_at ASC, id ASC \
                 ON CONFLICT(id) DO UPDATE SET \
                    project_id = excluded.project_id, \
                    instance_id = excluded.instance_id, \
                    repo_path = excluded.repo_path, \
                    prompt_content = excluded.prompt_content, \
                    model = excluded.model, \
                    session_id = excluded.session_id, \
                    status = excluded.status, \
                    created_at = excluded.created_at, \
                    updated_at = excluded.updated_at, \
                    image_payload = excluded.image_payload"
            ),
            [],
        )
        .map_err(AppError::from)? as usize;

    // Rewrite agm-asset:// references if images were materialized
    if !materialized_map.is_empty() {
        for (key, dest_path) in &materialized_map {
            let asset_marker = format!("agm-asset://{key}");
            let local_uri = format!("file:///{}", dest_path.to_string_lossy().replace('\\', "/"));
            tx.execute(
                "UPDATE active_prompts
                 SET prompt_content = replace(prompt_content, ?1, ?2),
                     image_payload = replace(image_payload, ?1, ?2)
                 WHERE prompt_content LIKE ?3 OR image_payload LIKE ?3",
                params![asset_marker, local_uri, format!("%{asset_marker}%")],
            )
            .map_err(AppError::from)?;
        }
    }

    tx.commit().map_err(AppError::from)?;
    live.execute("DETACH DATABASE importdb", [])
        .map_err(AppError::from)?;

    super::tree::invalidate_prompt_tree_cache(target_instance);

    crate::modules::logger::log_info(&format!(
        "[RepoDB] Imported {restored} prompt(s) and materialized {materialized_count} image(s) from SQLite tree export {path:?}"
    ));

    Ok(ImportResult {
        success: true,
        imported_count: restored,
        materialized_images_count: materialized_count,
        message: format!(
            "Successfully imported {restored} prompt(s) and materialized {materialized_count} image(s)"
        ),
    })
}

/// Web (non-Tauri) export path: snapshot to a temp file, base64-encode the
/// bytes, and return them for a browser blob download.
pub fn export_prompt_tree_db_bytes() -> Result<TreeExportPayload, AppError> {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let file_name = format!("prompt-tree-{stamp}.db");
    let tmp = std::env::temp_dir().join(&file_name);

    let mut stats = write_prompt_tree_db(&tmp)?;
    stats.path = file_name.clone();

    let bytes = fs::read(&tmp).map_err(AppError::from)?;
    let data_b64 = Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);

    // Justification: temp export cleanup is best-effort; a leftover temp file is harmless.
    crate::error::record_ignored(fs::remove_file(&tmp), "remove temp tree export db");

    Ok(TreeExportPayload {
        data_b64,
        file_name,
        stats,
    })
}

/// Web (non-Tauri) import path: decode base64 `.db` bytes into a temp file and
/// run the same validated import pipeline as the Tauri path.
pub fn import_prompt_tree_db_bytes(data_b64: &str) -> Result<usize, AppError> {
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(data_b64.trim())
        .map_err(|e| AppError::Config(format!("Invalid base64 database payload: {e}")))?;
    if bytes.is_empty() {
        return Err(AppError::Config("Database payload is empty".to_string()));
    }
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let tmp = std::env::temp_dir().join(format!("agm-tree-import-{stamp}.db"));
    fs::write(&tmp, &bytes).map_err(AppError::from)?;

    let result = import_prompt_tree_db(&tmp);

    // Justification: temp import cleanup is best-effort after the import completes.
    crate::error::record_ignored(fs::remove_file(&tmp), "remove temp tree import db");

    result
}

#[cfg(test)]
mod tree_sqlite_export_tests {
    use super::*;

    #[test]
    fn bogus_file_is_rejected_with_real_error() {
        let tmp = std::env::temp_dir().join("agm-tree-export-probe-notdb.txt");
        std::fs::write(&tmp, b"definitely not a database").unwrap();
        let err = probe_export_file(&tmp).expect_err("non-db file must be rejected");
        let msg = err.to_string();
        assert!(
            msg.contains("too small") || msg.contains("not a SQLite"),
            "unexpected error: {msg}"
        );
        let _ = std::fs::remove_file(&tmp);
    }

    #[test]
    fn wrong_kind_db_is_rejected() {
        let tmp = std::env::temp_dir().join("agm-tree-export-probe-wrongkind.db");
        let _ = std::fs::remove_file(&tmp);
        {
            let conn = Connection::open(&tmp).unwrap();
            conn.execute_batch(
                "CREATE TABLE meta (id INTEGER PRIMARY KEY AUTOINCREMENT, schema_version TEXT, kind TEXT, created_at TEXT, agm_version TEXT, notes TEXT);
                 INSERT INTO meta (schema_version, kind, created_at, agm_version) VALUES ('1.0', 'something/else', '0', 'test');
                 CREATE TABLE running_projects (id TEXT PRIMARY KEY, instance_id TEXT NOT NULL, repo_name TEXT NOT NULL, repo_path TEXT NOT NULL, workspace_storage_path TEXT, is_running INTEGER NOT NULL DEFAULT 1, last_detected_at INTEGER NOT NULL, updated_at INTEGER NOT NULL);
                 CREATE TABLE active_prompts (id TEXT PRIMARY KEY, project_id TEXT NOT NULL, instance_id TEXT NOT NULL, repo_path TEXT NOT NULL, prompt_content TEXT NOT NULL, model TEXT, session_id TEXT, status TEXT NOT NULL, created_at INTEGER NOT NULL, updated_at INTEGER NOT NULL, image_payload TEXT);
                 CREATE TABLE agm_project_sequences (project_key TEXT PRIMARY KEY, seq_id INTEGER NOT NULL UNIQUE, project_id TEXT NOT NULL, repo_name TEXT NOT NULL, repo_path TEXT NOT NULL, instance_id TEXT NOT NULL DEFAULT 'default', updated_at INTEGER NOT NULL);
                 CREATE TABLE agm_conversation_sequences (conversation_id TEXT PRIMARY KEY, seq_id INTEGER NOT NULL UNIQUE, project_key TEXT NOT NULL, title TEXT NOT NULL, instance_id TEXT NOT NULL DEFAULT 'default', updated_at INTEGER NOT NULL);",
            )
            .unwrap();
        }
        let err = probe_export_file(&tmp).expect_err("wrong-kind db must be rejected");
        assert!(err.to_string().contains("not an AGM prompt-tree export"));
        let _ = std::fs::remove_file(&tmp);
    }
}
