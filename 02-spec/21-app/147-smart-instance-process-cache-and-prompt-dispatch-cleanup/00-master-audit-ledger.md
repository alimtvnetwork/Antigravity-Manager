# Master Audit Ledger: Task 147 - Smart Instance Process Cache & Prompt Dispatch Cleanup

## 1. Executive Summary & Problem Classification
- **Classification**: Bugfix & Reliability Refactor (Prompt Dispatch Lifecycle, Process Liveness Caching, UI Tag Noise Reduction, Duration Formatter Hygiene).
- **Slug**: `147-smart-instance-process-cache-and-prompt-dispatch-cleanup`
- **Lead Maintainer / Attribution**: Strictly `@aukgit` (`(Thanks to @aukgit)`).
- **Core Defects Addressed**:
  1. **Prompt Dispatch / Enqueueing Broken & Phantom Restarts**:
     - Prompt sending (`Send Now`) and enqueuing (`Enqueue`) from the Instance Section / Prompt Tree Modal fail to take effect or repeatedly trigger unwanted instance restarts.
     - `enqueue_prompt` IPC command was completely missing from the Tauri backend registry (`src-tauri/src/lib.rs` and `commands/instance.rs`).
     - `focus_or_launch_instance` blindly re-launched IDE instances when window focusing returned `false` on Linux/Wayland, even if the instance process was already healthy and running.
  2. **Missing Smart Instance Process Cache & PID Verification**:
     - Lack of an active process cache tracking running Antigravity IDE instances, their PIDs, and health.
     - Failure to verify if a PID is alive before deciding whether to launch or focus.
  3. **Bracket Tag Clutter in Prompt Tree View**:
     - Every project node rendered bloated `[AGM:P006 | GM:#6]` tags and conversation items rendered `[AGM:C025 | GM:antigrav]` tags, dominating the line and creating visual noise.
  4. **Ghost 'RUNNING' Indicators & Duration Calculation Defects (`NaNm NaNs`)**:
     - Idle projects and completed turns showed false `1 RUNNING` indicators due to unbounded 600s/240s heuristic windows and unpruned `active_prompts` SQLite records.
     - In `PromptTreeViewModal.tsx`, timestamp string parsing returned `NaN`, producing `RUNNING (PID: 278499) NaNm NaNs`.

## 2. Invariant Rules & Boundaries
- **Strict Relative Git Paths**: All paths cited in documentation and code must be relative (e.g. `src-tauri/src/...`, `src/components/...`). Zero absolute paths or `file:///` URIs.
- **Positive Booleans**: Use `is` and `has` prefixes exclusively (`is_running`, `is_alive`, `is_cached`, `has_valid_pid`, `has_images`).
- **Attribution Discipline**: Attribute strictly `@aukgit` in `CHANGELOG.md` and release notes.
- **Search Discipline**: Search exclusively via GitMap (`gitmap aum search`, `gitmap find`); total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.

## 3. Subtask Breakdown & File Boundaries
| Subtask ID | Focus | Owned Files | Status |
|---|---|---|---|
| `147-01` | Backend Smart Instance Process Cache & PID Liveness | `src-tauri/src/modules/instance.rs` | COMPLETED |
| `147-02` | IPC Commands for Direct Send Now & Enqueue | `src-tauri/src/commands/instance.rs`, `src-tauri/src/modules/repo_db.rs`, `src-tauri/src/lib.rs` | COMPLETED |
| `147-03` | Frontend Prompt Tree View Tag Reduction & Dispatch Integration | `src/components/instances/PromptTreeViewModal.tsx`, `src/services/instanceService.ts` | COMPLETED |
| `147-04` | Ghost Running Indicator Fix & Duration Formatter Guard | `src-tauri/src/modules/repo_db.rs`, `src/components/instances/PromptTreeViewModal.tsx` | COMPLETED |
| `147-05` | Pre-flight Verification, CLI Parity & Minor Release (`v4.165.0`) | `package.json`, `src-tauri/Cargo.toml`, `README.md`, `README_EN.md`, `CHANGELOG.md`, `CHANGELOG_EN.md` | COMPLETED |
