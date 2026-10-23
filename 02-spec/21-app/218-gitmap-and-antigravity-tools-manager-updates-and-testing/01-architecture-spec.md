# Architecture Specification: GitMap & Antigravity Tools Manager Updates, Prompt Restore & E2E Testing

## Metadata
- **Spec ID**: `02-spec/21-app/218-gitmap-and-antigravity-tools-manager-updates-and-testing/01-architecture-spec.md`
- **Plan Reference**: `.ai-memory/plans/pending/218-gitmap-and-antigravity-tools-manager-updates-and-testing.md`
- **Slug**: `gitmap-and-antigravity-tools-manager-updates-and-testing`
- **Target Subsystems**: GitMap CLI, Antigravity-Manager Proxy, Auto-Switcher, Prompt Backup DB, Prompt Tree View, Coding Guidelines.

---

## 1. Executive Summary & Intent
This specification formalizes the end-to-end synchronization, code quality enforcement, and reliability remediation across GitMap and Antigravity Tools Manager:
1. **Fleet Synchronization & Historical Context**: Executing `gitmap pa` across all workspace repositories, auditing the last 50–80 commits across GitMap and Antigravity-Manager, reviewing completed and pending plans, and synchronizing recent updates from coding guidelines, canonical prompts, and Antigravity skills.
2. **Account Switching & Interrupted Prompt Recovery**: Addressing the critical defect where auto-switching succeeds, but prompt restoration fails. When an account depletes its quota, in-flight prompts stranded in `running` status must be identified, restored into `backed_up` status, and re-executed on the newly rotated account without loss of work or state.
3. **Rerun Randomizer**: Ensuring that the Rerun button auto-picks from diverse random follow-up nudges ("Is it finished?", "Is it released?", "Is it completed?", "Please verify and finish the remaining items.", etc.) and embeds the true original task prompt cleanly without recursive header nesting.
4. **Code Quality & CG Execute Suite**: Auditing GitMap and Antigravity-Manager against canonical coding standards (`02-spec/02-coding-guidelines/`), enforcing bounded functions (8–15 lines), positive booleans, PascalCase enums with `Type` suffix, AppError propagation, and zero silent errors.
5. **Two-Phase Build & E2E Validation Invariant**: Enforcing strictly ZERO builds or test runs during active code modification; all build checks, unit tests, and comprehensive E2E validation are executed only after all improvements are finalized.

---

## 2. Blast Radius & Architectural Contracts

### A. Subsystem Topology
```
[User Request / CLI]
       │
       ▼
[GitMap PA Fleet Sync] ──> [Git History Audit (Last 50-80 commits)]
       │
       ▼
[Antigravity-Manager Core]
  ├── [Auto-Switcher] ──> Candidate Selection with Headroom Fallback (> 5.0%)
  ├── [Prompt Backup DB] ──> Incomplete Prompt Sweep (running, dispatched, backed_up)
  ├── [Repo DB Auto-Resume] ──> Task Snapshot Reconnection & CDP Injection
  └── [Prompt Tree View] ──> Rerun Randomizer & Up-Next Visibility
```

### B. Invariant Contracts
1. **Strict FIFO Ordering**: All queries across `active_prompts`, `prompt_backups`, and `repo_prompts` must strictly enforce `ORDER BY created_at ASC, id ASC`.
2. **No Silent Errors**: Fallible paths must propagate via `AppError` or record using `crate::error::record_ignored` accompanied by a mandatory `// Justification:` comment in Rust, and `useErrorStore.getState().trackWarning` in TypeScript.
3. **Relative Git Paths**: All documentation, reports, plans, and code references must use strictly relative Git paths (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`). Absolute paths are strictly forbidden.
4. **Search Ban**: TOTAL BAN on `rg`, `ripgrep`, `grep`, `git grep`, and `Select-String`. All code searching must use GitMap (`gitmap aum search`, `gitmap find`, `gitmap cat`).
5. **Two-Phase Build/Test Invariant**: Zero builds and zero test runs during active code modification. Build and test validation only after all improvements are completed.
6. **Positive Booleans**: Affirmative naming (`is_*`, `has_*`, `can_*`, `should_*`), strictly zero literal equality checks (`== true` or `== false`).
7. **Function & File Size Bounds**: Target function size 8–15 lines, decomposing oversized methods into focused single-responsibility helpers.
8. **Honest Outcome Reporting**: Zero fake success; all recovery and restore routines report exact counts (`restored`, `dispatched`, `failed`, `skipped_live`).
