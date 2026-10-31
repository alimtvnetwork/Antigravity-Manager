# Specification: GitMap and Antigravity Tools Manager Updates and Testing
Sequence: 216
Slug: `216-gitmap-and-antigravity-tools-manager-updates-and-testing`

## 1. Executive Overview
This specification hardens the end-to-end reliability of account switching, prompt continuity, and rerun workflows in **Antigravity-Manager**, establishes fleet-wide synchronization with **GitMap**, audits code quality against the Coding Guidelines (`01-prompts/15-cg-execute/`), and details exhaustive deferred end-to-end test cases.

---

## 2. Core Defect & Problem Statement: Prompt Loss on Quota Finish
When an account depletes its quota (credit finished / rate limit 429 / 403), the auto-switcher rotates accounts. In legacy execution:
1. Prompts in `pending`, `queued`, or `blocked` states were not reliably transitioned into `backed_up` status during the rapid teardown.
2. In SQLite, prompts left with `status = 'running'` from terminated IDE sessions were incorrectly treated as "live" by the restore module in the new account session (`is_live_status_preserved`), causing them to be skipped (`outcome.skipped_live += 1`).
3. After the switch, prompts that failed or were abandoned when quota ran out were not automatically surfaced for rehydration and rerun.
4. In the UI, clicking "Rerun" required manual message selection instead of automatically picking a random completion/release check nudge.

---

## 3. Architecture & Functional Requirements

### 3.1 Prompt Recovery on Quota Exhaustion
- **Incomplete Prompt Sweep**:
  All non-terminal states (`running`, `in_flight`, `dispatched`, `pending`, `queued`, `blocked`) must be captured by `sweep_incomplete_prompts_for_instance` before any process termination.
- **Unambiguous Restore Invariant**:
  Only prompts with `status = 'completed'` are skipped during restore. Any prompt previously in `running` or other incomplete states MUST be re-injected into the new IDE session.
- **Auto-Switcher Restore Dispatch Guarantee**:
  `execute_profile_rotation_with_context` must explicitly invoke `schedule_prompt_restore_after_launch(target_inst_id)` in Step 3.
- **Quota Depletion Prompt Recovery**:
  Provide detection of prompts stranded by quota exhaustion, enabling instant recovery and rerun under the new profile.

### 3.2 Frontend Rerun Auto-Pick & Expanded Options
- Clicking "Rerun" in `PromptTreeViewModal` and `DetailHeader` defaults to `'randomized'`.
- Provide discrete options:
  - `randomized`: Randomized nudge (random pick from rich 18+ message catalog)
  - `is-it-done`: "Is it done?"
  - `is-it-finished`: "Is it finished?"
  - `is-it-published`: "Is it published?"
  - `is-it-released`: "Is it released?"
- Persist selection across page refreshes and IDE instances via localStorage.

### 3.3 GitMap Code Quality & CG Execute Standards
- Review `d:/work/gitmap` codebase against the 39 coding guideline families in `01-prompts/15-cg-execute/`.
- Verify zero explicit boolean comparisons (`== true` / `== false`).
- Enforce positive boolean naming and guard clause inversion.
- Enforce leaf error sovereignty via structured `*appfault.AppError`.
- Enforce strictly relative Git paths throughout all specs, docs, and commands.

### 3.4 Strict Two-Phase Build/Test Invariant
- Phase 1 (Authoring & Refactoring): STRICTLY ZERO builds (`cargo build`, `npm run build`, `vite build`) and ZERO test execution.
- Phase 2 (Deferred Verification): Perform end-to-end testing, CLI responsiveness verification, and package compilation only after all code improvements are finalized.

---

## 4. Subtask Decomposition
1. `01-gitmap-pa-audit-and-recent-commits-analysis.md`: Fleet synchronization, commit history analysis, and pending plans audit across both repos.
2. `02-prompt-backup-restore-and-quota-drop-recovery.md`: Backend prompt backup, restore dispatch guarantee, and quota depletion incomplete-prompt rehydration.
3. `03-rerun-randomizer-autopick-and-persistence.md`: Frontend rerun auto-pick, expanded message options, and persistence.
4. `04-gitmap-cg-execute-and-pending-plans-drain.md`: GitMap code quality audit and coding guideline enforcement across CG execute families.
5. `05-agm-and-gitmap-e2e-master-verification.md`: Master end-to-end verification plan, multi-instance test scenarios, and deferred test execution.
