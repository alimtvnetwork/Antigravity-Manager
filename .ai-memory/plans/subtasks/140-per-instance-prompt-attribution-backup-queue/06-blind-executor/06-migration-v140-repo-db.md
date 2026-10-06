# Step 06: one-time migration v140 for the repo DB

Legacy rows carry `instance_id` values `'__default__'`, `''` or `'all'`, captured ids look like `prompt-{cid}`, Layer 2 rows store the project id in `session_id`, and `agm_conversation_sequences` is keyed by `conversation_id` alone. This step adds one migration that fixes all of that exactly once, after a backup copy, inside one transaction, and then creates the identity index.

Order (fail before the point of no return):

1. If `agm_schema_migrations` has `v140_instance_ids`, return `Ok(0)`. Nothing else happens.
2. Copy the database with the SQLite backup API to `<db>.pre-v140.bak` (or `<db>.pre-v140.<unix>.bak` if that file already exists). If the copy fails, log `[Migration v140] backup copy failed; migration skipped`, return `Err`, change nothing.
3. Open one `IMMEDIATE` transaction and re-check the marker (another process may have finished first).
4. `'__default__'` becomes the default id in `active_prompts`, `running_projects`, `agm_project_sequences`, `agm_conversation_sequences`.
5. `session_id` that equals the project id (Layer 2 bug B10) becomes `NULL`.
6. Status `pending` becomes `queued`.
7. Captured rows that already have a real owner get the new id format `prompt-{instance}-{cid}` when that id is free.
8. Rows with `instance_id` `NULL`, `''` or `'all'`: exactly one owner for `session_id` in the owner index sets `instance_id` (and the new id format for `prompt-{cid}` ids). Zero owners: `status = 'orphaned'`, `status_reason = 'legacy_owner_unknown'`. Two or more owners, or the new id is taken: `status = 'orphaned'`, `status_reason = 'legacy_owner_ambiguous'`. Orphans keep `instance_id = ''` (DR-7).
9. Remaining duplicates of `(instance_id, session_id)` among `running` / `backed_up` rows: the newest stays, older ones become `orphaned` with `status_reason = 'legacy_duplicate'`. Without this, the unique index in the next step cannot be created.
10. Create `idx_active_prompts_identity`.
11. Rebuild `agm_conversation_sequences` with `PRIMARY KEY (instance_id, conversation_id)` and a globally unique `seq_id`.
12. Insert the marker row and commit.

The migration runs from `connect_db`, not from `init_tables`, because it needs the registry and the gemini dirs on disk. Tests call only the pure core `migrate_prompt_instance_ids_v140_core` with an in-memory database and an owner index built in the test.

The same step removes every `INSERT OR REPLACE INTO active_prompts`. SQLite `REPLACE` deletes **every** row that conflicts on **any** unique index, so once `idx_active_prompts_identity` exists, a `REPLACE` of row B with the same `(instance_id, session_id)` as running row A would silently delete A. Each site becomes `INSERT ... ON CONFLICT(id) DO UPDATE SET ...` that never writes `instance_id` (a row is never re-owned by another instance), and a conflict on the identity index now fails loudly instead of deleting a row.

## 1. Depends on

Steps 03 and 04. (Run it after step 05 as the step table orders it; it does not use any step 05 code.)

## 2. Files you may edit

- `src-tauri/src/modules/repo_db.rs`
- `src-tauri/src/modules/instance.rs` (only the three `INSERT OR REPLACE INTO active_prompts` statements inside the ignored test `test_local_e2e_instance_switch_and_prompt_restore`, change H)

## 3. Find it

| Change | Anchor | Unique search literal | Line hint (before steps 03 to 05; add about 120) |
|---|---|---|---|
| A. Call the migration | `pub fn connect_db() -> Result<Connection, String> {` | `init_tables(&conn)?;` | 149 to 160 |
| B. New sequences schema for fresh DBs | `fn init_tables(conn: &Connection) -> Result<(), String> {` | `CREATE TABLE IF NOT EXISTS agm_conversation_sequences` | 225 to 236 |
| C. Migration functions | after `init_tables` | `/// Purge corrupted and un-namespaced running_projects rows and clear stale prompt tree cache` | 310 |
| D. Per-instance sequence lookups | `fn ensure_conversation_sequence_in_conn(` | `fn ensure_conversation_sequence_in_conn(` | 3777 to 3817 |
| E. Tests | `mod tests {` | `fn test_uri_decoding()` | end of file |
| F. Upsert in `save_or_requeue_prompt` | `pub fn save_or_requeue_prompt(prompt: &ActivePrompt) -> Result<(), String> {` | `"INSERT OR REPLACE INTO active_prompts` (the hit inside this fn) | 2856 to 2873 |
| G. Upsert in `auto_resume_recent_prompts` | `// Mark / update status in active_prompts as dispatched` | `// Mark / update status in active_prompts as dispatched` | 3613 to 3630 |
| H. Test-only upserts | `fn test_local_e2e_instance_switch_and_prompt_restore()` in `instance.rs` | `"INSERT OR REPLACE INTO active_prompts` (3 hits in `instance.rs`) | 7090 to 7143 |

```text
gitmap aum search "init_tables(&conn)?;" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "CREATE TABLE IF NOT EXISTS agm_conversation_sequences" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "Purge corrupted and un-namespaced running_projects rows" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "fn ensure_conversation_sequence_in_conn" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "agm_conversation_sequences" src-tauri/src --ext .rs
gitmap aum search "agm_conversation_sequences" src-tauri/src/bin/agm.rs --ext .rs
gitmap aum search "Mark / update status in active_prompts as dispatched" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "INSERT OR REPLACE INTO active_prompts" src-tauri/src --ext .rs
gitmap aum search "INSERT OR REPLACE INTO active_prompts" src-tauri/src/bin/agm.rs --ext .rs
gitmap aum search "INSERT OR REPLACE INTO active_prompts" src-tauri/tests/auto_switcher_e2e_test.rs --ext .rs
gitmap aum search "INSERT OR REPLACE INTO active_prompts" src-tauri/tests/instance_cloning_and_sync_test.rs --ext .rs
gitmap aum search "INSERT OR REPLACE INTO active_prompts" src-tauri/tests/per_instance_prompt_liveness_test.rs --ext .rs
```

The two `agm_conversation_sequences` searches must show only `repo_db.rs` hits (the `CREATE TABLE` and the five queries inside `ensure_conversation_sequence_in_conn`) and 0 hits in `agm.rs`. If another file reads `agm_conversation_sequences`, STOP and report it.

The `INSERT OR REPLACE INTO active_prompts` search over `src-tauri/src` must show exactly 5 hits: `repo_db.rs` inside `save_or_requeue_prompt` (F) and inside `auto_resume_recent_prompts` (G), and 3 in `instance.rs` (H). The `agm.rs` and the three `src-tauri/tests` searches must show 0 hits (verified). If a hit exists anywhere else, STOP and report it. `INSERT OR REPLACE` into other tables (`running_projects`, `ItemTable`, ...) is out of scope.

## 4. Current code

### A. `connect_db` (lines 148 to 160)

```rust
/// Connect to the repo SQLite database with WAL mode and 5000ms busy timeout
pub fn connect_db() -> Result<Connection, String> {
    let path = get_repo_db_path()?;
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let conn =
        Connection::open(&path).map_err(|e| format!("Failed to open repo database: {}", e))?;
    let _ = conn.pragma_update(None, "journal_mode", "WAL");
    let _ = conn.pragma_update(None, "busy_timeout", 5000);
    init_tables(&conn)?;
    Ok(conn)
}
```

### B. Sequences table in `init_tables` (lines 225 to 236)

```rust
    conn.execute(
        "CREATE TABLE IF NOT EXISTS agm_conversation_sequences (
            conversation_id TEXT PRIMARY KEY,
            seq_id INTEGER NOT NULL UNIQUE,
            project_key TEXT NOT NULL,
            title TEXT NOT NULL,
            instance_id TEXT NOT NULL DEFAULT 'default',
            updated_at INTEGER NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create agm_conversation_sequences table: {}", e))?;
```

### C. Insertion point: the end of `init_tables` and the start of the next function (lines 305 to 311)

```rust
    });

    Ok(())
}

/// Purge corrupted and un-namespaced running_projects rows and clear stale prompt tree cache
pub fn purge_corrupted_running_projects(conn: &Connection) -> Result<(usize, usize), String> {
```

### D. `ensure_conversation_sequence_in_conn` (lines 3777 to 3817)

```rust
fn ensure_conversation_sequence_in_conn(
    conn: &Connection,
    conversation_id: &str,
    project_key: &str,
    title: &str,
    instance_id: &str,
) -> i64 {
    let now = Utc::now().timestamp();
    if let Ok(existing) = conn.query_row(
        "SELECT seq_id FROM agm_conversation_sequences WHERE conversation_id = ?1",
        params![conversation_id],
        |row| row.get::<_, i64>(0),
    ) {
        let _ = conn.execute(
            "UPDATE agm_conversation_sequences SET project_key = ?2, title = ?3, instance_id = ?4, updated_at = ?5 WHERE conversation_id = ?1",
            params![conversation_id, project_key, title, instance_id, now],
        );
        return existing;
    }

    let next_seq: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(seq_id), 0) + 1 FROM agm_conversation_sequences",
            [],
            |row| row.get(0),
        )
        .unwrap_or(1);

    let _ = conn.execute(
        "INSERT OR IGNORE INTO agm_conversation_sequences (conversation_id, seq_id, project_key, title, instance_id, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![conversation_id, next_seq, project_key, title, instance_id, now],
    );

    conn.query_row(
        "SELECT seq_id FROM agm_conversation_sequences WHERE conversation_id = ?1",
        params![conversation_id],
        |row| row.get(0),
    )
    .unwrap_or(next_seq)
}
```

Its callers (`repo_db.rs:4501`, `:4608`, and the test at `:6413`) keep the same signature; no caller changes.

### F. The insert at the end of `save_or_requeue_prompt`, as step 03 left it (lines 2856 to 2873 before step 03)

This is step 03's new code 5 C, followed by the existing `Ok(())` and closing brace:

```rust
    conn.execute(
        "INSERT OR REPLACE INTO active_prompts 
         (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload, source_dir)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        params![
            &prompt.id,
            &composite_proj_id,
            &canonical_inst,
            &prompt.repo_path,
            &prompt.prompt_content,
            &prompt.model,
            &prompt.session_id,
            &prompt.status,
            prompt.created_at,
            now,
            &prompt.image_payload,
            &prompt.source_dir,
        ],
    ).map_err(|e| format!("Failed to insert active prompt: {}", e))?;

    Ok(())
}
```

The signature `pub fn save_or_requeue_prompt(prompt: &ActivePrompt) -> Result<(), String>` does not change, so no caller changes. Callers (all searched; they keep compiling): `backup_prompts_db.rs:596`, `telegram_inbound.rs:2443`, `repo_db.rs:5112`, `repo_db.rs:5445`, `agm.rs:4563`, `agm.rs:14365`, `agm.rs:14366`, `agm.rs:14367`. `src-tauri/tests/*.rs`: 0 callers. Most callers discard the result with `let _ =`, so the function itself must log a failure.

### G. `auto_resume_recent_prompts` dispatched marker (lines 3613 to 3630)

```rust
        // Mark / update status in active_prompts as dispatched
        let _ = conn.execute(
            "INSERT OR REPLACE INTO active_prompts 
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES (?, ?, ?, ?, ?, ?, ?, 'dispatched', ?, ?, ?)",
            params![
                &prompt_id,
                &project.id,
                instance_id,
                &project.repo_path,
                &prompt_text,
                &prompt_model,
                &project.id,
                now,
                now,
                &image_payload,
            ],
        );
```

Steps 09, 10 and 14 edit other lines of this function; none of them quotes this block. Step 09's change H1 ends at the `fs::write` block directly above it.

### H. `instance.rs`, test `test_local_e2e_instance_switch_and_prompt_restore` (lines 7090 to 7143)

The three statements are identical except for the params. Each one starts with these exact three SQL lines:

```rust
                "INSERT OR REPLACE INTO active_prompts 
                 (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
```

The test is `#[ignore = "local_only_e2e"]`; the row ids are `prompt-{unique_id}-1`, `-2`, `-3` with sessions `sess-1`, `sess-2`, `sess-3`, so the rows never collide on either key.

Facts you can rely on (already verified):

- `agm_schema_migrations(name TEXT PRIMARY KEY, applied_at INTEGER NOT NULL)` and the columns `status_reason`, `source_dir`, `attempts` are created by `init_tables` (step 03).
- `gemini_dirs_tagged_in`, `get_canonical_host_home`, `InstanceScope` exist in this file after step 04; `crate::modules::instance::default_instance_id_in`, `load_registry`, `get_instances_dir` exist after step 01.
- `rusqlite` is version 0.32 with features `bundled` and `backup` (`src-tauri/Cargo.toml` line 44), so `Connection::backup(DatabaseName::Main, path, None)` and `Transaction::new_unchecked(&Connection, TransactionBehavior::Immediate)` are available.
- The repo DB does not enable `PRAGMA foreign_keys`, so changing `active_prompts.id` does not trip the `FOREIGN KEY(project_id)` clause.

## 5. New code

### A. Replace `connect_db` from 4 A with

```rust
/// Connect to the repo SQLite database with WAL mode and 5000ms busy timeout
pub fn connect_db() -> Result<Connection, String> {
    let path = get_repo_db_path()?;
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let conn =
        Connection::open(&path).map_err(|e| format!("Failed to open repo database: {}", e))?;
    let _ = conn.pragma_update(None, "journal_mode", "WAL");
    let _ = conn.pragma_update(None, "busy_timeout", 5000);
    init_tables(&conn)?;
    if let Err(e) = migrate_prompt_instance_ids_v140(&conn, &path) {
        crate::modules::logger::log_warn(&format!(
            "[Migration v140] repo DB left unmigrated: {}",
            e
        ));
    }
    Ok(conn)
}
```

A failed migration never makes `connect_db` fail; the app keeps working on the unmigrated rows and the backup copy (if any) stays on disk.

### B. Replace the sequences `CREATE TABLE` from 4 B with

```rust
    conn.execute(
        "CREATE TABLE IF NOT EXISTS agm_conversation_sequences (
            conversation_id TEXT NOT NULL,
            seq_id INTEGER NOT NULL UNIQUE,
            project_key TEXT NOT NULL,
            title TEXT NOT NULL,
            instance_id TEXT NOT NULL DEFAULT 'default',
            updated_at INTEGER NOT NULL,
            PRIMARY KEY (instance_id, conversation_id)
        )",
        [],
    )
    .map_err(|e| format!("Failed to create agm_conversation_sequences table: {}", e))?;
```

This only affects brand-new databases. Existing databases keep the old table until the migration rebuilds it.

### C. Insert this block between the closing `}` of `init_tables` and `/// Purge corrupted and un-namespaced running_projects rows ...`

Keep one empty line before and after the block.

```rust
const V140_MIGRATION_NAME: &str = "v140_instance_ids";
static V140_MIGRATION_ATTEMPTED: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(false);

fn schema_migration_applied(conn: &Connection, name: &str) -> Result<bool, String> {
    conn.query_row(
        "SELECT COUNT(*) FROM agm_schema_migrations WHERE name = ?1",
        params![name],
        |row| row.get::<_, i64>(0),
    )
    .map(|count| count > 0)
    .map_err(|e| format!("Failed to read agm_schema_migrations: {}", e))
}

fn v140_backup_path(db_path: &Path) -> PathBuf {
    let base = db_path.to_string_lossy().to_string();
    let first = PathBuf::from(format!("{}.pre-v140.bak", base));
    if first.exists() {
        PathBuf::from(format!("{}.pre-v140.{}.bak", base, Utc::now().timestamp()))
    } else {
        first
    }
}

fn summary_conversation_ids(summaries_db: &Path) -> Vec<String> {
    let Ok(conn) = Connection::open_with_flags(
        summaries_db,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
    ) else {
        return Vec::new();
    };
    let Ok(mut stmt) = conn.prepare("SELECT conversation_id FROM conversation_summaries") else {
        return Vec::new();
    };
    let Ok(rows) = stmt.query_map([], |row| row.get::<_, String>(0)) else {
        return Vec::new();
    };
    rows.flatten().collect()
}

/// Conversation id -> ids of the instances whose gemini dirs contain it
/// (a `conversation_summaries.db` row or a `brain/<cid>` folder).
pub fn build_conversation_owner_index(
    tagged_dirs: &[(String, PathBuf)],
) -> HashMap<String, std::collections::BTreeSet<String>> {
    let mut index: HashMap<String, std::collections::BTreeSet<String>> = HashMap::new();
    for (owner_id, base_dir) in tagged_dirs {
        let mut cids = Vec::new();
        let summaries_db = base_dir.join("conversation_summaries.db");
        if summaries_db.exists() {
            cids.extend(summary_conversation_ids(&summaries_db));
        }
        if let Ok(entries) = fs::read_dir(base_dir.join("brain")) {
            for entry in entries.flatten() {
                if entry.path().is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        cids.push(name.to_string());
                    }
                }
            }
        }
        for cid in cids {
            let cid = cid.trim();
            if !cid.is_empty() {
                index
                    .entry(cid.to_string())
                    .or_default()
                    .insert(owner_id.clone());
            }
        }
    }
    index
}

/// One-time v140 migration of the repo DB. Loads the registry and gemini dirs, then runs the core.
pub fn migrate_prompt_instance_ids_v140(conn: &Connection, db_path: &Path) -> Result<usize, String> {
    if schema_migration_applied(conn, V140_MIGRATION_NAME)? {
        return Ok(0);
    }
    if V140_MIGRATION_ATTEMPTED.swap(true, std::sync::atomic::Ordering::SeqCst) {
        return Ok(0);
    }
    let registry = crate::modules::instance::load_registry()?;
    let instances_root = crate::modules::instance::get_instances_dir()?;
    let default_id = crate::modules::instance::default_instance_id_in(&registry);
    let host_home = get_canonical_host_home();
    let tagged = gemini_dirs_tagged_in(
        &registry,
        &InstanceScope::All,
        host_home.as_deref(),
        &instances_root,
    );
    let owner_index = build_conversation_owner_index(&tagged);
    let backup_path = v140_backup_path(db_path);
    let changed =
        migrate_prompt_instance_ids_v140_core(conn, &backup_path, &default_id, &owner_index)?;
    crate::modules::logger::log_info(&format!(
        "[Migration v140] repo DB migrated: {} row changes, backup at {}",
        changed,
        backup_path.display()
    ));
    Ok(changed)
}

/// Pure core of the v140 migration. Idempotent through `agm_schema_migrations`.
pub fn migrate_prompt_instance_ids_v140_core(
    conn: &Connection,
    backup_path: &Path,
    default_id: &str,
    owner_index: &HashMap<String, std::collections::BTreeSet<String>>,
) -> Result<usize, String> {
    if schema_migration_applied(conn, V140_MIGRATION_NAME)? {
        return Ok(0);
    }
    if let Err(e) = conn.backup(rusqlite::DatabaseName::Main, backup_path, None) {
        let message = format!(
            "[Migration v140] backup copy failed; migration skipped: {}",
            e
        );
        crate::modules::logger::log_error(&message);
        return Err(message);
    }

    let tx = rusqlite::Transaction::new_unchecked(conn, rusqlite::TransactionBehavior::Immediate)
        .map_err(|e| format!("[Migration v140] begin failed: {}", e))?;
    if schema_migration_applied(&tx, V140_MIGRATION_NAME)? {
        return Ok(0);
    }
    let sql_err = |e: rusqlite::Error| format!("[Migration v140] {}", e);
    let mut changed = 0usize;

    for table in [
        "active_prompts",
        "running_projects",
        "agm_project_sequences",
        "agm_conversation_sequences",
    ] {
        changed += tx
            .execute(
                &format!(
                    "UPDATE {} SET instance_id = ?1 WHERE instance_id = '__default__'",
                    table
                ),
                params![default_id],
            )
            .map_err(sql_err)?;
    }

    changed += tx
        .execute(
            "UPDATE active_prompts SET session_id = NULL
             WHERE session_id IS NOT NULL AND session_id <> ''
               AND (session_id = project_id
                    OR substr(project_id, 1, length(session_id) + 2) = session_id || '__')",
            [],
        )
        .map_err(sql_err)?;

    changed += tx
        .execute(
            "UPDATE active_prompts SET status = 'queued' WHERE status = 'pending'",
            [],
        )
        .map_err(sql_err)?;

    changed += tx
        .execute(
            "UPDATE active_prompts
             SET id = 'prompt-' || instance_id || '-' || session_id
             WHERE id = 'prompt-' || session_id
               AND instance_id NOT IN ('', 'all')
               AND session_id IS NOT NULL AND session_id <> ''
               AND NOT EXISTS (
                   SELECT 1 FROM active_prompts AS other
                   WHERE other.id = 'prompt-' || active_prompts.instance_id || '-' || active_prompts.session_id
               )",
            [],
        )
        .map_err(sql_err)?;

    let legacy_rows: Vec<(String, Option<String>)> = {
        let mut stmt = tx
            .prepare(
                "SELECT id, session_id FROM active_prompts
                 WHERE instance_id IS NULL OR instance_id IN ('', 'all')",
            )
            .map_err(sql_err)?;
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, Option<String>>(1)?))
            })
            .map_err(sql_err)?;
        rows.flatten().collect()
    };

    for (old_id, session_id) in legacy_rows {
        let cid = session_id.as_deref().map(str::trim).unwrap_or("");
        let owners = if cid.is_empty() {
            None
        } else {
            owner_index.get(cid)
        };
        let unique_owner = owners
            .filter(|set| set.len() == 1)
            .and_then(|set| set.iter().next().cloned());
        let reason = if let Some(owner) = unique_owner {
            let new_id = if old_id.strip_prefix("prompt-") == Some(cid) {
                format!("prompt-{}-{}", owner, cid)
            } else {
                old_id.clone()
            };
            let is_taken = new_id != old_id
                && tx
                    .query_row(
                        "SELECT COUNT(*) FROM active_prompts WHERE id = ?1",
                        params![new_id],
                        |row| row.get::<_, i64>(0),
                    )
                    .map_err(sql_err)?
                    > 0;
            if !is_taken {
                changed += tx
                    .execute(
                        "UPDATE active_prompts SET instance_id = ?1, id = ?2 WHERE id = ?3",
                        params![owner, new_id, old_id],
                    )
                    .map_err(sql_err)?;
                continue;
            }
            "legacy_owner_ambiguous"
        } else if owners.is_some_and(|set| set.len() > 1) {
            "legacy_owner_ambiguous"
        } else {
            "legacy_owner_unknown"
        };
        changed += tx
            .execute(
                "UPDATE active_prompts SET status = 'orphaned', status_reason = ?1, instance_id = ''
                 WHERE id = ?2",
                params![reason, old_id],
            )
            .map_err(sql_err)?;
    }

    changed += tx
        .execute(
            "UPDATE active_prompts
             SET status = 'orphaned', status_reason = 'legacy_duplicate'
             WHERE status IN ('running', 'backed_up')
               AND session_id IS NOT NULL AND session_id <> ''
               AND EXISTS (
                   SELECT 1 FROM active_prompts AS newer
                   WHERE newer.instance_id = active_prompts.instance_id
                     AND newer.session_id = active_prompts.session_id
                     AND newer.status IN ('running', 'backed_up')
                     AND (newer.updated_at > active_prompts.updated_at
                          OR (newer.updated_at = active_prompts.updated_at
                              AND newer.id > active_prompts.id))
               )",
            [],
        )
        .map_err(sql_err)?;

    tx.execute(
        "CREATE UNIQUE INDEX IF NOT EXISTS idx_active_prompts_identity
         ON active_prompts(instance_id, session_id)
         WHERE status IN ('running', 'backed_up') AND session_id IS NOT NULL AND session_id <> ''",
        [],
    )
    .map_err(sql_err)?;

    tx.execute_batch(
        "DROP TABLE IF EXISTS agm_conversation_sequences_v2;
         CREATE TABLE agm_conversation_sequences_v2 (
             conversation_id TEXT NOT NULL,
             seq_id INTEGER NOT NULL UNIQUE,
             project_key TEXT NOT NULL,
             title TEXT NOT NULL,
             instance_id TEXT NOT NULL DEFAULT 'default',
             updated_at INTEGER NOT NULL,
             PRIMARY KEY (instance_id, conversation_id)
         );
         INSERT INTO agm_conversation_sequences_v2
             (conversation_id, seq_id, project_key, title, instance_id, updated_at)
         SELECT conversation_id, seq_id, project_key, title, instance_id, updated_at
         FROM agm_conversation_sequences;
         DROP TABLE agm_conversation_sequences;
         ALTER TABLE agm_conversation_sequences_v2 RENAME TO agm_conversation_sequences;",
    )
    .map_err(sql_err)?;

    tx.execute(
        "INSERT OR IGNORE INTO agm_schema_migrations (name, applied_at) VALUES (?1, ?2)",
        params![V140_MIGRATION_NAME, Utc::now().timestamp()],
    )
    .map_err(sql_err)?;
    tx.commit().map_err(sql_err)?;
    Ok(changed)
}
```

Why each guard exists (do not remove any):

- The marker check before the backup makes the second run cheap and side-effect free.
- `V140_MIGRATION_ATTEMPTED` stops a failing migration from re-copying the database on every `connect_db` call in the same process, and stops re-entry if `load_registry` ever reaches `connect_db`.
- Any `?` after `new_unchecked` drops `tx`, which rolls the whole transaction back.

### D. Replace `ensure_conversation_sequence_in_conn` from 4 D with

```rust
fn ensure_conversation_sequence_in_conn(
    conn: &Connection,
    conversation_id: &str,
    project_key: &str,
    title: &str,
    instance_id: &str,
) -> i64 {
    let now = Utc::now().timestamp();
    if let Ok(existing) = conn.query_row(
        "SELECT seq_id FROM agm_conversation_sequences WHERE conversation_id = ?1 AND instance_id = ?2",
        params![conversation_id, instance_id],
        |row| row.get::<_, i64>(0),
    ) {
        let _ = conn.execute(
            "UPDATE agm_conversation_sequences SET project_key = ?2, title = ?3, updated_at = ?5 WHERE conversation_id = ?1 AND instance_id = ?4",
            params![conversation_id, project_key, title, instance_id, now],
        );
        return existing;
    }

    let next_seq: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(seq_id), 0) + 1 FROM agm_conversation_sequences",
            [],
            |row| row.get(0),
        )
        .unwrap_or(1);

    let _ = conn.execute(
        "INSERT OR IGNORE INTO agm_conversation_sequences (conversation_id, seq_id, project_key, title, instance_id, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![conversation_id, next_seq, project_key, title, instance_id, now],
    );

    conn.query_row(
        "SELECT seq_id FROM agm_conversation_sequences WHERE conversation_id = ?1 AND instance_id = ?2",
        params![conversation_id, instance_id],
        |row| row.get(0),
    )
    .unwrap_or(next_seq)
}
```

The `UPDATE` no longer re-owns a conversation (`instance_id = ?4` moved from `SET` to `WHERE`). The parameter list is unchanged, so `?4` is still `instance_id` and `?5` is still `now`.

### F. `save_or_requeue_prompt` upsert

F1. Insert this new function directly above the line `/// Save or re-queue an active prompt into active_prompts and running_projects` (keep one empty line before and after):

```rust
/// Writes one `active_prompts` row keyed by `id`. Never re-owns a row of another instance
/// (0 changed rows) and never deletes a row that conflicts on `idx_active_prompts_identity`
/// (SQLite error); both cases return `Err`.
pub(crate) fn upsert_active_prompt_row(
    conn: &Connection,
    prompt: &ActivePrompt,
    project_id: &str,
    instance_id: &str,
    now: i64,
) -> Result<(), String> {
    let written = conn
        .execute(
            "INSERT INTO active_prompts
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload, source_dir)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
             ON CONFLICT(id) DO UPDATE SET
                project_id = excluded.project_id,
                repo_path = excluded.repo_path,
                prompt_content = excluded.prompt_content,
                model = excluded.model,
                session_id = excluded.session_id,
                status = excluded.status,
                created_at = excluded.created_at,
                updated_at = excluded.updated_at,
                image_payload = excluded.image_payload,
                source_dir = excluded.source_dir,
                status_reason = NULL,
                attempts = 0
             WHERE active_prompts.instance_id = excluded.instance_id",
            params![
                &prompt.id,
                project_id,
                instance_id,
                &prompt.repo_path,
                &prompt.prompt_content,
                &prompt.model,
                &prompt.session_id,
                &prompt.status,
                prompt.created_at,
                now,
                &prompt.image_payload,
                &prompt.source_dir,
            ],
        )
        .map_err(|e| {
            format!(
                "Failed to insert active prompt '{}' for instance '{}': {}",
                prompt.id, instance_id, e
            )
        })?;
    if written == 0 {
        return Err(format!(
            "Active prompt '{}' belongs to another instance; not re-owned to '{}'",
            prompt.id, instance_id
        ));
    }
    Ok(())
}
```

F2. Replace the whole block from 4 F (from `    conn.execute(` through the closing `}` of `save_or_requeue_prompt`) with:

```rust
    if let Err(e) =
        upsert_active_prompt_row(&conn, prompt, &composite_proj_id, &canonical_inst, now)
    {
        crate::modules::logger::log_warn(&format!("[RepoDB] save_or_requeue_prompt: {}", e));
        return Err(e);
    }

    Ok(())
}
```

The `DO UPDATE SET` writes every column the old `REPLACE` wrote except `instance_id`. `status_reason = NULL, attempts = 0` keeps the old `REPLACE` result for those two columns (it re-inserted them with their defaults). Do not use `map_err` with a logging closure here (clippy `manual_inspect`).

### G. Replace the block from 4 G with

```rust
        // Mark / update status in active_prompts as dispatched
        if let Err(e) = conn.execute(
            "INSERT INTO active_prompts 
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'dispatched', ?8, ?9, ?10)
             ON CONFLICT(id) DO UPDATE SET
                project_id = excluded.project_id,
                repo_path = excluded.repo_path,
                prompt_content = excluded.prompt_content,
                model = excluded.model,
                session_id = excluded.session_id,
                status = excluded.status,
                created_at = excluded.created_at,
                updated_at = excluded.updated_at,
                image_payload = excluded.image_payload,
                status_reason = NULL,
                attempts = 0
             WHERE active_prompts.instance_id = excluded.instance_id",
            params![
                &prompt_id,
                &project.id,
                instance_id,
                &project.repo_path,
                &prompt_text,
                &prompt_model,
                &project.id,
                now,
                now,
                &image_payload,
            ],
        ) {
            crate::modules::logger::log_warn(&format!(
                "[RepoDB] auto_resume_recent_prompts could not mark prompt '{}' dispatched: {}",
                prompt_id, e
            ));
        }
```

The params are unchanged (10 values; `'dispatched'` stays a literal). `source_dir` is not written, so a stamped `source_dir` survives (the old `REPLACE` reset it to `NULL`). A row owned by another instance is left alone (0 changes, no log needed: dispatch still proceeds exactly as before).

### H. In `instance.rs`, in each of the three statements from 4 H

Replace the three SQL lines with:

```rust
                "INSERT INTO active_prompts 
                 (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                 ON CONFLICT(id) DO NOTHING",
```

Leave the `let _ = conn.execute(` lines and the `rusqlite::params![...]` lists unchanged. After the edit, the `INSERT OR REPLACE INTO active_prompts` searches from section 3 must return 0 hits in `src-tauri/src`, `agm.rs` and every `src-tauri/tests` file.

## 6. Tests

Paste after `    use super::*;` in `mod tests` of `repo_db.rs`. All of them use `Connection::open_in_memory()` plus `init_tables(&conn)`, a temp folder for the backup file, and an owner index built in the test. None loads the registry or touches environment variables.

```rust
    fn insert_v140_prompt_row(
        conn: &Connection,
        id: &str,
        instance_id: &str,
        session_id: Option<&str>,
        status: &str,
        updated_at: i64,
    ) {
        conn.execute(
            "INSERT INTO active_prompts
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES (?1, 'proj-v140', ?2, 'D:/work/app', 'legacy prompt', NULL, ?3, ?4, ?5, ?5, NULL)",
            params![id, instance_id, session_id, status, updated_at],
        )
        .unwrap();
    }

    fn v140_owner_index(
        entries: Vec<(&str, Vec<&str>)>,
    ) -> HashMap<String, std::collections::BTreeSet<String>> {
        entries
            .into_iter()
            .map(|(cid, owners)| {
                (
                    cid.to_string(),
                    owners.into_iter().map(|o| o.to_string()).collect(),
                )
            })
            .collect()
    }

    fn v140_prompt_state(conn: &Connection) -> Vec<(String, String, String, String)> {
        let mut stmt = conn
            .prepare(
                "SELECT id, instance_id, status, IFNULL(status_reason, '') FROM active_prompts ORDER BY id",
            )
            .unwrap();
        stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)))
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
    }

    fn v140_state_row(id: &str, instance_id: &str, status: &str, reason: &str) -> (String, String, String, String) {
        (
            id.to_string(),
            instance_id.to_string(),
            status.to_string(),
            reason.to_string(),
        )
    }

    #[test]
    fn migration_assigns_unique_owner_for_empty_instance_row() {
        let tmp = tempfile::tempdir().unwrap();
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        insert_v140_prompt_row(&conn, "prompt-cid-a", "", Some("cid-a"), "backed_up", 10);
        let owners = v140_owner_index(vec![("cid-a", vec!["inst-a"])]);
        let backup = tmp.path().join("repo_prompts.db.pre-v140.bak");

        let changed =
            migrate_prompt_instance_ids_v140_core(&conn, &backup, "default", &owners).unwrap();

        assert!(changed >= 1);
        assert!(backup.exists());
        assert_eq!(
            v140_prompt_state(&conn),
            vec![v140_state_row("prompt-inst-a-cid-a", "inst-a", "backed_up", "")]
        );
    }

    #[test]
    fn migration_orphans_ambiguous_or_unknown_rows() {
        let tmp = tempfile::tempdir().unwrap();
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        insert_v140_prompt_row(&conn, "prompt-cid-both", "", Some("cid-both"), "backed_up", 10);
        insert_v140_prompt_row(&conn, "p-all", "all", Some("cid-none"), "backed_up", 10);
        insert_v140_prompt_row(&conn, "p-nosession", "", None, "queued", 10);
        insert_v140_prompt_row(&conn, "prompt-cid-c", "", Some("cid-c"), "backed_up", 10);
        insert_v140_prompt_row(&conn, "prompt-inst-a-cid-c", "inst-a", Some("cid-c"), "dispatched", 10);
        let owners = v140_owner_index(vec![
            ("cid-both", vec!["inst-a", "inst-b"]),
            ("cid-c", vec!["inst-a"]),
        ]);

        migrate_prompt_instance_ids_v140_core(&conn, &tmp.path().join("b.bak"), "default", &owners)
            .unwrap();

        assert_eq!(
            v140_prompt_state(&conn),
            vec![
                v140_state_row("p-all", "", "orphaned", "legacy_owner_unknown"),
                v140_state_row("p-nosession", "", "orphaned", "legacy_owner_unknown"),
                v140_state_row("prompt-cid-both", "", "orphaned", "legacy_owner_ambiguous"),
                v140_state_row("prompt-cid-c", "", "orphaned", "legacy_owner_ambiguous"),
                v140_state_row("prompt-inst-a-cid-c", "inst-a", "dispatched", ""),
            ]
        );
    }

    #[test]
    fn migration_is_idempotent() {
        let tmp = tempfile::tempdir().unwrap();
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        insert_v140_prompt_row(&conn, "prompt-cid-a", "", Some("cid-a"), "backed_up", 10);
        let owners = v140_owner_index(vec![("cid-a", vec!["inst-a"])]);
        migrate_prompt_instance_ids_v140_core(&conn, &tmp.path().join("first.bak"), "default", &owners)
            .unwrap();
        let before = v140_prompt_state(&conn);

        let second_backup = tmp.path().join("second.bak");
        let second =
            migrate_prompt_instance_ids_v140_core(&conn, &second_backup, "default", &owners).unwrap();

        assert_eq!(second, 0);
        assert_eq!(v140_prompt_state(&conn), before);
        assert!(!second_backup.exists());
    }

    #[test]
    fn migration_aborts_when_backup_copy_fails() {
        let tmp = tempfile::tempdir().unwrap();
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        insert_v140_prompt_row(&conn, "prompt-cid-a", "", Some("cid-a"), "backed_up", 10);
        let owners = v140_owner_index(vec![("cid-a", vec!["inst-a"])]);
        let unwritable = tmp.path().join("missing-dir").join("repo.bak");

        let result = migrate_prompt_instance_ids_v140_core(&conn, &unwritable, "default", &owners);

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("backup copy failed"));
        assert_eq!(
            v140_prompt_state(&conn),
            vec![v140_state_row("prompt-cid-a", "", "backed_up", "")]
        );
        let markers: i64 = conn
            .query_row("SELECT COUNT(*) FROM agm_schema_migrations", [], |row| row.get(0))
            .unwrap();
        assert_eq!(markers, 0);
    }

    #[test]
    fn migration_maps_legacy_default_pending_and_project_session_ids() {
        let tmp = tempfile::tempdir().unwrap();
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        insert_v140_prompt_row(&conn, "p-def", "__default__", Some("cid-d"), "pending", 10);
        insert_v140_prompt_row(&conn, "p-l2", "inst-a", Some("proj-v140"), "backed_up", 10);
        insert_v140_prompt_row(&conn, "prompt-cid-e", "inst-a", Some("cid-e"), "backed_up", 10);

        migrate_prompt_instance_ids_v140_core(
            &conn,
            &tmp.path().join("m.bak"),
            "default",
            &HashMap::new(),
        )
        .unwrap();

        assert_eq!(
            v140_prompt_state(&conn),
            vec![
                v140_state_row("p-def", "default", "queued", ""),
                v140_state_row("p-l2", "inst-a", "backed_up", ""),
                v140_state_row("prompt-inst-a-cid-e", "inst-a", "backed_up", ""),
            ]
        );
        let l2_session: Option<String> = conn
            .query_row("SELECT session_id FROM active_prompts WHERE id = 'p-l2'", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert!(l2_session.is_none());
    }

    #[test]
    fn migration_retires_duplicate_identity_rows_and_creates_index() {
        let tmp = tempfile::tempdir().unwrap();
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        insert_v140_prompt_row(&conn, "dup-old", "inst-a", Some("cid-x"), "backed_up", 10);
        insert_v140_prompt_row(&conn, "dup-new", "inst-a", Some("cid-x"), "running", 20);

        migrate_prompt_instance_ids_v140_core(
            &conn,
            &tmp.path().join("d.bak"),
            "default",
            &HashMap::new(),
        )
        .unwrap();

        assert_eq!(
            v140_prompt_state(&conn),
            vec![
                v140_state_row("dup-new", "inst-a", "running", ""),
                v140_state_row("dup-old", "inst-a", "orphaned", "legacy_duplicate"),
            ]
        );
        let index_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'index' AND name = 'idx_active_prompts_identity'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(index_count, 1);
        let duplicate = conn.execute(
            "INSERT INTO active_prompts
             (id, project_id, instance_id, repo_path, prompt_content, session_id, status, created_at, updated_at)
             VALUES ('dup-third', 'proj-v140', 'inst-a', 'D:/work/app', 'again', 'cid-x', 'backed_up', 30, 30)",
            [],
        );
        assert!(duplicate.is_err());
    }

    #[test]
    fn conversation_sequences_are_keyed_by_instance_after_migration() {
        let tmp = tempfile::tempdir().unwrap();
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        conn.execute_batch(
            "DROP TABLE agm_conversation_sequences;
             CREATE TABLE agm_conversation_sequences (
                 conversation_id TEXT PRIMARY KEY,
                 seq_id INTEGER NOT NULL UNIQUE,
                 project_key TEXT NOT NULL,
                 title TEXT NOT NULL,
                 instance_id TEXT NOT NULL DEFAULT 'default',
                 updated_at INTEGER NOT NULL
             );
             INSERT INTO agm_conversation_sequences VALUES ('cid-x', 1, 'app', 'Chat', '__default__', 1);",
        )
        .unwrap();

        migrate_prompt_instance_ids_v140_core(
            &conn,
            &tmp.path().join("s.bak"),
            "default",
            &HashMap::new(),
        )
        .unwrap();

        assert_eq!(
            ensure_conversation_sequence_in_conn(&conn, "cid-x", "app", "Chat", "default"),
            1
        );
        assert_eq!(
            ensure_conversation_sequence_in_conn(&conn, "cid-x", "app", "Chat", "inst-b"),
            2
        );
        assert_eq!(
            ensure_conversation_sequence_in_conn(&conn, "cid-x", "app", "Chat", "inst-b"),
            2
        );
    }

    #[test]
    fn owner_index_reads_summaries_and_brain_folders() {
        let tmp = tempfile::tempdir().unwrap();
        let a_dir = tmp.path().join("a").join(".gemini").join("antigravity");
        let b_dir = tmp.path().join("b").join(".gemini").join("antigravity");
        fs::create_dir_all(&a_dir).unwrap();
        {
            let summaries = Connection::open(a_dir.join("conversation_summaries.db")).unwrap();
            summaries
                .execute_batch(
                    "CREATE TABLE conversation_summaries (conversation_id TEXT);
                     INSERT INTO conversation_summaries VALUES ('cid-1');
                     INSERT INTO conversation_summaries VALUES ('cid-2');",
                )
                .unwrap();
        }
        fs::create_dir_all(b_dir.join("brain").join("cid-2")).unwrap();

        let index = build_conversation_owner_index(&[
            ("inst-a".to_string(), a_dir),
            ("inst-b".to_string(), b_dir),
        ]);

        let only_a: std::collections::BTreeSet<String> =
            ["inst-a".to_string()].into_iter().collect();
        let both: std::collections::BTreeSet<String> =
            ["inst-a".to_string(), "inst-b".to_string()].into_iter().collect();
        assert_eq!(index.get("cid-1"), Some(&only_a));
        assert_eq!(index.get("cid-2"), Some(&both));
        assert!(index.get("cid-3").is_none());
    }

    #[test]
    fn identity_conflict_never_deletes_other_row() {
        let tmp = tempfile::tempdir().unwrap();
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        migrate_prompt_instance_ids_v140_core(
            &conn,
            &tmp.path().join("i.bak"),
            "default",
            &HashMap::new(),
        )
        .unwrap();
        insert_v140_prompt_row(&conn, "prompt-inst-a-cid-x", "inst-a", Some("cid-x"), "running", 10);
        let original = vec![v140_state_row("prompt-inst-a-cid-x", "inst-a", "running", "")];

        let mut other = ActivePrompt {
            id: "prompt-other".to_string(),
            project_id: "proj-v140".to_string(),
            instance_id: "inst-a".to_string(),
            repo_path: "D:/work/app".to_string(),
            prompt_content: "second prompt".to_string(),
            model: None,
            session_id: Some("cid-x".to_string()),
            status: "backed_up".to_string(),
            created_at: 20,
            updated_at: 20,
            image_payload: None,
            source_dir: None,
        };

        let identity_clash = upsert_active_prompt_row(&conn, &other, "proj-v140", "inst-a", 20);
        assert!(identity_clash.is_err());
        assert_eq!(v140_prompt_state(&conn), original);

        other.id = "prompt-inst-a-cid-x".to_string();
        other.session_id = Some("cid-y".to_string());
        let reown = upsert_active_prompt_row(&conn, &other, "proj-v140", "inst-b", 30);
        assert!(reown.is_err());
        assert_eq!(v140_prompt_state(&conn), original);

        upsert_active_prompt_row(&conn, &other, "proj-v140", "inst-a", 40).unwrap();
        assert_eq!(
            v140_prompt_state(&conn),
            vec![v140_state_row("prompt-inst-a-cid-x", "inst-a", "backed_up", "")]
        );
    }
```

`identity_conflict_never_deletes_other_row` proves the three upsert outcomes: a different id with the same `(instance_id, session_id)` as a running row fails and keeps the running row (the old `REPLACE` deleted it); the same id from another instance fails and keeps the row on its owner; the same id from the owner updates in place.

The existing test `test_agm_sequences_and_200_word_preview` must still pass; it calls `ensure_conversation_sequence_in_conn` on a fresh in-memory database, which now gets the new composite key from `init_tables`.

## 7. Gate

```text
$env:CARGO_TARGET_DIR="$PWD\src-tauri\target"; $env:CARGO_BUILD_JOBS="1"
cd src-tauri; cargo fmt; cargo fmt -- --check; cargo clippy --all-targets --all-features; cargo test --lib -- --test-threads=1 migration_; cargo test --lib -- --test-threads=1 conversation_sequences_are_keyed_by_instance_after_migration; cargo test --lib -- --test-threads=1 owner_index_reads_summaries_and_brain_folders; cargo test --lib -- --test-threads=1 identity_conflict_never_deletes_other_row; cargo test --lib -- --test-threads=1 test_agm_sequences_and_200_word_preview; cargo test --lib -- --test-threads=1 test_repo_db_schema_initialization; cd ..
```

`migration_` must run at least the 6 tests above. Other modules may also have tests whose names contain `migration_`; they must pass too.

If the compiler says `no method named backup found for reference &Connection`, the `backup` feature of `rusqlite` is missing: STOP and report (do not edit `Cargo.toml` in this step).

## 8. Commit

```text
Fix: prompts - migrate repo DB prompt instance ids once (v140)
```

Stage only `src-tauri/src/modules/repo_db.rs` and `src-tauri/src/modules/instance.rs`.

## 9. Done when

- [ ] `connect_db` calls `migrate_prompt_instance_ids_v140(&conn, &path)` after `init_tables(&conn)?;` and only logs on error.
- [ ] `migrate_prompt_instance_ids_v140_core` takes `(&Connection, &Path, &str, &HashMap<String, BTreeSet<String>>)`, checks the marker first, copies the database before opening the transaction, and returns `Err` without changes when the copy fails.
- [ ] Orphaned rows keep `instance_id = ''` and get `status_reason` `legacy_owner_unknown`, `legacy_owner_ambiguous` or `legacy_duplicate`.
- [ ] `idx_active_prompts_identity` is created inside the migration transaction.
- [ ] `agm_conversation_sequences` has `PRIMARY KEY (instance_id, conversation_id)` both in `init_tables` and after the rebuild; every lookup in `ensure_conversation_sequence_in_conn` filters by `instance_id`.
- [ ] `gitmap aum search "INSERT OR REPLACE INTO active_prompts"` returns 0 hits in `src-tauri/src`, `src-tauri/src/bin/agm.rs` and each `src-tauri/tests/*.rs` file; every former site is `INSERT ... ON CONFLICT(id) ...` and no `DO UPDATE SET` list contains `instance_id`.
- [ ] `save_or_requeue_prompt` calls `upsert_active_prompt_row`, logs with `log_warn` and returns `Err` on an identity conflict or an instance mismatch; its signature is unchanged.
- [ ] Callers searched in `src-tauri/src`, `src-tauri/src/bin/agm.rs` and each `src-tauri/tests/*.rs` file: no signature, field, visibility or return-type change in this step needs a caller edit (`save_or_requeue_prompt` and `ensure_conversation_sequence_in_conn` keep their signatures; `upsert_active_prompt_row` is new).
- [ ] All 9 new tests (8 migration/sequence tests plus `identity_conflict_never_deletes_other_row`) and the 2 existing tests in the gate pass.
- [ ] Gate exited 0; commit pushed with the exact message.
