# Step 05: capture stamps the instance once (row id prompt-{instance}-{cid})

A captured prompt gets its instance id from the `(instance_id, dir)` pair that produced it, and nothing later rewrites it. After this step:

- `discover_running_prompts_in_dirs(&[(String, PathBuf)])` (pure core over dirs) builds `ActivePrompt` rows with id `prompt-{instance_id}-{cid}`, `instance_id` = the dir owner, `source_dir` = that dir. Deduplication is per `(instance_id, cid)`, so the same conversation id in two instances gives two rows.
- `discover_running_prompts_from_antigravity(&str)` keeps its signature. `"all"` scans `InstanceScope::All`; anything else goes through `canonical_instance_id` (unknown ids log and return nothing).
- Layer 1 of `backup_running_prompts` upserts through `upsert_captured_prompt`, whose `ON CONFLICT(id) DO UPDATE` never touches `instance_id`.
- Layer 2 stamps each row with the canonical id of the project that produced it (not the raw function argument, which may be `"all"`), looks up existing rows only within that instance, and stores `session_id = NULL` (it used to store the project id, bug B10).

## 1. Depends on

Steps 03 and 04.

## 2. Files you may edit

- `src-tauri/src/modules/repo_db.rs`

## 3. Find it

| Change | Anchor | Unique search literal | Line hint (before step 03/04; add about 40) |
|---|---|---|---|
| A. Replace discovery | `pub fn discover_running_prompts_from_antigravity(instance_id: &str) -> Vec<ActivePrompt> {` | `let p_id = format!` | 954 to 1098 |
| B. Add `upsert_captured_prompt` | `pub fn backup_running_prompts(instance_id: &str) -> Result<usize, String> {` | `/// Backup all currently running prompts across active projects before switching` | 1121 |
| C. Layer 1 upsert call | inside `backup_running_prompts` | `image_payload = excluded.image_payload",` | 1195 to 1216 |
| D. Layer 2 loop | inside `backup_running_prompts` | `for (project, mut extracted_prompts) in extracted_results {` | 1311 to 1420 |
| E. Tests | `mod tests {` | `fn test_uri_decoding()` | about 5900 |

```text
gitmap aum search "pub fn discover_running_prompts_from_antigravity" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "image_payload = excluded.image_payload" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "for (project, mut extracted_prompts) in extracted_results" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "discover_running_prompts_from_antigravity(" src-tauri/src --ext .rs
gitmap aum search "discover_running_prompts_from_antigravity|backup_running_prompts(" src-tauri/src/bin/agm.rs --ext .rs
gitmap aum search "discover_running_prompts_from_antigravity|backup_running_prompts(" src-tauri/tests/auto_switcher_e2e_test.rs --ext .rs
gitmap aum search "discover_running_prompts_from_antigravity|backup_running_prompts(" src-tauri/tests/instance_cloning_and_sync_test.rs --ext .rs
gitmap aum search "discover_running_prompts_from_antigravity|backup_running_prompts(" src-tauri/tests/per_instance_prompt_liveness_test.rs --ext .rs
```

Callers of `discover_running_prompts_from_antigravity` (no change needed, the signature stays `(&str) -> Vec<ActivePrompt>`):

- `src-tauri/src/modules/backup_prompts_db.rs:238` passes `target_inst` (`instance_id.unwrap_or("default")`).
- `src-tauri/src/modules/repo_db.rs:1182` (Layer 1 of `backup_running_prompts`, may pass `"all"`).
- `src-tauri/src/modules/repo_db.rs:4053` and `:4059` pass `"__default__"` and discard the result.
- `src-tauri/src/bin/agm.rs` and `src-tauri/tests/*.rs`: no callers.

Callers of `backup_running_prompts` (no change needed; its signature `(instance_id: &str) -> Result<usize, String>` stays):

- `src-tauri/src/bin/agm.rs`: `:3373`, `:4060`, `:7971` (`"default"`), `:10640`, `:10913`, `:14508`, `:14725` (all pass `&str`; the other hits are the `cmd_backup_running_prompts` CLI wrapper).
- `src-tauri/tests/auto_switcher_e2e_test.rs:135`: `let backup_repo_res = antigravity_tools_lib::modules::repo_db::backup_running_prompts(inst_id);` with `inst_id: &str = "test-e2e-instance"`. It still compiles, which is all the gate needs (`clippy --all-targets`). The test is `#[ignore]` (local-only) and not run by the gate; with an unregistered id, discovery now logs and returns no rows instead of stamping them `test-e2e-instance`.
- The other two test files: no callers.

## 4. Current code

### A. Discovery (lines 954 to 1098; after step 03 the `ActivePrompt` literal also has `source_dir: None,`)

```rust
/// Discover in-flight active conversations and running prompts directly from Antigravity core storage
/// (~/.gemini/antigravity/conversation_summaries.db and brain/<cid>/.system_generated/logs/transcript.jsonl)
pub fn discover_running_prompts_from_antigravity(instance_id: &str) -> Vec<ActivePrompt> {
    let mut prompts = Vec::new();
    let mut seen_cids = std::collections::HashSet::new();
    let candidate_dirs = gemini_dirs_for_instance(instance_id);

    for base_dir in candidate_dirs {
        let summaries_db = base_dir.join("conversation_summaries.db");
        if !summaries_db.exists() {
            continue;
        }

        let conn = match Connection::open_with_flags(
            &summaries_db,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
        ) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let now = Utc::now().timestamp();
        let rows = read_conversation_summary_rows(&conn);
        if rows.is_empty() {
            continue;
        }

        for item in rows {
            let cid = item.cid;
            let preview = item.preview;
            let status = item.status;
            let not_fully_idle = item.not_fully_idle;
            let ws_uris_opt = item.workspace_uris;
            if !seen_cids.insert(cid.clone()) {
                continue;
            }

            let is_running_or_recent =
                not_fully_idle != 0 || status.contains("RUNNING") || prompts.is_empty();
            if !is_running_or_recent && prompts.len() >= 5 {
                continue;
            }

            let Some(ws_uris_raw) = ws_uris_opt else {
                continue;
            };

            let ws_uris: Vec<String> = serde_json::from_str(&ws_uris_raw).unwrap_or_default();
            if ws_uris.is_empty() {
                continue;
            }

            let repo_path = decode_uri_to_path(&ws_uris[0]);
            let repo_name = Path::new(&repo_path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "antigravity-project".to_string());
            let project_id = format!(
                "{}-{}",
                repo_name.to_lowercase(),
                cid.chars().take(8).collect::<String>()
            );

            // Read transcript.jsonl from brain/<cid>/.system_generated/logs/transcript.jsonl
            let transcript_file = base_dir
                .join("brain")
                .join(&cid)
                .join(".system_generated")
                .join("logs")
                .join("transcript.jsonl");

            let mut user_prompt: Option<String> = None;
            let mut image_payload: Option<String> = None;

            if transcript_file.exists() {
                if let Ok(content) = fs::read_to_string(&transcript_file) {
                    for line in content.lines().rev() {
                        if !line.contains("USER_INPUT") {
                            continue;
                        }
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                            if val.get("type").and_then(|t| t.as_str()) == Some("USER_INPUT") {
                                if let Some(txt) = val.get("content").and_then(|c| c.as_str()) {
                                    if !txt.trim().is_empty() {
                                        user_prompt = Some(txt.to_string());
                                    }
                                }
                                if let Some(media_arr) = val.get("media").and_then(|m| m.as_array())
                                {
                                    for m_item in media_arr {
                                        if let Some(uri) =
                                            m_item.get("uri").and_then(|u| u.as_str())
                                        {
                                            let clean_path = decode_uri_to_path(uri);
                                            let p = Path::new(&clean_path);
                                            if p.exists() {
                                                if let Ok(bytes) = fs::read(p) {
                                                    let mime = m_item
                                                        .get("mime_type")
                                                        .and_then(|mt| mt.as_str())
                                                        .unwrap_or("image/png");
                                                    let b64 = STANDARD.encode(&bytes);
                                                    image_payload = Some(format!(
                                                        "data:{};base64,{}",
                                                        mime, b64
                                                    ));
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                                if user_prompt.is_some() {
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            if user_prompt.is_none() {
                user_prompt = live_prompt_text(&preview, "");
            }

            if let Some(prompt_text) = user_prompt {
                let p_id = format!("prompt-{}", cid);
                prompts.push(ActivePrompt {
                    id: p_id,
                    project_id,
                    instance_id: instance_id.to_string(),
                    repo_path,
                    prompt_content: prompt_text,
                    model: Some("gemini-pro".to_string()),
                    session_id: Some(cid),
                    status: "backed_up".to_string(),
                    created_at: now,
                    updated_at: now,
                    image_payload,
                    source_dir: None,
                });
            }
        }
    }

    prompts
}
```

### B. The line where `upsert_captured_prompt` goes (line 1121)

```rust
/// Backup all currently running prompts across active projects before switching
pub fn backup_running_prompts(instance_id: &str) -> Result<usize, String> {
```

### C. Layer 1 upsert statement (lines 1195 to 1216)

```rust
        let res = conn.execute(
            "INSERT INTO active_prompts 
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'backed_up', ?8, ?9, ?10)
             ON CONFLICT(id) DO UPDATE SET 
                 status = 'backed_up',
                 updated_at = excluded.updated_at,
                 prompt_content = excluded.prompt_content,
                 image_payload = excluded.image_payload",
            params![
                &p.id,
                &p.project_id,
                &p.instance_id,
                &p.repo_path,
                &p.prompt_content,
                &p.model,
                &p.session_id,
                p.created_at,
                now,
                &p.image_payload,
            ],
        );
```

### D. Layer 2 loop (lines 1311 to 1420; after step 03 the `ActivePrompt` literal also has `source_dir: None,`)

```rust
    for (project, mut extracted_prompts) in extracted_results {
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

        for (prompt_text, image_payload) in extracted_prompts {
            let existing_id: Option<String> = conn
                .query_row(
                    "SELECT id FROM active_prompts WHERE repo_path = ?1 AND prompt_content = ?2 LIMIT 1",
                    rusqlite::params![&project.repo_path, &prompt_text],
                    |r| r.get(0),
                )
                .ok();

            let (prompt_id, is_existing) = match existing_id {
                Some(eid) => (eid, true),
                None => (Uuid::new_v4().to_string(), false),
            };

            let prompt_model = Some("gemini-pro".to_string());
            let result = if is_existing {
                conn.execute(
                    "UPDATE active_prompts SET status = 'backed_up', updated_at = ?, image_payload = COALESCE(?, image_payload) WHERE id = ?",
                    params![now, &image_payload, &prompt_id],
                )
            } else {
                conn.execute(
                    "INSERT INTO active_prompts 
                     (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
                     VALUES (?, ?, ?, ?, ?, ?, ?, 'backed_up', ?, ?, ?)",
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
                )
            };

            if result.is_ok() {
                backed_up_count += 1;
                let (extracted_img, img_paths) = extract_image_payload_or_path(&prompt_text);
                let final_img = image_payload.clone().or(extracted_img);
                let has_image = final_img.is_some() || !img_paths.is_empty();

                // Write disk resume snapshot file inside project repo directory
                let task_file =
                    PathBuf::from(&project.repo_path).join(".antigravity_resume_task.json");
                let payload = serde_json::json!({
                    "prompt_id": prompt_id,
                    "project_id": project.id,
                    "instance_id": instance_id,
                    "repo_path": project.repo_path,
                    "prompt_content": prompt_text,
                    "model": prompt_model,
                    "session_id": project.id,
                    "image_payload": final_img,
                    "image_paths": img_paths,
                    "has_image": has_image,
                    "auto_boot": true,
                    "status": "backed_up",
                    "backed_up_at": now,
                });
                if let Ok(json_str) = serde_json::to_string_pretty(&payload) {
                    let _ = fs::write(&task_file, json_str);
                }

                let active_prompt = ActivePrompt {
                    id: prompt_id.clone(),
                    project_id: project.id.clone(),
                    instance_id: instance_id.to_string(),
                    repo_path: project.repo_path.clone(),
                    prompt_content: prompt_text,
                    model: prompt_model,
                    session_id: Some(project.id.clone()),
                    status: "backed_up".to_string(),
                    created_at: now,
                    updated_at: now,
                    image_payload: final_img,
                    source_dir: None,
                };
                if let Ok(mut map) = get_memory_prompts_map().lock() {
                    map.insert(prompt_id, active_prompt);
                }
            }
        }
    }
```

Facts you can rely on: `RunningProject` has a field `instance_id: String` (line 59). `read_conversation_summary_rows(&Connection)` (line 915) needs the columns `conversation_id, preview, status, not_fully_idle, workspace_uris, last_modified_time` and falls back to `conversation_id, preview, workspace_uris`. `live_prompt_text(preview, "")` returns the trimmed preview. `HashMap` and `HashSet` are imported at the top of the file.

## 5. New code

### A. Replace the whole block from 4 A with

```rust
/// Discover in-flight active conversations and running prompts directly from Antigravity core storage
/// (~/.gemini/antigravity/conversation_summaries.db and brain/<cid>/.system_generated/logs/transcript.jsonl)
pub fn discover_running_prompts_from_antigravity(instance_id: &str) -> Vec<ActivePrompt> {
    let scope = if instance_id.trim().eq_ignore_ascii_case("all") {
        InstanceScope::All
    } else {
        match crate::modules::instance::canonical_instance_id(instance_id) {
            Ok(id) => InstanceScope::One(id),
            Err(e) => {
                crate::modules::logger::log_warn(&format!(
                    "[RepoDB] Prompt discovery skipped: {}",
                    e
                ));
                return Vec::new();
            }
        }
    };
    discover_running_prompts_in_dirs(&gemini_dirs_tagged(&scope))
}

/// Captured rows are stamped with the owner of the dir they came from: id `prompt-{instance}-{cid}`.
fn discover_running_prompts_in_dirs(tagged_dirs: &[(String, PathBuf)]) -> Vec<ActivePrompt> {
    let mut prompts = Vec::new();
    let mut seen_cids: HashSet<(String, String)> = HashSet::new();
    let mut prompts_per_instance: HashMap<String, usize> = HashMap::new();

    for (owner_id, base_dir) in tagged_dirs {
        let summaries_db = base_dir.join("conversation_summaries.db");
        if !summaries_db.exists() {
            continue;
        }

        let conn = match Connection::open_with_flags(
            &summaries_db,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
        ) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let now = Utc::now().timestamp();
        let rows = read_conversation_summary_rows(&conn);
        if rows.is_empty() {
            continue;
        }

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

            let Some(ws_uris_raw) = ws_uris_opt else {
                continue;
            };

            let ws_uris: Vec<String> = serde_json::from_str(&ws_uris_raw).unwrap_or_default();
            if ws_uris.is_empty() {
                continue;
            }

            let repo_path = decode_uri_to_path(&ws_uris[0]);
            let repo_name = Path::new(&repo_path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "antigravity-project".to_string());
            let project_id = format!(
                "{}-{}",
                repo_name.to_lowercase(),
                cid.chars().take(8).collect::<String>()
            );

            // Read transcript.jsonl from brain/<cid>/.system_generated/logs/transcript.jsonl
            let transcript_file = base_dir
                .join("brain")
                .join(&cid)
                .join(".system_generated")
                .join("logs")
                .join("transcript.jsonl");

            let mut user_prompt: Option<String> = None;
            let mut image_payload: Option<String> = None;

            if transcript_file.exists() {
                if let Ok(content) = fs::read_to_string(&transcript_file) {
                    for line in content.lines().rev() {
                        if !line.contains("USER_INPUT") {
                            continue;
                        }
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(line) {
                            if val.get("type").and_then(|t| t.as_str()) == Some("USER_INPUT") {
                                if let Some(txt) = val.get("content").and_then(|c| c.as_str()) {
                                    if !txt.trim().is_empty() {
                                        user_prompt = Some(txt.to_string());
                                    }
                                }
                                if let Some(media_arr) = val.get("media").and_then(|m| m.as_array())
                                {
                                    for m_item in media_arr {
                                        if let Some(uri) =
                                            m_item.get("uri").and_then(|u| u.as_str())
                                        {
                                            let clean_path = decode_uri_to_path(uri);
                                            let p = Path::new(&clean_path);
                                            if p.exists() {
                                                if let Ok(bytes) = fs::read(p) {
                                                    let mime = m_item
                                                        .get("mime_type")
                                                        .and_then(|mt| mt.as_str())
                                                        .unwrap_or("image/png");
                                                    let b64 = STANDARD.encode(&bytes);
                                                    image_payload = Some(format!(
                                                        "data:{};base64,{}",
                                                        mime, b64
                                                    ));
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                                if user_prompt.is_some() {
                                    break;
                                }
                            }
                        }
                    }
                }
            }

            if user_prompt.is_none() {
                user_prompt = live_prompt_text(&preview, "");
            }

            if let Some(prompt_text) = user_prompt {
                let p_id = format!("prompt-{}-{}", owner_id, cid);
                prompts.push(ActivePrompt {
                    id: p_id,
                    project_id,
                    instance_id: owner_id.clone(),
                    repo_path,
                    prompt_content: prompt_text,
                    model: Some("gemini-pro".to_string()),
                    session_id: Some(cid),
                    status: "backed_up".to_string(),
                    created_at: now,
                    updated_at: now,
                    image_payload,
                    source_dir: Some(base_dir.to_string_lossy().to_string()),
                });
                *prompts_per_instance.entry(owner_id.clone()).or_insert(0) += 1;
            }
        }
    }

    prompts
}
```

The only differences from the old body: the loop iterates `(owner_id, base_dir)` pairs, `seen_cids` is keyed by `(owner_id, cid)`, the "first 5 idle rows" limit is counted per owner, the id is `prompt-{owner_id}-{cid}`, `instance_id` is `owner_id`, and `source_dir` is the dir.

### B. Insert directly above `/// Backup all currently running prompts across active projects before switching`

```rust
/// Upsert a captured prompt. The conflict update never changes `instance_id`.
fn upsert_captured_prompt(conn: &Connection, p: &ActivePrompt, now: i64) -> rusqlite::Result<usize> {
    conn.execute(
        "INSERT INTO active_prompts 
         (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload, source_dir)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'backed_up', ?8, ?9, ?10, ?11)
         ON CONFLICT(id) DO UPDATE SET 
             status = 'backed_up',
             updated_at = excluded.updated_at,
             prompt_content = excluded.prompt_content,
             image_payload = excluded.image_payload,
             source_dir = excluded.source_dir",
        params![
            &p.id,
            &p.project_id,
            &p.instance_id,
            &p.repo_path,
            &p.prompt_content,
            &p.model,
            &p.session_id,
            p.created_at,
            now,
            &p.image_payload,
            &p.source_dir,
        ],
    )
}

```

### C. Replace the statement from 4 C with

```rust
        let res = upsert_captured_prompt(&conn, &p, now);
        if let Err(e) = &res {
            crate::modules::logger::log_warn(&format!(
                "[Backup] capture upsert skipped for {} ({}): {}",
                p.id, p.instance_id, e
            ));
        }
```

Leave the `running_projects` insert above it and the `if res.is_ok() { ... }` block below it unchanged. After step 06 adds the identity index, a conflicting row makes this upsert return `Err`; the log line is how that shows up, so never drop it.

### D. Replace the whole loop from 4 D with

```rust
    let registry = crate::modules::instance::load_registry().unwrap_or_default();
    for (project, mut extracted_prompts) in extracted_results {
        let project_instance_id = match crate::modules::instance::canonical_instance_id_in(
            &registry,
            &project.instance_id,
        ) {
            Ok(id) => id,
            Err(e) => {
                crate::modules::logger::log_warn(&format!(
                    "[RepoDB] Layer 2 backup skipped project '{}': {}",
                    project.id, e
                ));
                continue;
            }
        };

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

        for (prompt_text, image_payload) in extracted_prompts {
            let existing_id: Option<String> = conn
                .query_row(
                    "SELECT id FROM active_prompts WHERE instance_id = ?1 AND repo_path = ?2 AND prompt_content = ?3 LIMIT 1",
                    rusqlite::params![&project_instance_id, &project.repo_path, &prompt_text],
                    |r| r.get(0),
                )
                .ok();

            let (prompt_id, is_existing) = match existing_id {
                Some(eid) => (eid, true),
                None => (Uuid::new_v4().to_string(), false),
            };

            let prompt_model = Some("gemini-pro".to_string());
            let result = if is_existing {
                conn.execute(
                    "UPDATE active_prompts SET status = 'backed_up', updated_at = ?, image_payload = COALESCE(?, image_payload) WHERE id = ?",
                    params![now, &image_payload, &prompt_id],
                )
            } else {
                conn.execute(
                    "INSERT INTO active_prompts 
                     (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload)
                     VALUES (?, ?, ?, ?, ?, ?, NULL, 'backed_up', ?, ?, ?)",
                    params![
                        &prompt_id,
                        &project.id,
                        &project_instance_id,
                        &project.repo_path,
                        &prompt_text,
                        &prompt_model,
                        now,
                        now,
                        &image_payload,
                    ],
                )
            };

            if result.is_ok() {
                backed_up_count += 1;
                let (extracted_img, img_paths) = extract_image_payload_or_path(&prompt_text);
                let final_img = image_payload.clone().or(extracted_img);
                let has_image = final_img.is_some() || !img_paths.is_empty();

                // Write disk resume snapshot file inside project repo directory
                let task_file =
                    PathBuf::from(&project.repo_path).join(".antigravity_resume_task.json");
                let payload = serde_json::json!({
                    "prompt_id": prompt_id,
                    "project_id": project.id,
                    "instance_id": project_instance_id,
                    "repo_path": project.repo_path,
                    "prompt_content": prompt_text,
                    "model": prompt_model,
                    "session_id": null,
                    "conversation_id": null,
                    "image_payload": final_img,
                    "image_paths": img_paths,
                    "has_image": has_image,
                    "auto_boot": true,
                    "status": "backed_up",
                    "backed_up_at": now,
                });
                if let Ok(json_str) = serde_json::to_string_pretty(&payload) {
                    let _ = fs::write(&task_file, json_str);
                }

                let active_prompt = ActivePrompt {
                    id: prompt_id.clone(),
                    project_id: project.id.clone(),
                    instance_id: project_instance_id.clone(),
                    repo_path: project.repo_path.clone(),
                    prompt_content: prompt_text,
                    model: prompt_model,
                    session_id: None,
                    status: "backed_up".to_string(),
                    created_at: now,
                    updated_at: now,
                    image_payload: final_img,
                    source_dir: None,
                };
                if let Ok(mut map) = get_memory_prompts_map().lock() {
                    map.insert(prompt_id, active_prompt);
                }
            }
        }
    }
```

The `LIKE ?3` fallback query is left as it is; it only reads prompt text and is replaced together with the resume hand-off in step 09. The resume file write here is also replaced by `write_resume_handoff` in step 09.

## 6. Tests

Paste after `    use super::*;` in `mod tests` of `repo_db.rs`. The fixtures are temp folders with a real `conversation_summaries.db`; the tagged dir list is passed directly, so no registry is loaded.

```rust
    fn write_summaries_fixture(gemini_dir: &Path, rows: &[(&str, &str, &str)]) {
        fs::create_dir_all(gemini_dir).unwrap();
        let conn = Connection::open(gemini_dir.join("conversation_summaries.db")).unwrap();
        conn.execute(
            "CREATE TABLE conversation_summaries (
                conversation_id TEXT,
                preview TEXT,
                status TEXT,
                not_fully_idle INTEGER,
                workspace_uris TEXT,
                last_modified_time INTEGER
            )",
            [],
        )
        .unwrap();
        for &(cid, preview, repo_uri) in rows {
            conn.execute(
                "INSERT INTO conversation_summaries VALUES (?1, ?2, 'CASCADE_RUN_STATUS_RUNNING', 1, ?3, 10)",
                params![cid, preview, format!("[\"{}\"]", repo_uri)],
            )
            .unwrap();
        }
    }

    #[test]
    fn captured_prompt_id_includes_instance() {
        let tmp = tempfile::tempdir().unwrap();
        let a_dir = tmp.path().join("a").join(".gemini").join("antigravity");
        write_summaries_fixture(&a_dir, &[("cid-1", "keep going", "file:///d:/work/app")]);

        let prompts = discover_running_prompts_in_dirs(&[("inst-a".to_string(), a_dir.clone())]);

        assert_eq!(prompts.len(), 1);
        assert_eq!(prompts[0].id, "prompt-inst-a-cid-1");
        assert_eq!(prompts[0].instance_id, "inst-a");
        assert_eq!(prompts[0].session_id.as_deref(), Some("cid-1"));
        assert_eq!(prompts[0].prompt_content, "keep going");
        let expected_dir = a_dir.to_string_lossy().to_string();
        assert_eq!(prompts[0].source_dir.as_deref(), Some(expected_dir.as_str()));
    }

    #[test]
    fn same_conversation_id_in_two_instances_gives_two_rows() {
        let tmp = tempfile::tempdir().unwrap();
        let a_dir = tmp.path().join("a").join(".gemini").join("antigravity");
        let b_dir = tmp.path().join("b").join(".gemini").join("antigravity");
        write_summaries_fixture(&a_dir, &[("cid-same", "same chat", "file:///d:/work/app")]);
        write_summaries_fixture(&b_dir, &[("cid-same", "same chat", "file:///d:/work/app")]);

        let prompts = discover_running_prompts_in_dirs(&[
            ("inst-a".to_string(), a_dir),
            ("inst-b".to_string(), b_dir),
        ]);

        assert_eq!(prompts.len(), 2);
        let ids: HashSet<&str> = prompts.iter().map(|p| p.id.as_str()).collect();
        assert!(ids.contains("prompt-inst-a-cid-same"));
        assert!(ids.contains("prompt-inst-b-cid-same"));

        let conn = Connection::open_in_memory().unwrap();
        assert!(init_tables(&conn).is_ok());
        for p in &prompts {
            upsert_captured_prompt(&conn, p, 20).unwrap();
        }
        let (rows, owners): (i64, i64) = conn
            .query_row(
                "SELECT COUNT(*), COUNT(DISTINCT instance_id) FROM active_prompts WHERE session_id = 'cid-same'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(rows, 2);
        assert_eq!(owners, 2);
    }

    #[test]
    fn layer1_upsert_never_rewrites_instance_id() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(init_tables(&conn).is_ok());
        let mut prompt = ActivePrompt {
            id: "prompt-inst-a-cid-9".to_string(),
            project_id: "app-cid-9".to_string(),
            instance_id: "inst-a".to_string(),
            repo_path: "D:/work/app".to_string(),
            prompt_content: "first".to_string(),
            model: None,
            session_id: Some("cid-9".to_string()),
            status: "backed_up".to_string(),
            created_at: 10,
            updated_at: 10,
            image_payload: None,
            source_dir: Some("A".to_string()),
        };
        upsert_captured_prompt(&conn, &prompt, 10).unwrap();

        prompt.instance_id = "inst-b".to_string();
        prompt.prompt_content = "second".to_string();
        upsert_captured_prompt(&conn, &prompt, 11).unwrap();

        let (instance_id, content, count): (String, String, i64) = conn
            .query_row(
                "SELECT instance_id, prompt_content, (SELECT COUNT(*) FROM active_prompts)
                 FROM active_prompts WHERE id = 'prompt-inst-a-cid-9'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();
        assert_eq!(instance_id, "inst-a");
        assert_eq!(content, "second");
        assert_eq!(count, 1);
    }
```

The original test `same_conversation_id_in_two_instances_gives_two_rows` also asked for "two sequence rows". The sequence table only gets a per-instance key in step 06, so that part is tested there (`conversation_sequences_are_keyed_by_instance_after_migration`).

## 7. Gate

```text
$env:CARGO_TARGET_DIR="$PWD\src-tauri\target"; $env:CARGO_BUILD_JOBS="1"
cd src-tauri; cargo fmt; cargo fmt -- --check; cargo clippy --all-targets --all-features; cargo test --lib -- --test-threads=1 captured_prompt_id_includes_instance; cargo test --lib -- --test-threads=1 same_conversation_id_in_two_instances_gives_two_rows; cargo test --lib -- --test-threads=1 layer1_upsert_never_rewrites_instance_id; cargo test --lib -- --test-threads=1 summary_rows_fall_back_when_status_columns_are_missing; cd ..
```

## 8. Commit

```text
Fix: prompts - stamp instance id once at capture
```

Stage only `src-tauri/src/modules/repo_db.rs`.

## 9. Done when

- [ ] `gitmap aum search "let p_id = format!" src-tauri/src/modules/repo_db.rs --ext .rs` returns exactly 1 hit, inside `discover_running_prompts_in_dirs`, and that line reads `let p_id = format!("prompt-{}-{}", owner_id, cid);`.
- [ ] `discover_running_prompts_from_antigravity` contains no loop; it builds a scope and calls `discover_running_prompts_in_dirs`.
- [ ] `upsert_captured_prompt`'s `DO UPDATE SET` list does not contain `instance_id`.
- [ ] Layer 2 inserts `session_id` as `NULL` and uses `project_instance_id`; the bare `instance_id` argument is no longer used inside the Layer 2 loop.
- [ ] The 3 new tests pass.
- [ ] Callers of `discover_running_prompts_from_antigravity` and `backup_running_prompts` were searched in `src-tauri/src`, `src-tauri/src/bin/agm.rs` and each `src-tauri/tests/*.rs` file; none needed an edit.
- [ ] Gate exited 0; commit pushed with the exact message.
