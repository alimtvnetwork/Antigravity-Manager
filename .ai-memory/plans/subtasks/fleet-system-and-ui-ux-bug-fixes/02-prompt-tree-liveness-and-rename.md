# Subtask Implementation Plan: Prompt Tree Liveness Detection & Conversation Rename Synchronization

> **Subtask ID:** `02-prompt-tree-liveness-and-rename`  
> **Parent Slug:** `fleet-system-and-ui-ux-bug-fixes`  
> **Spec Reference:** [02-spec/21-app/fleet-system-and-ui-ux-bug-fixes/02-component-spec.md](../../../../02-spec/21-app/fleet-system-and-ui-ux-bug-fixes/02-component-spec.md)  
> **Target Files:**
> - `src-tauri/src/commands/cmd_running_up_next.rs`
> - `src-tauri/src/modules/repo_db/tree_collect.rs`
> - `src-tauri/src/modules/repo_db/tree_phases.rs`
> - `src-tauri/src/modules/repo_db/transcript.rs`
> - `src-tauri/src/modules/repo_db/sequences.rs`
> - `src/components/instances/PromptTreeView/types.ts`
> - `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`
> - `src/components/instances/PromptTreeView/ConversationNode.tsx`
> - `src/components/instances/PromptTreeView/DetailHeader.tsx`
> - `src/components/instances/PromptTreeView/TreePanel.tsx`  
> **Status:** `[QUEUED]`  
> **Author:** `@aukgit`  

---

## 1. Context & Objectives

During end-to-end multi-instance observation in Antigravity-Manager, users observed two major UI/UX defects:
1. **Running Prompt Detection Failures:**
   - Active prompts entering the `dispatched` state were classified under "Up Next" rather than "Now Running" in `cmd_running_up_next.rs`.
   - Prompts with empty `instance_id` strings (common in default instances) were dropped entirely by `tree_collect.rs`.
   - Synthesized project entries in `tree_phases.rs` failed to mark projects with dispatched prompts as running.
   - Live transcript turns with empty step status (representing actively streaming turns) were prematurely treated as completed in `transcript.rs`.
2. **Conversation Rename Desynchronization:**
   - In `DetailHeader.tsx`, committing an edited conversation title mutated `selectedConversation.title` directly without notifying parent React state, causing the left sidebar tree to retain stale titles.
   - In `ConversationNode.tsx`, renaming a conversation did not forward `targetInst` to `onRenameConversation`, causing multi-instance title updates to target the wrong instance folder.
   - In `PromptTreeViewModal.tsx`, `handleRenameConversation` updated only `treeData` while leaving `selectedConversation` and `selectedProject` stale.
   - In `sequences.rs`, the SQLite virtual table `conversations_fts` was omitted from title updates, and no fallback search across `gemini_dirs_tagged(None)` existed if a specific instance directory yielded zero matches.

---

## 2. Step-by-Step Implementation Plan

### Step 1: Align Now Running & Up Next Status Queries (`src-tauri/src/commands/cmd_running_up_next.rs`)
1. In `query_now_running`:
   - Change `WHERE status = 'running'` to `WHERE status IN ('running', 'dispatched')`.
   - Ensure `ORDER BY created_at ASC, id ASC LIMIT 1` (strict FIFO invariant).
2. In `query_up_next`:
   - Change `WHERE status IN ('queued', 'pending', 'dispatched')` to `WHERE status IN ('queued', 'pending')`.
   - Ensure `ORDER BY created_at ASC, id ASC LIMIT 4`.
3. In `query_up_next_total`:
   - Change `WHERE status IN ('queued', 'pending', 'dispatched')` to `WHERE status IN ('queued', 'pending')`.

---

### Step 2: Preserve Default Instance Prompts in Tree Collection (`src-tauri/src/modules/repo_db/tree_collect.rs`)
1. In `collect_tree_data`:
   - Remove the early `continue` guard that drops empty instance IDs:
     ```rust
     // REMOVE:
     // if ap.instance_id.trim().is_empty() {
     //     continue;
     // }
     ```
   - Retain all prompts and normalize instance ID through `is_default_instance_match`:
     ```rust
     let key = normalize_path_for_compare(&ap.repo_path);
     let inst = if is_default_instance_match(&ap.instance_id) {
         "default".to_string()
     } else {
         crate::modules::instance::resolve_instance_id(&ap.instance_id)
             .unwrap_or_else(|_| ap.instance_id.clone())
     };
     active_prompts_by_inst_and_path
         .entry((inst, key))
         .or_default()
         .push(ap);
     ```

---

### Step 3: Include Dispatched Status in Project Liveness Flag (`src-tauri/src/modules/repo_db/tree_phases.rs`)
1. In `discover_fallback_projects`:
   - Locate the project synthesis block from `active_prompts`.
   - Update `is_running`:
     ```rust
     projects.push(RunningProject {
         id: project_key,
         instance_id: norm_inst,
         repo_name,
         repo_path: raw_repo_path,
         workspace_storage_path: None,
         is_running: status.equals_any(&["running", "in_flight", "dispatched"]),
         last_detected_at: if updated_at > 0 { updated_at } else { now_fb },
     });
     ```

---

### Step 4: Guard Planner Turn In-Flight Status (`src-tauri/src/modules/repo_db/transcript.rs`)
1. In `is_planner_turn_completed`:
   - Remove `s_status.is_empty()` from the completion check.
   - Refactor function to treat empty status as in-flight / non-terminal:
     ```rust
     pub(crate) fn is_planner_turn_completed(has_open_tools: bool, s_status: &str) -> bool {
         if has_open_tools {
             return false;
         }
         // An empty s_status indicates an in-flight, streaming, or non-finalized turn.
         // Only explicit terminal statuses (done, completed, idle, error) indicate turn completion.
         is_terminal_status(s_status)
     }
     ```

---

### Step 5: Extend Frontend Props & Signatures for Instance Routing
1. In `src/components/instances/PromptTreeView/types.ts`:
   - Export callback signature:
     ```typescript
     export type RenameConversationFn = (
         conversationId: string,
         newTitle: string,
         targetInst?: string
     ) => Promise<void> | void;
     ```
2. In `src/components/instances/PromptTreeView/ConversationNode.tsx`:
   - Update `ConversationNodeProps` to use `RenameConversationFn` for `onRenameConversation`.
   - In `handleSaveTitle`:
     ```typescript
     const targetInst = conv.instance_id || project.instance_id;
     if (onRenameConversation) {
         await onRenameConversation(conv.conversation_id, trimmed, targetInst);
     }
     ```
3. In `src/components/instances/PromptTreeView/TreePanel.tsx`:
   - Update `TreePanelProps` to forward `targetInst` in `onRenameConversation`.
4. In `src/components/instances/PromptTreeView/DetailHeader.tsx`:
   - Add `onRenameConversation?: RenameConversationFn` to `DetailHeaderProps`.
   - In `handleCommitTitle`, replace direct mutation `selectedConversation.title = trimmed` with parent callback invocation:
     ```typescript
     const targetInst = selectedConversation.instance_id || 'default';
     if (onRenameConversation) {
         await onRenameConversation(selectedConversation.conversation_id, trimmed, targetInst);
     } else {
         await invoke('rename_conversation', {
             instanceId: targetInst,
             conversationId: selectedConversation.conversation_id,
             newTitle: trimmed,
         });
         selectedConversation.title = trimmed;
     }
     ```

---

### Step 6: Reactive Multi-State Synchronization (`src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`)
1. Update `handleRenameConversation` in `PromptTreeViewModal.tsx`:
   - Accept `(conversationId: string, newTitle: string, targetInst?: string)`.
   - Resolve target instance: `targetInst || selectedProject?.instance_id || instanceId || 'default'`.
   - Invoke `rename_conversation` IPC.
   - Synchronize all three state tiers:
     - `setTreeData`: map across projects and conversations to update `title`.
     - `setSelectedConversation`: update `title` if `prev?.conversation_id === conversationId`.
     - `setSelectedProject`: update matching conversation inside `selectedProject.conversations`.
2. Pass `onRenameConversation={handleRenameConversation}` to `<DetailHeader ... />`.

---

### Step 7: Dual-Store Synchronization & FTS5 Update with Tagged Fallback (`src-tauri/src/modules/repo_db/sequences.rs`)
1. In `update_conversation_title`:
   - Track `let mut updated_summaries_count = 0;`.
   - In candidate database loop, update `conversation_summaries`:
     ```sql
     UPDATE conversation_summaries
     SET title = ?1, last_modified_time = ?2
     WHERE conversation_id = ?3;
     ```
     (with `LIKE ?3` fallback if exact match updates 0 rows).
   - Also update `conversations_fts` virtual table:
     ```sql
     UPDATE conversations_fts
     SET title = ?1
     WHERE conversation_id = ?2;
     ```
     (with `LIKE ?2` fallback if exact match updates 0 rows; wrap with `record_ignored` to tolerate databases without FTS).
   - Accumulate rows updated into `updated_summaries_count`.
2. Fallback search across `gemini_dirs_tagged(None)`:
   - If `updated_summaries_count == 0`:
     - Scan `gemini_dirs_tagged(None)`.
     - Attempt identical updates against `conversation_summaries.db` in tagged directories.
     - Break early once a matching database row is updated.
3. Update `agm_conversation_sequences` and `active_prompts` in `repo_prompts.db`.
4. Invalidate prompt tree cache for both `resolved_inst` and `"all"`.

---

## 3. Verification & Pre-flight Testing Checklist

- [ ] `cmd_running_up_next.rs`: Verify dispatched prompt appears in `now_running` and is excluded from `up_next`.
- [ ] `tree_collect.rs`: Verify active prompts with empty `instance_id` are grouped under `default` and not dropped.
- [ ] `tree_phases.rs`: Verify project with dispatched prompt has `is_running: true`.
- [ ] `transcript.rs`: Verify `is_planner_turn_completed(false, "")` returns `false`.
- [ ] `PromptTreeViewModal.tsx`: Verify conversation rename updates left tree, header title, and inspector immediately.
- [ ] `DetailHeader.tsx`: Verify renaming via double-click on header title triggers parent callback without direct prop mutation.
- [ ] `sequences.rs`: Verify `conversations_fts` is updated and full-text search reflects the new title.
- [ ] `cargo clippy --all-targets --all-features` passes cleanly without warnings or errors.
- [ ] `npm run build` succeeds without TypeScript type errors.
