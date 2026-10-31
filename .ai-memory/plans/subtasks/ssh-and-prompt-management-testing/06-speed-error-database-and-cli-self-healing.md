# Subtask Implementation Plan: High-Speed SQLite Error DB (`errors.db`) & Self-Healing Terminal CLI Tools

> **Subtask ID:** `06-speed-error-database-and-cli-self-healing`  
> **Parent Slug:** `ssh-and-prompt-management-testing`  
> **Spec References:**
> - `02-spec/21-app/ssh-and-prompt-management-testing/01-architecture-spec.md`
> - `02-spec/21-app/ssh-and-prompt-management-testing/03-migration-and-verification-spec.md`
> - `.agents/skills/agm-ai-diagnostics-and-logs/skill.md`  
> **Target Files:**
> - `src-tauri/src/modules/error_db/db.rs`
> - `src-tauri/src/modules/error_db/ops.rs`
> - `src-tauri/src/modules/error_db/diagnostics.rs`
> - `src-tauri/src/modules/error_db/types.rs`
> - `src-tauri/src/bin/agm/error_cmds.rs`
> - `src-tauri/src/bin/agm/help_text.rs`
> - `src/stores/error-store.ts`
> - `src/components/errors/error-modal.tsx`
> - `src/components/errors/error-modal/tabs.tsx`
> - `.agents/skills/agm-ai-diagnostics-and-logs/skill.md`  
> **Status:** `[QUEUED]`  
> **Author:** `@aukgit`  

---

## 1. Context & Objectives

The user explicitly requested:
> *"Enhance the `Antigravity Manager or Antigravity Tools` log section for AI. Add detail help section as in skill and other parts so that any AI can learn what to do, how to do it. How to view the logs, how to view the errors. See the system-level errors, haven't happened, and try to fix all these together. The error logs should also reside in the database, a speed Database (Database (Db)) of the errors, just like `gitmap`. Follow through `gitmap`'s path. These errors can also be seen from the terminal for you to fix it, understand it, and fix it. Save, export, these are there, but also have CLI level thing which can be used to fix it itself."*

### Core Objectives:
1. **Speed Error Database (`errors.db`)**: Model the SQLite error persistence after GitMap's speed DB architecture, utilizing `WAL` mode, `NORMAL` synchronous pragma, and indexed lookups for sub-millisecond recording.
2. **Frontend Error Streaming**: Persist captured frontend exceptions and unhandled rejections from `src/stores/error-store.ts` into `errors.db` asynchronously without blocking UI rendering or thread pools.
3. **AI Diagnostic Log Viewer**: Enhance the Antigravity Tools error modal and log inspection UI with structured AI diagnosis exports, one-click reports formatted for LLM ingestion, and error-code-specific troubleshooting recipes (`E1001`–`E9001`).
4. **Terminal Self-Healing Suite**: Expose and polish terminal CLI commands:
   - `agm errors list`: Tabular or JSON output of recent system faults.
   - `agm errors clear`: Purges error history from SQLite.
   - `agm errors fix`: Automated diagnostic engine that surgically repairs stale lockfiles, invalid configurations, corrupted databases, and resumption artifacts.
   - `agm failed-commands`: Displays command failure telemetry and suggestions.
5. **AI Skill Reference**: Maintain and expand `.agents/skills/agm-ai-diagnostics-and-logs/skill.md` as the canonical reference manual for AI agents operating on AGM.

---

## 2. Concrete Implementation Steps

### Step 1: Speed SQLite Database Optimization (`src-tauri/src/modules/error_db/`)
- In `src-tauri/src/modules/error_db/db.rs`:
  - Ensure `errors.db` initialization enforces high-throughput PRAGMAs:
    ```sql
    PRAGMA journal_mode = WAL;
    PRAGMA synchronous = NORMAL;
    PRAGMA busy_timeout = 5000;
    PRAGMA foreign_keys = ON;
    ```
  - Verify indices:
    - `idx_errors_created_at ON errors(created_at DESC)`
    - `idx_errors_code ON errors(error_code)`
    - `idx_errors_severity ON errors(severity)`
- In `src-tauri/src/modules/error_db/ops.rs`:
  - Implement fast batch insertion and sliding-window pruning to prevent unbounded disk growth.

### Step 2: Bridge Frontend Error Store to SQLite (`src/stores/error-store.ts`)
- In `src/stores/error-store.ts`:
  - In `captureError` and `trackWarning`, call Tauri IPC `log_client_error` (or HTTP `/api/errors/log` in web mode).
  - Use best-effort non-blocking dispatch (`void invoke(...).catch(...)`) to ensure error reporting never triggers recursive failure loops.
  - Store millisecond-precision timestamps (`requestedAt: new Date().toISOString()`).

### Step 3: Enhance UI Error Modal for AI Diagnostics (`src/components/errors/`)
- In `src/components/errors/error-modal.tsx` and `src/components/errors/error-modal/tabs.tsx`:
  - Add explicit "AI Diagnostic Report" button (`handleCopyAi`) that generates a standardized markdown prompt ready for LLM consumption:
    - Error code and severity
    - Human-readable message and diagnostic context
    - User interaction click path arrow (`Button#save -> Modal#settings -> Input#token`)
    - Cleaned stack frames with file paths relative to repository root
    - Skill reference link: `.agents/skills/agm-ai-diagnostics-and-logs/skill.md`
  - Render targeted troubleshooting remedies for recognized error codes (`E1001`–`E9001`).

### Step 4: Expand CLI Diagnostics & Self-Healing Suite (`src-tauri/src/bin/agm/`)
- In `src-tauri/src/bin/agm/error_cmds.rs`:
  - Polish `cmd_errors_list`:
    - Support `--limit <N>`, `--severity <S>`, `--json`, and search terms.
    - Format output in high-contrast ANSI terminal tables.
  - Expand `cmd_errors_fix`:
    - Check 1: Remove stale Chromium lockfiles (`lockfile`, `code.lock`, `DevToolsActivePort`).
    - Check 2: Run SQLite `PRAGMA quick_check;` across all active databases (`accounts.db`, `repo_prompts.db`, `errors.db`).
    - Check 3: Verify and sanitize resumption artifacts (`.antigravity_resume_task.json`).
    - Check 4: Probe reverse proxy loopback port (8045) and release hung sockets if necessary.
    - Support `--dry-run` to preview repairs without modifying files.
- In `src-tauri/src/bin/agm/help_text.rs`:
  - Document `agm errors list`, `agm errors fix`, and `agm failed-commands` in the main help output and `--json` manifest.

### Step 5: Document AI Diagnostics & Self-Healing Skill
- In `.agents/skills/agm-ai-diagnostics-and-logs/skill.md`:
  - Document all error code ranges (`E1000`–`E9000`), their root causes, and recommended automated or manual remediations.
  - Provide instructions for AI subagents on querying `errors.db` directly via SQLite and executing `agm errors fix`.

---

## 3. Invariants & Guardrails

- **Strict Relative Paths**: All paths cited in error reports, logs, and documentation must be relative to the repository root. Never log full Windows paths (`C:\Users\...`) to prevent credential leakage.
- **Positive Booleans**: Use positive flags (`is_resolved`, `is_dry_run`, `is_modal_open`).
- **Sub-millisecond Ordering**: Queries for recent errors must sort deterministically: `ORDER BY created_at DESC`.
- **Zero Silent Errors**: Swallowing errors is strictly forbidden. Any ignored fallible operation must call `crate::error::record_ignored` with a `// Justification:` comment.
- **Attribution**: Exclusively `@aukgit`.

---

## 4. Acceptance Criteria & Verification

### Acceptance Criteria:
1. Every error captured in the frontend is stored in `errors.db` with an accurate error code and timestamp.
2. Running `agm errors list` from the terminal displays all recorded errors in a clean table or JSON.
3. Running `agm errors fix` detects and reports status on stale lockfiles, SQLite integrity, and resumption files.
4. The error modal in the GUI provides a one-click "Copy AI Report" button containing relative file paths and skill references.
5. The documentation in `.agents/skills/agm-ai-diagnostics-and-logs/skill.md` correctly maps all error codes to automated fix actions.

### Verification Commands:
```powershell
# 1. Rust build and Clippy gate
cd src-tauri
cargo check --bin agm
cargo clippy --lib --all-features
cd ..

# 2. Targeted error_db unit tests
cd src-tauri
cargo test --lib modules::error_db
cd ..

# 3. CLI execution verification
cargo run --bin agm -- errors list --limit 5
cargo run --bin agm -- errors fix --dry-run
cargo run --bin agm -- errors count
```

---

*Maintained by @aukgit*
