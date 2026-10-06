# Step 09: Resume Hand-off Is DB-First (DR-1)

Goal: one writer, `write_resume_handoff`, replaces every inline write of `<repo>/.antigravity_resume_task.json`. The `active_prompts` row is the source of truth. The file keeps its legacy name and is written only when exactly one instance owns the repo; otherwise nothing is written and the outcome is `ResumeHandoffOutcome::DbOnly`. Every reader that evaluates a specific instance ignores a document whose `instance_id` is a different instance.

Never create `.antigravity_resume_task.<id>.json`. Do not add per-instance names to the gitignore helper in `src-tauri/src/bin/agm.rs`.

## 1. Depends on

- Step 03 (`ActivePrompt.source_dir: Option<String>`).
- Step 05 (capture stamps the instance; Layer 2 `session_id` fix).
- Step 08 (`normalize_path_for_compare` is `pub(crate)`; `prompt_tests_registry` test helper exists).

## 2. Files you may edit

- `src-tauri/src/modules/repo_db.rs`
- `src-tauri/src/modules/notification_hub.rs`
- `src-tauri/src/modules/agy_cleaner.rs`
- `src-tauri/src/bin/agm.rs`

## 3. Find it

GitMap does not descend into `src-tauri/src/bin` when you search `src-tauri/src`. Run both commands:

```text
gitmap aum search "antigravity_resume_task" src-tauri/src --ext .rs
gitmap aum search "antigravity_resume_task" src-tauri/src/bin/agm.rs --ext .rs
gitmap aum search "antigravity_resume_task" src-tauri/tests/auto_switcher_e2e_test.rs --ext .rs
gitmap aum search "antigravity_resume_task" src-tauri/tests/instance_cloning_and_sync_test.rs --ext .rs
gitmap aum search "antigravity_resume_task" src-tauri/tests/per_instance_prompt_liveness_test.rs --ext .rs
gitmap aum search "pub struct ActivePrompt" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "RESUME_TASK_FILE_NAME|ResumeHandoffOutcome|write_resume_handoff|repo_owner_instances|resume_doc_belongs_to|has_prompt_row" src-tauri/src --ext .rs
gitmap aum search "RESUME_TASK_FILE_NAME|ResumeHandoffOutcome|write_resume_handoff|repo_owner_instances|resume_doc_belongs_to|has_prompt_row" src-tauri/src/bin/agm.rs --ext .rs
```

The three test-file searches must return 0 hits (checked 2026-10-06). The last two searches must return 0 hits before you start (name collision check). This step changes no existing signature; the only callers it edits are the writers and readers listed below, including `agm.rs` K1 to K3.

Open `pub struct ActivePrompt` and confirm its fields are exactly: `id`, `project_id`, `instance_id`, `repo_path`, `prompt_content`, `model`, `session_id`, `status`, `created_at`, `updated_at`, `image_payload`, `source_dir`. If there is any other field, STOP.

| Change | File | Anchor (`fn`) | Unique search literal | Line hint (2026-10-06) |
|---|---|---|---|---|
| A. New types and functions | `repo_db.rs` | insert before `pub fn get_canonical_host_home() -> Option<PathBuf> {` | `pub fn get_canonical_host_home` | `:805` |
| B. Layer 1 writer | `repo_db.rs` | `pub fn backup_running_prompts(instance_id: &str)` | `session_id is the live conversation, so the IDE reopens that chat.` | `:1220` to `:1231` |
| C. Layer 2 reader | `repo_db.rs` | same fn, inside `std::thread::scope` | `Check if project has an existing .antigravity_resume_task.json on disk` | `:1280` to `:1301` |
| D1. Layer 2 writer, payload | `repo_db.rs` | same fn | `Write disk resume snapshot file inside project repo directory` followed by `let task_file =` on the next line | `:1376` to `:1400` |
| D2. Layer 2 writer, call | `repo_db.rs` | same fn | `map.insert(prompt_id, active_prompt);` | `:1415` to `:1417` |
| E. Direct dispatch writer | `repo_db.rs` | `pub fn dispatch_running_prompts(instance_id: &str)` | `Keep the conversation id on the file the IDE reads.` | `:1602` to `:1609` |
| F. Resend writer | `repo_db.rs` | `pub fn resend_running_commands_for_instance(` | `"resumed_at": now,` (first of two hits; the one inside this fn) | `:3369` to `:3394` |
| G. Auto-resume reader | `repo_db.rs` | `pub fn auto_resume_recent_prompts(` | `If not found in DB, check existing .antigravity_resume_task.json` | `:3532` to `:3559` |
| H1. Auto-resume writer, payload | `repo_db.rs` | same fn | `Dispatch directly to project folder via .antigravity_resume_task.json` | `:3594` to `:3611` |
| H2. Auto-resume writer, call | `repo_db.rs` | same fn | `spawn_prompt_via_agy(&prompt_obj);` | `:3645` |
| I. Reader | `notification_hub.rs` | `if let Some(proj) = matched_projs.first() {` | `join(".antigravity_resume_task.json");` | `:568` to `:573` |
| J. Reader | `agy_cleaner.rs` | inside the `running_projs` loop | `protect its session/conversation ID` | `:374` to `:380` |
| K1. Import writer | `agm.rs` | `fn cmd_prompts_import(args: &[String])` | `Also write .antigravity_resume_task.json in target repo` | `:2563` to `:2578` |
| K2. Dispatch writer | `agm.rs` | `fn cmd_prompt_dispatch(args: &[String])` | `"prefix_template": prefix_cat,` | `:3154` to `:3169` and `:3184` |
| K3. Rerun writer | `agm.rs` | `fn cmd_rerun(args: &[String])` | `"rerun_seq": idx + 1,` | `:3295` to `:3309` |
| L. Tests | `repo_db.rs` | `mod tests` | end of module | last line |

Not changed in this step (on purpose):

- `check_and_dispatch_enqueued_prompts` writer (`repo_db.rs:2174` to `:2186`). Step 11 replaces that whole function and its write goes through `dispatch_one_prompt`, which calls `write_resume_handoff_in`.
- `src-tauri/src/bin/agm.rs:7935` and `:8346` read the file for global status displays that have no target instance. `:11786` deletes it. `:14620`, `:14796` only check it exists in an e2e command. `src-tauri/src/modules/instance.rs:4941` checks existence only; `:7192` is a test. Leave all of them.

## 4. Current code

### A. Insertion point (`:802` to `:805`)

```rust
        "backed_up_at": at,
    })
}

pub fn get_canonical_host_home() -> Option<PathBuf> {
```

### B. Layer 1 writer (`:1218` to `:1235`)

```rust
        if res.is_ok() {
            backed_up_count += 1;
            let (extracted_img, img_paths) = extract_image_payload_or_path(&p.prompt_content);
            let final_img = p.image_payload.clone().or(extracted_img);
            // Write disk resume snapshot file inside project repo directory.
            // session_id is the live conversation, so the IDE reopens that chat.
            let task_file = PathBuf::from(&p.repo_path).join(".antigravity_resume_task.json");
            let mut payload = resume_task_document(&p, "backed_up", now, &img_paths);
            if final_img.is_some() {
                payload["image_payload"] = serde_json::json!(final_img);
            }
            if let Ok(json_str) = serde_json::to_string_pretty(&payload) {
                let _ = fs::write(&task_file, json_str);
            }
            if let Ok(mut map) = get_memory_prompts_map().lock() {
                map.insert(p.id.clone(), p);
            }
        }
```

### C. Layer 2 reader (`:1280` to `:1301`)

```rust
                        // Check if project has an existing .antigravity_resume_task.json on disk
                        if extracted.is_empty() {
                            let task_file = PathBuf::from(&project.repo_path)
                                .join(".antigravity_resume_task.json");
                            if task_file.exists() {
                                if let Ok(c) = fs::read_to_string(&task_file) {
                                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&c) {
                                        if let Some(txt) =
                                            v.get("prompt_content").and_then(|t| t.as_str())
                                        {
                                            if !txt.trim().is_empty() {
                                                let img = v
                                                    .get("image_payload")
                                                    .and_then(|i| i.as_str())
                                                    .map(|s| s.to_string());
                                                extracted.push((txt.to_string(), img));
                                            }
                                        }
                                    }
                                }
                            }
                        }
```

### D1. Layer 2 writer payload (`:1374` to `:1400`)

Step 05 may already have deleted the line `"session_id": project.id,` from this `json!` block. If that line is missing, the block still matches; continue.

```rust
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
```

### D2. Layer 2 writer call site (`:1415` to `:1417`)

```rust
                if let Ok(mut map) = get_memory_prompts_map().lock() {
                    map.insert(prompt_id, active_prompt);
                }
```

### E. `dispatch_running_prompts` writer (`:1602` to `:1609`)

```rust
        // Keep the conversation id on the file the IDE reads. A later rewrite must not drop it.
        let task_file = PathBuf::from(&prompt.repo_path).join(".antigravity_resume_task.json");
        let payload = resume_task_document(&prompt, "dispatched", now, &[]);
        let file_written = if let Ok(json_str) = serde_json::to_string_pretty(&payload) {
            fs::write(&task_file, json_str).is_ok()
        } else {
            false
        };
```

### F. `resend_running_commands_for_instance` writer (`:3369` to `:3394`)

```rust
        let (extracted_img, img_paths) = extract_image_payload_or_path(&prompt.prompt_content);
        if prompt.image_payload.is_none() && extracted_img.is_some() {
            prompt.image_payload = extracted_img.clone();
        }
        let has_image = prompt.image_payload.is_some() || !img_paths.is_empty();

        // Write .antigravity_resume_task.json to project directory
        let task_file = PathBuf::from(&prompt.repo_path).join(".antigravity_resume_task.json");
        let payload = serde_json::json!({
            "prompt_id": prompt.id,
            "project_id": prompt.project_id,
            "instance_id": prompt.instance_id,
            "repo_path": prompt.repo_path,
            "prompt_content": prompt.prompt_content,
            "model": prompt.model,
            "image_payload": prompt.image_payload,
            "image_paths": img_paths,
            "has_image": has_image,
            "auto_boot": true,
            "status": "dispatched",
            "resumed_at": now,
        });

        if let Ok(json_str) = serde_json::to_string_pretty(&payload) {
            let _ = fs::write(&task_file, json_str);
        }
```

### G. `auto_resume_recent_prompts` reader (`:3532` to `:3559`)

```rust
        // If not found in DB, check existing .antigravity_resume_task.json
        if maybe_prompt.is_none() {
            let task_file = PathBuf::from(&project.repo_path).join(".antigravity_resume_task.json");
            if task_file.exists() {
                if let Ok(c) = fs::read_to_string(&task_file) {
                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&c) {
                        if let Some(txt) = v.get("prompt_content").and_then(|t| t.as_str()) {
                            if !txt.trim().is_empty() {
                                let pid = v
                                    .get("prompt_id")
                                    .and_then(|i| i.as_str())
                                    .unwrap_or(&project.id)
                                    .to_string();
                                let m = v
                                    .get("model")
                                    .and_then(|m| m.as_str())
                                    .map(|s| s.to_string());
                                let img = v
                                    .get("image_payload")
                                    .and_then(|i| i.as_str())
                                    .map(|s| s.to_string());
                                maybe_prompt = Some((pid, txt.to_string(), m, img));
                            }
                        }
                    }
                }
            }
        }
```

### H1. `auto_resume_recent_prompts` writer payload (`:3592` to `:3611`)

```rust
        let has_image = image_payload.is_some();

        // Dispatch directly to project folder via .antigravity_resume_task.json to spin up boot process immediately
        let task_file = PathBuf::from(&project.repo_path).join(".antigravity_resume_task.json");
        let payload = serde_json::json!({
            "prompt_id": prompt_id,
            "project_id": project.id,
            "instance_id": instance_id,
            "repo_path": project.repo_path,
            "prompt_content": prompt_text,
            "model": prompt_model,
            "image_payload": image_payload,
            "has_image": has_image,
            "auto_boot": true,
            "resumed_at": now,
        });

        if let Ok(json_str) = serde_json::to_string_pretty(&payload) {
            let _ = fs::write(&task_file, json_str);
        }
```

### H2. `auto_resume_recent_prompts` spawn line (`:3645`)

```rust
        spawn_prompt_via_agy(&prompt_obj);
```

### I. `src-tauri/src/modules/notification_hub.rs` (`:568` to `:573`)

```rust
            if let Some(proj) = matched_projs.first() {
                let resume_file =
                    std::path::PathBuf::from(&proj.repo_path).join(".antigravity_resume_task.json");
                if resume_file.exists() {
                    if let Ok(content) = std::fs::read_to_string(&resume_file) {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
```

### J. `src-tauri/src/modules/agy_cleaner.rs` (`:374` to `:380`)

```rust
            // Check if .antigravity_resume_task.json exists and protect its session/conversation ID
            if !proj.repo_path.trim().is_empty() {
                let resume_file =
                    PathBuf::from(&proj.repo_path).join(".antigravity_resume_task.json");
                if resume_file.exists() {
                    if let Ok(content) = fs::read_to_string(&resume_file) {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
```

### K1. `src-tauri/src/bin/agm.rs` import writer (`:2563` to `:2578`)

```rust
            // Also write .antigravity_resume_task.json in target repo to trigger immediate rerun
            let task_file = PathBuf::from(&repo_path).join(".antigravity_resume_task.json");
            let task_payload = serde_json::json!({
                "prompt_id": id,
                "project_id": proj_id,
                "instance_id": inst_id,
                "repo_path": repo_path,
                "prompt_content": prompt_content,
                "model": model,
                "image_payload": img,
                "auto_boot": true,
                "imported_at": now,
            });
            if let Ok(js) = serde_json::to_string_pretty(&task_payload) {
                let _ = fs::write(&task_file, js);
            }
```

### K2. `src-tauri/src/bin/agm.rs` dispatch writer (`:3154` to `:3169`) and spawn line (`:3184`)

```rust
    let task_file = PathBuf::from(&cwd_str).join(".antigravity_resume_task.json");
    let payload = serde_json::json!({
        "prompt_id": prompt_id,
        "agm_seq_id": seq_label,
        "project_id": slug,
        "conversation_id": session_id,
        "instance_id": inst_id,
        "repo_path": cwd_str,
        "prompt_content": final_prompt,
        "prefix_template": prefix_cat,
        "suffix_template": suffix_cat,
        "dispatched_at": now,
    });
    if let Ok(js) = serde_json::to_string_pretty(&payload) {
        let _ = fs::write(&task_file, js);
    }
```

```rust
    let _ = repo_db::spawn_prompt_via_agy(&active_p);
```

### K3. `src-tauri/src/bin/agm.rs` rerun writer (`:3295` to `:3309`)

```rust
        let task_file = PathBuf::from(&p.repo_path).join(".antigravity_resume_task.json");
        let payload = serde_json::json!({
            "prompt_id": p.id,
            "project_id": p.project_id,
            "instance_id": p.instance_id,
            "repo_path": p.repo_path,
            "prompt_content": wrapped,
            "model": p.model,
            "image_payload": p.image_payload,
            "rerun_seq": idx + 1,
            "resumed_at": now,
        });
        if let Ok(js) = serde_json::to_string_pretty(&payload) {
            let _ = fs::write(&task_file, js);
        }
```

## 5. New code

### A. Insert before `pub fn get_canonical_host_home() -> Option<PathBuf> {` (after the closing `}` of `resume_task_document`)

```rust
pub const RESUME_TASK_FILE_NAME: &str = ".antigravity_resume_task.json";

/// Result of a resume hand-off. The active_prompts row is the source of truth; the repo
/// file keeps its legacy name and is written only when exactly one instance owns the repo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResumeHandoffOutcome {
    LegacyFile(PathBuf),
    DbOnly { legacy_skipped_owners: Vec<String> },
    Failed(String),
}

/// Instance ids whose running_projects rows point at this repo (normalized path equality).
pub fn repo_owner_instances_in(conn: &Connection, repo_path: &str) -> Vec<String> {
    let target = normalize_path_for_compare(repo_path);
    if target.is_empty() {
        return Vec::new();
    }
    let mut owners: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    if let Ok(mut stmt) = conn
        .prepare("SELECT instance_id, repo_path FROM running_projects WHERE trim(instance_id) <> ''")
    {
        if let Ok(rows) =
            stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        {
            for (instance_id, path) in rows.flatten() {
                if normalize_path_for_compare(&path) == target {
                    owners.insert(instance_id);
                }
            }
        }
    }
    owners.into_iter().collect()
}

pub fn repo_owner_instances(repo_path: &str) -> Vec<String> {
    connect_db()
        .map(|conn| repo_owner_instances_in(&conn, repo_path))
        .unwrap_or_default()
}

fn has_prompt_row(conn: &Connection, prompt_id: &str) -> bool {
    conn.query_row(
        "SELECT 1 FROM active_prompts WHERE id = ?1",
        params![prompt_id],
        |_| Ok(()),
    )
    .is_ok()
}

/// Pure core of the resume hand-off (DR-1). Never writes a per-instance file name.
pub fn write_resume_handoff_in(
    conn: &Connection,
    prompt: &ActivePrompt,
    status: &str,
) -> ResumeHandoffOutcome {
    if prompt.instance_id.trim().is_empty() {
        return ResumeHandoffOutcome::Failed(format!(
            "prompt '{}' has no instance id",
            prompt.id
        ));
    }
    let mut owners = repo_owner_instances_in(conn, &prompt.repo_path);
    if !owners.contains(&prompt.instance_id) {
        owners.push(prompt.instance_id.clone());
        owners.sort();
    }

    if owners.len() > 1 {
        if !has_prompt_row(conn, &prompt.id) {
            return ResumeHandoffOutcome::Failed(format!(
                "repo {} has several owners and prompt row '{}' is missing",
                prompt.repo_path, prompt.id
            ));
        }
        crate::modules::logger::log_info(&format!(
            "[ResumeHandoff] legacy file skipped: repo {} owned by {}",
            prompt.repo_path,
            owners.join(", ")
        ));
        return ResumeHandoffOutcome::DbOnly {
            legacy_skipped_owners: owners,
        };
    }

    let (extracted_img, img_paths) = extract_image_payload_or_path(&prompt.prompt_content);
    let mut doc = resume_task_document(prompt, status, Utc::now().timestamp(), &img_paths);
    if prompt.image_payload.is_none() {
        if let Some(img) = extracted_img {
            doc["image_payload"] = serde_json::json!(img);
        }
    }
    let task_file = PathBuf::from(&prompt.repo_path).join(RESUME_TASK_FILE_NAME);
    let write_result = serde_json::to_string_pretty(&doc)
        .map_err(|e| e.to_string())
        .and_then(|json| fs::write(&task_file, json).map_err(|e| e.to_string()));

    match write_result {
        Ok(()) => ResumeHandoffOutcome::LegacyFile(task_file),
        Err(e) if has_prompt_row(conn, &prompt.id) => {
            crate::modules::logger::log_warn(&format!(
                "[ResumeHandoff] legacy file write failed for repo {}: {}; row '{}' stays the source of truth",
                prompt.repo_path, e, prompt.id
            ));
            ResumeHandoffOutcome::DbOnly {
                legacy_skipped_owners: owners,
            }
        }
        Err(e) => ResumeHandoffOutcome::Failed(format!(
            "resume hand-off write failed for repo {}: {}",
            prompt.repo_path, e
        )),
    }
}

pub fn write_resume_handoff(prompt: &ActivePrompt, status: &str) -> ResumeHandoffOutcome {
    match connect_db() {
        Ok(conn) => write_resume_handoff_in(&conn, prompt, status),
        Err(e) => ResumeHandoffOutcome::Failed(format!("repo database unavailable: {}", e)),
    }
}

/// True only when the resume document names the same canonical instance as `instance_id`.
pub fn resume_doc_belongs_to_in(
    registry: &crate::modules::instance::InstanceRegistry,
    doc: &serde_json::Value,
    instance_id: &str,
) -> bool {
    let Some(doc_instance) = doc.get("instance_id").and_then(|v| v.as_str()) else {
        return false;
    };
    if doc_instance.trim().is_empty() || instance_id.trim().is_empty() {
        return false;
    }
    let doc_canonical = crate::modules::instance::canonical_instance_id_in(registry, doc_instance);
    let target_canonical = crate::modules::instance::canonical_instance_id_in(registry, instance_id);
    matches!((doc_canonical, target_canonical), (Ok(a), Ok(b)) if a == b)
}

pub fn resume_doc_belongs_to(doc: &serde_json::Value, instance_id: &str) -> bool {
    let registry = crate::modules::instance::load_registry().unwrap_or_default();
    resume_doc_belongs_to_in(&registry, doc, instance_id)
}

```

### B. Layer 1 writer: replace with

```rust
        if res.is_ok() {
            backed_up_count += 1;
            let _ = write_resume_handoff_in(&conn, &p, "backed_up");
            if let Ok(mut map) = get_memory_prompts_map().lock() {
                map.insert(p.id.clone(), p);
            }
        }
```

### C. Layer 2 reader: replace with

```rust
                        // Check if project has an existing .antigravity_resume_task.json on disk
                        if extracted.is_empty() {
                            let task_file = PathBuf::from(&project.repo_path)
                                .join(".antigravity_resume_task.json");
                            if task_file.exists() {
                                if let Ok(c) = fs::read_to_string(&task_file) {
                                    if let Some(v) = serde_json::from_str::<serde_json::Value>(&c)
                                        .ok()
                                        .filter(|v| resume_doc_belongs_to(v, &project.instance_id))
                                    {
                                        if let Some(txt) =
                                            v.get("prompt_content").and_then(|t| t.as_str())
                                        {
                                            if !txt.trim().is_empty() {
                                                let img = v
                                                    .get("image_payload")
                                                    .and_then(|i| i.as_str())
                                                    .map(|s| s.to_string());
                                                extracted.push((txt.to_string(), img));
                                            }
                                        }
                                    }
                                }
                            }
                        }
```

### D1. Layer 2 writer payload: replace with

```rust
            if result.is_ok() {
                backed_up_count += 1;
                let (extracted_img, _) = extract_image_payload_or_path(&prompt_text);
                let final_img = image_payload.clone().or(extracted_img);
```

Leave the `let active_prompt = ActivePrompt { ... };` initializer that follows unchanged (it still uses `final_img`).

### D2. Layer 2 writer call site: replace with

```rust
                let _ = write_resume_handoff_in(&conn, &active_prompt, "backed_up");
                if let Ok(mut map) = get_memory_prompts_map().lock() {
                    map.insert(prompt_id, active_prompt);
                }
```

### E. `dispatch_running_prompts` writer: replace with

```rust
        let file_written = matches!(
            write_resume_handoff_in(&conn, &prompt, "dispatched"),
            ResumeHandoffOutcome::LegacyFile(_)
        );
```

`file_written` is still read a few lines below (`if !sent && !(file_written && same_conversation)`); keep that line.

### F. `resend_running_commands_for_instance` writer: replace with

```rust
        let (extracted_img, _) = extract_image_payload_or_path(&prompt.prompt_content);
        if prompt.image_payload.is_none() {
            prompt.image_payload = extracted_img;
        }

        let _ = write_resume_handoff_in(&conn, &prompt, "dispatched");
```

### G. `auto_resume_recent_prompts` reader: replace with

```rust
        // If not found in DB, check existing .antigravity_resume_task.json
        if maybe_prompt.is_none() {
            let task_file = PathBuf::from(&project.repo_path).join(".antigravity_resume_task.json");
            if task_file.exists() {
                if let Ok(c) = fs::read_to_string(&task_file) {
                    if let Some(v) = serde_json::from_str::<serde_json::Value>(&c)
                        .ok()
                        .filter(|v| resume_doc_belongs_to(v, norm_inst))
                    {
                        if let Some(txt) = v.get("prompt_content").and_then(|t| t.as_str()) {
                            if !txt.trim().is_empty() {
                                let pid = v
                                    .get("prompt_id")
                                    .and_then(|i| i.as_str())
                                    .unwrap_or(&project.id)
                                    .to_string();
                                let m = v
                                    .get("model")
                                    .and_then(|m| m.as_str())
                                    .map(|s| s.to_string());
                                let img = v
                                    .get("image_payload")
                                    .and_then(|i| i.as_str())
                                    .map(|s| s.to_string());
                                maybe_prompt = Some((pid, txt.to_string(), m, img));
                            }
                        }
                    }
                }
            }
        }
```

### H1. `auto_resume_recent_prompts` writer payload: replace with

```rust
        let has_image = image_payload.is_some();
```

### H2. `auto_resume_recent_prompts` spawn line: replace with

```rust
        let _ = write_resume_handoff_in(&conn, &prompt_obj, "dispatched");
        spawn_prompt_via_agy(&prompt_obj);
```

### I. `notification_hub.rs`: replace with

```rust
            if let Some(proj) = matched_projs.first() {
                let resume_file =
                    std::path::PathBuf::from(&proj.repo_path).join(".antigravity_resume_task.json");
                if resume_file.exists() {
                    if let Ok(content) = std::fs::read_to_string(&resume_file) {
                        if let Some(val) = serde_json::from_str::<serde_json::Value>(&content)
                            .ok()
                            .filter(|v| {
                                crate::modules::repo_db::resume_doc_belongs_to(
                                    v,
                                    &details.instance_id,
                                )
                            })
                        {
```

Only the last line changed (it became six lines). Every line after it, including all closing braces, stays.

### J. `agy_cleaner.rs`: replace with

```rust
            // Check if .antigravity_resume_task.json exists and protect its session/conversation ID
            if !proj.repo_path.trim().is_empty() {
                let resume_file =
                    PathBuf::from(&proj.repo_path).join(".antigravity_resume_task.json");
                if resume_file.exists() {
                    if let Ok(content) = fs::read_to_string(&resume_file) {
                        if let Some(val) = serde_json::from_str::<serde_json::Value>(&content)
                            .ok()
                            .filter(|v| {
                                crate::modules::repo_db::resume_doc_belongs_to(
                                    v,
                                    &proj.instance_id,
                                )
                            })
                        {
```

Only the last line changed. Every line after it stays.

### K1. `agm.rs` import writer: replace with

```rust
            let imported = repo_db::ActivePrompt {
                id,
                project_id: proj_id,
                instance_id: inst_id,
                repo_path,
                prompt_content,
                model,
                session_id: None,
                status: "dispatched".to_string(),
                created_at: now,
                updated_at: now,
                image_payload: img,
                source_dir: Some("cli".to_string()),
            };
            let _ = repo_db::write_resume_handoff(&imported, "dispatched");
```

Before you paste, confirm `id`, `proj_id`, `inst_id`, `repo_path`, `prompt_content`, `model`, `img` are not used again after this block inside the loop (the next statement is `total_imported += 1;`). If the compiler reports "use of moved value" for one of them, add `.clone()` to that field only.

### K2. `agm.rs` dispatch writer

1. Delete the whole `let task_file = ...` block shown in section 4 (16 lines, through the closing `}` of `if let Ok(js) = ...`). Replace it with nothing.
2. Replace the spawn line with:

```rust
    let _ = repo_db::write_resume_handoff(&active_p, "dispatched");
    let _ = repo_db::spawn_prompt_via_agy(&active_p);
```

After the deletion, check that `prefix_cat` and `suffix_cat` are still used elsewhere in the function (`gitmap aum search "prefix_cat" src-tauri/src/bin/agm.rs --ext .rs`). They are used earlier to wrap the prompt; if the compiler reports either as unused, STOP and report.

### K3. `agm.rs` rerun writer: replace with

```rust
        let mut rerun = p.clone();
        rerun.prompt_content = wrapped.clone();
        rerun.status = "dispatched".to_string();
        rerun.updated_at = now;
        let _ = repo_db::write_resume_handoff(&rerun, "dispatched");
```

After K1, K2 and K3, check `agm.rs` still uses `fs::` and `PathBuf` elsewhere (it does in many places). Do not remove imports unless the compiler names them unused.

## 6. Tests

Paste at the end of `mod tests` in `src-tauri/src/modules/repo_db.rs`. They use `Connection::open_in_memory()` plus `init_tables(&conn)`, a `tempfile::tempdir()` as the repo folder, and the in-memory registry helper `prompt_tests_registry` from step 08. They never call `load_registry()`.

```rust
    fn handoff_test_prompt(id: &str, instance_id: &str, repo_path: &str) -> ActivePrompt {
        ActivePrompt {
            id: id.to_string(),
            project_id: "proj".to_string(),
            instance_id: instance_id.to_string(),
            repo_path: repo_path.to_string(),
            prompt_content: "keep going".to_string(),
            model: None,
            session_id: Some("conv-1".to_string()),
            status: "backed_up".to_string(),
            created_at: 10,
            updated_at: 10,
            image_payload: None,
            source_dir: None,
        }
    }

    fn seed_running_project(conn: &Connection, id: &str, instance_id: &str, repo_path: &str) {
        conn.execute(
            "INSERT INTO running_projects
             (id, instance_id, repo_name, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at)
             VALUES (?1, ?2, 'repo', ?3, NULL, 1, 10, 10)",
            params![id, instance_id, repo_path],
        )
        .unwrap();
    }

    fn seed_handoff_prompt_row(conn: &Connection, prompt: &ActivePrompt) {
        conn.execute(
            "INSERT INTO active_prompts
             (id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, NULL, ?6, ?7, 10, 10)",
            params![
                prompt.id,
                prompt.project_id,
                prompt.instance_id,
                prompt.repo_path,
                prompt.prompt_content,
                prompt.session_id,
                prompt.status
            ],
        )
        .unwrap();
    }

    #[test]
    fn two_instances_same_repo_do_not_share_legacy_handoff() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        let repo = tempfile::tempdir().unwrap();
        let repo_path = repo.path().to_string_lossy().to_string();
        let other_spelling = format!("{}/", repo_path.replace('\\', "/").to_uppercase());
        seed_running_project(&conn, "repo__inst-a", "inst-a", &repo_path);
        seed_running_project(&conn, "repo__inst-b", "inst-b", &other_spelling);
        let prompt = handoff_test_prompt("prompt-inst-a-conv-1", "inst-a", &repo_path);
        seed_handoff_prompt_row(&conn, &prompt);

        let outcome = write_resume_handoff_in(&conn, &prompt, "backed_up");

        assert_eq!(
            outcome,
            ResumeHandoffOutcome::DbOnly {
                legacy_skipped_owners: vec!["inst-a".to_string(), "inst-b".to_string()]
            }
        );
        assert!(!repo.path().join(RESUME_TASK_FILE_NAME).exists());
    }

    #[test]
    fn single_owner_repo_keeps_legacy_handoff() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        let repo = tempfile::tempdir().unwrap();
        let repo_path = repo.path().to_string_lossy().to_string();
        seed_running_project(&conn, "repo__inst-a", "inst-a", &repo_path);
        let prompt = handoff_test_prompt("prompt-inst-a-conv-1", "inst-a", &repo_path);

        let outcome = write_resume_handoff_in(&conn, &prompt, "backed_up");

        let expected_file = PathBuf::from(&repo_path).join(RESUME_TASK_FILE_NAME);
        assert_eq!(outcome, ResumeHandoffOutcome::LegacyFile(expected_file.clone()));
        let doc: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(&expected_file).unwrap()).unwrap();
        assert_eq!(doc["instance_id"], "inst-a");
        assert_eq!(doc["session_id"], "conv-1");
        assert_eq!(doc["conversation_id"], "conv-1");
        assert_eq!(doc["status"], "backed_up");
    }

    #[test]
    fn handoff_counts_the_prompt_instance_as_owner() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        let repo = tempfile::tempdir().unwrap();
        let repo_path = repo.path().to_string_lossy().to_string();
        let prompt = handoff_test_prompt("prompt-inst-a-conv-1", "inst-a", &repo_path);

        let outcome = write_resume_handoff_in(&conn, &prompt, "dispatched");

        assert!(matches!(outcome, ResumeHandoffOutcome::LegacyFile(_)));
    }

    #[test]
    fn handoff_fails_for_prompt_without_instance() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        let repo = tempfile::tempdir().unwrap();
        let repo_path = repo.path().to_string_lossy().to_string();
        let prompt = handoff_test_prompt("prompt-orphan", "", &repo_path);

        let outcome = write_resume_handoff_in(&conn, &prompt, "dispatched");

        assert!(matches!(outcome, ResumeHandoffOutcome::Failed(_)));
        assert!(!repo.path().join(RESUME_TASK_FILE_NAME).exists());
    }

    #[test]
    fn handoff_write_failure_falls_back_to_db_row() {
        let conn = Connection::open_in_memory().unwrap();
        init_tables(&conn).unwrap();
        let repo = tempfile::tempdir().unwrap();
        let missing_repo = repo.path().join("missing-sub").to_string_lossy().to_string();
        let prompt = handoff_test_prompt("prompt-inst-a-conv-1", "inst-a", &missing_repo);

        let without_row = write_resume_handoff_in(&conn, &prompt, "dispatched");
        assert!(matches!(without_row, ResumeHandoffOutcome::Failed(_)));

        seed_handoff_prompt_row(&conn, &prompt);
        let with_row = write_resume_handoff_in(&conn, &prompt, "dispatched");
        assert_eq!(
            with_row,
            ResumeHandoffOutcome::DbOnly {
                legacy_skipped_owners: vec!["inst-a".to_string()]
            }
        );
    }

    #[test]
    fn resume_reader_ignores_other_instance_document() {
        let registry = prompt_tests_registry(&["default", "inst-a", "inst-b"]);
        let doc_a = serde_json::json!({ "instance_id": "inst-a", "prompt_content": "x" });
        assert!(!resume_doc_belongs_to_in(&registry, &doc_a, "inst-b"));
        assert!(resume_doc_belongs_to_in(&registry, &doc_a, "inst-a"));
        assert!(resume_doc_belongs_to_in(&registry, &doc_a, "INST-A"));

        let doc_default = serde_json::json!({ "instance_id": "__default__" });
        assert!(resume_doc_belongs_to_in(&registry, &doc_default, "default"));
        assert!(!resume_doc_belongs_to_in(&registry, &doc_default, "inst-a"));

        let doc_missing = serde_json::json!({ "prompt_content": "x" });
        assert!(!resume_doc_belongs_to_in(&registry, &doc_missing, "default"));
        let doc_empty = serde_json::json!({ "instance_id": "" });
        assert!(!resume_doc_belongs_to_in(&registry, &doc_empty, "default"));
        let doc_unknown = serde_json::json!({ "instance_id": "ghost" });
        assert!(!resume_doc_belongs_to_in(&registry, &doc_unknown, "ghost"));
    }
```

## 7. Gate

```text
$env:CARGO_TARGET_DIR="$PWD\src-tauri\target"; $env:CARGO_BUILD_JOBS="1"
cd src-tauri; cargo fmt; cargo fmt -- --check; cargo clippy --all-targets --all-features; cargo test --lib -- --test-threads=1 handoff resume_reader; cd ..
```

Then check that no inline writer is left:

```text
gitmap aum search "fs::write(&task_file" src-tauri/src/modules/repo_db.rs --ext .rs
gitmap aum search "fs::write(&task_file" src-tauri/src/bin/agm.rs --ext .rs
gitmap aum search "antigravity_resume_task." src-tauri/src --ext .rs
```

The first must return exactly 2 hits: one inside `write_resume_handoff_in` (the new `.and_then(|json| fs::write(&task_file, json)` line) and one inside `check_and_dispatch_enqueued_prompts` (removed in step 11). The second must return 0 hits. The third must not show any per-instance name such as `.antigravity_resume_task.<id>.json`.

## 8. Commit

Stage exactly these four files: `src-tauri/src/modules/repo_db.rs`, `src-tauri/src/modules/notification_hub.rs`, `src-tauri/src/modules/agy_cleaner.rs`, `src-tauri/src/bin/agm.rs`.

```text
Fix: prompts - DB-first resume hand-off with single-owner file
```

## 9. Done when

- [ ] `ResumeHandoffOutcome` has exactly the variants `LegacyFile`, `DbOnly { legacy_skipped_owners }`, `Failed`.
- [ ] `write_resume_handoff`, `write_resume_handoff_in`, `repo_owner_instances`, `repo_owner_instances_in`, `resume_doc_belongs_to`, `resume_doc_belongs_to_in` exist in `repo_db.rs`.
- [ ] Every writer listed in section 3 (B, D, E, F, H, K1, K2, K3) calls `write_resume_handoff_in` or `write_resume_handoff`; none builds its own JSON.
- [ ] Readers C, G, I, J skip documents of another instance.
- [ ] No file name other than `.antigravity_resume_task.json` is written.
- [ ] The scheduler writer in `check_and_dispatch_enqueued_prompts` is untouched (step 11).
- [ ] Writers and readers were searched in `src-tauri/src`, `src-tauri/src/bin/agm.rs` and each `src-tauri/tests/*.rs` file; `agm.rs` K1 to K3 are converted and the test files have no hit.
- [ ] Gate exits 0. Only the four listed files are staged.
