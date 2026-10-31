//! Suffix Templates & "Resend As" Title Presets Subsystem
//!
//! Manages persistent verification suffix templates and customizable "Resend As"
//! title patterns in `repo_prompts.db` with full SQLite WAL support, 5-stage write
//! verification protocol, and strict FIFO retrieval.

use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Row model for `prompt_suffix_templates` table in `repo_prompts.db`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SuffixTemplateRow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub category: String,
    pub template: String,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

pub type SuffixTemplateRecord = SuffixTemplateRow;

/// Row model for `prompt_resend_as_templates` table in `repo_prompts.db`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ResendAsTemplateRow {
    pub id: String,
    pub name: String,
    pub title_pattern: String,
    pub target_mode: String, // "new" | "current"
    pub suffix_id: Option<String>,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

pub type ResendAsTemplateRecord = ResendAsTemplateRow;

/// Input struct for creating or updating a suffix template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSuffixTemplateInput {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub template: String,
    pub is_default: Option<bool>,
}

/// Input struct for creating or updating a "Resend As" title pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateResendAsTemplateInput {
    pub id: String,
    pub name: String,
    pub title_pattern: String,
    pub target_mode: Option<String>,
    pub suffix_id: Option<String>,
    pub is_default: Option<bool>,
}

const VALID_CATEGORIES: &[&str] = &[
    "custom",
    "verification",
    "release",
    "quality",
    "robustness",
    "security",
    "governance",
    "testing",
    "architecture",
];

/// Resolve filesystem path to `repo_prompts.db`.
pub fn get_repo_db_path() -> Result<PathBuf, String> {
    if let Ok(env_path) = std::env::var("ABV_DATA_DIR") {
        if !env_path.trim().is_empty() {
            let mut p = PathBuf::from(env_path.trim());
            p.push("repo_prompts.db");
            return Ok(p);
        }
    }

    let base = dirs::data_local_dir()
        .or_else(dirs::data_dir)
        .ok_or_else(|| "Failed to resolve local app data directory".to_string())?;

    let pointer = base.join("antigravity_data_dir.txt");
    let dir = if pointer.exists() {
        if let Ok(content) = fs::read_to_string(&pointer) {
            let trimmed = content.trim();
            if !trimmed.is_empty() {
                PathBuf::from(trimmed)
            } else {
                base.join("antigravity")
            }
        } else {
            base.join("antigravity")
        }
    } else {
        base.join("antigravity")
    };

    Ok(dir.join("repo_prompts.db"))
}

/// Connect to `repo_prompts.db` with WAL mode, 5000ms busy timeout, and schema initialization.
pub fn connect_db() -> Result<Connection, String> {
    let path = get_repo_db_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create repo database directory: {e}"))?;
    }
    let conn = Connection::open(&path)
        .map_err(|e| format!("Failed to open repo prompts database at {path:?}: {e}"))?;

    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|e| format!("Failed to set WAL mode: {e}"))?;
    conn.pragma_update(None, "busy_timeout", 5000)
        .map_err(|e| format!("Failed to set busy timeout: {e}"))?;

    ensure_templates_tables(&conn)?;
    Ok(conn)
}

/// Ensure database schema and default seeds exist in `repo_prompts.db`.
pub fn ensure_templates_tables(conn: &Connection) -> Result<(), String> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS prompt_suffix_templates (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            category TEXT NOT NULL DEFAULT 'custom',
            template TEXT NOT NULL,
            is_default INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create prompt_suffix_templates table: {e}"))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_prompt_suffix_templates_category 
         ON prompt_suffix_templates(category)",
        [],
    )
    .map_err(|e| format!("Failed to create idx_prompt_suffix_templates_category: {e}"))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_prompt_suffix_templates_default 
         ON prompt_suffix_templates(is_default)",
        [],
    )
    .map_err(|e| format!("Failed to create idx_prompt_suffix_templates_default: {e}"))?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS prompt_resend_as_templates (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            title_pattern TEXT NOT NULL,
            target_mode TEXT NOT NULL DEFAULT 'new',
            suffix_id TEXT,
            is_default INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create prompt_resend_as_templates table: {e}"))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_prompt_resend_as_default 
         ON prompt_resend_as_templates(is_default)",
        [],
    )
    .map_err(|e| format!("Failed to create idx_prompt_resend_as_default: {e}"))?;

    seed_default_templates_if_empty(conn)?;
    seed_default_resend_as_if_empty(conn)?;

    Ok(())
}

fn seed_default_templates_if_empty(conn: &Connection) -> Result<(), String> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM prompt_suffix_templates", [], |row| {
            row.get(0)
        })
        .map_err(|e| format!("Failed to count prompt_suffix_templates: {e}"))?;

    if count > 0 {
        return Ok(());
    }

    let defaults = [
        (
            "is-it-done",
            "Done? (Is it done?)",
            "Comprehensive completion audit & evidence scorecard",
            "verification",
            "### Task Completion Audit:\n- Verify all requirement acceptance criteria against grounded file inspection.\n- Provide line-numbered references (e.g. `src/...#L10-L25`) for all modifications.\n- Confirm zero orphan TODOs and clean working tree before marking done.",
            1,
        ),
        (
            "is-it-released",
            "Released? (Release Gates)",
            "14-location SemVer sync, strict @aukgit attribution & CI pre-flight",
            "release",
            "### Release Engineering Checklist:\n- Check branch alignment (main for stable, beta for preview).\n- Synchronize 14 version manifest locations via `npm run bump`.\n- Attributions strictly `@aukgit` (`(Thanks to @aukgit)`).\n- Synchronize README and README_EN update logs.\n- Run fmt, clippy, and build pre-flight gates.",
            0,
        ),
        (
            "are-you-confident",
            "Sure? (Quality Audit)",
            "Code quality, <= 15-line functions, positive booleans & error routing",
            "quality",
            "### Code Quality & Confidence Audit:\n- Keep core functions bounded to <= 15 lines with single responsibility.\n- Flatten nested conditionals with early returns and guard clauses.\n- Enforce positive boolean polarity (no double negatives).\n- Route all errors through AppError and useErrorStore (no bare catch {}).",
            0,
        ),
        (
            "edge-cases-and-tests",
            "Edge cases (Boundary Checks)",
            "Boundary checks, job breakaway fallbacks, timeouts & mock isolation",
            "robustness",
            "### Edge Cases & Defensive Architecture:\n- Validate empty inputs, collection bounds, and null/undefined handling.\n- Ensure Job Object breakaway fallbacks (spawn_with_job_breakaway_fallback).\n- Bound all network and process waits with explicit timeouts.\n- Verify multi-instance profile sandboxing and conscious PID matching.",
            0,
        ),
        (
            "security-audit",
            "Security (Secrets & Zero-Trust)",
            "Secrets scrub, credential protection, input validation & zero-trust",
            "security",
            "### Security & Zero-Trust Audit:\n- Scan for leaked secrets, API keys, tokens, or hardcoded passwords.\n- Ensure reverse proxy IP security and CIDR bounds.\n- Sanitize untrusted user inputs before SQLite queries or shell execution.\n- Validate AES-256-GCM encrypted credential vault isolation.",
            0,
        ),
        (
            "pr-review-ready",
            "PR Review (Single Scope & Clean Commits)",
            "Single problem scope, clean revertable commits & PR template match",
            "governance",
            "### PR Readiness & Governance Gate:\n- PR must address a single cohesive problem class.\n- Each commit must be self-contained and individually revertable.\n- Clean local trial-and-error noise into final-state commits.\n- Complete .github/pull_request_template.md sections.",
            0,
        ),
        (
            "test-and-coverage",
            "Tests & Cov (Mock Isolation & Suites)",
            "Targeted unit tests, mock isolation, error branch coverage & no OS mutation",
            "testing",
            "### Unit Testing & Mock Isolation:\n- Run targeted unit tests for touched modules locally.\n- Never execute real OS shutdown, reboot, or destructive system calls.\n- Inject mock executors and verify mock call durations.\n- Verify both happy-path and error-branch coverage.",
            0,
        ),
        (
            "refactor-dry",
            "Refactor DRY (Deduplication & Clean Helpers)",
            "Pattern deduplication, clean helper extraction & zero logic drift",
            "architecture",
            "### Code DRYness & Architectural Hygiene:\n- Identify repeated logic blocks across modules and extract shared helpers.\n- Eliminate redundant slice wrappers using variadic / spread patterns.\n- Preserve original business logic, comments, and docstrings.\n- Verify zero functional regression after extraction.",
            0,
        ),
    ];

    let base_time = Utc::now();
    for (idx, (id, name, desc, cat, template, is_def)) in defaults.into_iter().enumerate() {
        let item_now = (base_time + chrono::Duration::seconds(idx as i64)).to_rfc3339();
        conn.execute(
            "INSERT INTO prompt_suffix_templates (id, name, description, category, template, is_default, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![id, name, desc, cat, template, is_def, item_now, item_now],
        )
        .map_err(|e| format!("Failed to seed suffix template {id}: {e}"))?;
    }

    Ok(())
}

fn seed_default_resend_as_if_empty(conn: &Connection) -> Result<(), String> {
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM prompt_resend_as_templates",
            [],
            |row| row.get(0),
        )
        .map_err(|e| format!("Failed to count prompt_resend_as_templates: {e}"))?;

    if count > 0 {
        return Ok(());
    }

    let defaults = [
        (
            "verify",
            "Verify Task",
            "[Verify] {title}",
            "new",
            Some("is-it-done"),
            1,
        ),
        (
            "fix",
            "Fix Defects",
            "[Fix] {title}",
            "new",
            Some("edge-cases-and-tests"),
            0,
        ),
        (
            "rerun",
            "Rerun Execution",
            "[Rerun] {title}",
            "current",
            Some("rerun-nudge"),
            0,
        ),
        (
            "audit",
            "Code Audit",
            "[Audit] {title}",
            "new",
            Some("security-audit"),
            0,
        ),
        (
            "followup",
            "Follow-up Query",
            "[Follow-up] {title}",
            "new",
            None::<&str>,
            0,
        ),
    ];

    let base_time = Utc::now();
    for (idx, (id, name, pattern, mode, suffix, is_def)) in defaults.into_iter().enumerate() {
        let item_time = (base_time + chrono::Duration::seconds(idx as i64)).to_rfc3339();
        conn.execute(
            "INSERT INTO prompt_resend_as_templates (id, name, title_pattern, target_mode, suffix_id, is_default, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![id, name, pattern, mode, suffix, is_def, item_time, item_time],
        )
        .map_err(|e| format!("Failed to seed resend-as preset {id}: {e}"))?;
    }

    Ok(())
}

fn validate_id(id: &str) -> Result<(), String> {
    let trimmed = id.trim();
    if trimmed.len() < 2 || trimmed.len() > 64 {
        return Err(format!(
            "Template ID must be between 2 and 64 characters long (got {})",
            trimmed.len()
        ));
    }
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        return Err(format!(
            "Template ID '{trimmed}' contains invalid characters. Only alphanumeric, dashes, dots, and underscores are allowed."
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Suffix Templates CRUD & Write Verification
// ---------------------------------------------------------------------------

/// List all suffix templates ordered strictly by created_at ASC, id ASC (FIFO order).
pub fn list_suffix_templates_with_conn(
    conn: &Connection,
) -> Result<Vec<SuffixTemplateRow>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, category, template, is_default, created_at, updated_at 
             FROM prompt_suffix_templates 
             ORDER BY created_at ASC, id ASC",
        )
        .map_err(|e| format!("Failed to prepare list_suffix_templates query: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            let is_default_int: i32 = row.get(5)?;
            Ok(SuffixTemplateRow {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                category: row.get(3)?,
                template: row.get(4)?,
                is_default: is_default_int == 1,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| format!("Failed to query prompt_suffix_templates: {e}"))?;

    let mut list = Vec::new();
    for r in rows {
        list.push(r.map_err(|e| format!("Failed to read suffix template row: {e}"))?);
    }
    Ok(list)
}

pub fn list_suffix_templates() -> Result<Vec<SuffixTemplateRow>, String> {
    let conn = connect_db()?;
    list_suffix_templates_with_conn(&conn)
}

/// Retrieve a single suffix template by ID.
pub fn get_suffix_template_with_conn(
    conn: &Connection,
    id: &str,
) -> Result<Option<SuffixTemplateRow>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, description, category, template, is_default, created_at, updated_at 
             FROM prompt_suffix_templates 
             WHERE id = ?1",
        )
        .map_err(|e| format!("Failed to prepare get_suffix_template query: {e}"))?;

    let mut rows = stmt
        .query_map(params![id], |row| {
            let is_default_int: i32 = row.get(5)?;
            Ok(SuffixTemplateRow {
                id: row.get(0)?,
                name: row.get(1)?,
                description: row.get(2)?,
                category: row.get(3)?,
                template: row.get(4)?,
                is_default: is_default_int == 1,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| format!("Failed to query suffix template by ID: {e}"))?;

    if let Some(res) = rows.next() {
        let row = res.map_err(|e| format!("Failed to read suffix template: {e}"))?;
        Ok(Some(row))
    } else {
        Ok(None)
    }
}

pub fn get_suffix_template(id: &str) -> Result<Option<SuffixTemplateRow>, String> {
    let conn = connect_db()?;
    get_suffix_template_with_conn(&conn, id)
}

/// Save (create or update) a suffix template with 5-stage write verification.
pub fn save_suffix_template_with_conn(
    conn: &mut Connection,
    input: &CreateSuffixTemplateInput,
) -> Result<SuffixTemplateRow, String> {
    // Stage 1: Pre-write Semantic Validation
    validate_id(&input.id)?;

    let clean_id = input.id.trim().to_string();
    let clean_name = input.name.trim().to_string();
    if clean_name.len() < 2 || clean_name.len() > 100 {
        return Err(format!(
            "Template Name must be between 2 and 100 characters (got {})",
            clean_name.len()
        ));
    }

    let clean_template = input.template.trim().to_string();
    if clean_template.len() < 5 {
        return Err("Template body must be at least 5 characters long".to_string());
    }
    if clean_template.contains('\0') {
        return Err("Template body contains forbidden null bytes".to_string());
    }

    let clean_category = input
        .category
        .as_deref()
        .map(|c| c.trim().to_lowercase())
        .filter(|c| !c.is_empty())
        .unwrap_or_else(|| "custom".to_string());

    if !VALID_CATEGORIES.contains(&clean_category.as_str()) {
        return Err(format!(
            "Invalid category '{clean_category}'. Allowed categories: {}",
            VALID_CATEGORIES.join(", ")
        ));
    }

    let is_default = input.is_default.unwrap_or(false);
    let now = Utc::now().to_rfc3339();

    // Stage 2: Atomic SQLite Transaction with Retry / Exclusivity Guard
    let existing = get_suffix_template_with_conn(conn, &clean_id)?;
    let created_at = existing
        .map(|e| e.created_at)
        .unwrap_or_else(|| now.clone());

    let tx = conn
        .transaction()
        .map_err(|e| format!("Failed to begin transaction for save_suffix_template: {e}"))?;

    if is_default {
        tx.execute(
            "UPDATE prompt_suffix_templates SET is_default = 0 WHERE is_default = 1",
            [],
        )
        .map_err(|e| format!("Failed to unset previous default suffix templates: {e}"))?;
    }

    tx.execute(
        "INSERT INTO prompt_suffix_templates (id, name, description, category, template, is_default, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(id) DO UPDATE SET
             name = excluded.name,
             description = excluded.description,
             category = excluded.category,
             template = excluded.template,
             is_default = excluded.is_default,
             updated_at = excluded.updated_at",
        params![
            clean_id,
            clean_name,
            input.description.as_deref().map(|s| s.trim()),
            clean_category,
            clean_template,
            if is_default { 1 } else { 0 },
            created_at,
            now,
        ],
    )
    .map_err(|e| format!("Failed to insert/update prompt_suffix_templates: {e}"))?;

    tx.commit()
        .map_err(|e| format!("Failed to commit suffix template transaction: {e}"))?;

    // Stage 3: Immediate Read-Back Integrity Verification (Write-Then-Verify Invariant)
    let verified = get_suffix_template_with_conn(conn, &clean_id)?.ok_or_else(|| {
        format!("Post-write verification failed: template '{clean_id}' not found after commit")
    })?;

    if verified.name != clean_name {
        return Err(format!(
            "Post-write verification failed: name mismatch (expected '{clean_name}', found '{}')",
            verified.name
        ));
    }
    if verified.template != clean_template {
        return Err("Post-write verification failed: template body mismatch".to_string());
    }
    if verified.category != clean_category {
        return Err(format!(
            "Post-write verification failed: category mismatch (expected '{clean_category}', found '{}')",
            verified.category
        ));
    }
    if verified.is_default != is_default {
        return Err(format!(
            "Post-write verification failed: is_default mismatch (expected {is_default}, found {})",
            verified.is_default
        ));
    }

    Ok(verified)
}

pub fn save_suffix_template(
    input: &CreateSuffixTemplateInput,
) -> Result<SuffixTemplateRow, String> {
    let mut conn = connect_db()?;
    save_suffix_template_with_conn(&mut conn, input)
}

/// Delete a suffix template by ID.
pub fn delete_suffix_template_with_conn(conn: &Connection, id: &str) -> Result<bool, String> {
    let clean_id = id.trim();
    let rows_affected = conn
        .execute(
            "DELETE FROM prompt_suffix_templates WHERE id = ?1",
            params![clean_id],
        )
        .map_err(|e| format!("Failed to delete suffix template '{clean_id}': {e}"))?;

    Ok(rows_affected > 0)
}

pub fn delete_suffix_template(id: &str) -> Result<bool, String> {
    let conn = connect_db()?;
    delete_suffix_template_with_conn(&conn, id)
}

/// Mark a specific suffix template as the default.
pub fn set_default_suffix_template_with_conn(
    conn: &mut Connection,
    id: &str,
) -> Result<(), String> {
    let clean_id = id.trim();
    let _existing = get_suffix_template_with_conn(conn, clean_id)?.ok_or_else(|| {
        format!("Cannot set default: suffix template '{clean_id}' does not exist")
    })?;

    let tx = conn
        .transaction()
        .map_err(|e| format!("Failed to begin transaction for set_default_suffix_template: {e}"))?;

    tx.execute(
        "UPDATE prompt_suffix_templates SET is_default = 0 WHERE is_default = 1",
        [],
    )
    .map_err(|e| format!("Failed to clear current default suffix templates: {e}"))?;

    tx.execute(
        "UPDATE prompt_suffix_templates SET is_default = 1 WHERE id = ?1",
        params![clean_id],
    )
    .map_err(|e| format!("Failed to set '{clean_id}' as default: {e}"))?;

    tx.commit()
        .map_err(|e| format!("Failed to commit set_default_suffix_template: {e}"))?;

    // Verify
    let verified = get_suffix_template_with_conn(conn, clean_id)?
        .ok_or_else(|| format!("Read-back verification failed for '{clean_id}'"))?;
    if !verified.is_default {
        return Err(format!(
            "Post-write verification failed: template '{clean_id}' is_default is not true"
        ));
    }

    Ok(())
}

pub fn set_default_suffix_template(id: &str) -> Result<(), String> {
    let mut conn = connect_db()?;
    set_default_suffix_template_with_conn(&mut conn, id)
}

// ---------------------------------------------------------------------------
// "Resend As" Title Presets CRUD & Write Verification
// ---------------------------------------------------------------------------

/// List all "Resend As" title presets in strict FIFO order (`ORDER BY created_at ASC, id ASC`).
pub fn list_resend_as_templates_with_conn(
    conn: &Connection,
) -> Result<Vec<ResendAsTemplateRow>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, title_pattern, target_mode, suffix_id, is_default, created_at, updated_at 
             FROM prompt_resend_as_templates 
             ORDER BY created_at ASC, id ASC",
        )
        .map_err(|e| format!("Failed to prepare list_resend_as_templates query: {e}"))?;

    let rows = stmt
        .query_map([], |row| {
            let is_default_int: i32 = row.get(5)?;
            Ok(ResendAsTemplateRow {
                id: row.get(0)?,
                name: row.get(1)?,
                title_pattern: row.get(2)?,
                target_mode: row.get(3)?,
                suffix_id: row.get(4)?,
                is_default: is_default_int == 1,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| format!("Failed to query prompt_resend_as_templates: {e}"))?;

    let mut list = Vec::new();
    for r in rows {
        list.push(r.map_err(|e| format!("Failed to read resend-as template row: {e}"))?);
    }
    Ok(list)
}

pub fn list_resend_as_templates() -> Result<Vec<ResendAsTemplateRow>, String> {
    let conn = connect_db()?;
    list_resend_as_templates_with_conn(&conn)
}

/// Retrieve a single "Resend As" title preset by ID.
pub fn get_resend_as_template_with_conn(
    conn: &Connection,
    id: &str,
) -> Result<Option<ResendAsTemplateRow>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, name, title_pattern, target_mode, suffix_id, is_default, created_at, updated_at 
             FROM prompt_resend_as_templates 
             WHERE id = ?1",
        )
        .map_err(|e| format!("Failed to prepare get_resend_as_template query: {e}"))?;

    let mut rows = stmt
        .query_map(params![id], |row| {
            let is_default_int: i32 = row.get(5)?;
            Ok(ResendAsTemplateRow {
                id: row.get(0)?,
                name: row.get(1)?,
                title_pattern: row.get(2)?,
                target_mode: row.get(3)?,
                suffix_id: row.get(4)?,
                is_default: is_default_int == 1,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|e| format!("Failed to query resend-as template by ID: {e}"))?;

    if let Some(res) = rows.next() {
        let row = res.map_err(|e| format!("Failed to read resend-as template: {e}"))?;
        Ok(Some(row))
    } else {
        Ok(None)
    }
}

pub fn get_resend_as_template(id: &str) -> Result<Option<ResendAsTemplateRow>, String> {
    let conn = connect_db()?;
    get_resend_as_template_with_conn(&conn, id)
}

/// Save (create or update) a "Resend As" title preset with 5-stage write verification.
pub fn save_resend_as_template_with_conn(
    conn: &mut Connection,
    input: &CreateResendAsTemplateInput,
) -> Result<ResendAsTemplateRow, String> {
    // Stage 1: Pre-write Semantic Validation
    validate_id(&input.id)?;

    let clean_id = input.id.trim().to_string();
    let clean_name = input.name.trim().to_string();
    if clean_name.len() < 2 || clean_name.len() > 100 {
        return Err(format!(
            "Preset Name must be between 2 and 100 characters (got {})",
            clean_name.len()
        ));
    }

    let clean_pattern = input.title_pattern.trim().to_string();
    if clean_pattern.len() < 2 {
        return Err("Title pattern must be at least 2 characters long".to_string());
    }

    let clean_mode = input
        .target_mode
        .as_deref()
        .map(|m| m.trim().to_lowercase())
        .filter(|m| !m.is_empty())
        .unwrap_or_else(|| "new".to_string());

    if clean_mode != "new" && clean_mode != "current" {
        return Err(format!(
            "Invalid target_mode '{clean_mode}'. Allowed values: 'new', 'current'"
        ));
    }

    let clean_suffix_id = input
        .suffix_id
        .as_deref()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && s != "none");

    let is_default = input.is_default.unwrap_or(false);
    let now = Utc::now().to_rfc3339();

    // Stage 2: Atomic SQLite Transaction with Retry Guard
    let existing = get_resend_as_template_with_conn(conn, &clean_id)?;
    let created_at = existing
        .map(|e| e.created_at)
        .unwrap_or_else(|| now.clone());

    let tx = conn
        .transaction()
        .map_err(|e| format!("Failed to begin transaction for save_resend_as_template: {e}"))?;

    if is_default {
        tx.execute(
            "UPDATE prompt_resend_as_templates SET is_default = 0 WHERE is_default = 1",
            [],
        )
        .map_err(|e| format!("Failed to unset previous default resend-as templates: {e}"))?;
    }

    tx.execute(
        "INSERT INTO prompt_resend_as_templates (id, name, title_pattern, target_mode, suffix_id, is_default, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(id) DO UPDATE SET
             name = excluded.name,
             title_pattern = excluded.title_pattern,
             target_mode = excluded.target_mode,
             suffix_id = excluded.suffix_id,
             is_default = excluded.is_default,
             updated_at = excluded.updated_at",
        params![
            clean_id,
            clean_name,
            clean_pattern,
            clean_mode,
            clean_suffix_id,
            if is_default { 1 } else { 0 },
            created_at,
            now,
        ],
    )
    .map_err(|e| format!("Failed to insert/update prompt_resend_as_templates: {e}"))?;

    tx.commit()
        .map_err(|e| format!("Failed to commit resend-as template transaction: {e}"))?;

    // Stage 3: Immediate Read-Back Integrity Verification
    let verified = get_resend_as_template_with_conn(conn, &clean_id)?.ok_or_else(|| {
        format!("Post-write verification failed: preset '{clean_id}' not found after commit")
    })?;

    if verified.name != clean_name {
        return Err(format!(
            "Post-write verification failed: name mismatch (expected '{clean_name}', found '{}')",
            verified.name
        ));
    }
    if verified.title_pattern != clean_pattern {
        return Err(format!(
            "Post-write verification failed: pattern mismatch (expected '{clean_pattern}', found '{}')",
            verified.title_pattern
        ));
    }
    if verified.target_mode != clean_mode {
        return Err(format!(
            "Post-write verification failed: target_mode mismatch (expected '{clean_mode}', found '{}')",
            verified.target_mode
        ));
    }
    if verified.suffix_id != clean_suffix_id {
        return Err("Post-write verification failed: suffix_id mismatch".to_string());
    }
    if verified.is_default != is_default {
        return Err(format!(
            "Post-write verification failed: is_default mismatch (expected {is_default}, found {})",
            verified.is_default
        ));
    }

    Ok(verified)
}

pub fn save_resend_as_template(
    input: &CreateResendAsTemplateInput,
) -> Result<ResendAsTemplateRow, String> {
    let mut conn = connect_db()?;
    save_resend_as_template_with_conn(&mut conn, input)
}

/// Delete a "Resend As" title preset by ID.
pub fn delete_resend_as_template_with_conn(conn: &Connection, id: &str) -> Result<bool, String> {
    let clean_id = id.trim();
    let rows_affected = conn
        .execute(
            "DELETE FROM prompt_resend_as_templates WHERE id = ?1",
            params![clean_id],
        )
        .map_err(|e| format!("Failed to delete resend-as template '{clean_id}': {e}"))?;

    Ok(rows_affected > 0)
}

pub fn delete_resend_as_template(id: &str) -> Result<bool, String> {
    let conn = connect_db()?;
    delete_resend_as_template_with_conn(&conn, id)
}

/// Mark a specific "Resend As" title preset as default.
pub fn set_default_resend_as_template_with_conn(
    conn: &mut Connection,
    id: &str,
) -> Result<(), String> {
    let clean_id = id.trim();
    let _existing = get_resend_as_template_with_conn(conn, clean_id)?.ok_or_else(|| {
        format!("Cannot set default: resend-as preset '{clean_id}' does not exist")
    })?;

    let tx = conn.transaction().map_err(|e| {
        format!("Failed to begin transaction for set_default_resend_as_template: {e}")
    })?;

    tx.execute(
        "UPDATE prompt_resend_as_templates SET is_default = 0 WHERE is_default = 1",
        [],
    )
    .map_err(|e| format!("Failed to clear current default resend-as templates: {e}"))?;

    tx.execute(
        "UPDATE prompt_resend_as_templates SET is_default = 1 WHERE id = ?1",
        params![clean_id],
    )
    .map_err(|e| format!("Failed to set '{clean_id}' as default: {e}"))?;

    tx.commit()
        .map_err(|e| format!("Failed to commit set_default_resend_as_template: {e}"))?;

    let verified = get_resend_as_template_with_conn(conn, clean_id)?
        .ok_or_else(|| format!("Read-back verification failed for '{clean_id}'"))?;
    if !verified.is_default {
        return Err(format!(
            "Post-write verification failed: preset '{clean_id}' is_default is not true"
        ));
    }

    Ok(())
}

pub fn set_default_resend_as_template(id: &str) -> Result<(), String> {
    let mut conn = connect_db()?;
    set_default_resend_as_template_with_conn(&mut conn, id)
}

// ---------------------------------------------------------------------------
// Pattern Interpolation Engine
// ---------------------------------------------------------------------------

/// Interpolate title pattern with tokens `{title}`, `{date}`, `{time}`, `{seq}`.
pub fn apply_title_pattern(
    pattern: &str,
    current_title: &str,
    date_str: Option<&str>,
    seq_num: Option<u64>,
) -> String {
    let base_title = current_title.trim();
    let effective_date = date_str
        .map(|s| s.to_string())
        .unwrap_or_else(|| chrono::Local::now().format("%Y-%m-%d").to_string());
    let effective_time = chrono::Local::now().format("%H:%M").to_string();
    let effective_seq = seq_num.map(|n| format!("{:03}", n)).unwrap_or_default();

    let mut result = pattern
        .replace("{title}", base_title)
        .replace("{date}", &effective_date)
        .replace("{time}", &effective_time)
        .replace("{seq}", &effective_seq);

    // Fallback if title placeholder was omitted: append base title if non-empty
    if !pattern.contains("{title}") && !base_title.is_empty() {
        result = format!("{} - {}", result, base_title);
    }
    result.trim().to_string()
}

pub fn interpolate_title_pattern(
    pattern: &str,
    title: &str,
    date_str: Option<&str>,
    seq_num: Option<u64>,
) -> String {
    apply_title_pattern(pattern, title, date_str, seq_num)
}

// ---------------------------------------------------------------------------
// Unit Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.pragma_update(None, "journal_mode", "WAL").unwrap();
        conn.pragma_update(None, "busy_timeout", 5000).unwrap();
        ensure_templates_tables(&conn).unwrap();
        conn
    }

    #[test]
    fn test_seed_and_list_templates() {
        let conn = create_test_db();
        let list = list_suffix_templates_with_conn(&conn).unwrap();
        assert_eq!(list.len(), 8);
        assert_eq!(list[0].id, "is-it-done");
        assert!(list[0].is_default);

        let resend_list = list_resend_as_templates_with_conn(&conn).unwrap();
        assert_eq!(resend_list.len(), 5);
        assert_eq!(resend_list[0].id, "verify");
        assert!(resend_list[0].is_default);
    }

    #[test]
    fn test_save_suffix_template_and_verify() {
        let mut conn = create_test_db();
        let input = CreateSuffixTemplateInput {
            id: "my-test-check".to_string(),
            name: "My Test Check".to_string(),
            description: Some("Custom test check".to_string()),
            category: Some("robustness".to_string()),
            template: "### Test Checklist:\n- Ensure edge cases handled.".to_string(),
            is_default: Some(false),
        };

        let saved = save_suffix_template_with_conn(&mut conn, &input).unwrap();
        assert_eq!(saved.id, "my-test-check");
        assert_eq!(saved.name, "My Test Check");
        assert_eq!(saved.category, "robustness");

        let fetched = get_suffix_template_with_conn(&conn, "my-test-check")
            .unwrap()
            .unwrap();
        assert_eq!(fetched.id, saved.id);
        assert_eq!(fetched.template, saved.template);
    }

    #[test]
    fn test_save_suffix_template_validation_errors() {
        let mut conn = create_test_db();
        // Invalid ID with slash
        let res = save_suffix_template_with_conn(
            &mut conn,
            &CreateSuffixTemplateInput {
                id: "invalid/id".to_string(),
                name: "Bad ID".to_string(),
                description: None,
                category: None,
                template: "Some long template text".to_string(),
                is_default: None,
            },
        );
        assert!(res.is_err());

        // Body too short
        let res2 = save_suffix_template_with_conn(
            &mut conn,
            &CreateSuffixTemplateInput {
                id: "short-body".to_string(),
                name: "Short Body".to_string(),
                description: None,
                category: None,
                template: "abc".to_string(),
                is_default: None,
            },
        );
        assert!(res2.is_err());
    }

    #[test]
    fn test_toggle_default_suffix_template() {
        let mut conn = create_test_db();
        set_default_suffix_template_with_conn(&mut conn, "security-audit").unwrap();

        let sec = get_suffix_template_with_conn(&conn, "security-audit")
            .unwrap()
            .unwrap();
        assert!(sec.is_default);

        let done = get_suffix_template_with_conn(&conn, "is-it-done")
            .unwrap()
            .unwrap();
        assert!(!done.is_default);
    }

    #[test]
    fn test_save_and_delete_resend_as_template() {
        let mut conn = create_test_db();
        let input = CreateResendAsTemplateInput {
            id: "perf".to_string(),
            name: "Performance Benchmark".to_string(),
            title_pattern: "[Perf:{date}] {title}".to_string(),
            target_mode: Some("new".to_string()),
            suffix_id: Some("refactor-dry".to_string()),
            is_default: Some(false),
        };

        let saved = save_resend_as_template_with_conn(&mut conn, &input).unwrap();
        assert_eq!(saved.id, "perf");
        assert_eq!(saved.title_pattern, "[Perf:{date}] {title}");

        let deleted = delete_resend_as_template_with_conn(&conn, "perf").unwrap();
        assert!(deleted);

        let fetched = get_resend_as_template_with_conn(&conn, "perf").unwrap();
        assert!(fetched.is_none());
    }

    #[test]
    fn test_apply_title_pattern_interpolation() {
        let res = apply_title_pattern(
            "[Verify] {title}",
            "Fix Memory Leak",
            Some("2026-10-17"),
            Some(1),
        );
        assert_eq!(res, "[Verify] Fix Memory Leak");

        let res_date = apply_title_pattern(
            "[{date}] #{seq} {title}",
            "Task Alpha",
            Some("2026-10-17"),
            Some(42),
        );
        assert_eq!(res_date, "[2026-10-17] #042 Task Alpha");

        let res_no_placeholder =
            apply_title_pattern("[Audit]", "Security Scan", Some("2026-10-17"), None);
        assert_eq!(res_no_placeholder, "[Audit] - Security Scan");
    }
}
