# Master Audit Ledger: Task 148 - IDE Prompt Dispatch, Process Cache & Bracket Cleanup

## 1. Executive Summary & Problem Classification
- **Classification**: Bugfix, Reliability Refactor & UI Polish (IDE Prompt Dispatch Lifecycle, Smart Instance Process Cache with Closed-PID Re-Scan, Elimination of Bracket Tag Clutter, Ghost Running State Eradication, CLI Parity).
- **Slug**: `148-ide-prompt-dispatch-process-cache-and-bracket-cleanup`
- **Lead Maintainer / Attribution**: Strictly `@aukgit` (`(Thanks to @aukgit)`).
- **Core Defects Addressed**:
  1. **Prompt Dispatch / Enqueueing Broken & Repeated IDE Restarts**:
     - Prompt sending (`Send Now`) and enqueuing (`Enqueue`) from the Instance section / Prompt Tree Modal had no effect in the running IDE and repeatedly attempted duplicate IDE restarts.
     - Frontend double-invoked dispatch + focus, triggering a race condition where a second launch was initiated before the first finished initialization.
     - `spawn_prompt_via_agy` omitted `--user-data-dir`, causing the CLI to default to `$HOME/.config/Antigravity` instead of the cloned instance directory, failing to reach the active IDE instance.
  2. **Smart Instance Process Cache & Closed-PID Re-Scan**:
     - User-mandated 3-step sequence:
       - Step 1: When sending or queueing a prompt, first scan how many instances are running for Antigravity Tools Manager and cache those.
       - Step 2: If found in cache, check if that PID is closed. If closed, re-scan OS processes to verify if running again under a new PID.
       - Step 3: If not running after re-scan, then and only then reopen the IDE and send the prompt.
     - `is_pid_alive_os` on Unix recycled PIDs without validating process identity; updated to verify genuine Antigravity process signature.
     - `sysinfo` in `close_instance` omitted `.with_cmd(...)`, failing process argument match and failing to terminate old instances.
  3. **Bracket Tag Clutter in Prompt Tree View**:
     - As evidenced in screenshot `media_1791477466124.png`, every project node rendered `[AGM:P006 | GM:#6]` and every turn rendered `[AGM:C025 | GM:antigrav]`, consuming over 40% of horizontal line space.
     - Replaced with sleek, borderless, bracket-free `#6`, `P006`, and `C025` sequence indicators.
  4. **Ghost '1 RUNNING' Indicators on Idle Projects**:
     - `auto_resume_recent_prompts` manufactured synthetic fake crash recovery prompts with status `'dispatched'` for projects without previous prompts, polluting `active_prompts`.
     - `compute_project_conversation_tree` blindly marked projects as `1 RUNNING` when `is_inst_alive` was true without checking actual in-flight process or transcript status.
     - Purged synthetic prompt generation and required empirical worker/transcript verification.
  5. **CLI Parity (`agm prompts` & `agm doctor`)**:
     - Neither `prompts` nor `doctor` were routed in `src-tauri/src/modules/cli.rs`.
     - Implemented full CLI handlers for `prompts ls`, `tree`, `send`, `enqueue`, `backup`, `restore` and `doctor` with human and `--json` support.

## 2. Invariant Rules & Boundaries
- **Strict Relative Git Paths**: All paths cited in documentation and code must be relative (e.g. `src-tauri/src/...`, `src/components/...`). Zero absolute paths or `file:///` URIs.
- **Positive Booleans**: Use `is` and `has` prefixes exclusively (`is_running`, `is_alive`, `is_cached`, `has_valid_pid`).
- **Attribution Discipline**: Attribute strictly `@aukgit` in `CHANGELOG.md` and release notes.
- **Search Discipline**: Search exclusively via GitMap (`gitmap aum search`, `gitmap find`); total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.

## 3. Subtask Breakdown & File Boundaries
| Subtask ID | Focus | Owned Files | Status |
|---|---|---|---|
| `148-01` | Smart Process Cache, Closed-PID Re-Scan & agy Dispatch | `src-tauri/src/modules/instance.rs`, `src-tauri/src/modules/repo_db.rs` | COMPLETED |
| `148-02` | Bracket Tag Reduction & Double-Launch Removal | `src/components/instances/PromptTreeViewModal.tsx` | COMPLETED |
| `148-03` | Ghost Running State Eradication | `src-tauri/src/modules/repo_db.rs` | COMPLETED |
| `148-04` | CLI Parity: `prompts` & `doctor` | `src-tauri/src/modules/cli.rs` | COMPLETED |
| `148-05` | Pre-flight Quality Gates, Testing & Minor Bump (`v4.166.0`) | `package.json`, `src-tauri/Cargo.toml`, `changelog.md`, `changelog_en.md`, `readme.md`, `readme_en.md` | COMPLETED |
