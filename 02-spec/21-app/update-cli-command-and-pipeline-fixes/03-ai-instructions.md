# Master AI Instructions: Prompt Backup Continuity, CLI Parity & UI Conversation Targeting

- **Task Identifier**: `update-cli-command-and-pipeline-fixes`
- **Specification Reference**: `02-spec/21-app/update-cli-command-and-pipeline-fixes/02-component-spec.md`
- **Applicable Subtasks**:
  - `.ai-memory/plans/subtasks/update-cli-command-and-pipeline-fixes/03-prompt-backup-and-restore-switch-fix.md`
  - `.ai-memory/plans/subtasks/update-cli-command-and-pipeline-fixes/04-ui-conversation-choice-fix.md`
  - `.ai-memory/plans/subtasks/update-cli-command-and-pipeline-fixes/05-preflight-release-ceremony.md`
- **Target Release**: `v4.186.0`
- **Author**: `@aukgit`
- **Status**: `Ready for Execution`

---

## 1. Non-Negotiable Architectural Invariants

AI workers executing these modifications must strictly adhere to the following repo invariants:

1. **Relative Git Paths Only**: All file paths in documentation, comments, imports, and logs must be relative to the repository root (`src-tauri/...`, `src/...`, `02-spec/...`). Never emit absolute filesystem paths (`D:\...`, `C:\...`).
2. **Positive Booleans Only**: Every boolean identifier, field, and function must have positive polarity (`is_active`, `has_match`, `is_success`, `is_current_conversation`, `is_new_session`). Never introduce negative or double-negative names (`is_not_empty`, `disable_sync`).
3. **Strict FIFO Queue & Restore Order**: All prompt selection, restore, and queue ordering queries across SQLite tables (`active_prompts`, `prompt_backups`) MUST strictly enforce First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`. LIFO sorting (`ORDER BY updated_at DESC`) is strictly forbidden.
4. **Zero Silent Errors**: Every fallible operation must route its error into `AppError` or be recorded via `crate::error::record_ignored(..., "<operation_name>")` accompanied by an explicit `// Justification:` comment explaining why it is safe to proceed. Never leave a bare `let _ = ...`, `.ok()`, or empty `catch {}`.
5. **UI Dark-Glass Capsule Standard**: Adjacent toolbar controls or window controls must be grouped inside contiguous segmented pill capsules (`rounded-full`, shared border, subtle dividers, dark-glass backdrop). Inline dropdowns inside capsules MUST use transparent borderless styling: `bg-transparent border-0 focus:ring-0 focus:outline-none cursor-pointer`.
6. **Maintainer Attribution**: All release notes, changelogs, commits, and attribution must attribute strictly `@aukgit`. Zero other GitHub usernames or handles.
7. **No Modifying Git Commands**: Workers must NOT run `git add`, `git commit`, `git push`, or `git checkout`. Version tags and commit ceremonies are handled by the designated release step.

---

## 2. Step-by-Step Backend Implementation Guide

### 2.1 File 1: `src-tauri/src/modules/repo_db/backup.rs`

#### Step 2.1.1: Expand Status Transition Set in `backup_running_prompts`
Locate line 65–83 in `src-tauri/src/modules/repo_db/backup.rs`.

**Problem**: The SQL transition query currently filters strictly on `status = 'running'`. Prompts that are `'in_flight'` or `'dispatched'` are left unbacked-up and get orphaned when the IDE closes.

**Surgical Change**:
Expand the SQL query to transition `('running', 'in_flight', 'dispatched')`:

```rust
// Transition in-flight 'running', 'in_flight', and 'dispatched' prompts in active_prompts to 'backed_up' before switch
let transitioned = if instance_id == "all" {
    conn.execute(
        "UPDATE active_prompts SET status = 'backed_up', updated_at = ?1 
         WHERE status IN ('running', 'in_flight', 'dispatched')",
        params![now],
    )
    .unwrap_or(0)
} else if is_all_or_default {
    conn.execute(
        "UPDATE active_prompts SET status = 'backed_up', updated_at = ?1 
         WHERE status IN ('running', 'in_flight', 'dispatched') 
           AND instance_id IN ('default', '__default__')",
        params![now],
    )
    .unwrap_or(0)
} else {
    conn.execute(
        "UPDATE active_prompts SET status = 'backed_up', updated_at = ?1 
         WHERE status IN ('running', 'in_flight', 'dispatched') 
           AND instance_id = ?2",
        params![now, instance_id],
    )
    .unwrap_or(0)
};
```

#### Step 2.1.2: Update In-Memory Cache in `backup_running_prompts`
Locate line 86–97 in `src-tauri/src/modules/repo_db/backup.rs`.

**Surgical Change**:
```rust
if let Ok(mut map) = get_memory_prompts_map().lock() {
    for p in map.values_mut() {
        let matches_inst = instance_id == "all"
            || (is_all_or_default
                && (p.instance_id == "default" || p.instance_id == "__default__"))
            || p.instance_id == instance_id;
        let is_target_status = p.status == "running" || p.status == "in_flight" || p.status == "dispatched";
        if is_target_status && matches_inst {
            p.status = "backed_up".to_string();
            p.updated_at = now;
        }
    }
}
```

---

### 2.2 File 2: `src-tauri/src/modules/repo_db/auto_resume.rs`

#### Step 2.2.1: Include `'backed_up'` Status in Query
Locate lines 86–95 in `src-tauri/src/modules/repo_db/auto_resume.rs`.

**Problem**: The query checks `WHERE status IN ('queued', 'pending')`, so prompts transitioned to `'backed_up'` are never auto-resumed.

**Surgical Change**:
```rust
let mut prompt_stmt = conn
    .prepare(
        "SELECT id, prompt_content, model, image_payload 
         FROM active_prompts 
         WHERE status IN ('queued', 'pending', 'backed_up')
           AND (project_id = ?1 OR repo_path = ?2 OR project_id LIKE ?3)
           AND (instance_id = ?4 OR (?4 = 'default' AND (instance_id = 'default' OR instance_id = '__default__')))
         ORDER BY created_at ASC, id ASC LIMIT 1",
    )
    .map_err(|e| format!("Failed to prepare prompt query: {}", e))?;
```

#### Step 2.2.2: Include `'backed_up'` in Task File Fallback Check
Locate line 118 in `src-tauri/src/modules/repo_db/auto_resume.rs`.

**Surgical Change**:
```rust
let is_pending_status = task_status == "queued" || task_status == "pending" || task_status == "backed_up";
```

---

### 2.3 File 3: `src-tauri/src/modules/auto_switcher/rotation.rs`

#### Step 2.3.1: Cross-Instance Prompt Re-Attribution
Locate lines 203–225 in `src-tauri/src/modules/auto_switcher/rotation.rs` where `target.instance_id != current_instance_id` is handled.

**Problem**: Prompts backed up for `current_instance_id` stay associated with `current_instance_id`. When `target.instance_id` is launched, Step 3 calls `resend_running_commands_for_instance(Some(&target_inst_id_clone), 20)`, finding zero records.

**Surgical Change**:
Directly after requeueing/saving in-flight prompts (lines 211–215), re-attribute backed-up and queued prompts to the target instance:

```rust
// Re-attribute backed-up and queued prompts from depleted instance to target instance
if let Ok(conn) = crate::modules::repo_db::connect_db() {
    let now = chrono::Utc::now().timestamp();
    // Justification: Best-effort prompt re-attribution; logged on failure
    let reattribute_res = conn.execute(
        "UPDATE active_prompts 
         SET instance_id = ?1, updated_at = ?2 
         WHERE instance_id = ?3 
           AND status IN ('backed_up', 'queued', 'pending')",
        rusqlite::params![&target.instance_id, now, &current_instance_id],
    );
    match reattribute_res {
        Ok(count) if count > 0 => {
            logger::log_info(&format!(
                "[AutoSwitcher] Re-attributed {} backed-up/queued prompts from instance '{}' to target '{}'",
                count, current_instance_id, target.instance_id
            ));
        }
        Ok(_) => {}
        Err(e) => {
            logger::log_warn(&format!(
                "[AutoSwitcher] Failed to re-attribute prompts from '{}' to '{}': {}",
                current_instance_id, target.instance_id, e
            ));
        }
    }
}
```

---

### 2.4 File 4: `src-tauri/src/bin/agm/prompt_cmds.rs`

#### Step 2.4.1: Add `qe` Alias in `dispatch_prompt_domain`
Locate line 41 in `src-tauri/src/bin/agm/prompt_cmds.rs`.

**Surgical Change**:
```rust
"queue" | "qe" => cmd_prompts_queue(args),
```

#### Step 2.4.2: Add Flag Support & Current Directory Defaulting in `cmd_prompts_send`
Locate lines 106–109 in `src-tauri/src/bin/agm/prompt_cmds.rs`.

**Surgical Change**:
Replace static `"scratch/test-repo"` fallback with `std::env::current_dir()`:
```rust
let repo_path = ctx
    .repo_path
    .clone()
    .unwrap_or_else(|| {
        std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".to_string())
    });
```

Support `--new`, `--existing`, `--conversation <id>`:
```rust
let explicit_conv_id = ctx.conversation_id.clone();
let is_new_session = ctx.has_flag("new");
let is_existing_session = ctx.has_flag("existing");

let session_id = if is_new_session {
    Some(format!("conv-{}", uuid::Uuid::new_v4()))
} else if let Some(ref conv_id) = explicit_conv_id {
    Some(conv_id.clone())
} else if is_existing_session {
    repo_db::get_latest_session_id_for_project(&repo_path, &resolved_inst)
        .ok()
        .flatten()
        .or_else(|| Some(format!("conv-{}", uuid::Uuid::new_v4())))
} else {
    Some(format!("conv-{}", uuid::Uuid::new_v4()))
};
```

#### Step 2.4.3: Add Flag Support & Current Directory Defaulting in `cmd_prompts_queue`
Locate lines 288–292 and lines 325–339 in `src-tauri/src/bin/agm/prompt_cmds.rs`.

**Surgical Change**:
1. Default repository path to `std::env::current_dir()`:
```rust
let repo_path = ctx
    .repo_path
    .clone()
    .unwrap_or_else(|| {
        std::env::current_dir()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_else(|_| ".".to_string())
    });
```
2. Handle target conversation session:
```rust
let target_session_id = if ctx.has_flag("new") {
    None
} else if let Some(ref conv_id) = ctx.conversation_id {
    Some(conv_id.clone())
} else if ctx.has_flag("existing") {
    repo_db::get_latest_session_id_for_project(&repo_path, &resolved_inst)
        .ok()
        .flatten()
} else {
    None
};
```
And insert `target_session_id` into the `session_id` column of `active_prompts`.

---

### 2.5 File 5: `src-tauri/src/bin/agm/instance_cmds.rs`

#### Step 2.5.1: Add `agm instance prompt` Domain Routing
Locate lines 33–45 in `src-tauri/src/bin/agm/instance_cmds.rs`.

**Surgical Change**:
In `dispatch_instance_domain`, add route for `"prompt" | "prompts"`:
```rust
pub(crate) fn dispatch_instance_domain(subcommand: &str, args: &[String]) {
    match subcommand {
        "list" | "ls" => cmd_instances_list(args),
        "status" => cmd_instances_status(args),
        "create" | "add" | "new" => cmd_instances_create(args),
        "start" | "launch" => cmd_instances_start(args),
        "stop" | "kill" | "close" => cmd_instances_stop(args),
        "restart" => cmd_instances_restart(args),
        "switch" | "use" => cmd_instances_switch(args),
        "copy" | "clone" | "duplicate" => cmd_instances_copy(args),
        "delete" | "rm" => cmd_instances_delete(args),
        "logs" | "log" => cmd_instances_logs(args),
        "prompt" | "prompts" => crate::prompt_cmds::dispatch_prompt_domain_from_args(args),
        _ => crate::common::handle_unknown_domain_command("instances", subcommand, args),
    }
}
```

---

## 3. Step-by-Step Frontend Implementation Guide

### 3.1 File 6: `src/components/instances/PromptTreeView/types.ts`

#### Step 3.1.1: Export `ConversationTargetMode` Type
Add the following union type:
```typescript
export type ConversationTargetMode = 'current' | 'new';
```

---

### 3.2 File 7: `src/components/instances/PromptTreeView/DetailHeader.tsx`

#### Step 3.2.1: Add Props to `DetailHeaderProps`
```typescript
import type { ConversationTargetMode } from './types';

export interface DetailHeaderProps {
    // Existing props...
    targetMode: ConversationTargetMode;
    setTargetMode: (mode: ConversationTargetMode) => void;
}
```

#### Step 3.2.2: Render Target Mode Dropdown in Capsule 2
In `DetailHeader.tsx`, locate Capsule 2 (Execution & Workflow, line 194):
Directly beside the `confirmationSuffix` dropdown, add the `targetMode` dropdown:

```tsx
{/* Conversation Target Mode Selector */}
<div className="flex items-center px-1.5 py-0.5">
    <select
        value={props.targetMode}
        onChange={(e) => props.setTargetMode(e.target.value as ConversationTargetMode)}
        className="bg-transparent border-0 text-slate-700 dark:text-slate-200 text-[10.5px] font-semibold focus:ring-0 focus:outline-none cursor-pointer max-w-[95px] truncate pr-1 py-0"
        title="Target current conversation or start a new conversation on Send / Queue"
    >
        <option value="current" className="bg-white dark:bg-[#0c2438]">Target: Current</option>
        <option value="new" className="bg-white dark:bg-[#0c2438]">Target: New Conv</option>
    </select>
</div>
```

---

### 3.3 File 8: `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`

#### Step 3.3.1: Accept and Utilize `targetMode` in Dispatch Handlers
In `usePromptTreePromptActions.ts`:
1. Destructure `targetMode` from `state`.
2. In `handleResendPrompt`:
```typescript
const targetConversationId = targetMode === 'new' ? undefined : conv?.conversation_id;
await sendPromptNow(targetInstId, repoPath, promptContent, targetConversationId);
```
3. In `handleEnqueuePrompt`:
```typescript
const targetConversationId = targetMode === 'new' ? undefined : selectedConversation.conversation_id;
await invoke('enqueue_prompt', {
    instanceId: targetInstId,
    repoPath,
    promptContent,
    conversationId: targetConversationId,
});
```

---

## 4. Verification & Pre-Flight Gate Commands

AI workers must run the targeted validation suite before declaring completion:

1. **Rust Code Formatting**:
   ```bash
   cd src-tauri && cargo fmt -- --check
   ```
2. **Rust Clippy Gate**:
   ```bash
   cd src-tauri && cargo clippy --all-targets --all-features
   ```
3. **Frontend Compilation**:
   ```bash
   npm run build
   ```
4. **Targeted Backend Tests**:
   ```bash
   cd src-tauri && cargo test --lib modules::repo_db -- --nocapture
   ```

---

## 5. Release Ceremony Directives (`v4.186.0`)

Upon successful pre-flight gate completion, the release ceremony proceeds via:
1. **Atomic Version Bump**:
   ```bash
   npm run bump patch
   ```
   *(Verifies bump from `v4.185.0` to `v4.186.0` across all 14 manifests).*
2. **Documentation & Changelog Synchronization**:
   - Update `changelog.md` and `changelog_en.md` under `## [v4.186.0]`.
   - Update `README.md` ("## 📝 Changelog") and `README_EN.md` ("## 📝 Changelog").
   - Strict Attribution: Maintainer attribution is exclusively `@aukgit` (`(Thanks to @aukgit)`).
