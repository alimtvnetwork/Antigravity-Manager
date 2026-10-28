# Execution Plan: 50-instant-rerun-enqueue-and-minor-release

## Strategic Intent & Systems Reasoning
Eliminate all artificial latency and delay sources in the prompt Rerun and Enqueue pipelines:
1. **Backend Scheduler 30s Backoff Root Cause**: The background prompt queue scheduler (`start_prompt_queue_scheduler` in `src-tauri/src/modules/scheduler.rs`) slept for `PROMPT_QUEUE_IDLE_BACKOFF_INTERVAL = 30s` whenever the queue had been empty. `enqueue_prompt` wrote to SQLite but never notified the scheduler or triggered dispatch. An enqueued prompt on an idle instance was parked for up to 30 seconds before dispatching.
   - *Fix*: Introduce `tokio::sync::Notify` (`notify_prompt_queue()`) to immediately wake the scheduler loop on enqueue, and trigger an immediate idle dispatch check (`check_and_dispatch_enqueued_prompts(Some(&resolved_id))`) in the `enqueue_prompt` IPC handler.
2. **Backend Blocking CDP Delay**: `enqueue_prompt_for_instance` attempted synchronous CDP DOM injection with a 4-iteration 400ms retry loop (1.6s).
   - *Fix*: Bypass blocking CDP on pure enqueue operations so SQLite write completes in sub-milliseconds.
3. **Frontend Pacing Delays**: `dispatchRerunMessages` executed `await new Promise((r) => setTimeout(r, 300))` between every round, plus repeated heavy clipboard conversions.
   - *Fix*: Eliminate artificial pacing sleeps and perform clipboard copy once.
4. **Minor Release Ceremony**:
   - Execute `npm run bump minor` to advance version from `4.267.0` to `4.268.0`.
   - Update `CHANGELOG.md`, `CHANGELOG_EN.md`, `README.md`, and `README_EN.md` strictly attributing `@aukgit` and strictly using relative Git paths.
   - Pass preflight gates, tag `v4.268.0`, commit, and push.

## User Request (Verbatim)
```text
also rerun is dispathcing after sometime later why? fix it , rerun should enque immediattely thats it, fix it core logic and do a minor bump and release please

## Must follow and spawn agent using

execute-parent-task-with-n-steps-v6

## Additional Instructions

- plan first before doing work for improved work or less friction, faster execution for AI and coding without errors and following Coding Guidelines.
- learn from gitmap skill to leverage GitMap high-speed search, toolchain discovery, and caching.
- Track in-flight file touches and prevent write collisions using GitMap RC Heatmap (`gitmap rc heatmap status` / `gitmap rc heatmap tree` / `gitmap rc heatmap live` / `gitmap rc heatmap sync`).
- Only add relative paths, never add absolute paths during work; this must be respected on release page and in release notes as well.
```

## Architecture Boundaries & File Ownership
- Spec Files:
  - `02-spec/21-app/50-instant-rerun-enqueue-and-minor-release/01-architecture-spec.md` (Spec Author 01)
  - `02-spec/21-app/50-instant-rerun-enqueue-and-minor-release/02-component-spec.md` (Spec Author 02)
  - `02-spec/21-app/50-instant-rerun-enqueue-and-minor-release/03-migration-and-verification-spec.md` (Spec Author 03)
- Plan Subtasks:
  - `.ai-memory/plans/subtasks/50-instant-rerun-enqueue-and-minor-release/01-subtask-backend-queue-latency-fix.md` (Spec Author 01)
  - `.ai-memory/plans/subtasks/50-instant-rerun-enqueue-and-minor-release/02-subtask-frontend-instant-rerun.md` (Spec Author 02)
  - `.ai-memory/plans/subtasks/50-instant-rerun-enqueue-and-minor-release/03-subtask-minor-bump-and-release.md` (Spec Author 03)
- Source Implementation Files:
  - `src-tauri/src/modules/scheduler.rs`: `tokio::sync::Notify` queue wakeup mechanism
  - `src-tauri/src/commands/instance.rs`: Trigger immediate dispatch check on enqueue
  - `src-tauri/src/modules/repo_db/agy.rs`: Eliminate blocking CDP on enqueue
  - `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`: Remove pacing delay in `dispatchRerunMessages`
  - `src/components/instances/PromptTreeView/DetailHeader.tsx`: Immediate cache invalidation and tree reload on rerun
  - Version manifests & documentation (`package.json`, `src-tauri/Cargo.toml`, `CHANGELOG.md`, `CHANGELOG_EN.md`, `README.md`, `README_EN.md`)

## Verification Evidence Contract
- Pre-flight checks: `npm run build`, `cargo check --lib`, `cargo fmt -- --check`.
- Unit tests: `rerunActions.test.ts` passing.
- Version sync: `4.268.0` synchronized across all manifests.
- Git tag: `v4.268.0` pushed to `origin/main`.
