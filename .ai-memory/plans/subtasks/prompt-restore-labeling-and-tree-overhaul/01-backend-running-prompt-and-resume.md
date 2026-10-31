# Subtask Plan: TASK-01 - Correct Running Prompt Matching & 10s Restore Labeling

> **Task Slug**: `prompt-restore-labeling-and-tree-overhaul`  
> **Subtask Code**: `TASK-01`  
> **Plan File**: `.ai-memory/plans/subtasks/prompt-restore-labeling-and-tree-overhaul/01-backend-running-prompt-and-resume.md`  
> **Target Version**: `v4.222.0`  
> **Assigned Worker**: Worker 01  
> **Dependencies**: None (Wave 1 kickoff)  
> **Reference Specification**: `02-spec/21-app/prompt-restore-labeling-and-tree-overhaul/01-running-prompt-and-restore-spec.md`  

---

## Subtask Objective

Eliminate the 7 isolated flaws causing the wrong prompt to be captured during running prompt detection and backup/restore, while building the full 10-second post-switch resume tracking system in the Rust backend and exposing interactive status pill badges with countdown and inspection modal in the React frontend.

---

## Target File Ownership & Changes

| Area | Relative File Path | Planned Modifications |
| :--- | :--- | :--- |
| **Backend** | `src-tauri/src/modules/repo_db/discovery.rs` | Refactor lines 152–200: integrate `inspect_first_line_is_subagent`, filter out system messages/subagents, eliminate stale turn-0 fallback, and leverage `inspect_conversation_transcript`. |
| **Backend** | `src-tauri/src/modules/repo_db/backup.rs` | Fix Flaw D (sanitize `state.vscdb` queries) and Flaw E (replace LIFO query at line 580 with FIFO `ORDER BY created_at ASC, id ASC`). |
| **Backend** | `src-tauri/src/modules/instance/switch.rs` | Fix Flaw G (line 155): snapshot running prompts from `registry.active_instance_id` (source) instead of target `instance.id`. |
| **Backend** | `src-tauri/src/modules/backup_prompts_db/restore.rs` | Fix Flaw F (line 114): preserve `rec.created_at` instead of destroying timestamps with `now`. |
| **Backend** | `src-tauri/src/modules/instance/post_switch_resume.rs` | Expand `POST_SWITCH_RESUME_TASKS` to store `ScheduledResumeEntry` with `prompts: Vec<PendingPromptPreview>`. Implement query, cancellation, and force-dispatch methods. Emit Tauri events. |
| **Backend** | `src-tauri/src/commands/instance.rs` | Expose IPC commands: `get_pending_post_switch_resumes`, `get_instance_pending_resume`, `cancel_post_switch_resume`, `force_dispatch_pending_resume`. |
| **Backend** | `src-tauri/src/lib/run.rs` | Register the 4 new commands in the Tauri command invoke handler. |
| **Frontend** | `src/services/instanceService/lifecycle.ts` | Add TypeScript wrapper functions for the 4 new Tauri IPC commands. |
| **Frontend** | `src/hooks/usePendingResumes.ts` | Create custom hook listening to `prompts://resume-scheduled`, `prompts://resume-cancelled`, and `prompts://restored` with local 1s countdown tick. |
| **Frontend** | `src/components/instances/PendingResumeModal.tsx` | Create modal to review pending prompts, countdown timer, '⚡ Restore Now' and '✕ Cancel' buttons. |
| **Frontend** | `src/pages/instances/InstanceCard.tsx` | Add amber status pill badge `[RotateCw animate-spin] Restoring N in Xs` to card header with click handler opening modal. |
| **Frontend** | `src/components/instances/table/InstanceRowCells.tsx` | Update `StatusCell` to display the amber pill badge when a resume is pending for the instance row. |

---

## Detailed Step-by-Step Implementation Checklist

### Phase 2.1: Backend Flaw Remediation (Prompt Correctness)

- [ ] **Step 1: Fix Flaw A, B, C in `src-tauri/src/modules/repo_db/discovery.rs`**
  - [ ] Replace the raw `fs::read_to_string(&tf)` reverse line scan with `inspect_conversation_transcript(&base_dir, &cid)`.
  - [ ] Verify `inspection.latest_prompt` is used when available.
  - [ ] If falling back to line inspection, check `is_subagent` and skip `<SYSTEM_MESSAGE>`, `[Message] timestamp=`, and role assignments (`You are ...`).
  - [ ] Sanitize prompt using `extract_clean_user_prompt(&txt)`.
  - [ ] Ensure non-empty user prompt does not fall back to turn-0 `live_prompt_text(&preview, "")`.
  - [ ] Ensure media/images attached to the genuine user turn are extracted properly.

- [ ] **Step 2: Fix Flaw D & Flaw E in `src-tauri/src/modules/repo_db/backup.rs`**
  - [ ] In `backup.rs:488`, restrict `state.vscdb` queries to validated interactive chat input history keys. Strip arbitrary UI panel/telemetry configuration strings.
  - [ ] In `backup.rs:580`, replace `ORDER BY updated_at DESC LIMIT 1` with `ORDER BY created_at ASC, id ASC LIMIT 1` to strictly preserve FIFO queue invariant.

- [ ] **Step 3: Fix Flaw F in `src-tauri/src/modules/backup_prompts_db/restore.rs`**
  - [ ] In `restore.rs:114`, change `created_at: now` to:
    ```rust
    created_at: if rec.created_at > 0 { rec.created_at } else { now },
    ```
  - [ ] Ensure `updated_at` is set to `now` to indicate the restore execution time while maintaining the original creation timestamp.

- [ ] **Step 4: Fix Flaw G in `src-tauri/src/modules/instance/switch.rs`**
  - [ ] In `switch.rs:155`, locate the snapshot step prior to stopping the IDE.
  - [ ] Determine the active source instance id:
    ```rust
    let source_id = if !registry.active_instance_id.is_empty() {
        &registry.active_instance_id
    } else {
        &instance.id
    };
    ```
  - [ ] Execute `backup_running_prompts(source_id)` and `backup_active_running_prompts(Some(source_id), None)`.
  - [ ] Log clearly: `"[INSTANCE_SWITCH:STAGE_1_SNAPSHOT] Backed up {} prompts from active source instance '{}' for target '{}'"`

---

### Phase 2.2: Post-Switch 10-Second Resume Engine Expansion

- [ ] **Step 5: Expand `src-tauri/src/modules/instance/post_switch_resume.rs`**
  - [ ] Define struct `PendingPromptPreview` (`id`, `project_id`, `repo_name`, `repo_path`, `prompt_summary`, `full_prompt`, `created_at`, `has_image`).
  - [ ] Define struct `PendingResumeTaskInfo` (`instance_id`, `scheduled_at_ms`, `fire_at_ms`, `remaining_seconds`, `total_delay_seconds`, `prompt_count`, `prompts: Vec<PendingPromptPreview>`).
  - [ ] Define struct `ScheduledResumeEntry` containing `handle: JoinHandle<()>`, `instance_id`, `scheduled_at_ms`, `fire_at_ms`, `total_delay_seconds`, and `prompts`.
  - [ ] Update `POST_SWITCH_RESUME_TASKS` static map to `Lazy<Mutex<HashMap<String, ScheduledResumeEntry>>>`.
  - [ ] In `schedule_post_switch_resume(instance_id: String)`:
    - Query backed-up prompts awaiting restore for `instance_id`.
    - Generate previews with `extract_smart_prompt_summary(&prompt.prompt_content, 120)`.
    - Compute `scheduled_at_ms` and `fire_at_ms`.
    - Insert into registry and spawn timer.
    - Emit `prompts://resume-scheduled` event with `PendingResumeTaskInfo`.
  - [ ] Implement `get_all_pending_resumes() -> Vec<PendingResumeTaskInfo>`.
  - [ ] Implement `get_instance_pending_resume(instance_id: &str) -> Option<PendingResumeTaskInfo>`.
  - [ ] Implement `cancel_post_switch_resume(instance_id: &str, reason: &str)`:
    - Abort join handle, remove registry entry.
    - Emit `prompts://resume-cancelled` event with `{ "instance_id": instance_id }`.
  - [ ] Implement `force_dispatch_pending_resume(instance_id: &str) -> Result<usize, String>`:
    - Abort timer handle, remove entry.
    - Call `run_resume_with_retry(instance_id)`.
    - Emit `prompts://restored` event with `{ "instance_id": instance_id, "restored_count": restored }`.

- [ ] **Step 6: Expose IPC Commands & Register in Tauri Handler**
  - [ ] In `src-tauri/src/commands/instance.rs`, add:
    - `get_pending_post_switch_resumes`
    - `get_instance_pending_resume`
    - `cancel_post_switch_resume`
    - `force_dispatch_pending_resume`
  - [ ] In `src-tauri/src/lib/run.rs`, add the 4 command symbols to `tauri::generate_handler![...]`.

---

### Phase 2.3: Frontend Implementation & UI Integration

- [ ] **Step 7: Lifecycle Service & Event Hook**
  - [ ] In `src/services/instanceService/lifecycle.ts`, add:
    - `getPendingPostSwitchResumes(): Promise<PendingResumeTaskInfo[]>`
    - `getInstancePendingResume(instanceId: string): Promise<PendingResumeTaskInfo | null>`
    - `cancelPostSwitchResume(instanceId: string): Promise<void>`
    - `forceDispatchPendingResume(instanceId: string): Promise<number>`
  - [ ] In `src/hooks/usePendingResumes.ts`:
    - Listen to `prompts://resume-scheduled`, `prompts://resume-cancelled`, and `prompts://restored`.
    - Maintain state map of pending resumes by `instance_id`.
    - Set up a 1-second interval to decrement remaining countdown seconds until 0.

- [ ] **Step 8: Pending Resume Modal Component**
  - [ ] Create `src/components/instances/PendingResumeModal.tsx`.
  - [ ] Render dark-glass modal container with backdrop blur.
  - [ ] Show title, instance name, countdown badge (`Restoring in Xs`), and visual progress indicator.
  - [ ] Render list of pending prompt cards showing repo name, timestamp, and clean prompt text.
  - [ ] Add `⚡ Restore Now` button triggering `forceDispatchPendingResume`.
  - [ ] Add `✕ Cancel Restore` button triggering `cancelPostSwitchResume`.
  - [ ] Add `Close` button to dismiss modal while keeping timer alive.

- [ ] **Step 9: UI Integration into `InstanceCard.tsx` and `InstanceTable.tsx`**
  - [ ] In `src/pages/instances/InstanceCard.tsx`:
    - Check if active pending resume exists for `inst.config.id`.
    - If active, render amber pill badge:
      `[RotateCw animate-spin] Restoring N in Xs`
    - Clicking the badge opens `PendingResumeModal`.
  - [ ] In `src/components/instances/table/InstanceRowCells.tsx` (`StatusCell`):
    - Check if pending resume exists for row.
    - If active, render the amber pill badge inside the `Status & PID` cell.
    - Clicking opens `PendingResumeModal`.

---

## Verification & Quality Gates

- [ ] **Cargo Format & Clippy**:
  ```bash
  cd src-tauri && cargo fmt -- --check
  cd src-tauri && cargo clippy --all-targets --all-features
  ```
- [ ] **Frontend Build**:
  ```bash
  npm run build
  ```
- [ ] **E2E Behavior Verification**:
  - [ ] Perform an account switch or instance switch.
  - [ ] Verify amber badge appears immediately on the target instance card and table row.
  - [ ] Verify countdown ticks from 10 down to 0 every second.
  - [ ] Click the badge: verify `PendingResumeModal` displays the clean user prompt (not subagent text or `<SYSTEM_MESSAGE>`).
  - [ ] Test `⚡ Restore Now`: verify immediate dispatch into IDE without waiting for the 10s countdown.
  - [ ] Test `✕ Cancel Restore`: verify timer is cleanly aborted and badge disappears.
