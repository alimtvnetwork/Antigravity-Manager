# 01 — Architecture Spec: Antigravity-Manager Package Repository Update & Prompt Functionality Testing

- Spec: `02-spec/21-app/154-antigravity-manager-package-repository-update-and-prompt-functionality-testing/`
- Parent plan: `.ai-memory/plans/154-antigravity-manager-package-repository-update-and-prompt-functionality-testing.md`
- Repo: `alimtvnetwork/Antigravity-Manager` (Tauri desktop app, "Antigravity Manager Tools")
- Baseline: `main` @ `c7efb910` (pulled 2026-10-19, tree clean)
- Author: Spec Author 1 (backend + panel contracts). Companion spec files are owned by Spec Author 2 — do not edit them.

## 1. User Request (Verbatim)

```text
# `Antigravity Manager or Antigravity Tools` Package Repository Update and Prompt Functionality Testing: high priority instruction, non-negotiable task

Make sure you get the latest of the `Antigravity Manager or Antigravity Tools` package repository, and also make sure the prompts backup and restore with image works perfectly. You can do end-to-end testing there. And also at the same time, if the prompts are running and we are in the prompt tree view, it should show me the first prompt which is running right now, and next four prompts that's four or whatever the prompts is in the queue. It should show me that, and then I could rerun, send the same prompt using image as well. So all the test needs to be sent with putting the ``Id`` with some screenshot and image, make sure of that. Is it clear?

## task: `Antigravity Manager or Antigravity Tools` Package Repository Update and Prompt Functionality Testing
## slug: `Antigravity Manager or Antigravity Tools`-package-repository-update-and-prompt-functionality-testing

# Actionable Items Must Follow Non-Negotiable

1. Git pull BEFORE starting any work — first try `gitmap pull <repo path>` (or run `gitmap pull` in repository folder); if that fails, fall back to regular `git pull` and resolve any merge conflicts so the repository is completely up to date and clean before starting.
2. Write spec under 02-spec/21-app/`Antigravity Manager or Antigravity Tools`-package-repository-update-and-prompt-functionality-testing/ and enqueue plan task in .ai-memory/plans/`Antigravity Manager or Antigravity Tools`-package-repository-update-and-prompt-functionality-testing.md (subtasks in .ai-memory/plans/subtasks/`Antigravity Manager or Antigravity Tools`-package-repository-update-and-prompt-functionality-testing/) first
3. Search codebase exclusively via GitMap (gitmap aum search, gitmap find, gitmap cat, gitmap ps, gitmap py, gitmap llm train); TOTAL BAN on rg, ripgrep, grep, git grep, Select-String
4. Strictly use relative Git paths (02-spec/..., .ai-memory/..., cmd/...); only add the relative paths, never add the absolute path during your work, and ensure this is respected on the release page and in release notes as well
5. Ensure the latest version of the `Antigravity Manager or Antigravity Tools` package repository is obtained.
6. Verify that prompts backup and restore with image functionality works perfectly.
7. Conduct end-to-end testing of the prompt functionality.
8. In the prompt tree view, ensure it displays the first running prompt and the next four prompts in the queue.
9. Implement functionality to rerun and send the same prompt using an image.
10. Include ``Id`` with screenshots and images for all tests.

## Must follow and spawn agent using

[execute-parent-task-with-n-steps-v6](file;.agents/skills/execute-parent-task-with-n-steps-v6)

## Additional Instructions

- [/plan](slashCommand;plan) first before doing the work to reduce the credits.
- [/learn](slashCommand;learn) from [gitmap](file;.agents/skills/gitmap) skill to leverage GitMap high-speed search, toolchain discovery, and caching.
- Only add the relative paths, never add the absolute path during your work; this should be respected on the release page and in release notes as well.
```

## 2. Discrete Deliverables (Task-01 … Task-07)

| Task-ID | Title | Scope | Subtask file |
|---|---|---|---|
| Task-01 | Backend queue / backup / restore verification & fixes | FIFO dispatch, backup & restore with images, multi-image extraction fix, `session_id` preservation on restore, no double-dispatch on resend path | `01-backend-queue-backup-restore.md` (this spec) |
| Task-02 | "Now Running + Up Next" panel data contract & UI | IPC/API returning 1 running + up to 4 queued (FIFO), liveness TTL, refresh cadence, empty/overflow states, prompt tree view integration | `02-panel-data-contracts.md` (this spec) |
| Task-03 | Rerun / re-send same prompt with image | Attachment lifecycle (`.antigravity/attachments/` GC/retention), original-vs-rewritten source at rerun, broken-link failure handling | companion subtask (Spec Author 2) |
| Task-04 | E2E catalog with prompt-Id + screenshot/image evidence | Transient sandbox per test (`agm instances test-create` / `test-clean`), every test row carries prompt `Id` + screenshot | companion subtask (Spec Author 2) |
| Task-05 | Image round-trip byte-identical verification | SHA256(original) == SHA256(restored) for every image; `has_images` preserved; missing image = loud error, never silent skip | `01-backend-queue-backup-restore.md` (this spec) |
| Task-06 | Last-100-commits review + spec-folder ambiguity sweep | Improvement proposals filed as spec amendments, not drive-by edits | companion subtask (Spec Author 2) |
| Task-07 | CI green + atomic commit/push | Pre-flight gates, one `gitmap cpf`/`cpb` per task, push | lead |

## 3. Scope Definitions (ambiguity resolutions)

- **A5 — "package repository update".** Resolved: it means `git pull` latest, which is DONE (`main` @ `c7efb910`, clean). Dependency updates (npm/cargo) are OUT of scope unless CI is red — then the minimal bump needed to go green is in scope.
- **A9 — restore paths covered by e2e.** The e2e MUST cover exactly these restore paths: (a) `agm` CLI restore (`agm restore-running-prompts` / `rrp`); (b) UI restore (restore button → IPC → same Rust fn, CLI/GUI parity); (c) FIFO resume via `.antigravity_resume_task.json`; (d) per-instance restore (`instance_id` scoping — restore for instance X must not touch instance Y's rows).
- **A7 — relationship to plan 140.** Plan `140-per-instance-prompt-attribution-backup-queue` owns the per-instance attribution ground. This task REUSES its E2E catalog pattern (`.ai-memory/e2e-testing/`) and does NOT re-verify 140- or 153-covered ground. New work here: image round-trip byte-identity, the "Up Next" panel contract, rerun-with-image lifecycle.

## 4. Backend Architecture (verified read-only, 2026-10-19)

### 4.1 Modules

| Module | Key symbols | Role |
|---|---|---|
| `src-tauri/src/modules/backup_prompts_db/` (`mod.rs`, `types.rs`, `backup.rs`, `restore.rs`, `db.rs`, `listing.rs`, `green_projects.rs`, `tests.rs`) | `PromptBackupRecord{has_images, images_payload}`, `backup_queued_prompts_for_instance`, `restore_running_prompts_for_instance` (maps `images_payload` → `ActivePrompt.image_payload`) | Durable backup/restore of prompts |
| `src-tauri/src/modules/repo_db/queue.rs` | `check_and_dispatch_enqueued_prompts` — FIFO `ORDER BY created_at ASC, id ASC` | Queue dispatch |
| `src-tauri/src/modules/repo_db/agy.rs` | `enqueue_prompt_for_instance`, `spawn_prompt_via_agy` (dispatch choke point), `drain_next_queued_prompt_for_workspace` (on worker exit) | Enqueue + spawn |
| `src-tauri/src/modules/repo_db/schema.rs` | `active_prompts` statuses: `running` / `backed_up` / `queued` / `pending` / `dispatched` / `completed` / `failed`; `image_payload` column | Schema |
| `src-tauri/src/modules/repo_db/detection.rs` | `extract_image_payload_or_path` (~:250 — captures only the FIRST `data:image/` URI; file paths all collected), `resume_task_document` (:330 builds `.antigravity_resume_task.json`) | Image extraction, resume doc |
| `src-tauri/src/modules/repo_db/backup.rs` | `resend_queued_prompts_for_instance` (:84) — documents that `restore_running_prompts_for_instance` must NOT run on the resend path (double-dispatch) | Resend pipeline |
| `src-tauri/src/commands/cmd_queue_resend.rs` | `resend_queued_prompts` (GUI twin of `agm prompts resend-queued`), `backup_queued_prompts` (GUI twin of `agm prompts backup-queued`) | IPC |
| CLI (`src-tauri/src/bin/agm.rs`) | `agm prompts resend-queued`, `agm prompts backup-queued`, `agm rerun prompts <N>` (reruns last N in ascending order), `agm prompt enqueue` (`--instance`), `agm instances test-create` / `test-clean`, `agm which-prompts-running` (`wpr`), `agm backup-running-prompts` (`brp`), `agm restore-running-prompts` (`rrp`) | CLI surface |

### 4.2 Key callers

- UI prompt tree: `src/components/instances/PromptTreeView/usePromptTreeCoreActions.ts` (documents itself as GUI twin of `agm prompts resend-queued`).
- Dispatch choke point: everything that sends a prompt funnels through `spawn_prompt_via_agy`.
- Worker exit triggers `drain_next_queued_prompt_for_workspace` → next FIFO prompt dispatches.

## 5. Data Flow

### 5.1 Prompt lifecycle

```text
running → backed_up → queued → dispatched → running → completed / failed
```

- Enqueue: `enqueue_prompt_for_instance` inserts with status `queued`.
- Dispatch: `check_and_dispatch_enqueued_prompts` picks the FIFO head (`ORDER BY created_at ASC, id ASC`), `spawn_prompt_via_agy` sends it (status → `dispatched` → `running`).
- Worker exit: `drain_next_queued_prompt_for_workspace` dispatches the next queued prompt.
- Backup: `backup_queued_prompts_for_instance` snapshots queued rows into `prompt_backups` (preserving FIFO order).
- Restore: `restore_running_prompts_for_instance` moves backup rows back to `active_prompts` with `images_payload` → `image_payload`. It must NEVER run on the resend path (double-dispatch — already documented in `repo_db/backup.rs`).
- Resend: `resend_queued_prompts_for_instance` takes a durable FIFO snapshot, re-injects unrestored queued backup records, runs a SINGLE dispatch pass; already-dispatched prompts are skipped (RCA-15 dedup); returns real counts.

### 5.2 Image side-channel

Images ride OUT-OF-BAND via `.antigravity_resume_task.json` (built by `resume_task_document`). The `agy -p` argument and CDP submit are TEXT-ONLY — so any image must be re-attached from the resume doc / `image_payload` at dispatch time. If the side-channel is broken, the prompt sends text-only: this must be a loud error, never a silent downgrade.

## 6. "Now Running + Up Next" Data Contract (A1/A10)

### 6.1 Contract

New IPC command (name proposed: `get_running_and_up_next`; subtask 01 implements, subtask 02 consumes):

```text
get_running_and_up_next(instance_id: string | null) -> {
  now_running: PromptRow | null,   // status = 'running', FIFO earliest for the instance
  up_next: PromptRow[],             // up to 4, status IN ('queued','pending','dispatched'), FIFO
  up_next_total: number,            // FULL queued count ("four or whatever the queue holds")
  instance_id: string,
  as_of_ms: number                 // backend timestamp; UI uses it for staleness display
}
```

`PromptRow` carries at minimum: `id` (the prompt `Id` the user asked to see on every test), `instance_id`, `status`, `origin_tier` (4-tier: `USER_PROMPT` / `SUBAGENT_INSTRUCTION` / `SYSTEM_MESSAGE` / `TOOL_OUTPUT`), `created_at`, `has_images`.

### 6.2 Source queries (all FIFO)

```sql
-- now_running
SELECT ... FROM active_prompts
WHERE instance_id = ?1 AND status = 'running'
ORDER BY created_at ASC, id ASC LIMIT 1;

-- up_next (limit 5 rows total on screen = 1 running + 4 queued)
SELECT ... FROM active_prompts
WHERE instance_id = ?1 AND status IN ('queued','pending','dispatched')
ORDER BY created_at ASC, id ASC LIMIT 4;

-- up_next_total
SELECT COUNT(*) FROM active_prompts
WHERE instance_id = ?1 AND status IN ('queued','pending','dispatched');
```

`dispatched` is included in "up next": it has left the queue but is not yet observed running — hiding it would make prompts vanish between dispatch and first heartbeat.

### 6.3 Liveness signal + TTL (no flicker)

- A row counts as "running right now" iff its worker is alive (PID check) OR its last heartbeat is within TTL.
- TTL proposed: 30s. A prompt transitions `running` → stale ONLY after TTL expiry — never on a single missed poll (no flicker).
- Stale rows are surfaced with a "stale" badge, not silently dropped (no silent errors).

### 6.4 Refresh cadence, count semantics, empty/overflow

- Refresh: UI polls the IPC every 3s while the prompt tree view is mounted (cheap indexed queries); backend pushes Tauri events on dispatch/completion for immediacy.
- Count semantics: show `min(4, up_next_total)` rows; always show the total ("Up next (3)" / "Up next (4 of 11)").
- Empty: `now_running = null` → "No prompt running"; `up_next = []` → "Queue empty".
- Overflow: `up_next_total > 4` → "+N more in queue" affordance.
- Every row shows its prompt `Id` and 4-tier origin badge (Sky/Purple/Zinc/Amber per AGENTS.md).

## 7. Backup → Restore Image Round-Trip Contract (A2)

### 7.1 How `images_payload` is populated at backup time

`extract_image_payload_or_path(prompt_content)` returns `(Option<data_uri>, Vec<path>)`. Contract:

- **Data URIs are stored verbatim** (byte-identical, no re-encoding, no truncation).
- **File paths are copied** into `.antigravity/attachments/` under content-hash filenames (`sha256(ext)`) at backup time, so later moves/deletes of the original file cannot break restore.
- `has_images = true` iff at least one image (URI or path) was captured.

### 7.2 Known risk — fix candidate (Task-01)

`extract_image_payload_or_path` captures only the FIRST `data:image/` URI (verified `src-tauri/src/modules/repo_db/detection.rs` ~:250: `content.find("data:image/")` takes the first match). Prompts with 2+ inline images silently lose images 2..n at backup. Fix: collect ALL data URIs into `Vec<String>`. This is a Task-01 fix, not a new feature.

### 7.3 Acceptance criterion — byte-identical restoration

For every image in the prompt: `SHA256(original bytes) == SHA256(restored bytes)` after backup → restore → re-dispatch. `has_images` flag preserved. Any image missing at restore aborts with `AppError` naming the prompt `Id` — never a silent skip.

## 8. Rerun / Re-send With Image Lifecycle (A3)

- **Rerun source:** the ORIGINAL prompt content + ORIGINAL image payloads (from the live row or its backup record). Never the rewritten relative links.
- **`.antigravity/attachments/` GC/retention:** an attachment file may be deleted ONLY if (a) no row in `active_prompts` or `prompt_backups` references it AND (b) it is older than the retention TTL (proposed: 7 days). Sandbox attachments are exempt from the TTL and are removed by `agm instances test-clean`.
- **Broken-link failure handling:** if an image file is missing at rerun time, abort the rerun with an `AppError` naming the prompt `Id` and the missing path. Text-only fallback is FORBIDDEN (would violate no-silent-errors and the user's "with image" requirement).

## 9. Invariants Re-affirmed (by reference, not restated)

- FIFO invariant: every restore/dispatch/queue query uses `ORDER BY created_at ASC, id ASC`; LIFO (`ORDER BY updated_at DESC`) forbidden in dispatch logic — `AGENTS.md`, `.agents/rules/prompt-tree-and-restore-invariants.md`.
- 4-tier prompt origin classification in tree UI (Sky/Purple/Zinc/Amber, `↳`/`└──` nesting) — `AGENTS.md`.
- No silent errors: Rust → `AppError` + `tracing`; TS → error store (`captureError`/`trackWarning`) — `AGENTS.md`.
- Relative git paths only, lowercase filenames — `AGENTS.md`, R11.
- Pre-flight: `cargo fmt --check`, `cargo clippy --all-targets --all-features`, `npm run build` — `AGENTS.md`.
- Tests use transient sandboxes (`agm instances test-create` / `test-clean`); caller IDE self-preservation (`detect_caller_instance_id`) — `AGENTS.md`.
- Release channels: `main` = stable, `beta` = preview; never cross — `AGENTS.md`.
- Backend fix strategy: pipeline-first, adapters last resort — `AGENTS.md`.

## 10. Open Ambiguities Log

| ID | Ambiguity | Status |
|---|---|---|
| A6 | Image-resend status contradiction (learned notes) | LOGGED — do not fix in this task; file as follow-up |
| A8 | Duplicate spec files in `02-spec/` | LOGGED — do not fix in this task; file as follow-up |
| A11 (new) | Liveness TTL value (proposed 30s) | Proposed; confirm during Task-02 review |
| A12 (new) | Attachment GC TTL (proposed 7 days) | Proposed; confirm during Task-03 review |
| A13 (new) | Whether `pending` rows belong in "Up Next" | Proposed YES (spec §6.2); confirm during Task-02 review |
