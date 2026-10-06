# Step 07: Backup DB Migration Twin, Identity Index, and Dedupe Key

Goal: the backup database (`prompt_backups`) gets the same one-time v140 migration as the repo DB (step 06), a partial unique index on `(instance_id, prompt_id)`, and a dedupe rule that never moves a row from one instance to another.

## 1. Depends on

- Step 01 (`InstanceScope`, `canonical_instance_id` in `src-tauri/src/modules/instance.rs`).
- Step 04 (`gemini_dirs_tagged(scope: &InstanceScope) -> Vec<(String, PathBuf)>` in `src-tauri/src/modules/repo_db.rs`).
- Step 06 (repo DB migration committed; this step is its twin for the backup DB).

Confirm with `git log --oneline -1` that the step 06 commit is the latest one.

## 2. Files you may edit

- `src-tauri/src/modules/backup_prompts_db.rs`

Nothing else.

## 3. Find it

| Change | Anchor (`fn` signature) | Unique search literal | Line hint |
|---|---|---|---|
| A. Imports | top of file | `use crate::modules::repo_db::{self, ActivePrompt};` | `:1` to `:10` |
| B. Split schema init out of `connect_backup_db`, run migration, create identity index | `pub fn connect_backup_db(custom_file: Option<&str>) -> Result<Connection, String>` | `Failed to initialize backup prompts tables` | `:112` to `:185` |
| C. Dedupe block inside the backup loop | `pub fn backup_active_running_prompts_for_instance(` | `Deduplicate records to prevent duplicate reinjections` | `:341` to `:404` |
| D. New helper `upsert_prompt_backup_record` | add right after the closing `}` of `backup_active_running_prompts_for_instance` | `/// List all backup batches and total counts` (insert above this line) | `:412` |
| E. Tests | `mod tests` | `fn test_backup_db_lifecycle()` | `:728` to `:783` |

Search commands:

```text
gitmap aum search "Failed to initialize backup prompts tables" src-tauri/src/modules/backup_prompts_db.rs --ext .rs
gitmap aum search "Deduplicate records to prevent duplicate reinjections" src-tauri/src/modules/backup_prompts_db.rs --ext .rs
gitmap aum search "List all backup batches and total counts" src-tauri/src/modules/backup_prompts_db.rs --ext .rs
gitmap aum search "pub fn gemini_dirs_tagged" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "pub enum InstanceScope" src-tauri/src/modules/instance.rs --ext .rs
gitmap aum search "pub fn canonical_instance_id" src-tauri/src/modules/instance.rs --ext .rs
```

The last three must each return a hit (steps 01 and 04). `gemini_dirs_tagged` must take `&InstanceScope` and return `Vec<(String, PathBuf)>`. If any is missing or has a different signature, STOP.

Caller check (GitMap does not descend into `src-tauri/src/bin` or `src-tauri/tests` from `src-tauri/src`):

```text
gitmap aum search "connect_backup_db|backup_active_running_prompts_for_instance|init_backup_tables|upsert_prompt_backup_record" src-tauri/src --ext .rs
gitmap aum search "connect_backup_db|backup_active_running_prompts_for_instance|init_backup_tables|upsert_prompt_backup_record" src-tauri/src/bin/agm.rs --ext .rs
gitmap aum search "connect_backup_db|backup_active_running_prompts_for_instance|init_backup_tables|upsert_prompt_backup_record" src-tauri/tests/auto_switcher_e2e_test.rs --ext .rs
gitmap aum search "connect_backup_db|backup_active_running_prompts_for_instance|init_backup_tables|upsert_prompt_backup_record" src-tauri/tests/instance_cloning_and_sync_test.rs --ext .rs
gitmap aum search "connect_backup_db|backup_active_running_prompts_for_instance|init_backup_tables|upsert_prompt_backup_record" src-tauri/tests/per_instance_prompt_liveness_test.rs --ext .rs
```

This step keeps the signatures of `connect_backup_db` and `backup_active_running_prompts_for_instance`; the new functions are `pub(crate)` or private. `init_backup_tables` and `upsert_prompt_backup_record` must have 0 hits before you start (name collision check). No caller in `agm.rs` or `src-tauri/tests` needs an edit (checked 2026-10-06).

## 4. Current code

### A. Imports (`:1` to `:10`)

```rust
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::modules::account;
use crate::modules::agy_cleaner;
use crate::modules::repo_db::{self, ActivePrompt};
```

### B. `connect_backup_db` (`:112` to `:185`)

```rust
/// Open and initialize the split SQLite database connection
pub fn connect_backup_db(custom_file: Option<&str>) -> Result<Connection, String> {
    let db_path = get_backup_prompts_db_path(custom_file)?;
    if let Some(parent) = db_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let conn = Connection::open(&db_path).map_err(|e| {
        format!(
            "Failed to open backup prompts database '{:?}': {}",
            db_path, e
        )
    })?;

    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA foreign_keys = ON;

         CREATE TABLE IF NOT EXISTS backup_batches (
             id TEXT PRIMARY KEY,
             created_at INTEGER NOT NULL,
             prompts_count INTEGER NOT NULL,
             file_path TEXT NOT NULL,
             retention_days INTEGER NOT NULL DEFAULT 1,
             is_fully_restored BOOLEAN NOT NULL DEFAULT 0
         );

         CREATE TABLE IF NOT EXISTS prompt_backups (
             id TEXT PRIMARY KEY,
             backup_batch_id TEXT NOT NULL,
             prompt_id TEXT NOT NULL,
             project_name TEXT NOT NULL,
             project_path TEXT NOT NULL,
             project_id TEXT NOT NULL,
             conversation_id TEXT NOT NULL,
             conversation_name TEXT,
             sequence_id INTEGER NOT NULL,
             prompt_text TEXT NOT NULL,
             has_images BOOLEAN NOT NULL DEFAULT 0,
             images_payload TEXT,
             status TEXT NOT NULL DEFAULT 'queued',
             created_at INTEGER NOT NULL,
             is_restored BOOLEAN NOT NULL DEFAULT 0,
             restored_at INTEGER,
             instance_id TEXT DEFAULT 'default',
             FOREIGN KEY(backup_batch_id) REFERENCES backup_batches(id) ON DELETE CASCADE
         );
         CREATE INDEX IF NOT EXISTS idx_prompt_backups_batch ON prompt_backups(backup_batch_id);
         CREATE INDEX IF NOT EXISTS idx_prompt_backups_restored ON prompt_backups(is_restored, restored_at);

         CREATE TABLE IF NOT EXISTS green_projects (
             id TEXT PRIMARY KEY,
             project_identifier TEXT NOT NULL UNIQUE,
             project_path TEXT NOT NULL,
             added_at INTEGER NOT NULL,
             status TEXT NOT NULL DEFAULT 'pending',
             last_checked_at INTEGER
         );",
    )
    .map_err(|e| format!("Failed to initialize backup prompts tables: {}", e))?;

    // Migrate existing DB if instance_id is missing
    let _ = conn.execute(
        "ALTER TABLE prompt_backups ADD COLUMN instance_id TEXT DEFAULT 'default'",
        [],
    );
    let _ = conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_prompt_backups_instance ON prompt_backups(instance_id)",
        [],
    );

    Ok(conn)
}
```

### C. Dedupe block in `backup_active_running_prompts_for_instance` (`:341` to `:404`)

The lines right before this block are the `let record = PromptBackupRecord { ... };` initializer ending with `instance_id: Some(target_inst.to_string()),` and `};`. The line right after this block is `        records.push(record);` followed by `    }` (end of the `for` loop). Those surrounding lines stay.

```rust
        // Deduplicate records to prevent duplicate reinjections into prompt_backups
        let is_live_prompt = running_prompts.iter().any(|rp| rp.id == record.prompt_id);
        let existing_unrestored: Option<String> = conn
            .query_row(
                "SELECT id FROM prompt_backups WHERE (prompt_id = ?1 OR (project_path = ?2 AND prompt_text = ?3)) AND is_restored = 0 LIMIT 1",
                params![record.prompt_id, record.project_path, record.prompt_text],
                |r| r.get(0),
            )
            .ok();

        if let Some(existing_rec_id) = existing_unrestored {
            let _ = conn.execute(
                "UPDATE prompt_backups SET backup_batch_id = ?1, created_at = ?2, instance_id = ?3, is_restored = 0 WHERE id = ?4",
                params![record.backup_batch_id, now, target_inst, existing_rec_id],
            );
            records.push(record);
            continue;
        }

        // If previously restored for this instance, reactivate for this new switch batch
        let existing_restored: Option<String> = conn
            .query_row(
                "SELECT id FROM prompt_backups WHERE (prompt_id = ?1 OR (project_path = ?2 AND prompt_text = ?3)) AND (instance_id = ?4 OR instance_id IS NULL OR instance_id = '') LIMIT 1",
                params![record.prompt_id, record.project_path, record.prompt_text, target_inst],
                |r| r.get(0),
            )
            .ok();

        if let Some(existing_rec_id) = existing_restored {
            let _ = conn.execute(
                "UPDATE prompt_backups SET backup_batch_id = ?1, created_at = ?2, instance_id = ?3, is_restored = 0, restored_at = NULL WHERE id = ?4",
                params![record.backup_batch_id, now, target_inst, existing_rec_id],
            );
            records.push(record);
            continue;
        }

        conn.execute(
            "INSERT INTO prompt_backups (
                id, backup_batch_id, prompt_id, project_name, project_path, project_id,
                conversation_id, conversation_name, sequence_id, prompt_text, has_images,
                images_payload, status, created_at, is_restored, restored_at, instance_id
             ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                record.id,
                record.backup_batch_id,
                record.prompt_id,
                record.project_name,
                record.project_path,
                record.project_id,
                record.conversation_id,
                record.conversation_name,
                record.sequence_id,
                record.prompt_text,
                record.has_images,
                record.images_payload,
                record.status,
                record.created_at,
                record.is_restored,
                record.restored_at,
                record.instance_id.as_deref().unwrap_or(target_inst),
            ],
        )
        .map_err(|e| format!("Failed to insert prompt backup record: {}", e))?;
```

### D. Insertion point (`:410` to `:412`)

```rust
    Ok((batch_info, records))
}

/// List all backup batches and total counts
```

### E. End of `mod tests` (`:778` to `:783`)

```rust
        let removed = auto_cleanup_expired(Some(custom_file), -10).unwrap();
        assert_eq!(removed, 1);

        let _ = fs::remove_dir_all(temp_dir);
    }
}
```

## 5. New code

### A. Imports: replace the block with

```rust
use chrono::Utc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

use crate::modules::account;
use crate::modules::agy_cleaner;
use crate::modules::instance::InstanceScope;
use crate::modules::repo_db::{self, ActivePrompt};
```

### B. Replace the whole `connect_backup_db` function with

```rust
/// Open and initialize the split SQLite database connection
pub fn connect_backup_db(custom_file: Option<&str>) -> Result<Connection, String> {
    let db_path = get_backup_prompts_db_path(custom_file)?;
    if let Some(parent) = db_path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let conn = Connection::open(&db_path).map_err(|e| {
        format!(
            "Failed to open backup prompts database '{:?}': {}",
            db_path, e
        )
    })?;

    init_backup_tables(&conn)?;
    if let Err(e) = migrate_backup_instance_ids_v140(&conn, &db_path) {
        crate::modules::logger::log_error(&format!(
            "[Migration v140] backup prompts migration not applied: {}",
            e
        ));
    }
    create_backup_identity_index(&conn);

    Ok(conn)
}

/// Create the backup prompt tables on any connection (file or in-memory).
pub(crate) fn init_backup_tables(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "PRAGMA journal_mode = WAL;
         PRAGMA synchronous = NORMAL;
         PRAGMA foreign_keys = ON;

         CREATE TABLE IF NOT EXISTS backup_batches (
             id TEXT PRIMARY KEY,
             created_at INTEGER NOT NULL,
             prompts_count INTEGER NOT NULL,
             file_path TEXT NOT NULL,
             retention_days INTEGER NOT NULL DEFAULT 1,
             is_fully_restored BOOLEAN NOT NULL DEFAULT 0
         );

         CREATE TABLE IF NOT EXISTS prompt_backups (
             id TEXT PRIMARY KEY,
             backup_batch_id TEXT NOT NULL,
             prompt_id TEXT NOT NULL,
             project_name TEXT NOT NULL,
             project_path TEXT NOT NULL,
             project_id TEXT NOT NULL,
             conversation_id TEXT NOT NULL,
             conversation_name TEXT,
             sequence_id INTEGER NOT NULL,
             prompt_text TEXT NOT NULL,
             has_images BOOLEAN NOT NULL DEFAULT 0,
             images_payload TEXT,
             status TEXT NOT NULL DEFAULT 'queued',
             created_at INTEGER NOT NULL,
             is_restored BOOLEAN NOT NULL DEFAULT 0,
             restored_at INTEGER,
             instance_id TEXT DEFAULT 'default',
             FOREIGN KEY(backup_batch_id) REFERENCES backup_batches(id) ON DELETE CASCADE
         );
         CREATE INDEX IF NOT EXISTS idx_prompt_backups_batch ON prompt_backups(backup_batch_id);
         CREATE INDEX IF NOT EXISTS idx_prompt_backups_restored ON prompt_backups(is_restored, restored_at);

         CREATE TABLE IF NOT EXISTS green_projects (
             id TEXT PRIMARY KEY,
             project_identifier TEXT NOT NULL UNIQUE,
             project_path TEXT NOT NULL,
             added_at INTEGER NOT NULL,
             status TEXT NOT NULL DEFAULT 'pending',
             last_checked_at INTEGER
         );",
    )
    .map_err(|e| format!("Failed to initialize backup prompts tables: {}", e))?;

    // Migrate existing DB if instance_id is missing
    let _ = conn.execute(
        "ALTER TABLE prompt_backups ADD COLUMN instance_id TEXT DEFAULT 'default'",
        [],
    );
    let _ = conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_prompt_backups_instance ON prompt_backups(instance_id)",
        [],
    );

    conn.execute(
        "CREATE TABLE IF NOT EXISTS agm_schema_migrations (
             name TEXT PRIMARY KEY,
             applied_at INTEGER NOT NULL
         )",
        [],
    )
    .map_err(|e| format!("Failed to create agm_schema_migrations table: {}", e))?;

    Ok(())
}

/// One unrestored backup per (instance, prompt). Created after the v140 migration.
pub(crate) fn create_backup_identity_index(conn: &Connection) {
    if let Err(e) = conn.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_prompt_backups_identity
         ON prompt_backups(instance_id, prompt_id) WHERE is_restored = 0",
        [],
    ) {
        crate::modules::logger::log_error(&format!(
            "[BackupDB] Failed to create idx_prompt_backups_identity: {}",
            e
        ));
    }
}

const BACKUP_MIGRATION_V140: &str = "v140_instance_ids";

fn is_backup_migration_applied(conn: &Connection) -> bool {
    conn.query_row(
        "SELECT 1 FROM agm_schema_migrations WHERE name = ?1",
        params![BACKUP_MIGRATION_V140],
        |_| Ok(()),
    )
    .is_ok()
}

fn count_legacy_backup_rows(conn: &Connection) -> i64 {
    conn.query_row(
        "SELECT COUNT(*) FROM prompt_backups
         WHERE instance_id IS NULL OR instance_id IN ('', '__default__', 'all')",
        [],
        |r| r.get(0),
    )
    .unwrap_or(0)
}

/// Conversation id -> instance ids that have it in their own gemini dirs.
fn backup_conversation_owner_index() -> HashMap<String, BTreeSet<String>> {
    let mut owners: HashMap<String, BTreeSet<String>> = HashMap::new();
    for (instance_id, dir) in repo_db::gemini_dirs_tagged(&InstanceScope::All) {
        let summaries_db = dir.join("conversation_summaries.db");
        if summaries_db.exists() {
            if let Ok(summaries) = Connection::open_with_flags(
                &summaries_db,
                rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
            ) {
                if let Ok(mut stmt) =
                    summaries.prepare("SELECT conversation_id FROM conversation_summaries")
                {
                    if let Ok(rows) = stmt.query_map([], |r| r.get::<_, String>(0)) {
                        for cid in rows.flatten() {
                            owners.entry(cid).or_default().insert(instance_id.clone());
                        }
                    }
                }
            }
        }
        if let Ok(entries) = fs::read_dir(dir.join("brain")) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    owners
                        .entry(entry.file_name().to_string_lossy().to_string())
                        .or_default()
                        .insert(instance_id.clone());
                }
            }
        }
    }
    owners
}

/// Thin loader: guard, `.pre-v140.bak` copy, registry and gemini dirs, then the pure core.
fn migrate_backup_instance_ids_v140(conn: &Connection, db_path: &Path) -> Result<usize, String> {
    if is_backup_migration_applied(conn) {
        return Ok(0);
    }
    if count_legacy_backup_rows(conn) == 0 {
        return migrate_backup_instance_ids_v140_core(conn, "default", &HashMap::new());
    }
    let backup_path = PathBuf::from(format!("{}.pre-v140.bak", db_path.to_string_lossy()));
    if let Err(e) = conn.backup(rusqlite::DatabaseName::Main, &backup_path, None) {
        crate::modules::logger::log_error(&format!(
            "[Migration v140] backup copy failed; migration skipped: {}",
            e
        ));
        return Err(format!("backup copy failed: {}", e));
    }
    let default_id = crate::modules::instance::canonical_instance_id("default")?;
    let owners = backup_conversation_owner_index();
    migrate_backup_instance_ids_v140_core(conn, &default_id, &owners)
}

/// Pure core of the backup DB v140 migration. Runs once per database.
/// Rows whose conversation has exactly one owner get that owner; all other legacy rows
/// become orphaned and restored (with `restored_at = NULL`) so no restore ever picks them.
pub(crate) fn migrate_backup_instance_ids_v140_core(
    conn: &Connection,
    default_id: &str,
    owners: &HashMap<String, BTreeSet<String>>,
) -> Result<usize, String> {
    if is_backup_migration_applied(conn) {
        return Ok(0);
    }
    let now = Utc::now().timestamp();
    let tx = conn
        .unchecked_transaction()
        .map_err(|e| format!("Failed to open backup migration transaction: {}", e))?;

    let mut changed = tx
        .execute(
            "UPDATE prompt_backups SET instance_id = ?1 WHERE instance_id = '__default__'",
            params![default_id],
        )
        .map_err(|e| format!("Failed to map default alias backup rows: {}", e))?;

    let mut legacy_rows: Vec<(String, String)> = Vec::new();
    {
        let mut stmt = tx
            .prepare(
                "SELECT id, conversation_id FROM prompt_backups
                 WHERE instance_id IS NULL OR instance_id IN ('', 'all')",
            )
            .map_err(|e| format!("Failed to prepare legacy backup query: {}", e))?;
        let rows = stmt
            .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| format!("Failed to query legacy backup rows: {}", e))?;
        legacy_rows.extend(rows.flatten());
    }

    for (row_id, conversation_id) in legacy_rows {
        let unique_owner = owners
            .get(conversation_id.trim())
            .filter(|ids| ids.len() == 1)
            .and_then(|ids| ids.iter().next());
        let updated = match unique_owner {
            Some(owner) => tx.execute(
                "UPDATE prompt_backups SET instance_id = ?1 WHERE id = ?2",
                params![owner, row_id],
            ),
            None => tx.execute(
                "UPDATE prompt_backups
                 SET instance_id = '', status = 'orphaned', is_restored = 1, restored_at = NULL
                 WHERE id = ?1",
                params![row_id],
            ),
        }
        .map_err(|e| format!("Failed to migrate backup row {}: {}", row_id, e))?;
        changed += updated;
    }

    changed += tx
        .execute(
            "UPDATE prompt_backups SET is_restored = 1, restored_at = ?1
             WHERE is_restored = 0
               AND rowid NOT IN (
                   SELECT MAX(rowid) FROM prompt_backups
                   WHERE is_restored = 0
                   GROUP BY instance_id, prompt_id
               )",
            params![now],
        )
        .map_err(|e| format!("Failed to collapse duplicate backup rows: {}", e))?;

    tx.execute(
        "INSERT INTO agm_schema_migrations (name, applied_at) VALUES (?1, ?2)",
        params![BACKUP_MIGRATION_V140, now],
    )
    .map_err(|e| format!("Failed to record backup migration: {}", e))?;
    tx.commit()
        .map_err(|e| format!("Failed to commit backup migration: {}", e))?;

    if changed > 0 {
        crate::modules::logger::log_info(&format!(
            "[Migration v140] backup prompts: {} rows migrated",
            changed
        ));
    }
    Ok(changed)
}
```

### C. Replace the dedupe block (everything from `// Deduplicate records to prevent duplicate reinjections into prompt_backups` down to and including `.map_err(|e| format!("Failed to insert prompt backup record: {}", e))?;`) with

```rust
        upsert_prompt_backup_record(&conn, &record, target_inst, now)?;
```

The next line must still be `        records.push(record);`. The removed `let is_live_prompt = ...` was never read anywhere (check with `gitmap aum search "is_live_prompt" src-tauri/src/modules/backup_prompts_db.rs --ext .rs`; after the edit it must return 0 hits).

### D. Insert this function between `Ok((batch_info, records))` + `}` and `/// List all backup batches and total counts`

```rust
/// Insert one backup record, or refresh the unrestored or restored row of the same instance.
/// Dedupe key: (instance_id, prompt_id) first, then (instance_id, project_path, prompt_text).
/// A row owned by another instance is never updated.
pub(crate) fn upsert_prompt_backup_record(
    conn: &Connection,
    record: &PromptBackupRecord,
    target_inst: &str,
    now: i64,
) -> Result<(), String> {
    let existing_unrestored: Option<String> = conn
        .query_row(
            "SELECT id FROM prompt_backups
             WHERE instance_id = ?4
               AND (prompt_id = ?1 OR (project_path = ?2 AND prompt_text = ?3))
               AND is_restored = 0
             LIMIT 1",
            params![record.prompt_id, record.project_path, record.prompt_text, target_inst],
            |r| r.get(0),
        )
        .ok();

    if let Some(existing_rec_id) = existing_unrestored {
        let _ = conn.execute(
            "UPDATE prompt_backups SET backup_batch_id = ?1, created_at = ?2, is_restored = 0 WHERE id = ?3",
            params![record.backup_batch_id, now, existing_rec_id],
        );
        return Ok(());
    }

    let existing_restored: Option<String> = conn
        .query_row(
            "SELECT id FROM prompt_backups
             WHERE instance_id = ?4
               AND status <> 'orphaned'
               AND (prompt_id = ?1 OR (project_path = ?2 AND prompt_text = ?3))
             LIMIT 1",
            params![record.prompt_id, record.project_path, record.prompt_text, target_inst],
            |r| r.get(0),
        )
        .ok();

    if let Some(existing_rec_id) = existing_restored {
        let _ = conn.execute(
            "UPDATE prompt_backups SET backup_batch_id = ?1, created_at = ?2, is_restored = 0, restored_at = NULL WHERE id = ?3",
            params![record.backup_batch_id, now, existing_rec_id],
        );
        return Ok(());
    }

    conn.execute(
        "INSERT INTO prompt_backups (
            id, backup_batch_id, prompt_id, project_name, project_path, project_id,
            conversation_id, conversation_name, sequence_id, prompt_text, has_images,
            images_payload, status, created_at, is_restored, restored_at, instance_id
         ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            record.id,
            record.backup_batch_id,
            record.prompt_id,
            record.project_name,
            record.project_path,
            record.project_id,
            record.conversation_id,
            record.conversation_name,
            record.sequence_id,
            record.prompt_text,
            record.has_images,
            record.images_payload,
            record.status,
            record.created_at,
            record.is_restored,
            record.restored_at,
            record.instance_id.as_deref().unwrap_or(target_inst),
        ],
    )
    .map_err(|e| format!("Failed to insert prompt backup record: {}", e))?;

    Ok(())
}
```

Compiler notes you may hit:

- `conn.backup(...)` needs the rusqlite `backup` feature. It is already enabled in `src-tauri/Cargo.toml` (`features = ["bundled", "backup"]`). Do not edit `Cargo.toml`.
- If the compiler says `the type of the progress argument cannot be inferred` on `conn.backup(..., None)`, change `None` to `None::<fn(rusqlite::backup::Progress)>` on that line only.

## 6. Tests

Paste these inside `mod tests` of `src-tauri/src/modules/backup_prompts_db.rs`, after the closing `}` of `fn test_backup_db_lifecycle()` and before the final `}` of the module. They use only `Connection::open_in_memory()` plus `init_backup_tables`, never the registry, never real data folders.

```rust
    fn seed_backup_batch(conn: &Connection, batch_id: &str) {
        conn.execute(
            "INSERT INTO backup_batches (id, created_at, prompts_count, file_path, retention_days, is_fully_restored)
             VALUES (?1, 1, 1, 'memory', 1, 0)",
            params![batch_id],
        )
        .unwrap();
    }

    fn backup_test_record(
        id: &str,
        batch_id: &str,
        prompt_id: &str,
        instance_id: &str,
    ) -> PromptBackupRecord {
        PromptBackupRecord {
            id: id.to_string(),
            backup_batch_id: batch_id.to_string(),
            prompt_id: prompt_id.to_string(),
            project_name: "app".to_string(),
            project_path: "D:/work/app".to_string(),
            project_id: "app-1".to_string(),
            conversation_id: "conv-1".to_string(),
            conversation_name: None,
            sequence_id: 1,
            prompt_text: "fix the build".to_string(),
            has_images: false,
            images_payload: None,
            status: "running".to_string(),
            created_at: 1,
            is_restored: false,
            restored_at: None,
            instance_id: Some(instance_id.to_string()),
        }
    }

    fn seed_legacy_backup(
        conn: &Connection,
        id: &str,
        conversation_id: &str,
        instance_id: Option<&str>,
    ) {
        conn.execute(
            "INSERT INTO prompt_backups (id, backup_batch_id, prompt_id, project_name, project_path, project_id,
                conversation_id, sequence_id, prompt_text, created_at, is_restored, instance_id)
             VALUES (?1, 'b-legacy', ?1, 'app', 'D:/work/app', 'app-1', ?2, 1, 'legacy text', 1, 0, ?3)",
            params![id, conversation_id, instance_id],
        )
        .unwrap();
    }

    fn backup_row_state(conn: &Connection, id: &str) -> (String, String, bool, Option<i64>) {
        conn.query_row(
            "SELECT IFNULL(instance_id, ''), status, is_restored, restored_at FROM prompt_backups WHERE id = ?1",
            params![id],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap()
    }

    fn backup_owner_index(pairs: &[(&str, &[&str])]) -> HashMap<String, BTreeSet<String>> {
        pairs
            .iter()
            .map(|(cid, ids)| {
                (
                    cid.to_string(),
                    ids.iter().map(|id| id.to_string()).collect(),
                )
            })
            .collect()
    }

    #[test]
    fn backup_dedupe_does_not_steal_other_instance_row() {
        let conn = Connection::open_in_memory().unwrap();
        init_backup_tables(&conn).unwrap();
        seed_backup_batch(&conn, "b-a");
        seed_backup_batch(&conn, "b-b");

        let rec_a = backup_test_record("rec-a", "b-a", "prompt-inst-a-conv-1", "inst-a");
        upsert_prompt_backup_record(&conn, &rec_a, "inst-a", 100).unwrap();
        let rec_b = backup_test_record("rec-b", "b-b", "prompt-inst-b-conv-1", "inst-b");
        upsert_prompt_backup_record(&conn, &rec_b, "inst-b", 200).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM prompt_backups", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 2);
        let (inst_a, _, _, _) = backup_row_state(&conn, "rec-a");
        assert_eq!(inst_a, "inst-a");
        let batch_a: String = conn
            .query_row(
                "SELECT backup_batch_id FROM prompt_backups WHERE id = 'rec-a'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(batch_a, "b-a");
        let (inst_b, _, _, _) = backup_row_state(&conn, "rec-b");
        assert_eq!(inst_b, "inst-b");
    }

    #[test]
    fn backup_dedupe_same_prompt_id_in_two_instances_gives_two_rows() {
        let conn = Connection::open_in_memory().unwrap();
        init_backup_tables(&conn).unwrap();
        seed_backup_batch(&conn, "b-a");
        seed_backup_batch(&conn, "b-b");

        let rec_a = backup_test_record("rec-a", "b-a", "prompt-shared", "inst-a");
        upsert_prompt_backup_record(&conn, &rec_a, "inst-a", 100).unwrap();
        let rec_b = backup_test_record("rec-b", "b-b", "prompt-shared", "inst-b");
        upsert_prompt_backup_record(&conn, &rec_b, "inst-b", 200).unwrap();

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM prompt_backups WHERE prompt_id = 'prompt-shared'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 2);
    }

    #[test]
    fn backup_dedupe_same_instance_reuses_row() {
        let conn = Connection::open_in_memory().unwrap();
        init_backup_tables(&conn).unwrap();
        seed_backup_batch(&conn, "b-1");
        seed_backup_batch(&conn, "b-2");

        let first = backup_test_record("rec-1", "b-1", "prompt-inst-a-conv-1", "inst-a");
        upsert_prompt_backup_record(&conn, &first, "inst-a", 100).unwrap();
        let second = backup_test_record("rec-2", "b-2", "prompt-inst-a-conv-1", "inst-a");
        upsert_prompt_backup_record(&conn, &second, "inst-a", 200).unwrap();

        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM prompt_backups", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
        let (batch, created_at): (String, i64) = conn
            .query_row(
                "SELECT backup_batch_id, created_at FROM prompt_backups WHERE id = 'rec-1'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(batch, "b-2");
        assert_eq!(created_at, 200);
    }

    #[test]
    fn backup_migration_assigns_unique_owner() {
        let conn = Connection::open_in_memory().unwrap();
        init_backup_tables(&conn).unwrap();
        seed_backup_batch(&conn, "b-legacy");
        seed_legacy_backup(&conn, "legacy-1", "cid-a", Some(""));
        seed_legacy_backup(&conn, "legacy-2", "cid-a2", None);
        let owners = backup_owner_index(&[("cid-a", &["inst-a"]), ("cid-a2", &["inst-a"])]);

        let changed = migrate_backup_instance_ids_v140_core(&conn, "default", &owners).unwrap();
        assert!(changed >= 2);
        let (inst_1, status_1, restored_1, _) = backup_row_state(&conn, "legacy-1");
        assert_eq!(inst_1, "inst-a");
        assert_ne!(status_1, "orphaned");
        assert!(!restored_1);
        let (inst_2, _, restored_2, _) = backup_row_state(&conn, "legacy-2");
        assert_eq!(inst_2, "inst-a");
        assert!(!restored_2);
    }

    #[test]
    fn backup_migration_orphans_unknown_and_ambiguous_rows() {
        let conn = Connection::open_in_memory().unwrap();
        init_backup_tables(&conn).unwrap();
        seed_backup_batch(&conn, "b-legacy");
        seed_legacy_backup(&conn, "legacy-ambiguous", "cid-both", Some(""));
        seed_legacy_backup(&conn, "legacy-unknown", "cid-none", Some("all"));
        let owners = backup_owner_index(&[("cid-both", &["inst-a", "inst-b"])]);

        migrate_backup_instance_ids_v140_core(&conn, "default", &owners).unwrap();
        for id in ["legacy-ambiguous", "legacy-unknown"] {
            let (inst, status, restored, restored_at) = backup_row_state(&conn, id);
            assert_eq!(inst, "");
            assert_eq!(status, "orphaned");
            assert!(restored);
            assert_eq!(restored_at, None);
        }
    }

    #[test]
    fn backup_migration_maps_default_alias() {
        let conn = Connection::open_in_memory().unwrap();
        init_backup_tables(&conn).unwrap();
        seed_backup_batch(&conn, "b-legacy");
        seed_legacy_backup(&conn, "legacy-default", "cid-x", Some("__default__"));

        migrate_backup_instance_ids_v140_core(&conn, "default", &HashMap::new()).unwrap();
        let (inst, status, restored, _) = backup_row_state(&conn, "legacy-default");
        assert_eq!(inst, "default");
        assert_ne!(status, "orphaned");
        assert!(!restored);
    }

    #[test]
    fn backup_migration_is_idempotent() {
        let conn = Connection::open_in_memory().unwrap();
        init_backup_tables(&conn).unwrap();
        seed_backup_batch(&conn, "b-legacy");
        seed_legacy_backup(&conn, "legacy-1", "cid-a", Some(""));
        let owners = backup_owner_index(&[("cid-a", &["inst-a"])]);

        migrate_backup_instance_ids_v140_core(&conn, "default", &owners).unwrap();
        seed_legacy_backup(&conn, "legacy-late", "cid-a", Some(""));
        let second = migrate_backup_instance_ids_v140_core(&conn, "default", &owners).unwrap();
        assert_eq!(second, 0);
        let (inst_late, _, _, _) = backup_row_state(&conn, "legacy-late");
        assert_eq!(inst_late, "");
    }

    #[test]
    fn backup_migration_aborts_when_backup_copy_fails() {
        let conn = Connection::open_in_memory().unwrap();
        init_backup_tables(&conn).unwrap();
        seed_backup_batch(&conn, "b-legacy");
        seed_legacy_backup(&conn, "legacy-1", "cid-a", Some(""));
        let temp = tempfile::tempdir().unwrap();
        let unwritable = temp.path().join("missing-dir").join("backup-prompts.db");

        let result = migrate_backup_instance_ids_v140(&conn, &unwritable);
        assert!(result.is_err());
        assert!(!is_backup_migration_applied(&conn));
        let (inst, status, restored, _) = backup_row_state(&conn, "legacy-1");
        assert_eq!(inst, "");
        assert_ne!(status, "orphaned");
        assert!(!restored);
    }

    #[test]
    fn backup_identity_index_exists_after_init() {
        let conn = Connection::open_in_memory().unwrap();
        init_backup_tables(&conn).unwrap();
        migrate_backup_instance_ids_v140_core(&conn, "default", &HashMap::new()).unwrap();
        create_backup_identity_index(&conn);
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = 'idx_prompt_backups_identity'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(count, 1);
    }
```

Why `backup_migration_aborts_when_backup_copy_fails` never touches the registry: the loader returns at the failed copy, before `canonical_instance_id` and `backup_conversation_owner_index` are called.

## 7. Gate

From the repo root:

```text
$env:CARGO_TARGET_DIR="$PWD\src-tauri\target"; $env:CARGO_BUILD_JOBS="1"
cd src-tauri; cargo fmt; cargo fmt -- --check; cargo clippy --all-targets --all-features; cargo test --lib -- --test-threads=1 backup_dedupe backup_migration backup_identity_index; cd ..
```

All commands must exit 0. Do not add `test_backup_db_lifecycle` to the filter (it writes to the real data folder through `restore_running_prompts`).

## 8. Commit

Stage only `src-tauri/src/modules/backup_prompts_db.rs`.

```text
Fix: prompts - backup DB v140 migration and per-instance dedupe
```

## 9. Done when

- [ ] `connect_backup_db` calls `init_backup_tables`, then `migrate_backup_instance_ids_v140`, then `create_backup_identity_index`, in that order.
- [ ] `agm_schema_migrations` exists in the backup DB and gets the row `v140_instance_ids` exactly once.
- [ ] A failed `.pre-v140.bak` copy returns `Err` and changes no row.
- [ ] Legacy rows with no unique owner end as `instance_id = ''`, `status = 'orphaned'`, `is_restored = 1`, `restored_at = NULL`.
- [ ] Neither dedupe `UPDATE` writes `instance_id` any more (`gitmap aum search "instance_id = ?3, is_restored" src-tauri/src/modules/backup_prompts_db.rs --ext .rs` returns 0 hits).
- [ ] Both dedupe lookups contain `instance_id = ?4`, and the restored lookup no longer contains `instance_id IS NULL OR instance_id = ''`.
- [ ] `idx_prompt_backups_identity` is created.
- [ ] The candidate selection (`is_inst_match`) and the conversation matching (`matched_conv`) in `backup_active_running_prompts_for_instance` are unchanged; step 15 owns them.
- [ ] The caller searches in section 3 were run on `src-tauri/src`, `src-tauri/src/bin/agm.rs` and each `src-tauri/tests/*.rs` file; no caller outside `backup_prompts_db.rs` needed a change.
- [ ] Gate exits 0. Only `src-tauri/src/modules/backup_prompts_db.rs` is staged.
