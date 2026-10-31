# Subtask Plan 230-02: Restored Status Alignment, Strict FIFO Queue & Reactive Dispatch

- **Subtask ID:** `230-02`
- **Subtask Slug:** `02-subtask-restore-status-and-queue`
- **Parent Task:** `230-running-prompt-restore-after-account-switch`
- **Component Spec:** `02-spec/21-app/230-running-prompt-restore-after-account-switch/02-component-spec.md`
- **Status:** READY FOR IMPLEMENTATION
- **Author:** Spec Author 02

---

## 1. Context & Objectives

When Antigravity Manager switches accounts (manually or via the background auto-profile switcher), active prompts must be safely restored to the new account without prompt loss, duplicates, queue order inversion, or multi-second stalls.

This subtask implements the surgical backend modifications required to:
1. Align restored prompt statuses across queries so prompts restored as `"in_progress"` are immediately recognized and dispatched.
2. Enforce strict First-In-First-Out (FIFO) ordering across dispatch helpers and listing queries, eliminating LIFO inversions (`pick_latest_prompt` and `ORDER BY updated_at DESC`).
3. Reactively wake up the background prompt queue scheduler immediately upon restore completion via `notify_prompt_queue()`, avoiding idle polling delays.
4. Eliminate false-positive subagent prompt dropping (`you are ` prefix) and resolve narrow schema fallback drops in transcript discovery.

---

## 2. Step-by-Step Implementation Roadmap

### Step 1: Restored Status & Resend Query Alignment (`src-tauri/src/modules/repo_db/resend.rs`)

- **Location:** `src-tauri/src/modules/repo_db/resend.rs:77` and `src-tauri/src/modules/repo_db/resend.rs:180`
- **Changes:**
  1. Update line 77 to include `'in_progress'` in the SQL status filter.
  2. Update line 180 to include `"in_progress"` in the deduplication cache bypass arm.

#### Code Modification Details:
```rust
// In resend_single_pass() (Line 77):
let mut stmt = conn
    .prepare(
        "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload 
         FROM active_prompts 
         WHERE status IN ('backed_up', 'queued', 'pending', 'running', 'in_progress')
         ORDER BY created_at ASC, id ASC LIMIT ?",
    )
    .map_err(|e| format!("Failed to prepare resend query: {}", e))?;
```

```rust
// In already_dispatched check (Line 180):
let already_dispatched = match prompt.status.as_str() {
    "backed_up" | "running" | "pending" | "queued" | "in_progress" => false,
    _ => {
        let mut cache = match get_dispatched_prompts_cache().lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let seen = cache.contains(&prompt.id) || cache.contains(&sig);
        if !seen {
            cache.insert(prompt.id.clone());
            cache.insert(sig);
        }
        seen
    }
};
```

---

### Step 2: Queue Dispatch Query Alignment (`src-tauri/src/modules/repo_db/queue.rs`)

- **Location:** `src-tauri/src/modules/repo_db/queue.rs:154`
- **Changes:** Include `'in_progress'` in the `check_and_dispatch_enqueued_prompts` query so restored in-progress prompts are dispatched as soon as the project is idle.

#### Code Modification Details:
```rust
// In check_and_dispatch_enqueued_prompts() (Line 150-156):
let prompt_opt: Option<ActivePrompt> = conn
    .query_row(
        "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload
         FROM active_prompts
         WHERE (project_id = ?1 OR repo_path = ?2)
           AND (instance_id = ?3 OR (?3 = 'default' AND (instance_id = 'default' OR instance_id = '__default__')))
           AND status IN ('queued', 'pending', 'in_progress')
         ORDER BY created_at ASC, id ASC
         LIMIT 1",
        params![&project_id, &repo_path, &inst_id],
        |row| {
            // ...
        },
    )
    .optional()
    .map_err(|e| format!("Failed to fetch enqueued prompt for {}: {}", project_id, e))?;
```

---

### Step 3: Strict FIFO Selection & Listing Invariant (`src-tauri/src/modules/repo_db/dispatch.rs` & `tree_format.rs`)

- **Location 1:** `src-tauri/src/modules/repo_db/dispatch.rs:393-403` and `dispatch.rs:585`
- **Location 2:** `src-tauri/src/modules/repo_db/tree_format.rs:116`
- **Changes:**
  1. Rename `pick_latest_prompt` to `pick_first_queued_prompt`.
  2. Invert sorting from `max_by` on `updated_at` to `min_by` on `created_at` (tie-break on `id`).
  3. Update caller at line 585 in `dispatch_single_latest_prompt`.
  4. In `tree_format.rs:116`, change `ORDER BY updated_at DESC` to `ORDER BY created_at ASC, id ASC`.

#### Code Modification Details:
```rust
// In src-tauri/src/modules/repo_db/dispatch.rs:
/// Pick the earliest enqueued prompt in strict FIFO order: smallest `created_at`,
/// ties broken deterministically by smallest `id`.
fn pick_first_queued_prompt(prompts: &[ActivePrompt]) -> Option<ActivePrompt> {
    prompts
        .iter()
        .min_by(|a, b| {
            a.created_at
                .cmp(&b.created_at)
                .then_with(|| a.id.cmp(&b.id))
        })
        .cloned()
}

// In dispatch_single_latest_prompt() (around line 585):
let Some(prompt) = pick_first_queued_prompt(&candidates) else {
    crate::modules::logger::log_info(&format!(
        "[RepoDB] Single-prompt resume: no backed-up prompt for instance '{}'; nothing to dispatch",
        target_inst
    ));
    return Ok(0);
};
```

```rust
// In src-tauri/src/modules/repo_db/tree_format.rs:116:
let mut stmt = conn.prepare(
    "SELECT id, project_id, instance_id, repo_path, prompt_content, model, session_id, status, created_at, updated_at, image_payload
     FROM active_prompts
     WHERE (instance_id = ?1 OR (instance_id = '__default__' AND ?1 = 'default'))
       AND status IN ('running', 'in_flight', 'dispatched')
     ORDER BY created_at ASC, id ASC"
).map_err(|e| e.to_string())?;
```

---

### Step 4: Reactive Post-Switch Scheduler Wakeup (`src-tauri/src/modules/instance/workspace.rs` & `post_switch_resume.rs`)

- **Location 1:** `src-tauri/src/modules/instance/workspace.rs:210-231`
- **Location 2:** `src-tauri/src/modules/instance/post_switch_resume.rs:362, 382, 521`
- **Changes:** Trigger immediate `notify_prompt_queue()` and spawn `check_and_dispatch_enqueued_prompts(Some(instance_id))` right after prompt restoration finishes.

#### Code Modification Details:
```rust
// In src-tauri/src/modules/instance/workspace.rs:
// Immediately after dispatch_single_latest_prompt() or pushed count calculation:
crate::modules::scheduler::notify_prompt_queue();
let inst_owned = instance_id.to_string();
tauri::async_runtime::spawn(async move {
    if let Err(err) = crate::modules::repo_db::check_and_dispatch_enqueued_prompts(Some(&inst_owned)) {
        crate::modules::logger::log_warn(&format!(
            "[INSTANCE_SWITCH:STAGE_5_RESTORE] Async queue dispatch failed for '{inst_owned}': {err}"
        ));
    }
});
```

```rust
// In src-tauri/src/modules/instance/post_switch_resume.rs:
// In run_post_switch_resume_after_delay(), arm_resume_timer(), and force_dispatch_resume():
// After run_resume_with_retry(instance_id).await completes:
crate::modules::scheduler::notify_prompt_queue();
let inst_for_dispatch = instance_id.to_string();
tauri::async_runtime::spawn(async move {
    let _ = crate::modules::repo_db::check_and_dispatch_enqueued_prompts(Some(&inst_for_dispatch));
});
```

---

### Step 5: Subagent Prompt Filter Fix & Narrow Schema Fallback (`src-tauri/src/modules/repo_db/discovery.rs`)

- **Location 1:** `src-tauri/src/modules/repo_db/discovery.rs:81`
- **Location 2:** `src-tauri/src/modules/repo_db/discovery.rs:45-64`
- **Location 3:** `src-tauri/src/modules/repo_db/discovery.rs:243-250`
- **Changes:**
  1. Remove `|| lower.starts_with("you are ")` from `is_subagent_or_system_content`.
  2. Set `status: "RUNNING".to_string()` in `read_conversation_summary_rows` narrow fallback.
  3. Ensure discovery filter respects conversations where status was populated via fallback.

#### Code Modification Details:
```rust
// In src-tauri/src/modules/repo_db/discovery.rs:
fn is_subagent_or_system_content(content: &str) -> bool {
    let trimmed = content.trim();
    if trimmed.contains("<SYSTEM_MESSAGE>")
        || trimmed.contains("[Message] timestamp=")
        || trimmed.contains("You are Research")
        || trimmed.contains("You are Worker")
        || trimmed.contains("You are Spec Author")
        || trimmed.starts_with("Task id ")
    {
        return true;
    }
    let lower = trimmed.to_lowercase();
    lower.contains("subagent")
        || lower.contains("invoked by a caller agent")
    // NOTE: Removed `|| lower.starts_with("you are ")` to avoid dropping valid user prompts
}
```

```rust
// In read_conversation_summary_rows() (Narrow Query Fallback, Lines 52-61):
let Ok(rows) = stmt.query_map([], |row| {
    Ok(ConversationSummaryRow {
        cid: row.get(0)?,
        preview: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
        status: "RUNNING".to_string(), // Default to RUNNING instead of empty string
        not_fully_idle: 1,
        workspace_uris: row.get(2)?,
        last_modified_time: None,
    })
}) else {
    return Vec::new();
};
```

---

## 3. Acceptance Criteria

- [ ] Prompts with status `'in_progress'` are retrieved by `resend_single_pass` and dispatched without caching drops.
- [ ] Prompts with status `'in_progress'` are considered by `check_and_dispatch_enqueued_prompts`.
- [ ] `pick_first_queued_prompt` selects the candidate with the earliest `created_at` timestamp.
- [ ] `list_running_prompts_for_instance` orders rows by `created_at ASC, id ASC`.
- [ ] Post-switch restore triggers `notify_prompt_queue()` and `check_and_dispatch_enqueued_prompts` immediately.
- [ ] User prompts starting with "you are " (e.g. "you are an expert at typescript...") are not flagged as subagent content.
- [ ] Narrow schema fallback rows are not dropped by the running status filter.
- [ ] All file paths in documentation and code comments are repository-relative Git paths.
