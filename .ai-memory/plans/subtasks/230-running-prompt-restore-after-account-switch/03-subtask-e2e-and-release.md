---
plan: 230-running-prompt-restore-after-account-switch
subtask: "03"
title: Comprehensive E2E Testing, Frontend Settling Countdown Visibility, and Minor Bump Release Ceremony
domain: fullstack-rust-typescript
depends_on: "01,02"
citations:
  architecture_spec: ../../../../02-spec/21-app/230-running-prompt-restore-after-account-switch/01-rca-and-architecture-spec.md
  component_spec: ../../../../02-spec/21-app/230-running-prompt-restore-after-account-switch/02-component-and-queue-spec.md
  e2e_and_release_spec: ../../../../02-spec/21-app/230-running-prompt-restore-after-account-switch/03-e2e-and-release-spec.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src-tauri/src/modules/backup_prompts_db/tests.rs
  - src-tauri/src/modules/instance/tests.rs
  - src-tauri/src/modules/repo_db/queue.rs
  - src-tauri/src/modules/repo_db/backup.rs
  - src/pages/instances/useInstancePage.ts
  - src/pages/instances/InstanceCard.tsx
  - src/pages/instances/InstancesStates.tsx
  - CHANGELOG.md
  - CHANGELOG_EN.md
  - README.md
  - README_EN.md
status: pending
---

# Subtask 03: Comprehensive E2E Testing, Frontend Settling Countdown Visibility, and Minor Bump Release Ceremony

## 1. Context & Motivation

Following the implementation of Subtask 01 (RCA & Architecture: deep transcript extraction, process termination, settling delay, and queue re-injection) and Subtask 02 (Component & Queue: FIFO query invariants, SQLite repository pattern, and prompt status alignment), this subtask delivers the end-to-end verification, operator UI countdown visibility, and the full minor release ceremony.

Without comprehensive automated test coverage for the five core failure modes (unrestored prompt discovery, status discoverability, FIFO ordering, default NULL/empty scope matching, and settling timer wakeup), regressions could easily re-introduce prompt loss during account switching. Furthermore, exposing the 10-second settling window directly in the UI eliminates operator uncertainty, providing a clear countdown and force-dispatch option. Finally, running the strict pre-flight gates and executing the atomic version bump to `v4.271.0` completes the task lifecycle cleanly.

---

## 2. Target Files & Symbols Breakdown

| Target File | Operations & Scope |
| :--- | :--- |
| `src-tauri/src/modules/backup_prompts_db/tests.rs` | Implement Test 1 (`test_has_unrestored_backups_for_target_instance`), Test 2 (`test_prompt_status_after_restore_discoverability`), and Test 4 (`test_count_backed_up_prompts_default_includes_null_and_empty`). |
| `src-tauri/src/modules/instance/tests.rs` | Implement Test 3 (`test_strict_fifo_queue_ordering`) and Test 5 (`test_settling_timer_triggers_prompt_queue_notification`). |
| `src/pages/instances/useInstancePage.ts` | Wire `prompts://resume-scheduled` and `prompts://resume-cancelled` events alongside `prompts://restored` to maintain a reactive 10s settling countdown state (`settlingResumeState`). |
| `src/pages/instances/InstanceCard.tsx` | Render an animated countdown badge (`⚡ Restoring in Xs (N prompts)`) on the active instance card during the settling window. |
| `src/pages/instances/InstancesStates.tsx` | Render an interactive settling banner in `InstancesAlerts` allowing the operator to force dispatch restored prompts immediately. |
| `CHANGELOG.md` & `CHANGELOG_EN.md` | Author the `[4.271.0]` release section with exclusive attribution to `@aukgit`. |
| `README.md` & `README_EN.md` | Update the `## 📝 Changelog` section to mirror `v4.271.0` release notes. |

---

## 3. Concrete Implementation Steps

### 3.1 Step 1: Rust Automated Test Suite Implementation

1. Open `src-tauri/src/modules/backup_prompts_db/tests.rs` and add:
   - **Test 1**: `test_has_unrestored_backups_for_target_instance`: Seed two prompt backups under separate instance IDs, verify `count_unrestored_prompts_for_instance_with_file` returns 1 for target, 0 for unmatched, and 0 after marking restored.
   - **Test 2**: `test_prompt_status_after_restore_discoverability`: Iterate through the status matrix (`running`, `in_progress`, `queued`, `dispatched`, `backed_up`), validating that `prompt_status_after_restore` returns discoverable statuses (`'in_progress'`, `'queued'`, `'pending'`).
   - **Test 4**: `test_count_backed_up_prompts_default_includes_null_and_empty`: Verify that `count_backed_up_prompts("default")` counts rows with `instance_id` = `'default'`, `'__default__'`, `''`, and `NULL`.
2. Open `src-tauri/src/modules/instance/tests.rs` (or companion module) and add:
   - **Test 3**: `test_strict_fifo_queue_ordering`: Insert prompts with staggered `created_at` timestamps (100, 200, 300) and inverted `updated_at` timestamps (999, 500, 300). Confirm that `ORDER BY created_at ASC, id ASC` selects the oldest created prompt.
   - **Test 5**: `test_settling_timer_triggers_prompt_queue_notification`: Listen on `PROMPT_QUEUE_NOTIFY.notified()` and assert that `notify_prompt_queue()` triggers the watcher within 500ms.

### 3.2 Step 2: Frontend UI Settling Countdown Visibility

1. In `src/pages/instances/useInstancePage.ts`:
   - Define state: `const [settlingResume, setSettlingResume] = useState<SettlingResumeState | null>(null);`.
   - In the Tauri event listener registration block:
     ```typescript
     listen<ScheduledResumeEntry>('prompts://resume-scheduled', (event) => {
         setSettlingResume({
             instanceId: event.payload.instance_id,
             promptCount: event.payload.prompt_count,
             totalDelaySeconds: event.payload.total_delay_seconds,
             remainingSeconds: event.payload.remaining_seconds,
             scheduledAt: event.payload.scheduled_at_ms,
             fireAt: event.payload.fire_at_ms,
         });
     });
     listen('prompts://resume-cancelled', () => setSettlingResume(null));
     listen<{ instance_id: string; restored_count: number }>('prompts://restored', (event) => {
         setSettlingResume(null);
         setResumeIndicator({
             instanceId: event.payload.instance_id,
             restoredCount: event.payload.restored_count,
             at: Date.now(),
         });
         window.setTimeout(() => setResumeIndicator(null), 6000);
     });
     ```
   - Add a 1-second interval ticker decrementing `settlingResume.remainingSeconds`.
   - Expose `settlingResume` on `InstancePageApi`.
2. In `src/pages/instances/InstanceCard.tsx`:
   - Inspect `api.settlingResume`. If `settlingResume?.instanceId === inst.config.id`, render an animated countdown pill:
     ```tsx
     <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-bold bg-amber-500/15 text-amber-700 dark:text-amber-300 border border-amber-500/30 animate-pulse">
         ⚡ Restoring {settlingResume.promptCount} prompt(s) in {settlingResume.remainingSeconds}s
     </span>
     ```
3. In `src/pages/instances/InstancesStates.tsx` (`InstancesAlerts`):
   - When `api.settlingResume` is active, display an interactive alert banner with a `Force Dispatch` button calling `invoke('force_dispatch_post_switch_resume', { instanceId: settlingResume.instanceId })`.

### 3.3 Step 3: Quality Gates Verification

Execute the standard local pre-flight checks:
1. `cd src-tauri && cargo fmt -- --check`
2. `cd src-tauri && cargo clippy --all-targets --all-features`
3. `npm run build`

### 3.4 Step 4: Atomic Minor Bump & Manifest Synchronization

Execute the atomic bump script:
```bash
npm run bump minor
```
Verify that the version advances from `4.270.0` to `4.271.0` in:
- `package.json`
- `package-lock.json`
- `version.json`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/tauri.conf.json`
- `Casks/antigravity-tools.rb`

### 3.5 Step 5: Changelog & Documentation Synchronization

1. In `CHANGELOG.md` and `CHANGELOG_EN.md`, add:
   ```markdown
   ## [4.271.0] - 2026-10-27

   ### Fixed
   - **Prompt Restore**: Resolved running prompt restore failures across manual and automated account switching. (Thanks to @aukgit)
   - **FIFO Restore Invariant**: Enforced strict First-In-First-Out prompt restoration ordering (`ORDER BY created_at ASC, id ASC`). (Thanks to @aukgit)
   - **Settling Delay & Wakeup**: Hardened the 10-second post-switch settling timer to reactively trigger prompt queue dispatch via `notify_prompt_queue()`. (Thanks to @aukgit)
   - **Default Scope Invariant**: Fixed `count_backed_up_prompts("default")` to properly match empty string and NULL `instance_id` rows. (Thanks to @aukgit)
   - **Frontend Observability**: Added real-time countdown badge and settling progress visibility in the instances dashboard. (Thanks to @aukgit)
   ```
   *Strict Attribution Check: Confirm no user handles other than `@aukgit` are present.*
2. In `README.md` and `README_EN.md`, synchronize the `## 📝 Changelog` section with the new release highlights.

### 3.6 Step 6: GitMap Release Commit & Push

Stage and release atomically via GitMap:
```bash
gitmap cpr
```
Commit message:
`RELEASE(RELEASE): v4.271.0 - running prompt restore after account switch, FIFO queue invariant, 10s settling timer wakeup, and UI progress visibility`

---

## 4. Acceptance Criteria Checklist

- [ ] All 5 Rust unit/integration tests pass with green results.
- [ ] During the 10-second post-switch settling window, `settlingResume` renders on both `InstanceCard` and `InstancesAlerts`.
- [ ] When the 10-second settling delay completes, `notify_prompt_queue()` immediately wakes the dispatch loop.
- [ ] `cargo fmt -- --check` passes cleanly.
- [ ] `cargo clippy --all-targets --all-features` passes with 0 warnings.
- [ ] `npm run build` generates production frontend assets without errors.
- [ ] Version is bumped to `v4.271.0` across all manifest files.
- [ ] `CHANGELOG.md` and `CHANGELOG_EN.md` attribute solely `@aukgit`.
- [ ] `README.md` and `README_EN.md` changelog sections are fully synchronized.
- [ ] Zero absolute paths exist in code, specifications, or commit logs.
