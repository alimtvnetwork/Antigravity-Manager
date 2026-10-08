# Master Plan: Task 148 - IDE Prompt Dispatch, Process Cache & Bracket Cleanup

## 1. Overview
Resolve the prompt dispatch failure, repeated IDE instance launches, false running states, bracket tag clutter, and CLI omissions through:
1. User-mandated 3-step smart process caching with closed-PID re-scan logic in `src-tauri/src/modules/instance.rs`.
2. Removal of the frontend double-launch race condition in `PromptTreeViewModal.tsx` and injection of `--user-data-dir` into `spawn_prompt_via_agy`.
3. Total elimination of heavy bracket wrappers (`[AGM:P006 | GM:#6]`, `[AGM:C025 | GM:antigrav]`, omission bracket tags) in `PromptTreeViewModal.tsx`.
4. Permanent eradication of ghost running states caused by synthetic prompt manufacturing and unverified in-flight status in `repo_db.rs`.
5. Full CLI command routing and implementation for `agm prompts` and `agm doctor` in `src-tauri/src/modules/cli.rs`.
6. Full pre-flight verification, minor version bump to `v4.166.0`, and upstream release to `main`.

## 2. Invariant Rules
- Non-negotiable relative Git paths only.
- Strict positive booleans (`is_running`, `is_alive`, `is_cached`, `has_valid_pid`).
- Attribution strictly `@aukgit` (`(Thanks to @aukgit)`).
- Search exclusively via GitMap (`gitmap aum search`, `gitmap find`).
- Pre-flight checks before tagging: `cargo fmt -- --check`, `cargo clippy --all-targets --all-features`, `npm run build`.

## 3. Subtask Index
1. **Subtask 01**: Smart Process Cache, Closed-PID Re-Scan & agy Dispatch (`.ai-memory/plans/subtasks/148-ide-prompt-dispatch-process-cache-and-bracket-cleanup/01-smart-process-cache-and-closed-pid-rescan.md`)
2. **Subtask 02**: Bracket Tag Reduction & Double-Launch Removal (`.ai-memory/plans/subtasks/148-ide-prompt-dispatch-process-cache-and-bracket-cleanup/02-ui-bracket-reduction-and-ghost-running-cleanup.md`)
3. **Subtask 03**: Ghost Running State Eradication (`.ai-memory/plans/subtasks/148-ide-prompt-dispatch-process-cache-and-bracket-cleanup/03-ghost-running-state-eradication.md`)
4. **Subtask 04**: CLI Parity: `prompts` & `doctor` (`.ai-memory/plans/subtasks/148-ide-prompt-dispatch-process-cache-and-bracket-cleanup/04-cli-parity-prompts-and-doctor.md`)
5. **Subtask 05**: Pre-flight Quality Gates, Testing & Minor Bump (`.ai-memory/plans/subtasks/148-ide-prompt-dispatch-process-cache-and-bracket-cleanup/05-preflight-testing-and-minor-bump.md`)
