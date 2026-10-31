# 238 - Antigravity Manager and Tools UI Issues: Architecture Specification

## 1. Overview & Strategic Intent
This document defines the architectural redesign, pipeline invariants, and synchronization mechanics for resolving high-priority issues across **Antigravity Manager** and **Antigravity Tools**.

Key architecture focus areas:
1. **Immediate CDP Enqueue & Rerun Dispatch**: Eliminating prompt dispatch stalls by routing enqueue and rerun operations through the Chrome DevTools Protocol (`submit_prompt_via_cdp_for_instance_with_mode` with `cancel_running_first: false`) when the IDE is active, ensuring prompt injection into Antigravity IDE immediately without background `agy` headless detachment.
2. **Account Auto-Switching 50x Stability & Prompt Restore**: Replacing fragile background `agy` re-spawning with reliable CDP prompt re-injection and SQLite status settlement across 50 consecutive switches.
3. **5–10s Conversation Turn Privacy Masking**: Masking active session conversation turns and tokens in the UI during the 5–10s settling window post-switch until restored prompt acceptance.
4. **Cluster SSH Auto-Refresh & First-Launch Auto-Sync**: Auto-seeding SSH nodes on initial application startup from known seed configurations (`repo-secrets/gitmap-ssh-nodes.json`, `04-ssh-nodes.json`) and running periodic background TCP liveness refresh loops in the fleet engine.
5. **Instance-Scoped Improvements Flywheel**: Relocating the 100 Improvements Flywheel inside each instance card, defaulting to 20 improvement requests enqueued directly to the IDE using canonical coding guideline prompts (`01-prompts/30-improvements/`, `01-prompts/17-coding-guidelines/`), with user-configurable task counts.
6. **Harmonized Segmented Pill Capsules**: Standardizing Copy and Send/Enqueue/Rerun action capsules to continuous `rounded-full` dark-glass geometry.
7. **Rust / Cargo Toolchain on C: Drive**: Ensuring Rust toolchain is properly installed on the host system at the standard `C:\Users\Administrator\.cargo\bin` location.
8. **CLI UI Manipulation Parity**: Supporting programmatic GUI navigation, opening, clicking, and status inspection via `agm ui` commands.

---

## 2. Core Architecture & Pipeline Invariants

### 2.1 CDP Direct Enqueue Pipeline
- **Problem**: Enqueued prompts previously entered SQLite `active_prompts` with status `queued` or `pending`. The queue scheduler in `src-tauri/src/modules/repo_db/queue.rs` stalled when gate heuristics detected active turns within 300s, and when running, only invoked `spawn_prompt_via_agy`, never sending the prompt text to the open Antigravity IDE GUI.
- **Solution Pipeline**:
  ```
  User clicks Rerun / Enqueue
         │
         ▼
  Tauri Command: enqueue_prompt_for_instance(instance_id, prompt)
         │
         ├───────────────────────────────────────────┐
         ▼                                           ▼
  [1] Insert SQLite active_prompts           [2] Check IDE Liveness
      (status = 'queued', FIFO order)             (find_cdp_endpoint)
                                                     │
                                                     ▼ IDE Running?
                                            ┌────────┴────────┐
                                            │ YES             │ NO
                                            ▼                 ▼
                                    CDP Direct Inject      Background
                                    (cancel_running=false) Queue Poll
                                    types into IDE input
                                    and sends Enter
  ```
- **FIFO Queue Review Invariant**: When an enqueued turn exists in `active_prompts`, the conversation tree generator flags `conv.is_queued = true` and `turn.status = QUEUED`. The UI Detail Header displays `{enqueuedCount} prompt(s) enqueued in FIFO queue for this conversation.` This truthfully communicates pending turns awaiting sequential consumption.

### 2.2 50x Account Auto-Switch & Prompt Restoration
- **Root Cause of Restore Failure**: During auto-switching, `push_backed_up_prompts_after_channel` dispatched prompts via headless `agy` CLI rather than ensuring the newly authenticated instance window was ready to receive the prompt via CDP.
- **Stabilization Invariant**:
  - Pre-switch: Record running turn snapshot to `prompt_backups` with `account_id`, `instance_id`, `channel`, and `created_at`.
  - Account Switch: Update credentials, restart instance with clean profile token.
  - Settle Window (5–10s): Trigger UI masking state (`isSettling: true`).
  - Post-switch CDP Ingestion: Await IDE CDP readiness, then re-inject the backed-up prompt directly into the IDE input field.
  - Mark backup restored and emit queue dispatch notification.

### 2.3 Post-Switch Conversation Privacy Masking
- **UI State Contract**:
  - When `isSettling` or `resumeCountdown > 0` (5–10s timer), all conversation turn messages and prompt bubbles render a privacy shield with animated shimmer / blur:
    ```tsx
    {isMasked ? (
      <div className="h-16 rounded-xl bg-surface-2/60 animate-pulse flex items-center justify-center gap-2 border border-border/40">
        <Lock className="w-4 h-4 text-accent-cyan" />
        <span className="text-xs text-text-secondary">Securing session tokens & restoring prompt in {secondsLeft}s...</span>
      </div>
    ) : (
      <TurnContent text={turn.content} />
    )}
    ```

### 2.4 Cluster SSH Auto-Refresh & First-Launch Sync
- **First Open Sync**: `src-tauri/src/lib/setup_app.rs` calls `auto_sync_cluster_on_startup()`.
- **Seed Discovery**: Checks `candidate_seed_node_paths()`:
  - `repo-secrets/gitmap-ssh-nodes.json`
  - `04-ssh-nodes.json`
- **Periodic Liveness**: Background tokio task polls node connectivity every 30s, updating latency and instance counts in SQLite `ssh_nodes.db` and publishing Tauri event `fleet://status_updated`.

### 2.5 Instance-Scoped Improvements Flywheel
- **Architecture**:
  - Moved from global `src/pages/Instances.tsx` to `src/pages/instances/InstanceCard.tsx`.
  - Scoped to `instance.id` and `instance.workspace_path`.
  - Batch configuration: 20 improvements default, with dropdown selector (`[5, 10, 20, 50]`).
  - Prompts sourced from `01-prompts/30-improvements/` and `01-prompts/17-coding-guidelines/`.
  - Enqueue action dispatches sequentially via the CDP Enqueue pipeline into the instance's active IDE.
