# Subtask Plan: Prompt Dispatch & Unified FIFO Enqueue Pipeline

> **Subtask ID:** `02-prompt-dispatch-and-fifo-enqueue-pipeline`  
> **Parent Slug:** `149-smart-instance-process-cache-and-prompt-enqueue-fix`  
> **Target Files:** `src-tauri/src/commands/instance.rs`, `src-tauri/src/lib.rs`, `src-tauri/src/modules/repo_db.rs`, `src/components/instances/PromptTreeViewModal.tsx`, `src/services/instanceService.ts`  
> **Status:** `[READY]`  
> **Author:** @aukgit  
> **Protocol:** `execute-parent-task-with-n-steps-v6`  

---

## 1. Context & Objectives

In Antigravity Manager, queueing or sending prompts from the Instance Prompt Tree failed due to duplicate shadowed command definitions in Rust, inconsistent payload schemas, and redundant frontend IPC calls that triggered repeated window launches.

This subtask resolves these issues:
1. **Unify `enqueue_prompt` Tauri Command**: Eliminate conflicting duplicate definitions in `src-tauri/src/commands/instance.rs` (lines 265 & 420) and duplicate registration in `src-tauri/src/lib.rs` (lines 1125 & 1130).
2. **Consolidate `enqueue_prompt_for_instance` in `repo_db.rs`**: Merge conflicting implementations (lines 2251 & 3614) into one definitive function that copies the prompt to the clipboard, writes `.antigravity_resume_task.json` with status `'queued'`, and inserts into SQLite `active_prompts`.
3. **Enforce Strict FIFO Ordering**: All queries that inspect, list, or dispatch queued prompts MUST enforce `ORDER BY created_at ASC, id ASC`.
4. **Streamline `handleResendPrompt` in `PromptTreeViewModal.tsx`**: Eliminate the redundant second launch/focus call in the frontend that races with backend prompt dispatch.

---

## 2. Step-by-Step Implementation Plan

### Step 1: Unify `enqueue_prompt` in `src-tauri/src/commands/instance.rs`

#### Current Problem
- Line 265 defines `enqueue_prompt(instance_id, repo_path, prompt_content, conversation_id)`.
- Line 420 defines `enqueue_prompt(instance_id, prompt_text, prompt_content, workspace_path, repo_path, conversation_id, project_id)`.
- Line 420 shadows line 265. However, line 420 only writes a row to SQLite and omits `.antigravity_resume_task.json` and clipboard copying.

#### Action
1. Remove both definitions and replace with a single canonical command:
   ```rust
   #[tauri::command]
   pub async fn enqueue_prompt(
       instance_id: String,
       repo_path: Option<String>,
       workspace_path: Option<String>,
       prompt_content: Option<String>,
       prompt_text: Option<String>,
       conversation_id: Option<String>,
       project_id: Option<String>,
   ) -> Result<crate::modules::repo_db::ActivePrompt, String> {
       let resolved_id = crate::modules::instance::resolve_instance_id(&instance_id)
           .unwrap_or(instance_id);
       let prompt = prompt_content
           .or(prompt_text)
           .unwrap_or_default();
       let target_repo = repo_path
           .or(workspace_path)
           .unwrap_or_default();

       tokio::task::spawn_blocking(move || {
           crate::modules::repo_db::enqueue_prompt_for_instance(
               &resolved_id,
               &target_repo,
               &prompt,
               conversation_id.as_deref(),
               project_id.as_deref(),
           )
       })
       .await
       .map_err(|e| format!("Task execution failed: {}", e))?
   }
   ```
2. This handler accepts either parameter convention from legacy or modern frontend callers and returns the full `ActivePrompt` entity.

---

### Step 2: Remove Duplicate Registration in `src-tauri/src/lib.rs`

#### Current Problem
- `src-tauri/src/lib.rs` registers `commands::enqueue_prompt` twice:
  - Line 1125: `commands::enqueue_prompt,`
  - Line 1130: `commands::enqueue_prompt,`

#### Action
1. Audit lines 1120-1135 in `src-tauri/src/lib.rs`.
2. Delete line 1130 so `commands::enqueue_prompt` appears exactly once in `generate_handler![]`.

---

### Step 3: Implement Unified `enqueue_prompt_for_instance` in `src-tauri/src/modules/repo_db.rs`

#### Current Problem
- Line 2251: `pub fn enqueue_prompt_for_instance(instance_id, prompt_text, workspace_path) -> Result<i64, AppError>`
- Line 3614: `pub fn enqueue_prompt_for_instance(instance_id, repo_path, prompt_content, conversation_id) -> Result<ActivePrompt, String>`

#### Action
1. Consolidate into a single definitive function:
   ```rust
   /// Enqueue a prompt into the FIFO queue for a specific instance,
   /// saving to SQLite split DB and writing .antigravity_resume_task.json with status queued.
   pub fn enqueue_prompt_for_instance(
       instance_id: &str,
       repo_path: &str,
       prompt_content: &str,
       conversation_id: Option<&str>,
       project_id: Option<&str>,
   ) -> Result<ActivePrompt, String> {
       crate::modules::instance::scan_and_cache_all_running_instances();
       copy_to_system_clipboard(prompt_content);

       let clean_inst = if instance_id.trim().is_empty() || instance_id == "__default__" {
           "default"
       } else {
           instance_id.trim()
       };
       let canonical_inst = crate::modules::instance::resolve_instance_id(clean_inst)
           .unwrap_or_else(|_| clean_inst.to_string());

       let prompt_id = format!("queued-{}", &uuid::Uuid::new_v4().to_string()[..8]);
       let clean_repo_name = project_id
           .map(|s| s.to_string())
           .unwrap_or_else(|| {
               Path::new(repo_path)
                   .file_name()
                   .map(|n| n.to_string_lossy().to_string())
                   .filter(|s| !s.is_empty())
                   .unwrap_or_else(|| "project".to_string())
           });
       let now = Utc::now().timestamp();

       let active_prompt = ActivePrompt {
           id: prompt_id,
           project_id: clean_repo_name,
           instance_id: canonical_inst.clone(),
           repo_path: repo_path.to_string(),
           prompt_content: prompt_content.to_string(),
           model: Some("gemini-2.5-pro".to_string()),
           session_id: conversation_id.map(|s| s.to_string()),
           status: "queued".to_string(),
           created_at: now,
           updated_at: now,
           image_payload: None,
       };

       save_or_requeue_prompt(&active_prompt)?;

       // Write resume task document to target repo
       if !repo_path.trim().is_empty() {
           let ws_dir = PathBuf::from(repo_path);
           if ws_dir.exists() {
               let payload = serde_json::json!({
                   "prompt_id": active_prompt.id,
                   "project_id": active_prompt.project_id,
                   "instance_id": canonical_inst,
                   "repo_path": repo_path,
                   "prompt_content": prompt_content,
                   "model": "gemini-2.5-pro",
                   "auto_boot": false,
                   "status": "queued",
                   "queued_at": now,
               });
               let serialized = serde_json::to_string_pretty(&payload).unwrap_or_default();
               let _ = fs::write(ws_dir.join(".antigravity_resume_task.json"), &serialized);
               let _ = fs::write(
                   ws_dir.join(format!(".antigravity_resume_task.{}.json", canonical_inst)),
                   &serialized,
               );
           }
       }

       invalidate_prompt_tree_cache(Some(&canonical_inst));
       Ok(active_prompt)
   }
   ```
2. Audit all SQLite queries selecting queued prompts (e.g. line 2150 in `dispatch_running_prompts`) and verify strict FIFO ordering:
   ```sql
   ORDER BY created_at ASC, id ASC
   ```

---

### Step 4: Streamline `handleResendPrompt` in `PromptTreeViewModal.tsx`

#### Current Problem
- In `src/components/instances/PromptTreeViewModal.tsx` (lines 1705-1725):
  1. Calls `sendPromptNow(targetInstId, repoPath, promptContent, conv?.conversation_id)`.
  2. Inside `send_prompt_now_for_instance`, backend already calls `ensure_instance_running_smart(&canonical_inst, Some(repo_path))` and focuses the window.
  3. Immediately following, frontend calls `focusInstanceWorkspace(targetInstId, repoPath, repoName)` or `focusOrLaunchInstance(targetInstId)`.
  4. This second call triggers a second `ensure_instance_running_smart` check while the first is still stabilizing, triggering race conditions and relaunches.

#### Action
1. In `handleResendPrompt`:
   - Keep `sendPromptNow`.
   - Remove the redundant call to `focusInstanceWorkspace` / `focusOrLaunchInstance` during prompt resend, because `send_prompt_now_for_instance` handles window focus natively on the backend.
   - If explicit focus is desired as a fallback, call `focus_instance_workspace` with error suppression without re-triggering launch.

---

### Step 5: Frontend Service Parity in `src/services/instanceService.ts`

#### Action
1. Ensure `enqueuePrompt` in `src/services/instanceService.ts` matches the unified signature:
   ```typescript
   export async function enqueuePrompt(
       instanceId: string,
       repoPath: string,
       promptContent: string,
       conversationId?: string,
       projectId?: string
   ): Promise<ActivePrompt> {
       try {
           return await invoke('enqueue_prompt', {
               instanceId,
               repoPath,
               promptContent,
               conversationId,
               projectId,
           });
       } catch (e: any) {
           const captured = useErrorStore.getState().captureError(e, {
               source: 'instanceService.enqueuePrompt',
               endpoint: 'enqueue_prompt',
               triggerAction: 'enqueue_prompt',
               context: { instanceId, repoPath },
           });
           useErrorStore.getState().openErrorModal(captured);
           throw e;
       }
   }
   ```

---

## 3. Verification & Safety Checklist

- [ ] `commands::enqueue_prompt` defined exactly once in `src-tauri/src/commands/instance.rs`.
- [ ] `commands::enqueue_prompt` registered exactly once in `src-tauri/src/lib.rs`.
- [ ] `enqueue_prompt_for_instance` defined exactly once in `src-tauri/src/modules/repo_db.rs`.
- [ ] Enqueuing a prompt writes `.antigravity_resume_task.json` with `"status": "queued"` and `"auto_boot": false`.
- [ ] All queue fetch queries enforce `ORDER BY created_at ASC, id ASC`.
- [ ] Clicking "Send Now" in Prompt Tree dispatches prompt without duplicate window opens.
- [ ] Pre-flight checks pass:
  - `cd src-tauri && cargo fmt -- --check`
  - `cd src-tauri && cargo clippy --all-targets --all-features`
  - `npm run build`
