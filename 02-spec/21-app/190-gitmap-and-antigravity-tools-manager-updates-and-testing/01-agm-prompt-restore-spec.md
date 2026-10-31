# 01 — AGM Prompt Backup / Auto-Switch / Restore / Rerun Spec

Slug: 190-gitmap-and-antigravity-tools-manager-updates-and-testing
Scope: Antigravity-Manager (AGM). Companion specs: 02 (gitmap CG), 03 (E2E + release).

## Problem statement

Account switching and prompt backup work; auto-switch on quota works; **prompt
restore does not**. Root causes established by code inspection at HEAD b4a9b9d3
(static analysis; live E2E confirmation is spec 03's job).

## Flow (as-is)

- Statuses in `active_prompts`: queued, pending, running, in_flight, backed_up,
  dispatched, completed. Completion is recorded only by the agy monitor thread
  on agy exit 0 (`src-tauri/src/modules/repo_db/agy.rs` ~L382-389). Non-zero
  exit writes nothing.
- Backup: `src-tauri/src/modules/repo_db/backup.rs` `backup_running_prompts`
  (SQL transitions + live discovery) and
  `src-tauri/src/modules/backup_prompts_db/backup.rs` (durable split-DB
  snapshots into `prompt_backups`).
- Auto-switch: `src-tauri/src/modules/auto_switcher/quota.rs` (quota-% trigger),
  `src-tauri/src/modules/auto_switcher/rotation.rs` (cross-instance path).
- Restore entry points (three, divergent): `backup_prompts_db/restore.rs`,
  `repo_db/backup.rs` `resend_queued_prompts_for_instance`,
  `repo_db/restore.rs` (JSON import, dispatches nothing).
- Post-switch trigger: `src-tauri/src/modules/instance/switch.rs` Stage 5 →
  `instance/workspace.rs` `push_backed_up_prompts_after_channel`, timed by the
  in-memory scheduler `instance/post_switch_resume.rs` (10s delay, one 7s retry).
- Dispatch: `repo_db/resend.rs` (FIFO, ≤1 prompt per repo_path per pass),
  `repo_db/dispatch.rs`, queue drain only from the agy monitor on process exit.

## Root causes (ranked) and required fixes

- **RC-1 (primary): 120s in-flight window.** `backup_running_prompts` moves
  `dispatched` rows to `backed_up` only if dispatched within 120s
  (`repo_db/backup.rs:439,447-465`). Long-running prompts — the normal case —
  fall out of every restore query and vanish after auto-switch.
  FIX: post-switch **incomplete-prompt sweep** — transition ALL
  `running`/`in_flight`/`dispatched` rows not marked `completed` (any age) to
  `backed_up` on the switch path, and re-run the sweep for the target instance
  in `push_backed_up_prompts_after_channel` before dispatch.
- **RC-2: manual switch orphans durable backups.** `switch.rs` Stage 1
  re-attributes only `active_prompts`; `prompt_backups` rows keep the source
  instance while restore is scoped to the target. Rotation path already has
  the fix (~L290) — port it to `switch.rs`; audit `launch.rs`/`spawn.rs`
  scoped restores to fall back to unscoped semantics when scoped restore
  returns 0 but unrestored records exist.
- **RC-3: fake success.** `backup_prompts_db/restore.rs` swallows dispatch in
  `record_ignored` and returns Ok regardless; `switch.rs:770` audit writes
  `restored_prompts_count = backed_up_count` before restore runs; rotation
  notification passes `backed_up_count` as both backed-up and restored counts.
  FIX: propagate real dispatched/failed counts end to end.
- **RC-4: stalled queues.** ≤1 prompt per repo per pass + drain only on agy
  exit. FIX: after a restore pass, schedule drain retries via the post-switch
  timer until no progress, or loop resend passes while progress is made.
- **RC-5: backup durability.** `enforce_max_backup_prompts_cap` deletes rows
  beyond the newest 100 including unrestored ones; backup candidates require
  `updated_at` within 2h. FIX: never delete `is_restored=0` rows in the cap;
  drop the 2h freshness filter for `queued` rows.
- **RC-6: re-injection clobber.** `save_or_requeue_prompt` uses
  `INSERT OR REPLACE` with no live-status dedup on the backup-DB restore path.
  FIX: apply the same dedup as `resend_queued_prompts_for_instance`; never
  overwrite a row whose live status is `running`/`completed`.
- **RC-7: memory-only resume.** Persist the scheduled post-switch resume
  (config/DB) in `post_switch_resume.rs` and re-arm on app start.
- **RC-8: split-DB path ambiguity.** `backup_prompts_db/db.rs` prefers a
  CWD-relative `data/backup-prompts/SQL.db`; GUI and CLI can open different
  databases. FIX: single app-data-dir path for both.

## Rerun auto-pick (user requirement)

- Options exist: `is-it-done` | `is-it-published` | `randomized`
  (`src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`).
  Today: picker initializes to `is-it-done` on every mount (DetailHeader.tsx
  ~L244), count defaults to 2 (clamped 1..5), random set has only 5 messages,
  and the context-menu rerun hardcodes `is-it-done`
  (ConversationContextMenu.tsx:459).
- REQUIRED: default option = `randomized`; persist picker + count; route the
  context-menu rerun through the same persisted option; enlarge
  `RERUN_RANDOM_MESSAGE_SET`; raise/auto-derive the default count so "a bunch"
  of nudges are selected and executed automatically.

## Invariants

- FIFO everywhere in dispatch/restore: `ORDER BY created_at ASC, id ASC`.
- GUI/CLI parity for every lifecycle operation touched.
- No silent errors: failures flow into AppError / error store; a failed
  dispatch must never be recorded or reported as dispatched.
- No builds/tests during implementation (user instruction); verification is
  spec 03 (end phase) + CI.

## Acceptance

- A prompt dispatched >120s before a quota auto-switch is restored and
  re-dispatched on the target instance (proven in E2E, spec 03).
- Manual GUI switch restores the same prompt set as auto-rotation.
- Restore reporting shows real dispatched/failed counts.
- Rerun fires with the randomized option by default, persisted across mounts.
