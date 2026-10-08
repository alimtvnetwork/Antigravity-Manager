# Master Plan: Task 147 - Smart Instance Process Cache & Prompt Dispatch Cleanup

## 1. Overview
Resolve the prompt dispatch failure, infinite IDE re-launch defect, ghost running indicators, and bracket tag clutter through a comprehensive backend smart process cache, first-class IPC commands (`send_prompt_now`, `enqueue_prompt`), and UI de-cluttering in `PromptTreeViewModal.tsx`.

## 2. Invariant Rules
- Non-negotiable relative Git paths only.
- Strict positive booleans (`is_running`, `is_alive`, `is_cached`, `has_valid_pid`).
- Attribution strictly `@aukgit` (`(Thanks to @aukgit)`).
- Search exclusively via GitMap (`gitmap aum search`, `gitmap find`).
- Pre-flight checks before tagging: `cargo fmt -- --check`, `cargo clippy --all-targets --all-features`, `npm run build`.

## 3. Subtask Index
1. **Subtask 01**: Backend Smart Instance Process Cache & PID Liveness (`.ai-memory/plans/subtasks/147-smart-instance-process-cache-and-prompt-dispatch-cleanup/01-backend-smart-process-cache-and-liveness.md`)
2. **Subtask 02**: Prompt Dispatch & Enqueue IPC Commands (`.ai-memory/plans/subtasks/147-smart-instance-process-cache-and-prompt-dispatch-cleanup/02-prompt-dispatch-enqueue-and-ipc-commands.md`)
3. **Subtask 03**: Frontend Dispatch Integration & Bracket Tag Reduction (`.ai-memory/plans/subtasks/147-smart-instance-process-cache-and-prompt-dispatch-cleanup/03-frontend-dispatch-and-bracket-tag-reduction.md`)
4. **Subtask 04**: Ghost Running Cleanup & Duration Formatter Hygiene (`.ai-memory/plans/subtasks/147-smart-instance-process-cache-and-prompt-dispatch-cleanup/04-ghost-running-cleanup-and-duration-fix.md`)
5. **Subtask 05**: CLI Verification, E2E Testing & Minor Release Ceremony (`.ai-memory/plans/subtasks/147-smart-instance-process-cache-and-prompt-dispatch-cleanup/05-cli-e2e-testing-and-minor-release.md`)
