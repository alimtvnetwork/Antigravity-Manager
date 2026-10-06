# Step 03: new active_prompts columns, agm_schema_migrations, ActivePrompt.source_dir

You add three columns to `active_prompts` (`status_reason`, `source_dir`, `attempts`), create the `agm_schema_migrations` table, and add the field `source_dir: Option<String>` to the `ActivePrompt` struct. Every place that builds an `ActivePrompt` must get the new field, or the build fails. Readers that map rows by column index also read the new column, and `save_or_requeue_prompt` writes it, so the value survives its `INSERT OR REPLACE`.

No behavior changes yet: every initializer sets `source_dir: None`. Step 05 fills it at capture.

## 1. Depends on

Step 01.

## 2. Files you may edit

- `src-tauri/src/modules/repo_db.rs`
- `src-tauri/src/modules/backup_prompts_db.rs`
- `src-tauri/src/modules/telegram_inbound.rs`
- `src-tauri/src/bin/agm.rs` (5 `ActivePrompt` initializers, sites E9 to E13)

## 3. Find it

| Change | File | Anchor | Unique search literal | Line hint |
|---|---|---|---|---|
| A. Struct field | `repo_db.rs` | `pub struct ActivePrompt {` | `pub struct ActivePrompt {` | 82 |
| B. Columns and migrations table | `repo_db.rs` | `fn init_tables(conn: &Connection) -> Result<(), String> {` | `ALTER TABLE active_prompts ADD COLUMN image_payload TEXT` | 198 to 202 |
| C. Persist `source_dir` | `repo_db.rs` | `pub fn save_or_requeue_prompt(prompt: &ActivePrompt) -> Result<(), String> {` | `"INSERT OR REPLACE INTO active_prompts` (the hit inside this fn) | 2856 to 2873 |
| D. Row mappers (5 sites) | `repo_db.rs` | see section 4 D | `image_payload: row.get(10).ok(),` | 1527, 2158, 2386, 2462, 3266 |
| E. Plain initializers (13 sites) | 4 files | see section 4 E | `ActivePrompt {` | see table in 4 E |
| F. Tests | `repo_db.rs` | `mod tests {` | `fn test_uri_decoding()` | 5880 |

Find every initializer first and confirm the count. GitMap does not descend into `src-tauri/src/bin` or `src-tauri/tests` when you search `src-tauri/src`, so run every line:

```text
gitmap aum search "ActivePrompt {" src-tauri/src --ext .rs
gitmap aum search "ActivePrompt {" src-tauri/src/bin/agm.rs --ext .rs
gitmap aum search "ActivePrompt" src-tauri/tests/auto_switcher_e2e_test.rs --ext .rs
gitmap aum search "ActivePrompt" src-tauri/tests/instance_cloning_and_sync_test.rs --ext .rs
gitmap aum search "ActivePrompt" src-tauri/tests/per_instance_prompt_liveness_test.rs --ext .rs
gitmap aum search "image_payload: row.get(10).ok()," src-tauri/src --ext .rs
gitmap aum search "image_payload: row.get(10).ok()," src-tauri/src/bin/agm.rs --ext .rs
```

Expected on 2026-10-06:

- `src-tauri/src`: 14 hits for `ActivePrompt {` (1 is the struct definition at `repo_db.rs:82`, 13 are initializers: 5 row mappers D1 to D5 and 8 plain initializers E1 to E8).
- `src-tauri/src/bin/agm.rs`: 5 hits (`:3171`, `:4550`, `:14325`, `:14338`, `:14351`), plain initializers E9 to E13.
- Each `src-tauri/tests/*.rs` file: 0 hits for `ActivePrompt`.
- Row-mapper literal: 5 hits, all in `repo_db.rs`; 0 hits in `agm.rs`.

If any count differs, STOP and report.

## 4. Current code

### A. Struct (`repo_db.rs` lines 80 to 95)

```rust
/// Represents a prompt captured from a running project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivePrompt {
    pub id: String,
    pub project_id: String,
    pub instance_id: String,
    pub repo_path: String,
    pub prompt_content: String,
    pub model: Option<String>,
    pub session_id: Option<String>,
    pub status: String, // "running", "backed_up", "dispatched", "completed"
    pub created_at: i64,
    pub updated_at: i64,
    #[serde(default)]
    pub image_payload: Option<String>,
}
```

### B. `init_tables` (`repo_db.rs` lines 198 to 209)

```rust
    // Migration: add image_payload column if it doesn't exist yet
    let _ = conn.execute(
        "ALTER TABLE active_prompts ADD COLUMN image_payload TEXT",
        [],
    );

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_active_prompts_instance_status 
         ON active_prompts(instance_id, status)",
        [],
    )
    .map_err(|e| format!("Failed to create index on active_prompts: {}", e))?;
```

Note: migrations in this file are idempotent `ALTER TABLE ... ADD COLUMN` calls whose error is ignored with `let _ =`. Schema setup runs in `init_tables` (line 163), which `connect_db` (line 149) calls. Tests call `init_tables` directly on `Connection::open_in_memory()`.

### C. `save_or_requeue_prompt` insert (`repo_db.rs` lines 2856 to 2873)

```rust
    conn.execute(
        "INSERT OR REPLACE INTO active_prompts 
         (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
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
        ],
    ).map_err(|e| format!("Failed to insert active prompt: {}", e))?;
```

### D. Row mappers (5 sites, all in `repo_db.rs`)

Each site has a `SELECT` whose column list ends with `updated_at, image_payload`, and a closure that builds `ActivePrompt` with `image_payload: row.get(10).ok(),` as its last field. The five sites:

| Site | Enclosing fn | SELECT line | Mapper last-field line |
|---|---|---|---|
| D1 | `pub fn dispatch_running_prompts(instance_id: &str) -> Result<usize, String> {` (line 1491) | 1507 | 1527 |
| D2 | `pub fn check_and_dispatch_enqueued_prompts(target_instance: Option<&str>) -> Result<usize, String> {` (line 2054) | 2138 | 2158 |
| D3 | `pub fn list_backed_up_prompts() -> Result<Vec<ActivePrompt>, String> {` (line 2364) | 2368 | 2386 |
| D4 | `pub fn list_all_prompts() -> Result<Vec<ActivePrompt>, String> {` (line 2440) | 2444 | 2462 |
| D5 | `pub fn resend_running_commands_for_instance(` (line 3237) | 3246 | 3266 |

Current SELECT first line at D1, D3, D4, D5 (12 spaces of indent, a trailing space after `image_payload`):

```rust
            "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload 
```

Current SELECT first line at D2 (16 spaces of indent, no trailing space):

```rust
                "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload
```

Current mapper last field at all five sites (indent is 16 spaces at D1, D3, D4, D5 and 24 spaces at D2):

```rust
                image_payload: row.get(10).ok(),
            })
```

### E. Plain initializers (13 sites)

Each one ends with an `image_payload` line followed by `};` or `});`. You add one line after the `image_payload` line.

| Site | File:line of `ActivePrompt {` | Enclosing fn | Current last line before `}` |
|---|---|---|---|
| E1 | `repo_db.rs:1081` | `pub fn discover_running_prompts_from_antigravity(instance_id: &str) -> Vec<ActivePrompt> {` | `                    image_payload,` (line 1092) |
| E2 | `repo_db.rs:1402` | `pub fn backup_running_prompts(instance_id: &str) -> Result<usize, String> {` | `                    image_payload: final_img,` (line 1413) |
| E3 | `repo_db.rs:3632` | `pub fn auto_resume_recent_prompts(` | `            image_payload: image_payload.clone(),` (line 3643) |
| E4 | `repo_db.rs:5095` | `pub fn prompt_target_by_sequence_scoped(` | `        image_payload: None,` (line 5109) |
| E5 | `repo_db.rs:5429` | `pub fn start_prompt_goal_heartbeat(` | `        image_payload: None,` (line 5443) |
| E6 | `repo_db.rs:5927` | test `fn resume_document_keeps_the_same_conversation_id()` | `            image_payload: None,` (line 5938) |
| E7 | `backup_prompts_db.rs:583` | restore loop (`let active_p = ActivePrompt {`) | `            image_payload: rec.images_payload.clone(),` (line 594) |
| E8 | `telegram_inbound.rs:2429` | `let active_prompt = repo_db::ActivePrompt {` | `            image_payload: None,` (line 2440) |
| E9 | `agm.rs:3171` | `fn cmd_prompt_dispatch(args: &[String])`, `let active_p = repo_db::ActivePrompt {` | `        image_payload: None,` (line 3182) |
| E10 | `agm.rs:4550` | `fn cmd_running_prompts_import(args: &[String])`, `let active_p = ActivePrompt {` | `                image_payload: images_payload,` (line 4561) |
| E11 | `agm.rs:14325` | e2e command, `let running_prompt = ActivePrompt {` | `        image_payload: None,` (line 14336) |
| E12 | `agm.rs:14338` | same function, `let queued_prompt_1 = ActivePrompt {` | `        image_payload: None,` (line 14349) |
| E13 | `agm.rs:14351` | same function, `let queued_prompt_2 = ActivePrompt {` | `        image_payload: None,` (line 14362) |

E9 current code (`agm.rs` lines 3171 to 3183):

```rust
    let active_p = repo_db::ActivePrompt {
        id: prompt_id,
        project_id: slug.clone(),
        instance_id: inst_id.clone(),
        repo_path: cwd_str.clone(),
        prompt_content: final_prompt.clone(),
        model: Some("gemini-3.8-flash-high".to_string()),
        session_id: Some(session_id.clone()),
        status: "running".to_string(),
        created_at: now,
        updated_at: now,
        image_payload: None,
    };
```

E10 current code (`agm.rs` lines 4550 to 4562):

```rust
            let active_p = ActivePrompt {
                id,
                project_id: proj_id,
                instance_id: "default".to_string(),
                repo_path,
                prompt_content: prompt_text,
                model: Some("gemini-3.8-flash-high".to_string()),
                session_id: None,
                status: "queued".to_string(),
                created_at: now,
                updated_at: now,
                image_payload: images_payload,
            };
```

E11 current code (`agm.rs` lines 14325 to 14337); E12 (`:14338`) and E13 (`:14351`) have the same shape with `queued_prompt_1` / `queued_prompt_2`, other `id` / `prompt_content` / `status` values, and the same last line `        image_payload: None,`:

```rust
    let running_prompt = ActivePrompt {
        id: format!("prompt-{}-running-1", new_inst.id),
        project_id: "gitmap-test".to_string(),
        instance_id: new_inst.id.clone(),
        repo_path: gitmap_dir.to_string(),
        prompt_content: "Running the Gitmap tests and verifying test inventory".to_string(),
        model: Some("gemini-2.5-pro".to_string()),
        session_id: Some(format!("session-{}-1", new_inst.id)),
        status: "running".to_string(),
        created_at: now_ts,
        updated_at: now_ts,
        image_payload: None,
    };
```

Example, E7 current code (`backup_prompts_db.rs` lines 583 to 595):

```rust
        let active_p = ActivePrompt {
            id: rec.prompt_id.clone(),
            project_id: rec.project_id.clone(),
            instance_id: eff_inst.to_string(),
            repo_path: rec.project_path.clone(),
            prompt_content: rec.prompt_text.clone(),
            model: Some("gemini-3.8-flash-high".to_string()),
            session_id: Some(rec.conversation_id.clone()),
            status: restored_status.to_string(),
            created_at: now,
            updated_at: now,
            image_payload: rec.images_payload.clone(),
        };
```

Example, E8 current code (`telegram_inbound.rs` lines 2429 to 2441):

```rust
        let active_prompt = repo_db::ActivePrompt {
            id: prompt_id.clone(),
            project_id: final_proj_id.clone(),
            instance_id: resolved_instance.clone(),
            repo_path: repo_path.clone(),
            prompt_content: resolved_prompt.clone(),
            model: None,
            session_id: resolved_conv_id.clone(),
            status: "running".to_string(),
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
            image_payload: None,
        };
```

### F. Test module start (`repo_db.rs` lines 5879 to 5888)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uri_decoding() {
        let uri = "file:///d:/work/My%20Project";
        let path = decode_uri_to_path(uri);
        assert!(path.contains("My Project"));
    }
```

## 5. New code

### A. Struct: replace the block from 4 A with

```rust
/// Represents a prompt captured from a running project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivePrompt {
    pub id: String,
    pub project_id: String,
    pub instance_id: String,
    pub repo_path: String,
    pub prompt_content: String,
    pub model: Option<String>,
    pub session_id: Option<String>,
    pub status: String, // "running", "backed_up", "dispatched", "completed"
    pub created_at: i64,
    pub updated_at: i64,
    #[serde(default)]
    pub image_payload: Option<String>,
    #[serde(default)]
    pub source_dir: Option<String>,
}
```

### B. `init_tables`: replace the block from 4 B with

```rust
    // Migration: add image_payload column if it doesn't exist yet
    let _ = conn.execute(
        "ALTER TABLE active_prompts ADD COLUMN image_payload TEXT",
        [],
    );

    // Migration v140: reason codes, capture source and dispatch attempts
    for ddl in [
        "ALTER TABLE active_prompts ADD COLUMN status_reason TEXT",
        "ALTER TABLE active_prompts ADD COLUMN source_dir TEXT",
        "ALTER TABLE active_prompts ADD COLUMN attempts INTEGER NOT NULL DEFAULT 0",
    ] {
        let _ = conn.execute(ddl, []);
    }

    conn.execute(
        "CREATE TABLE IF NOT EXISTS agm_schema_migrations (
            name TEXT PRIMARY KEY,
            applied_at INTEGER NOT NULL
        )",
        [],
    )
    .map_err(|e| format!("Failed to create agm_schema_migrations table: {}", e))?;

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_active_prompts_instance_status 
         ON active_prompts(instance_id, status)",
        [],
    )
    .map_err(|e| format!("Failed to create index on active_prompts: {}", e))?;
```

### C. `save_or_requeue_prompt`: replace the block from 4 C with

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
```

### D. Row mappers: at each of the 5 sites D1 to D5

1. In the SELECT first line, change `updated_at, image_payload` to `updated_at, image_payload, source_dir`. Keep the rest of the string (trailing space, following lines) unchanged. New first line at D1, D3, D4, D5:

```rust
            "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload, source_dir 
```

New first line at D2:

```rust
                "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload, source_dir
```

2. Directly after `image_payload: row.get(10).ok(),` add one line with the same indent:

```rust
                image_payload: row.get(10).ok(),
                source_dir: row.get(11).ok(),
            })
```

Do the SELECT edit and the mapper edit inside the same function. Column 11 exists only because of step 1; do not add `source_dir: row.get(11)` to any query you did not edit in step 1.

### E. Plain initializers: at each of the 13 sites E1 to E13

Add `source_dir: None,` on the line directly after the `image_payload` line, with the same indent. E7 becomes:

```rust
        let active_p = ActivePrompt {
            id: rec.prompt_id.clone(),
            project_id: rec.project_id.clone(),
            instance_id: eff_inst.to_string(),
            repo_path: rec.project_path.clone(),
            prompt_content: rec.prompt_text.clone(),
            model: Some("gemini-3.8-flash-high".to_string()),
            session_id: Some(rec.conversation_id.clone()),
            status: restored_status.to_string(),
            created_at: now,
            updated_at: now,
            image_payload: rec.images_payload.clone(),
            source_dir: None,
        };
```

E8 becomes:

```rust
        let active_prompt = repo_db::ActivePrompt {
            id: prompt_id.clone(),
            project_id: final_proj_id.clone(),
            instance_id: resolved_instance.clone(),
            repo_path: repo_path.clone(),
            prompt_content: resolved_prompt.clone(),
            model: None,
            session_id: resolved_conv_id.clone(),
            status: "running".to_string(),
            created_at: Utc::now().timestamp(),
            updated_at: Utc::now().timestamp(),
            image_payload: None,
            source_dir: None,
        };
```

E1 (`image_payload,` shorthand) becomes:

```rust
                    image_payload,
                    source_dir: None,
                });
```

E9 (`agm.rs`), last two lines become:

```rust
        image_payload: None,
        source_dir: None,
    };
```

E10 (`agm.rs`), last two lines become:

```rust
                image_payload: images_payload,
                source_dir: None,
            };
```

E11, E12 and E13 (`agm.rs`), last two lines of each become:

```rust
        image_payload: None,
        source_dir: None,
    };
```

After all edits, re-run:

```text
gitmap aum search "source_dir" src-tauri/src --ext .rs
gitmap aum search "source_dir" src-tauri/src/bin/agm.rs --ext .rs
```

Expect in `src-tauri/src`: 1 struct field, 1 `ADD COLUMN`, 2 hits in `save_or_requeue_prompt`, 5 SELECT lines plus 5 mapper lines, and 8 `source_dir: None` lines. Expect in `agm.rs`: exactly 5 `source_dir: None,` lines (E9 to E13).

## 6. Tests

Paste after `    use super::*;` in `mod tests` of `repo_db.rs` (before `fn test_uri_decoding`):

```rust
    #[test]
    fn init_tables_adds_v140_columns_idempotently() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(init_tables(&conn).is_ok());
        assert!(init_tables(&conn).is_ok());

        let columns: Vec<String> = conn
            .prepare("SELECT name FROM pragma_table_info('active_prompts')")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .flatten()
            .collect();
        for name in ["status_reason", "source_dir", "attempts"] {
            assert!(columns.iter().any(|c| c == name), "missing column {}", name);
        }

        let migrations_tables: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'agm_schema_migrations'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(migrations_tables, 1);

        conn.execute(
            "INSERT INTO active_prompts
             (id, project_id, instance_id, repo_path, prompt_content, status, created_at, updated_at)
             VALUES ('p-v140', 'proj', 'default', 'D:/work/app', 'text', 'queued', 1, 1)",
            [],
        )
        .unwrap();
        let (attempts, source_dir): (i64, Option<String>) = conn
            .query_row(
                "SELECT attempts, source_dir FROM active_prompts WHERE id = 'p-v140'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(attempts, 0);
        assert!(source_dir.is_none());
    }

    #[test]
    fn active_prompt_source_dir_defaults_when_missing_in_json() {
        let json = r#"{
            "id": "p-1",
            "project_id": "proj",
            "instance_id": "default",
            "repo_path": "D:/work/app",
            "prompt_content": "keep going",
            "model": null,
            "session_id": "cid-1",
            "status": "backed_up",
            "created_at": 1,
            "updated_at": 1
        }"#;
        let prompt: ActivePrompt = serde_json::from_str(json).unwrap();
        assert!(prompt.source_dir.is_none());
        assert!(prompt.image_payload.is_none());
    }
```

## 7. Gate

```text
$env:CARGO_TARGET_DIR="$PWD\src-tauri\target"; $env:CARGO_BUILD_JOBS="1"
cd src-tauri; cargo fmt; cargo fmt -- --check; cargo clippy --all-targets --all-features; cargo test --lib -- --test-threads=1 init_tables_adds_v140_columns_idempotently; cargo test --lib -- --test-threads=1 active_prompt_source_dir_defaults_when_missing_in_json; cargo test --lib -- --test-threads=1 resume_document_keeps_the_same_conversation_id; cargo test --lib -- --test-threads=1 test_repo_db_schema_initialization; cd ..
```

If clippy reports `missing field source_dir in initializer of ActivePrompt`, it names the file and line: add `source_dir: None,` there and report the extra site in your step report. If that file is not in section 2, STOP (section 6 of `00-start-here.md`).

## 8. Commit

```text
Fix: prompts - add v140 prompt columns and ActivePrompt source_dir
```

Stage exactly: `src-tauri/src/modules/repo_db.rs`, `src-tauri/src/modules/backup_prompts_db.rs`, `src-tauri/src/modules/telegram_inbound.rs`, `src-tauri/src/bin/agm.rs`.

## 9. Done when

- [ ] `ActivePrompt` has `#[serde(default)] pub source_dir: Option<String>` as its last field.
- [ ] `init_tables` adds `status_reason`, `source_dir`, `attempts` and creates `agm_schema_migrations`.
- [ ] `save_or_requeue_prompt` writes `source_dir` (12 columns, 12 `?`, 12 params).
- [ ] All 5 row mappers select `source_dir` and read `row.get(11).ok()`.
- [ ] All 13 plain initializers have `source_dir: None` (8 in `src-tauri/src`, 5 in `src-tauri/src/bin/agm.rs`).
- [ ] Initializers were searched in `src-tauri/src`, `src-tauri/src/bin/agm.rs` and each `src-tauri/tests/*.rs` file (the test files have none).
- [ ] Both new tests and the two existing tests in the gate pass.
- [ ] Gate exited 0; commit pushed with the exact message.
