# Step 14: Running Detection Uses Live Turns Only

Goal: a prompt counts as running only when its conversation is in a live turn (`not_fully_idle > 0`, status contains `RUNNING`, last turn within 600 s, DR-8) in a live instance; idle `running` rows are swept instead of being retired to `dispatched`; Layer 2, Gate 4, project execution status and auto-resume compare exact instance ids and exact normalized paths. Fixes B05, B08, B09, B11, B12, B14 and the `LIKE` part of auto-resume. Spec section 6.7.

Do NOT edit `is_instance_running` in `src-tauri/src/modules/instance.rs` (DR-4). This step only calls it, through `is_registered_instance_alive` from step 12.

## 1. Depends on

- Step 12 (`select_instance_prompts_by_status`, `is_registered_instance_alive`).
- Step 04 (`gemini_dirs_tagged(&InstanceScope) -> Vec<(String, PathBuf)>`).
- Step 05 (`discover_running_prompts_in_dirs`, `project_instance_id` in the Layer 2 loop, `write_summaries_fixture`).
- Step 08 (`normalize_path_for_compare` is `pub(crate)`; `resolve_instance_id` replaced in `repo_db.rs`).
- Step 10 (`prompt_status`).

## 2. Files you may edit

- `src-tauri/src/modules/repo_db.rs`

## 3. Find it

All changes are in `src-tauri/src/modules/repo_db.rs`.

| Change | Function | Unique search literal | Line hint |
|---|---|---|---|
| A | `struct ConversationSummaryRow` and `fn read_conversation_summary_rows(conn: &Connection) -> Vec<ConversationSummaryRow>` | `struct ConversationSummaryRow {` | `:907` |
| B | `pub fn discover_running_prompts_from_antigravity(instance_id: &str) -> Vec<ActivePrompt>` (thin wrapper after step 05; liveness gate) | `discover_running_prompts_in_dirs(&gemini_dirs_tagged(&scope))` | after `:956` |
| C | `fn discover_running_prompts_in_dirs(tagged_dirs: &[(String, PathBuf)]) -> Vec<ActivePrompt>` (row filter, added by step 05) | `let is_running_or_recent =` | after `:970` |
| D | `pub fn backup_running_prompts(instance_id: &str) -> Result<usize, String>` (stale retire) | `Step 0: Retire stale 'running' prompts older than 2 hours` | `:1127` |
| E | same function, Layer 1 discovery line | `let ag_prompts = discover_running_prompts_from_antigravity(` | `:1182` |
| F | same function, Layer 2 project list | `Layer 2: VS Code / Instance Workspace Storage` | `:1238` |
| G | same function, Layer 2 fallback query | `let proj_like = format!("%{}%", project.repo_name.to_lowercase());` (first hit, inside `backup_running_prompts`) | `:1314` |
| H | same function, Layer 2 dedupe query (verify only; step 05 already filters by instance) | `SELECT id FROM active_prompts WHERE instance_id = ?1 AND repo_path = ?2 AND prompt_content = ?3 LIMIT 1` | `:1337` |
| I | `pub fn detect_running_projects(instance_id: &str) -> Result<Vec<RunningProject>, String>` | `DELETE FROM running_projects WHERE workspace_storage_path IS NULL OR instr(id, '__') = 0` | `:626` |
| J | `pub fn is_prompt_running_for_project(project_id: &str, instance_id: &str) -> bool` (Gate 4) | `.starts_with(&format!("{}-", folder_name)));` | `:2012` |
| K | `pub fn get_project_execution_status(` | `Helper to get live execution status for a specific project` | `:2691` |
| L | `pub fn auto_resume_recent_prompts(` (prompt lookup) | `Project was active within < 1 hour! Find its most recent prompt + image` | `:3506` |
| M | `fn normalize_path_for_compare(p: &str) -> String` (insert helpers after it) | `fn normalize_path_for_compare(p: &str) -> String {` | `:2473` |
| N | test `summary_rows_fall_back_when_status_columns_are_missing` | `assert_eq!(rows[0].not_fully_idle, 1);` | `:5963` |
| O | test helper `fn write_summaries_fixture` (added by step 05) | `'CASCADE_RUN_STATUS_RUNNING', 1, ?3, 10)` | test module |

```text
gitmap aum search "struct ConversationSummaryRow" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "discover_running_prompts_in_dirs(&gemini_dirs_tagged" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "let is_running_or_recent" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "prompts_per_instance" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "CASCADE_RUN_STATUS_RUNNING', 1, ?3, 10" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "Retire stale 'running' prompts" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "let ag_prompts = discover_running_prompts_from_antigravity" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "Layer 2: VS Code" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "let proj_like" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "AND repo_path = ?2 AND prompt_content = ?3" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "instr(id, '__') = 0" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "folder_name" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "Helper to get live execution status" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "Find its most recent prompt" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "fn normalize_path_for_compare" src-tauri/src/modules/repo_db.rs --ext .rs
```

`let proj_like` has two hits: one in `backup_running_prompts` (Change G) and one in `auto_resume_recent_prompts` (Change L). Both are removed by this step.

Before adding new names, check they do not exist yet:

```text
gitmap aum search "LIVE_TURN_TTL_SECS|fn is_live_turn|fn live_summary_rows|fn summary_time_to_epoch|fn sweep_idle_running_prompts|fn prune_running_projects_for_instance|fn is_same_project_path|fn select_project_execution_status|fn latest_prompt_for_project" src-tauri/src --ext .rs
```

## 4. Current code

Expected drift: steps 04, 05 and 08 edit other lines of `backup_running_prompts` and `detect_running_projects` (upserts, canonical id). Blocks B, C, G, H and O below are quoted as step 05 leaves them (step 05 file, section 5), not as the original repository code. Compare only the exact blocks quoted below. If a quoted block itself differs (not just whitespace), STOP, except where a block says otherwise.

### Change A (`:907` to `:952`)

```rust
struct ConversationSummaryRow {
    cid: String,
    preview: String,
    status: String,
    not_fully_idle: i32,
    workspace_uris: Option<String>,
}

fn read_conversation_summary_rows(conn: &Connection) -> Vec<ConversationSummaryRow> {
    let wide = "SELECT conversation_id, preview, status, not_fully_idle, workspace_uris
         FROM conversation_summaries
         ORDER BY last_modified_time DESC
         LIMIT 25";
    if let Ok(mut stmt) = conn.prepare(wide) {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok(ConversationSummaryRow {
                cid: row.get(0)?,
                preview: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                status: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                not_fully_idle: row.get::<_, Option<i32>>(3)?.unwrap_or(0),
                workspace_uris: row.get(4)?,
            })
        }) {
            return rows.flatten().collect();
        }
    }
    let narrow = "SELECT conversation_id, preview, workspace_uris
         FROM conversation_summaries
         ORDER BY rowid DESC
         LIMIT 25";
    let Ok(mut stmt) = conn.prepare(narrow) else {
        return Vec::new();
    };
    let Ok(rows) = stmt.query_map([], |row| {
        Ok(ConversationSummaryRow {
            cid: row.get(0)?,
            preview: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
            status: String::new(),
            not_fully_idle: 1,
            workspace_uris: row.get(2)?,
        })
    }) else {
        return Vec::new();
    };
    rows.flatten().collect()
}
```

### Change B (last line of `discover_running_prompts_from_antigravity`, as step 05 wrote it)

```rust
    discover_running_prompts_in_dirs(&gemini_dirs_tagged(&scope))
}
```

### Change C (three places inside `discover_running_prompts_in_dirs`, as step 05 wrote it)

C1, the counter declaration near the top of the function:

```rust
    let mut prompts_per_instance: HashMap<String, usize> = HashMap::new();
```

C2, the row loop head:

```rust
        for item in rows {
            let cid = item.cid;
            let preview = item.preview;
            let status = item.status;
            let not_fully_idle = item.not_fully_idle;
            let ws_uris_opt = item.workspace_uris;
            if !seen_cids.insert((owner_id.clone(), cid.clone())) {
                continue;
            }

            let owner_count = prompts_per_instance.get(owner_id).copied().unwrap_or(0);
            let is_running_or_recent =
                not_fully_idle != 0 || status.contains("RUNNING") || owner_count == 0;
            if !is_running_or_recent && owner_count >= 5 {
                continue;
            }
```

C3, the counter increment directly after the `prompts.push(ActivePrompt { ... });` statement:

```rust
                *prompts_per_instance.entry(owner_id.clone()).or_insert(0) += 1;
```

### Change D (`:1127` to `:1132`)

```rust
    // Step 0: Retire stale 'running' prompts older than 2 hours to 'dispatched' so they don't shadow active prompts
    let stale_cutoff = now - 7200;
    let _ = conn.execute(
        "UPDATE active_prompts SET status = 'dispatched', updated_at = ? WHERE status = 'running' AND updated_at < ?",
        params![now, stale_cutoff],
    );
```

### Change E (`:1181` to `:1182`)

```rust
    // Layer 1: Core Antigravity Live Conversations Discovery (~/.gemini/antigravity)
    let ag_prompts = discover_running_prompts_from_antigravity(instance_id);
```

If step 05 or 08 changed the argument (for example to a canonical local such as `&inst`), note the exact argument; you reuse it in Change D.

### Change F (`:1238` to `:1239`)

```rust
    // Layer 2: VS Code / Instance Workspace Storage & Active Projects Discovery (Parallelized)
    let projects = detect_running_projects(instance_id).unwrap_or_default();
```

### Change G (inside the Layer 2 loop, directly after step 05's `let project_instance_id = match ... };` block)

```rust
        // Check if SQLite active_prompts already has an existing prompt for this repo_path or project
        if extracted_prompts.is_empty() {
            let proj_like = format!("%{}%", project.repo_name.to_lowercase());
            let mut check_stmt = conn
                .prepare(
                    "SELECT prompt_content, image_payload FROM active_prompts \
                     WHERE repo_path = ?1 OR project_id = ?2 OR project_id LIKE ?3 \
                     ORDER BY updated_at DESC LIMIT 1",
                )
                .ok();
            if let Some(ref mut c_stmt) = check_stmt {
                if let Ok((txt, img)) = c_stmt.query_row(
                    rusqlite::params![&project.repo_path, &project.id, &proj_like],
                    |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?)),
                ) {
                    if !txt.trim().is_empty() {
                        extracted_prompts.push((txt, img));
                    }
                }
            }
        }
```

### Change H (`:1334` to `:1341`)

```rust
        for (prompt_text, image_payload) in extracted_prompts {
            let existing_id: Option<String> = conn
                .query_row(
                    "SELECT id FROM active_prompts WHERE instance_id = ?1 AND repo_path = ?2 AND prompt_content = ?3 LIMIT 1",
                    rusqlite::params![&project_instance_id, &project.repo_path, &prompt_text],
                    |r| r.get(0),
                )
                .ok();
```

This is step 05's version. Change H is verify-only: if the block reads exactly like this, change nothing. If it still reads `WHERE repo_path = ?1 AND prompt_content = ?2 LIMIT 1`, step 05 was not applied: STOP.

### Change I (`:623` to `:651`, inside `detect_running_projects`)

```rust
    // Persist discovered projects into repo database
    if let Ok(conn) = connect_db() {
        let _ = conn.execute(
            "DELETE FROM running_projects WHERE workspace_storage_path IS NULL OR instr(id, '__') = 0",
            [],
        );

        let discovered_ids: std::collections::HashSet<String> =
            projects.iter().map(|p| p.id.clone()).collect();
        if let Ok(mut stmt) = conn.prepare(
            "SELECT id, workspace_storage_path FROM running_projects WHERE instance_id = ?1",
        ) {
            let existing_rows: Vec<(String, Option<String>)> = stmt
                .query_map(params![target_id], |r| Ok((r.get(0)?, r.get(1)?)))
                .map(|iter| iter.flatten().collect())
                .unwrap_or_default();
            for (old_id, wspath_opt) in existing_rows {
                let path_exists = wspath_opt
                    .as_deref()
                    .map(|p| Path::new(p).exists())
                    .unwrap_or(false);
                if !discovered_ids.contains(&old_id) || !path_exists {
                    let _ = conn.execute(
                        "DELETE FROM running_projects WHERE id = ?1",
                        params![&old_id],
                    );
                }
            }
        }
```

The block continues with `for p in &projects {`; that loop stays. The local `target_id` is the instance id this function resolved at its top (step 08 may have changed how it is computed; the name `target_id` must still exist, otherwise STOP).

### Change J (`:1999` to `:2013`, Gate 4 inside `is_prompt_running_for_project`)

```rust
                                for u in ws_uris {
                                    let clean_p =
                                        normalize_path_for_compare(&decode_uri_to_path(&u));
                                    let folder_name = Path::new(&clean_p)
                                        .file_name()
                                        .and_then(|n| n.to_str())
                                        .unwrap_or("")
                                        .to_lowercase();
                                    let has_target = !clean_target.is_empty();
                                    let is_target_matched = has_target
                                        && (clean_p == clean_target
                                            || folder_name == clean_target
                                            || clean_target
                                                .starts_with(&format!("{}-", folder_name)));
                                    if is_target_matched {
```

### Change K (`:2691` to `:2711`)

```rust
/// Helper to get live execution status for a specific project
pub fn get_project_execution_status(
    project_id: &str,
    instance_id: &str,
) -> Option<ProjectExecutionInfo> {
    if project_id.trim().is_empty() {
        return None;
    }
    let norm_target = normalize_path_for_compare(project_id);
    let norm_inst = crate::modules::instance::resolve_instance_id(instance_id)
        .unwrap_or_else(|_| instance_id.to_string())
        .to_lowercase();
    let infos = get_live_project_execution_info();
    infos.into_iter().find(|p| {
        (p.project_id == project_id
            || (!norm_target.is_empty() && normalize_path_for_compare(&p.repo_path) == norm_target))
            && (instance_id == "all"
                || instance_id.is_empty()
                || p.project_id.to_lowercase().contains(&norm_inst))
    })
}
```

Expected drift: step 08 may have replaced the `resolve_instance_id` line with `canonical_instance_id`. The whole function is replaced, so that drift is fine.

### Change L (`:3506` to `:3530`, inside `auto_resume_recent_prompts`)

```rust
        // Project was active within < 1 hour! Find its most recent prompt + image
        let proj_like = format!("%{}%", project.repo_name.to_lowercase());
        let mut prompt_stmt = conn
            .prepare(
                "SELECT id, prompt_content, model, image_payload 
                 FROM active_prompts 
                 WHERE (project_id = ?1 OR repo_path = ?2 OR project_id LIKE ?3)
                   AND (instance_id = ?4 OR (?4 = 'default' AND (instance_id = 'default' OR instance_id = '__default__')))
                 ORDER BY created_at DESC LIMIT 1",
            )
            .map_err(|e| format!("Failed to prepare prompt query: {}", e))?;

        let mut maybe_prompt = prompt_stmt
            .query_row(
                rusqlite::params![&project.id, &project.repo_path, &proj_like, norm_inst],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, Option<String>>(3)?,
                    ))
                },
            )
            .ok();
```

The resume-file fallback that follows (`// If not found in DB, check existing .antigravity_resume_task.json`) belongs to step 09. Do not change it here. The local `norm_inst` is the instance id this function computed at its top; it must still exist, otherwise STOP.

### Change M (`:2472` to `:2475`)

```rust
/// Normalize filesystem path for reliable cross-platform comparison
pub(crate) fn normalize_path_for_compare(p: &str) -> String {
    p.replace('\\', "/").trim_end_matches('/').to_lowercase()
}
```

(Step 08 made it `pub(crate)`. This function stays unchanged; new helpers are inserted directly after it.)

### Change N (test module, `:5946` to `:5964`)

```rust
    #[test]
    fn summary_rows_fall_back_when_status_columns_are_missing() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE conversation_summaries (conversation_id TEXT, preview TEXT, workspace_uris TEXT)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO conversation_summaries VALUES ('cid-1', 'running work', '[\"file:///d:/work/app\"]')",
            [],
        )
        .unwrap();
        let rows = read_conversation_summary_rows(&conn);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].cid, "cid-1");
        assert_eq!(rows[0].preview, "running work");
        assert_eq!(rows[0].not_fully_idle, 1);
    }
```

### Change O (test helper `write_summaries_fixture`, added by step 05)

```rust
        for &(cid, preview, repo_uri) in rows {
            conn.execute(
                "INSERT INTO conversation_summaries VALUES (?1, ?2, 'CASCADE_RUN_STATUS_RUNNING', 1, ?3, 10)",
                params![cid, preview, format!("[\"{}\"]", repo_uri)],
            )
            .unwrap();
        }
```

The fixture stores `last_modified_time = 10` (year 1970). After this step such a row is no longer a live turn, so step 05's tests `captured_prompt_id_includes_instance` and `same_conversation_id_in_two_instances_gives_two_rows` would find nothing. Change O stamps the current time instead.

## 5. New code

### Change A (replaces the struct and `read_conversation_summary_rows`)

```rust
/// Seconds after the last turn during which a RUNNING conversation still counts as live.
pub const LIVE_TURN_TTL_SECS: i64 = 600;

struct ConversationSummaryRow {
    cid: String,
    preview: String,
    status: String,
    not_fully_idle: i32,
    workspace_uris: Option<String>,
    last_turn_at: i64,
}

/// A conversation is in a live turn only when it is not idle, reports RUNNING, and its last
/// turn is within `LIVE_TURN_TTL_SECS`.
pub fn is_live_turn(not_fully_idle: i64, status: &str, last_turn_at: i64, now: i64) -> bool {
    not_fully_idle > 0 && status.contains("RUNNING") && now - last_turn_at <= LIVE_TURN_TTL_SECS
}

fn live_summary_rows(rows: Vec<ConversationSummaryRow>, now: i64) -> Vec<ConversationSummaryRow> {
    rows.into_iter()
        .filter(|row| {
            is_live_turn(
                i64::from(row.not_fully_idle),
                &row.status,
                row.last_turn_at,
                now,
            )
        })
        .collect()
}

fn summary_time_to_epoch(value: rusqlite::types::Value) -> i64 {
    match value {
        rusqlite::types::Value::Integer(ts) => parse_flexible_timestamp(&ts.to_string()),
        rusqlite::types::Value::Real(ts) => parse_flexible_timestamp(&(ts as i64).to_string()),
        rusqlite::types::Value::Text(text) => parse_flexible_timestamp(&text),
        _ => 0,
    }
}

fn read_conversation_summary_rows(conn: &Connection) -> Vec<ConversationSummaryRow> {
    let wide = "SELECT conversation_id, preview, status, not_fully_idle, workspace_uris, last_modified_time
         FROM conversation_summaries
         ORDER BY last_modified_time DESC
         LIMIT 25";
    if let Ok(mut stmt) = conn.prepare(wide) {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok(ConversationSummaryRow {
                cid: row.get(0)?,
                preview: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                status: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                not_fully_idle: row.get::<_, Option<i32>>(3)?.unwrap_or(0),
                workspace_uris: row.get(4)?,
                last_turn_at: summary_time_to_epoch(row.get::<_, rusqlite::types::Value>(5)?),
            })
        }) {
            return rows.flatten().collect();
        }
    }
    let narrow = "SELECT conversation_id, preview, workspace_uris
         FROM conversation_summaries
         ORDER BY rowid DESC
         LIMIT 25";
    let Ok(mut stmt) = conn.prepare(narrow) else {
        return Vec::new();
    };
    let Ok(rows) = stmt.query_map([], |row| {
        Ok(ConversationSummaryRow {
            cid: row.get(0)?,
            preview: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
            status: String::new(),
            not_fully_idle: 0,
            workspace_uris: row.get(2)?,
            last_turn_at: 0,
        })
    }) else {
        return Vec::new();
    };
    rows.flatten().collect()
}
```

`parse_flexible_timestamp` is defined later in the same file (`pub fn parse_flexible_timestamp(time_str: &str) -> i64`); calling it from here is fine.

### Change B (only dirs whose owner instance process is alive are scanned)

```rust
    let live_dirs: Vec<(String, PathBuf)> = gemini_dirs_tagged(&scope)
        .into_iter()
        .filter(|(owner_id, _)| is_registered_instance_alive(owner_id))
        .collect();
    discover_running_prompts_in_dirs(&live_dirs)
}
```

This keeps `discover_running_prompts_in_dirs` pure (step 05's fixture tests call it directly with temp dirs and never touch the registry). If the compiler says `gemini_dirs_tagged` does not return `Vec<(String, PathBuf)>`, STOP and report the real return type.

### Change C

C1: delete the line `let mut prompts_per_instance: HashMap<String, usize> = HashMap::new();`.

C2: replace the loop head with:

```rust
        for item in live_summary_rows(rows, now) {
            let cid = item.cid;
            let preview = item.preview;
            let ws_uris_opt = item.workspace_uris;
            if !seen_cids.insert((owner_id.clone(), cid.clone())) {
                continue;
            }
```

C3: delete the line `*prompts_per_instance.entry(owner_id.clone()).or_insert(0) += 1;`.

After the edit, `gitmap aum search "prompts_per_instance|owner_count|is_running_or_recent" src-tauri/src/modules/repo_db.rs --ext .rs` must return 0 hits. Also search the rest of the loop body for `status` and `not_fully_idle` used as plain locals (not the `status:` field inside `ActivePrompt { ... }`). Step 05's body has none. If one exists, STOP.

### Change D (replaces the stale retire; uses the discovery result for the sweep)

```rust
    let ag_prompts = discover_running_prompts_from_antigravity(instance_id);
    let live_cids: HashSet<String> = ag_prompts
        .iter()
        .filter_map(|p| p.session_id.clone())
        .filter(|cid| !cid.trim().is_empty())
        .collect();
    let swept = if is_registered_instance_alive(instance_id) {
        sweep_idle_running_prompts(&conn, instance_id, &live_cids)
    } else {
        0
    };
    if swept > 0 {
        crate::modules::logger::log_info(&format!(
            "[RepoDB] Liveness sweep removed {} idle running prompts for instance '{}'",
            swept, instance_id
        ));
    }
```

If Change E showed a different argument than `instance_id` (for example `&inst`), use that same argument in every place above (`discover_running_prompts_from_antigravity`, `is_registered_instance_alive`, `sweep_idle_running_prompts`, and the log).

The sweep runs only while the instance process is alive. When the instance is dead, discovery returns nothing, and sweeping would delete every `running` row; instead those rows fall through to the existing `running` to `backed_up` transition below and are preserved.

### Change E (Layer 1 keeps its comment; the discovery line is deleted because Change D now computes `ag_prompts`)

```rust
    // Layer 1: Core Antigravity Live Conversations Discovery (~/.gemini/antigravity)
```

The next line stays `for p in ag_prompts {`.

### Change F

```rust
    // Layer 2: VS Code / Instance Workspace Storage & Active Projects Discovery (Parallelized)
    let projects: Vec<RunningProject> = detect_running_projects(instance_id)
        .unwrap_or_default()
        .into_iter()
        .filter(|p| p.is_running)
        .collect();
```

`detect_running_projects` already scans exactly one instance (it resolves one registry entry and stamps every project with it), and step 05's loop re-checks `project_instance_id` per project, so only the `is_running` filter is new.

### Change G

```rust
        // Check if SQLite active_prompts already has an existing prompt for this repo_path or project
        if extracted_prompts.is_empty() {
            if let Some((_, txt, _, img)) = latest_prompt_for_project(
                &conn,
                &project_instance_id,
                &project.id,
                &project.repo_path,
            ) {
                if !txt.trim().is_empty() {
                    extracted_prompts.push((txt, img));
                }
            }
        }
```

### Change H

No edit (verify only, see section 4).

### Change O (test helper)

```rust
        let now = Utc::now().timestamp();
        for &(cid, preview, repo_uri) in rows {
            conn.execute(
                "INSERT INTO conversation_summaries VALUES (?1, ?2, 'CASCADE_RUN_STATUS_RUNNING', 1, ?3, ?4)",
                params![cid, preview, format!("[\"{}\"]", repo_uri), now],
            )
            .unwrap();
        }
```

### Change I

```rust
    // Persist discovered projects into repo database
    if let Ok(conn) = connect_db() {
        let discovered_ids: std::collections::HashSet<String> =
            projects.iter().map(|p| p.id.clone()).collect();
        prune_running_projects_for_instance(&conn, target_id, &discovered_ids);
```

The `for p in &projects {` loop and everything after it stay unchanged.

### Change J

```rust
                                for u in ws_uris {
                                    let clean_p =
                                        normalize_path_for_compare(&decode_uri_to_path(&u));
                                    let is_target_matched =
                                        is_same_project_path(&clean_p, &clean_target);
                                    if is_target_matched {
```

### Change K (replaces the whole function)

```rust
/// Helper to get live execution status for a specific project
pub fn get_project_execution_status(
    project_id: &str,
    instance_id: &str,
) -> Option<ProjectExecutionInfo> {
    if project_id.trim().is_empty() {
        return None;
    }
    let inst = if instance_id == "all" || instance_id.is_empty() {
        None
    } else {
        Some(crate::modules::instance::canonical_instance_id(instance_id).ok()?)
    };
    select_project_execution_status(
        get_live_project_execution_info(),
        project_id,
        inst.as_deref(),
    )
}
```

### Change L

```rust
        // Project was active within < 1 hour! Find its most recent prompt + image
        let mut maybe_prompt =
            latest_prompt_for_project(&conn, norm_inst, &project.id, &project.repo_path);
```

If `norm_inst` is a `String` after step 08 (not a `&str`), write `&norm_inst` instead; the compiler error names this line.

### Change M (insert directly after `normalize_path_for_compare`)

```rust
/// True when both paths name the same folder after normalization. Never matches a prefix or a
/// neighbor folder such as `app` against `app-v2`.
pub(crate) fn is_same_project_path(candidate: &str, target: &str) -> bool {
    let target_key = normalize_path_for_compare(target);
    !target_key.is_empty() && normalize_path_for_compare(candidate) == target_key
}

/// Removes `running` rows of one instance whose conversation is not in `live_cids`. Rows of other
/// instances and rows in any other status are never touched.
pub(crate) fn sweep_idle_running_prompts(
    conn: &Connection,
    instance_id: &str,
    live_cids: &HashSet<String>,
) -> usize {
    let mut removed = 0;
    for prompt in select_instance_prompts_by_status(conn, instance_id, &[prompt_status::RUNNING]) {
        let cid = prompt.session_id.clone().unwrap_or_default();
        if !cid.trim().is_empty() && live_cids.contains(&cid) {
            continue;
        }
        crate::modules::logger::log_instance_prompt_audit(
            instance_id,
            instance_id,
            &prompt.project_id,
            &prompt.repo_path,
            "",
            None,
            &format!("LivenessSweep:ConversationNotLive prompt={}", prompt.id),
            false,
            "RUNNING_ROW_REMOVED_NOT_LIVE",
        );
        removed += conn
            .execute(
                "DELETE FROM active_prompts WHERE id = ?1 AND status = ?2",
                params![&prompt.id, prompt_status::RUNNING],
            )
            .unwrap_or(0);
        if let Ok(mut map) = get_memory_prompts_map().lock() {
            map.remove(&prompt.id);
        }
    }
    removed
}

/// Deletes this instance's legacy `running_projects` ids (no `__{instance}` suffix) and its
/// workspace rows that were not rediscovered or whose workspace folder is gone. Rows written by
/// the other discovery path (no workspace path) and rows of other instances are kept.
pub(crate) fn prune_running_projects_for_instance(
    conn: &Connection,
    instance_id: &str,
    discovered_ids: &HashSet<String>,
) -> usize {
    let mut removed = conn
        .execute(
            "DELETE FROM running_projects WHERE instance_id = ?1 AND substr(id, -length('__' || ?1)) <> '__' || ?1",
            params![instance_id],
        )
        .unwrap_or(0);
    let mut workspace_rows: Vec<(String, String)> = Vec::new();
    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, workspace_storage_path FROM running_projects
         WHERE instance_id = ?1 AND workspace_storage_path IS NOT NULL AND trim(workspace_storage_path) != ''",
    ) {
        if let Ok(mapped) = stmt.query_map(params![instance_id], |r| Ok((r.get(0)?, r.get(1)?))) {
            workspace_rows.extend(mapped.flatten());
        }
    }
    for (id, workspace_path) in workspace_rows {
        if !discovered_ids.contains(&id) || !Path::new(&workspace_path).exists() {
            removed += conn
                .execute("DELETE FROM running_projects WHERE id = ?1", params![&id])
                .unwrap_or(0);
        }
    }
    removed
}

/// First entry whose project id ends with exactly `__{instance_id}` (or any entry when
/// `instance_id` is `None`) and whose id or normalized repo path equals `project_id`.
pub(crate) fn select_project_execution_status(
    infos: Vec<ProjectExecutionInfo>,
    project_id: &str,
    instance_id: Option<&str>,
) -> Option<ProjectExecutionInfo> {
    let target_key = normalize_path_for_compare(project_id);
    let suffix = instance_id.map(|inst| format!("__{}", inst));
    infos.into_iter().find(|p| {
        let is_project_match = p.project_id == project_id
            || (!target_key.is_empty() && normalize_path_for_compare(&p.repo_path) == target_key);
        let is_instance_match = suffix
            .as_deref()
            .map(|s| p.project_id.ends_with(s))
            .unwrap_or(true);
        is_project_match && is_instance_match
    })
}

/// Newest prompt of exactly this instance whose project id or normalized repo path matches.
/// Returns `(id, prompt_content, model, image_payload)`.
pub(crate) fn latest_prompt_for_project(
    conn: &Connection,
    instance_id: &str,
    project_id: &str,
    repo_path: &str,
) -> Option<(String, String, Option<String>, Option<String>)> {
    let repo_key = normalize_path_for_compare(repo_path);
    let mut stmt = conn
        .prepare(
            "SELECT id, prompt_content, model, image_payload, project_id, repo_path
             FROM active_prompts
             WHERE instance_id = ?1
             ORDER BY created_at DESC, updated_at DESC",
        )
        .ok()?;
    let rows = stmt
        .query_map(params![instance_id], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, Option<String>>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
            ))
        })
        .ok()?;
    for (id, content, model, image, row_project_id, row_repo_path) in rows.flatten() {
        let is_match = row_project_id == project_id
            || (!repo_key.is_empty() && normalize_path_for_compare(&row_repo_path) == repo_key);
        if is_match {
            return Some((id, content, model, image));
        }
    }
    None
}
```

`HashSet`, `Path`, `params` and `Connection` are already imported at the top of `repo_db.rs`.

### Change N (edit the last assertion of the existing test)

```rust
        assert_eq!(rows[0].not_fully_idle, 0);
        assert_eq!(rows[0].last_turn_at, 0);
    }
```

## 6. Tests

Paste into the existing `#[cfg(test)] mod tests` of `src-tauri/src/modules/repo_db.rs` (near `:5880`), before its final closing `}`.

```rust
    fn live_seed_row(conn: &Connection, id: &str, instance_id: &str, session_id: Option<&str>, status: &str) {
        conn.execute(
            "INSERT INTO active_prompts
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES (?1, 'proj', ?2, '/work/app', 'keep going', NULL, ?3, ?4, 1, 1, NULL)",
            params![id, instance_id, session_id, status],
        )
        .unwrap();
    }

    fn live_row_exists(conn: &Connection, id: &str) -> bool {
        conn.query_row(
            "SELECT COUNT(*) FROM active_prompts WHERE id = ?1",
            params![id],
            |r| r.get::<_, i64>(0),
        )
        .unwrap()
            == 1
    }

    #[test]
    fn is_live_turn_respects_live_turn_ttl() {
        assert!(is_live_turn(1, "CASCADE_RUN_STATUS_RUNNING", 400, 1000));
        assert!(!is_live_turn(1, "CASCADE_RUN_STATUS_RUNNING", 399, 1000));
        assert!(!is_live_turn(0, "CASCADE_RUN_STATUS_RUNNING", 1000, 1000));
        assert!(!is_live_turn(1, "CASCADE_RUN_STATUS_IDLE", 1000, 1000));
    }

    #[test]
    fn fallback_schema_row_is_never_live() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE conversation_summaries (conversation_id TEXT, preview TEXT, workspace_uris TEXT)",
            [],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO conversation_summaries VALUES ('cid-1', 'running work', '[\"file:///work/app\"]')",
            [],
        )
        .unwrap();
        let rows = read_conversation_summary_rows(&conn);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].not_fully_idle, 0);
        assert!(live_summary_rows(rows, Utc::now().timestamp()).is_empty());
    }

    #[test]
    fn first_summary_row_is_not_running_when_idle() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE conversation_summaries (conversation_id TEXT, preview TEXT, status TEXT, not_fully_idle INTEGER, workspace_uris TEXT, last_modified_time TEXT)",
            [],
        )
        .unwrap();
        let now = Utc::now().timestamp();
        let recent = (now - 5).to_string();
        conn.execute(
            "INSERT INTO conversation_summaries VALUES ('cid-idle', 'done work', 'CASCADE_RUN_STATUS_IDLE', 0, '[\"file:///work/app\"]', ?1)",
            params![recent],
        )
        .unwrap();
        let rows = read_conversation_summary_rows(&conn);
        assert_eq!(rows.len(), 1);
        assert!(live_summary_rows(rows, now).is_empty());

        conn.execute(
            "INSERT INTO conversation_summaries VALUES ('cid-live', 'busy work', 'CASCADE_RUN_STATUS_RUNNING', 1, '[\"file:///work/app\"]', ?1)",
            params![recent],
        )
        .unwrap();
        let live = live_summary_rows(read_conversation_summary_rows(&conn), now);
        assert_eq!(live.len(), 1);
        assert_eq!(live[0].cid, "cid-live");
    }

    #[test]
    fn liveness_sweep_removes_only_idle_running_rows_of_instance() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        live_seed_row(&conn, "a-live", "inst-a", Some("cid-live"), "running");
        live_seed_row(&conn, "a-idle", "inst-a", Some("cid-idle"), "running");
        live_seed_row(&conn, "a-nocid", "inst-a", None, "running");
        live_seed_row(&conn, "a-backed", "inst-a", Some("cid-old"), "backed_up");
        live_seed_row(&conn, "b-idle", "inst-b", Some("cid-b"), "running");

        let live: HashSet<String> = ["cid-live".to_string()].into_iter().collect();
        let removed = sweep_idle_running_prompts(&conn, "inst-a", &live);

        assert_eq!(removed, 2);
        assert!(live_row_exists(&conn, "a-live"));
        assert!(!live_row_exists(&conn, "a-idle"));
        assert!(!live_row_exists(&conn, "a-nocid"));
        assert!(live_row_exists(&conn, "a-backed"));
        assert!(live_row_exists(&conn, "b-idle"));
    }

    #[test]
    fn detect_running_projects_keeps_other_writer_rows() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        let existing_dir = std::env::temp_dir().to_string_lossy().to_string();
        let rows: [(&str, &str, Option<&str>); 5] = [
            ("app-abcd1234__inst-a", "inst-a", None),
            ("legacy-app", "inst-a", None),
            ("legacy-b", "inst-b", None),
            ("app-gone__inst-a", "inst-a", Some("/no/such/agm/workspace/dir")),
            ("app-live__inst-a", "inst-a", Some(existing_dir.as_str())),
        ];
        for (id, inst, ws) in rows {
            conn.execute(
                "INSERT INTO running_projects
                 (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
                 VALUES (?1, ?2, 'app', '/work/app', ?3, 0, 1, 1)",
                params![id, inst, ws],
            )
            .unwrap();
        }
        let discovered: HashSet<String> = ["app-live__inst-a".to_string()].into_iter().collect();

        prune_running_projects_for_instance(&conn, "inst-a", &discovered);

        let remaining: Vec<String> = conn
            .prepare("SELECT id FROM running_projects ORDER BY id")
            .unwrap()
            .query_map([], |r| r.get(0))
            .unwrap()
            .flatten()
            .collect();
        assert_eq!(
            remaining,
            vec![
                "app-abcd1234__inst-a".to_string(),
                "app-live__inst-a".to_string(),
                "legacy-b".to_string(),
            ]
        );
    }

    #[test]
    fn gate4_does_not_match_prefix_folder() {
        assert!(!is_same_project_path("d:/work/app-v2", "D:\\work\\app"));
        assert!(!is_same_project_path("d:/work/app", "D:\\work\\app-v2"));
        assert!(is_same_project_path("D:/Work/App/", "d:\\work\\app"));
        assert!(!is_same_project_path("", ""));
    }

    #[test]
    fn execution_status_default_does_not_match_default_copy() {
        let info = |project_id: &str, repo_path: &str| ProjectExecutionInfo {
            project_id: project_id.to_string(),
            repo_name: "x".to_string(),
            repo_path: repo_path.to_string(),
            is_running: true,
            is_idle: false,
            status: "RUNNING".to_string(),
            active_prompt: None,
            last_detected_at: 1,
        };
        let infos = vec![info("x__default-copy-8159", "/work/x")];
        assert!(select_project_execution_status(infos.clone(), "/work/x", Some("default")).is_none());
        assert!(
            select_project_execution_status(infos.clone(), "/work/x", Some("default-copy-8159"))
                .is_some()
        );
        assert!(select_project_execution_status(infos, "/work/x", None).is_some());
    }

    #[test]
    fn auto_resume_prompt_lookup_ignores_neighbor_repo_and_other_instance() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        let rows = [
            ("p-app", "inst-a", "/work/app", 10),
            ("p-app-v2", "inst-a", "/work/app-v2", 20),
            ("p-other-inst", "inst-b", "/work/app", 30),
        ];
        for (id, inst, repo, created) in rows {
            conn.execute(
                "INSERT INTO active_prompts
                 (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
                 VALUES (?1, 'unrelated', ?2, ?3, ?1, NULL, NULL, 'dispatched', ?4, ?4, NULL)",
                params![id, inst, repo, created],
            )
            .unwrap();
        }
        let found = latest_prompt_for_project(&conn, "inst-a", "app__inst-a", "D:/../work/app")
            .map(|(id, _, _, _)| id);
        assert_eq!(found, None);
        let found = latest_prompt_for_project(&conn, "inst-a", "app__inst-a", "/WORK/APP/")
            .map(|(id, _, _, _)| id);
        assert_eq!(found.as_deref(), Some("p-app"));
    }
```

`ProjectExecutionInfo` derives `Clone`, so `infos.clone()` compiles.

## 7. Gate

```text
$env:CARGO_TARGET_DIR="$PWD\src-tauri\target"; $env:CARGO_BUILD_JOBS="1"
cd src-tauri; cargo fmt; cargo fmt -- --check; cargo clippy --all-targets --all-features; cargo test --lib -- --test-threads=1 is_live_turn_respects_live_turn_ttl; cd ..
cd src-tauri; cargo test --lib -- --test-threads=1 fallback_schema_row_is_never_live; cargo test --lib -- --test-threads=1 first_summary_row_is_not_running_when_idle; cargo test --lib -- --test-threads=1 summary_rows_fall_back_when_status_columns_are_missing; cd ..
cd src-tauri; cargo test --lib -- --test-threads=1 liveness_sweep_; cargo test --lib -- --test-threads=1 detect_running_projects_keeps_other_writer_rows; cargo test --lib -- --test-threads=1 gate4_does_not_match_prefix_folder; cd ..
cd src-tauri; cargo test --lib -- --test-threads=1 execution_status_default_does_not_match_default_copy; cargo test --lib -- --test-threads=1 auto_resume_prompt_lookup_; cd ..
cd src-tauri; cargo test --lib -- --test-threads=1 captured_prompt_id_includes_instance; cargo test --lib -- --test-threads=1 same_conversation_id_in_two_instances_gives_two_rows; cd ..
```

Every command must exit 0 and each filter must run at least one test.

## 8. Commit

```text
Fix: prompts - running only for live turns of a live instance
```

Stage only:

```text
git add src-tauri/src/modules/repo_db.rs
```

## 9. Done when

- [ ] `ConversationSummaryRow` has `last_turn_at`; the narrow fallback sets `not_fully_idle: 0` and `last_turn_at: 0`.
- [ ] `is_live_turn` and `LIVE_TURN_TTL_SECS = 600` exist; `discover_running_prompts_in_dirs` iterates `live_summary_rows(rows, now)`; `prompts_per_instance`, `owner_count` and `is_running_or_recent` are gone.
- [ ] `discover_running_prompts_from_antigravity` scans only dirs whose owner passes `is_registered_instance_alive`.
- [ ] Step 05's `write_summaries_fixture` stamps the current time.
- [ ] `backup_running_prompts` no longer retires `running` rows to `dispatched`; it runs `sweep_idle_running_prompts` with the live conversation ids only when `is_registered_instance_alive` is true, and calls `discover_running_prompts_from_antigravity` only once.
- [ ] Layer 2 only scans projects with `is_running`; its fallback uses `latest_prompt_for_project` with `project_instance_id` and contains no `LIKE`.
- [ ] `detect_running_projects` no longer runs the global `DELETE ... instr(id, '__') = 0`; it calls `prune_running_projects_for_instance`.
- [ ] Gate 4 uses `is_same_project_path`; `folder_name` no longer exists in `is_prompt_running_for_project`.
- [ ] `get_project_execution_status` uses `select_project_execution_status` (exact `__{id}` suffix).
- [ ] `auto_resume_recent_prompts` uses `latest_prompt_for_project`; `proj_like` no longer exists anywhere in `repo_db.rs`.
- [ ] `is_instance_running` in `instance.rs` is untouched.
- [ ] Callers searched in `src-tauri/src`, `src-tauri/src/bin/agm.rs` and each `src-tauri/tests/*.rs` file. No caller edit is needed. The only struct change is the private `ConversationSummaryRow`, whose two initializers inside `read_conversation_summary_rows` are both covered above. `backup_running_prompts`, `detect_running_projects`, `get_project_execution_status`, `auto_resume_recent_prompts` and `is_prompt_running_for_project` keep their signatures and return types. Every new helper is private or `pub(crate)` and new.
- [ ] Gate exits 0.
- [ ] Only `repo_db.rs` is staged.
