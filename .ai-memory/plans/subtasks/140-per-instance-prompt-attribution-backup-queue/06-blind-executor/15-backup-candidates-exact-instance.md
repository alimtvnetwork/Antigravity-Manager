# Step 15: Backup Candidates Come From the Exact Instance Only

Goal: `backup_active_running_prompts_for_instance` takes a required instance id, canonicalizes it, picks only prompts whose `instance_id` equals it exactly (an empty `instance_id` never matches), and attaches a conversation only when the conversation belongs to that same instance and matches by session id first, then by exact normalized workspace path. No `contains` on paths or project ids. Fixes B15 and B16. Spec section 6.8.

## 1. Depends on

- Step 01 (`canonical_instance_id`, `InstanceScope`).
- Step 04 (`repo_db::gemini_dirs_tagged(&InstanceScope) -> Vec<(String, PathBuf)>`).
- Step 07 (imports block of `backup_prompts_db.rs` includes `use crate::modules::instance::InstanceScope;`; the dedupe block is replaced by `upsert_prompt_backup_record(&conn, &record, target_inst, now)?;`).
- Step 08 (`repo_db::normalize_path_for_compare` is `pub(crate)`).
- Step 10 (`repo_db::prompt_status`).

## 2. Files you may edit

- `src-tauri/src/modules/backup_prompts_db.rs`
- `src-tauri/src/modules/integration.rs` (one call site)

## 3. Find it

| Change | File | Function | Unique search literal | Line hint |
|---|---|---|---|---|
| A | `backup_prompts_db.rs` | `pub fn backup_active_running_prompts(` (wrapper) | `/// Backward compatible wrapper with instance support` | `:221` |
| B | `backup_prompts_db.rs` | `pub fn backup_active_running_prompts_for_instance(` (head) | `let target_inst = instance_id.unwrap_or("default");` | `:234` |
| C | `backup_prompts_db.rs` | same function, candidate list | `// Consolidate candidate prompts to back up: prioritize live discovered prompts` | `:243` |
| D | `backup_prompts_db.rs` | same function, conversation match | `let repo_norm = p.repo_path.to_lowercase().replace('\\', "/");` | `:299` |
| E | `backup_prompts_db.rs` | new helpers, inserted directly above `/// Backward compatible wrapper with instance support` | `/// Backward compatible wrapper with instance support` | `:221` |
| F | `integration.rs` | `on_account_switch` (desktop) | `backup_active_running_prompts_for_instance(` | `:342` |

```text
gitmap aum search "Backward compatible wrapper with instance support" src-tauri/src/modules/backup_prompts_db.rs --ext .rs
gitmap aum search "let target_inst = instance_id.unwrap_or" src-tauri/src/modules/backup_prompts_db.rs --ext .rs
gitmap aum search "Consolidate candidate prompts" src-tauri/src/modules/backup_prompts_db.rs --ext .rs
gitmap aum search "let repo_norm" src-tauri/src/modules/backup_prompts_db.rs --ext .rs
gitmap aum search "backup_active_running_prompts" src-tauri --ext .rs
gitmap aum search "agy_cleaner" src-tauri/src/modules/backup_prompts_db.rs --ext .rs
```

Before adding new names, check they do not exist yet:

```text
gitmap aum search "struct BackupConversation|fn load_backup_conversations|fn is_backup_candidate|fn select_backup_candidates|fn match_backup_conversation|fn first_workspace_path|fn backup_path_key" src-tauri/src --ext .rs
```

Callers today. GitMap's `src-tauri/src` search skips `agm.rs` and `src-tauri/tests`, so search those separately:

```text
gitmap aum search "backup_active_running_prompts" src-tauri/src --ext .rs
gitmap aum search "backup_active_running_prompts" src-tauri/src/bin/agm.rs --ext .rs
gitmap aum search "backup_active_running_prompts" src-tauri/tests/auto_switcher_e2e_test.rs --ext .rs
gitmap aum search "backup_active_running_prompts" src-tauri/tests/instance_cloning_and_sync_test.rs --ext .rs
gitmap aum search "backup_active_running_prompts" src-tauri/tests/per_instance_prompt_liveness_test.rs --ext .rs
```

- `src-tauri/src/modules/integration.rs:342` calls `backup_active_running_prompts_for_instance(Some("default"), None)`. Change F updates it. This is the only direct caller of the changed function anywhere: `agm.rs` and `src-tauri/tests` have 0 direct calls.
- `src-tauri/src/modules/auto_switcher.rs:1433` and `:2212`, `src-tauri/src/modules/instance.rs:4791`, `src-tauri/src/modules/telegram_inbound.rs:1386`, `:2730` and `:2789`, and `src-tauri/tests/auto_switcher_e2e_test.rs:139` call the wrapper `backup_active_running_prompts(Some(..), None)`. They keep compiling unchanged because the wrapper keeps its `Option<&str>` signature. Step 16 changes some of them.
- `src-tauri/src/bin/agm.rs` calls the wrapper four times. They keep compiling unchanged, so do not edit `agm.rs` in this step:
  - `:4061` `match backup_prompts_db::backup_active_running_prompts(Some(&target_instance), custom_file) {`
  - `:4492` `match backup_prompts_db::backup_active_running_prompts(Some("default"), Some(&file_path)) {`
  - `:14509` `let backup_batch = backup_prompts_db::backup_active_running_prompts(Some(&new_inst.id), None);`
  - `:14726` `let backup_batch_2 = backup_prompts_db::backup_active_running_prompts(Some(&new_inst.id), None);`

## 4. Current code

Expected drift: step 07 replaced the dedupe block near the end of the loop and the imports block. Those are not quoted here. Compare only the blocks below.

### Change A (`:221` to `:227`)

```rust
/// Backward compatible wrapper with instance support
pub fn backup_active_running_prompts(
    instance_id: Option<&str>,
    custom_file: Option<&str>,
) -> Result<(BackupBatchInfo, Vec<PromptBackupRecord>), String> {
    backup_active_running_prompts_for_instance(instance_id, custom_file)
}
```

### Change B (`:229` to `:241`)

```rust
/// Backup all currently active and queued running prompts into the split SQLite database scoped to an instance
pub fn backup_active_running_prompts_for_instance(
    instance_id: Option<&str>,
    custom_file: Option<&str>,
) -> Result<(BackupBatchInfo, Vec<PromptBackupRecord>), String> {
    let target_inst = instance_id.unwrap_or("default");
    let _ = auto_cleanup_expired(custom_file, 86400);

    let all_prompts = repo_db::list_all_prompts().unwrap_or_default();
    let running_prompts = repo_db::discover_running_prompts_from_antigravity(target_inst);
    let conversations = agy_cleaner::scan_conversations(100);
    let now = Utc::now().timestamp();
    let freshness_cutoff = now - 7200;
```

### Change C (`:243` to `:265`)

```rust
    // Consolidate candidate prompts to back up: prioritize live discovered prompts
    let mut candidate_prompts = Vec::new();
    for p in running_prompts.clone() {
        candidate_prompts.push(p);
    }
    for p in all_prompts {
        let is_inst_match = p.instance_id.is_empty()
            || p.instance_id == target_inst
            || (target_inst == "default"
                && (p.instance_id == "default" || p.instance_id.is_empty()));
        if is_inst_match
            && (p.status == "running" || p.status == "queued" || p.status == "backed_up")
            && p.updated_at >= freshness_cutoff
            && !candidate_prompts.iter().any(|c| {
                c.id == p.id
                    || (c.repo_path.to_lowercase().replace('\\', "/")
                        == p.repo_path.to_lowercase().replace('\\', "/")
                        && c.prompt_content.trim() == p.prompt_content.trim())
            })
        {
            candidate_prompts.push(p);
        }
    }
```

### Change D (`:297` to `:304`)

```rust
    for (idx, p) in candidate_prompts.iter().enumerate() {
        let repo_norm = p.repo_path.to_lowercase().replace('\\', "/");
        let matched_conv = conversations.iter().find(|c| {
            let uris_norm = c.workspace_uris.to_lowercase().replace('\\', "/");
            (!repo_norm.is_empty() && uris_norm.contains(&repo_norm))
                || uris_norm.contains(&p.project_id.to_lowercase())
        });
```

The lines right after stay unchanged:

```rust
        let conv_id = matched_conv
            .map(|c| c.conversation_id.clone())
            .or_else(|| p.session_id.clone())
            .unwrap_or_else(|| "-".to_string());

        let conv_name = matched_conv.and_then(|c| {
            if c.title.trim().is_empty() {
                None
            } else {
                Some(c.title.clone())
            }
        });
```

They read only `conversation_id` and `title`, which the new `BackupConversation` type also has.

### Change F (`integration.rs:341` to `:345`)

```rust
        let _ = crate::modules::repo_db::backup_running_prompts("default");
        let _ = crate::modules::backup_prompts_db::backup_active_running_prompts_for_instance(
            Some("default"),
            None,
        );
```

Keep `"default"` here: the desktop switch path only ever switches the default instance.

## 5. New code

### Change A

```rust
/// Backward compatible wrapper with instance support. `None` is refused: a backup always names
/// the instance it belongs to.
pub fn backup_active_running_prompts(
    instance_id: Option<&str>,
    custom_file: Option<&str>,
) -> Result<(BackupBatchInfo, Vec<PromptBackupRecord>), String> {
    match instance_id {
        Some(id) => backup_active_running_prompts_for_instance(id, custom_file),
        None => Err("Prompt backup needs an instance id".to_string()),
    }
}
```

### Change B

```rust
/// Backup all currently active and queued running prompts into the split SQLite database scoped to an instance
pub fn backup_active_running_prompts_for_instance(
    instance_id: &str,
    custom_file: Option<&str>,
) -> Result<(BackupBatchInfo, Vec<PromptBackupRecord>), String> {
    let inst = crate::modules::instance::canonical_instance_id(instance_id)?;
    let target_inst = inst.as_str();
    let _ = auto_cleanup_expired(custom_file, 86400);

    let all_prompts = repo_db::list_all_prompts().unwrap_or_default();
    let running_prompts = repo_db::discover_running_prompts_from_antigravity(target_inst);
    let conversations =
        load_backup_conversations(&repo_db::gemini_dirs_tagged(&InstanceScope::One(inst.clone())));
    let now = Utc::now().timestamp();
    let freshness_cutoff = now - 7200;
```

The local `target_inst: &str` keeps its name, so the rest of the function (record `instance_id: Some(target_inst.to_string())` and step 07's `upsert_prompt_backup_record(&conn, &record, target_inst, now)?;`) compiles unchanged.

### Change C

```rust
    // Consolidate candidate prompts to back up: prioritize live discovered prompts
    let candidate_prompts =
        select_backup_candidates(running_prompts.clone(), all_prompts, target_inst, freshness_cutoff);
```

Before saving, check that `running_prompts` is still used later in the function (`gitmap aum search "running_prompts" src-tauri/src/modules/backup_prompts_db.rs --ext .rs`). If its only use is now this line, remove `.clone()` and pass `running_prompts`. If clippy reports `redundant_clone` here, do the same.

### Change D

```rust
    for (idx, p) in candidate_prompts.iter().enumerate() {
        let matched_conv = match_backup_conversation(p, &conversations);
```

### Change E (insert directly above `/// Backward compatible wrapper with instance support`)

```rust
/// One summarized conversation of a gemini dir, tagged with the instance that owns the dir.
#[derive(Debug, Clone)]
pub(crate) struct BackupConversation {
    pub owner_id: String,
    pub conversation_id: String,
    pub title: String,
    pub workspace_uris: String,
}

/// Thin loader: reads `conversation_summaries.db` of every tagged dir, read-only.
fn load_backup_conversations(tagged_dirs: &[(String, PathBuf)]) -> Vec<BackupConversation> {
    let mut out = Vec::new();
    for (owner_id, dir) in tagged_dirs {
        let summaries_db = dir.join("conversation_summaries.db");
        if !summaries_db.exists() {
            continue;
        }
        let Ok(summaries) = Connection::open_with_flags(
            &summaries_db,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_URI,
        ) else {
            continue;
        };
        let Ok(mut stmt) = summaries.prepare(
            "SELECT conversation_id, title, workspace_uris FROM conversation_summaries ORDER BY rowid DESC LIMIT 200",
        ) else {
            continue;
        };
        let Ok(rows) = stmt.query_map([], |r| {
            Ok(BackupConversation {
                owner_id: owner_id.clone(),
                conversation_id: r.get(0)?,
                title: r.get::<_, Option<String>>(1)?.unwrap_or_default(),
                workspace_uris: r.get::<_, Option<String>>(2)?.unwrap_or_default(),
            })
        }) else {
            continue;
        };
        out.extend(rows.flatten());
    }
    out
}

fn backup_path_key(path: &str) -> String {
    repo_db::normalize_path_for_compare(path)
        .trim_start_matches('/')
        .to_string()
}

/// First workspace folder of a summary row. `workspace_uris` is a JSON array of file URIs; a
/// plain single URI is accepted too.
fn first_workspace_path(workspace_uris: &str) -> Option<String> {
    let trimmed = workspace_uris.trim();
    if trimmed.is_empty() {
        return None;
    }
    let first = match serde_json::from_str::<Vec<String>>(trimmed) {
        Ok(list) => list.into_iter().next()?,
        Err(_) => trimmed.to_string(),
    };
    let path = repo_db::decode_uri_to_path_pub(&first);
    if path.trim().is_empty() {
        None
    } else {
        Some(path)
    }
}

/// A prompt is backed up for `instance_id` only when it belongs to exactly that instance, is
/// still pending work, and was touched after `freshness_cutoff`.
pub(crate) fn is_backup_candidate(p: &ActivePrompt, instance_id: &str, freshness_cutoff: i64) -> bool {
    let is_pending = p.status == repo_db::prompt_status::RUNNING
        || p.status == repo_db::prompt_status::QUEUED
        || p.status == repo_db::prompt_status::BACKED_UP;
    !instance_id.is_empty()
        && p.instance_id == instance_id
        && is_pending
        && p.updated_at >= freshness_cutoff
}

/// Live discovered prompts of the instance first, then stored prompts that pass
/// `is_backup_candidate` and are not the same prompt (same id, or same repo and text).
pub(crate) fn select_backup_candidates(
    live: Vec<ActivePrompt>,
    stored: Vec<ActivePrompt>,
    instance_id: &str,
    freshness_cutoff: i64,
) -> Vec<ActivePrompt> {
    let mut candidates: Vec<ActivePrompt> = live
        .into_iter()
        .filter(|p| !instance_id.is_empty() && p.instance_id == instance_id)
        .collect();
    for p in stored {
        if !is_backup_candidate(&p, instance_id, freshness_cutoff) {
            continue;
        }
        let repo_key = backup_path_key(&p.repo_path);
        let is_duplicate = candidates.iter().any(|c| {
            c.id == p.id
                || (backup_path_key(&c.repo_path) == repo_key
                    && c.prompt_content.trim() == p.prompt_content.trim())
        });
        if !is_duplicate {
            candidates.push(p);
        }
    }
    candidates
}

/// Conversation of the prompt's own instance: same session id first, then the conversation whose
/// first workspace folder is exactly the prompt's repo path.
pub(crate) fn match_backup_conversation<'a>(
    p: &ActivePrompt,
    conversations: &'a [BackupConversation],
) -> Option<&'a BackupConversation> {
    let session_id = p
        .session_id
        .as_deref()
        .map(str::trim)
        .filter(|sid| !sid.is_empty() && *sid != "-");
    if let Some(sid) = session_id {
        let by_session = conversations
            .iter()
            .find(|c| c.owner_id == p.instance_id && c.conversation_id == sid);
        if by_session.is_some() {
            return by_session;
        }
    }
    let repo_key = backup_path_key(&p.repo_path);
    if repo_key.is_empty() {
        return None;
    }
    conversations.iter().find(|c| {
        c.owner_id == p.instance_id
            && first_workspace_path(&c.workspace_uris)
                .map(|path| backup_path_key(&path) == repo_key)
                .unwrap_or(false)
    })
}
```

After Changes B and D, `agy_cleaner` may no longer be used in this file. Run `gitmap aum search "agy_cleaner::" src-tauri/src/modules/backup_prompts_db.rs --ext .rs`. If it returns 0 hits, delete the line `use crate::modules::agy_cleaner;` from the imports block. If it still has hits, keep the import.

`serde_json` is a crate dependency already used across `src-tauri/src/modules`; no `use` line is needed for the fully qualified call.

### Change F

```rust
        let _ = crate::modules::repo_db::backup_running_prompts("default");
        let _ = crate::modules::backup_prompts_db::backup_active_running_prompts_for_instance(
            "default", None,
        );
```

(`cargo fmt` may reflow this call; accept its output.)

## 6. Tests

Paste inside `mod tests` of `src-tauri/src/modules/backup_prompts_db.rs`, before the final `}` of the module (after step 07's and step 12's tests). They are pure: no database, no registry, no environment variables.

```rust
    fn candidate_prompt(id: &str, instance_id: &str, repo_path: &str, status: &str) -> ActivePrompt {
        ActivePrompt {
            id: id.to_string(),
            project_id: "app-1".to_string(),
            instance_id: instance_id.to_string(),
            repo_path: repo_path.to_string(),
            prompt_content: format!("work for {}", id),
            model: None,
            session_id: None,
            status: status.to_string(),
            created_at: 100,
            updated_at: 100,
            image_payload: None,
            source_dir: None,
        }
    }

    fn summary_conversation(owner_id: &str, conversation_id: &str, uri: &str) -> BackupConversation {
        BackupConversation {
            owner_id: owner_id.to_string(),
            conversation_id: conversation_id.to_string(),
            title: format!("title {}", conversation_id),
            workspace_uris: format!("[\"{}\"]", uri),
        }
    }

    #[test]
    fn empty_instance_legacy_row_not_in_any_backup() {
        let stored = vec![
            candidate_prompt("legacy", "", "D:/work/app", "backed_up"),
            candidate_prompt("mine", "default", "D:/work/other", "backed_up"),
        ];
        let for_default = select_backup_candidates(Vec::new(), stored.clone(), "default", 0);
        let ids: Vec<&str> = for_default.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(ids, vec!["mine"]);

        let for_named = select_backup_candidates(Vec::new(), stored, "inst-a", 0);
        assert!(for_named.is_empty());
        assert!(!is_backup_candidate(
            &candidate_prompt("legacy", "", "D:/work/app", "backed_up"),
            "",
            0
        ));
    }

    #[test]
    fn two_instances_same_repo_backup_separately() {
        let stored = vec![
            candidate_prompt("a-1", "inst-a", "D:/work/app", "running"),
            candidate_prompt("b-1", "inst-b", "D:/work/app", "running"),
            candidate_prompt("a-old", "inst-a", "D:/work/app", "dispatched"),
            candidate_prompt("a-stale", "inst-a", "D:/work/app2", "queued"),
        ];
        let mut stale = stored.clone();
        stale[3].updated_at = 10;

        let for_a = select_backup_candidates(Vec::new(), stale.clone(), "inst-a", 50);
        let for_b = select_backup_candidates(Vec::new(), stale, "inst-b", 50);

        let ids_a: Vec<&str> = for_a.iter().map(|p| p.id.as_str()).collect();
        let ids_b: Vec<&str> = for_b.iter().map(|p| p.id.as_str()).collect();
        assert_eq!(ids_a, vec!["a-1"]);
        assert_eq!(ids_b, vec!["b-1"]);

        let live_other = vec![candidate_prompt("live-b", "inst-b", "D:/work/app", "backed_up")];
        let with_live = select_backup_candidates(live_other, stored, "inst-a", 0);
        assert!(with_live.iter().all(|p| p.instance_id == "inst-a"));
    }

    #[test]
    fn backup_conversation_match_uses_session_then_exact_path() {
        let conversations = vec![
            summary_conversation("inst-b", "cid-1", "file:///d:/work/app"),
            summary_conversation("inst-a", "cid-v2", "file:///d:/work/app-v2"),
            summary_conversation("inst-a", "cid-path", "file:///d:/work/app"),
            summary_conversation("inst-a", "cid-1", "file:///d:/work/elsewhere"),
        ];

        let mut by_session = candidate_prompt("p1", "inst-a", "D:/work/app", "running");
        by_session.session_id = Some("cid-1".to_string());
        let found = match_backup_conversation(&by_session, &conversations).map(|c| c.conversation_id.as_str());
        assert_eq!(found, Some("cid-1"));
        assert_eq!(
            match_backup_conversation(&by_session, &conversations).map(|c| c.owner_id.as_str()),
            Some("inst-a")
        );

        let by_path = candidate_prompt("p2", "inst-a", "D:\\work\\app", "running");
        let found = match_backup_conversation(&by_path, &conversations).map(|c| c.conversation_id.as_str());
        assert_eq!(found, Some("cid-path"));

        let neighbor = candidate_prompt("p3", "inst-a", "D:/work/ap", "running");
        assert!(match_backup_conversation(&neighbor, &conversations).is_none());

        let other_instance = candidate_prompt("p4", "inst-c", "D:/work/app", "running");
        assert!(match_backup_conversation(&other_instance, &conversations).is_none());
    }
```

If `ActivePrompt` has more fields than `source_dir` after step 03, the compiler names them in `candidate_prompt`; set each to `None` or its default and report it.

The plan's test `two_instances_same_repo_backup_separately` also asked that the two instances end up as two rows in `prompt_backups`. Writing the rows is step 07's `upsert_prompt_backup_record`, already covered by step 07's `backup_dedupe_same_prompt_id_in_two_instances_gives_two_rows` and `backup_dedupe_does_not_steal_other_instance_row`; this test covers the candidate side only.

## 7. Gate

```text
$env:CARGO_TARGET_DIR="$PWD\src-tauri\target"; $env:CARGO_BUILD_JOBS="1"
cd src-tauri; cargo fmt; cargo fmt -- --check; cargo clippy --all-targets --all-features; cargo test --lib -- --test-threads=1 empty_instance_legacy_row_not_in_any_backup; cd ..
cd src-tauri; cargo test --lib -- --test-threads=1 two_instances_same_repo_backup_separately; cargo test --lib -- --test-threads=1 backup_conversation_match_uses_session_then_exact_path; cargo test --lib -- --test-threads=1 backup_dedupe_; cargo test --lib -- --test-threads=1 test_backup_db_lifecycle; cd ..
```

Every command must exit 0 and each filter must run at least one test.

## 8. Commit

```text
Fix: prompts - backup candidates from the exact instance only
```

Stage only:

```text
git add src-tauri/src/modules/backup_prompts_db.rs src-tauri/src/modules/integration.rs
```

## 9. Done when

- [ ] `backup_active_running_prompts_for_instance` takes `instance_id: &str` and starts with `canonical_instance_id(instance_id)?`.
- [ ] The wrapper `backup_active_running_prompts` returns `Err` for `None`.
- [ ] `unwrap_or("default")`, `is_inst_match`, `repo_norm` and `uris_norm` no longer exist in `backup_prompts_db.rs`.
- [ ] Conversations come from `load_backup_conversations(&repo_db::gemini_dirs_tagged(&InstanceScope::One(..)))`, not `agy_cleaner::scan_conversations`.
- [ ] `match_backup_conversation` checks the owner instance, then session id, then exact path key; it never calls `contains`.
- [ ] `integration.rs` passes `"default"` (not `Some("default")`).
- [ ] Callers searched in `src-tauri/src`, `src-tauri/src/bin/agm.rs` and each `src-tauri/tests/*.rs` file. `integration.rs:342` is the only direct caller of `backup_active_running_prompts_for_instance` and is updated here. The wrapper callers (including `agm.rs:4061`, `:4492`, `:14509`, `:14726` and `tests/auto_switcher_e2e_test.rs:139`) compile unchanged.
- [ ] Gate exits 0.
- [ ] Only the two files are staged.
