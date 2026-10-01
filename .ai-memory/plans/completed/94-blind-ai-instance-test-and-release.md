# Plan 94: Blind AI Instance Test and Release Verification

> **Canonical Specification:** `02-spec/21-app/94-blind-ai-instance-test-and-release.md`  
> **Status:** COMPLETED  
> **Target Subsystems:** `instance.rs`, `backup_prompts_db.rs`, `repo_db.rs`, `notification_hub.rs`, `supabase_sync.rs`, `agm.rs`, `scripts/dev-tool-clear.ps1`.  
> **Verified Date:** 2026-10-01  

---

## 🎯 Executive Summary & Architectural Overview

The Blind AI Instance Test and Release Verification rigorously audited, verified, and hardened the repository implementation against the **8 Laws of Instance Sandboxing, Account Switching, Prompt Recovery, and Release Readiness** specified in `02-spec/21-app/94-blind-ai-instance-test-and-release.md`.

Multi-agent subagents (Wave 1: Worker 01 and Worker 02) audited both the Rust core and the TypeScript frontend, validated database migrations, hardened Supabase network error diagnostics against PostgREST error codes (such as `PGRST205`), and safeguarded Cargo build caches in cleanup scripts.

---

## 🛡️ The 8 Laws Audit & Compliance Matrix

| Law | Principle | Implementation Mechanism | Verdict |
| :--- | :--- | :--- | :--- |
| **L1** | **Host Process PID Safety** | `instance::close_instance` explicitly skips processes containing `cursor`, `agm`, or `agm-alim`. Windows process termination strictly invokes `taskkill /F /T /PID <pid>`; zero `taskkill /IM` calls exist repository-wide. `agm test-instance-flow` registers host PIDs in `protected_pids` upfront and validates them before any operation. | **VERIFIED** |
| **L2** | **Folder Owns PID** | Processes are matched strictly by `--user-data-dir` containing the instance data path. `find_pids_for_data_dir` traverses up to 10 ancestor generations to isolate child processes. Processes outside the sandbox lineage are never touched. | **VERIFIED** |
| **L3** | **Clone vs New** | CLI enforces mutual exclusivity between `--from` and `--new`. `--from default` copies `state.vscdb` and sanitizes auth sessions, while `--new` constructs a clean profile skeleton without copying `state.vscdb`. | **VERIFIED** |
| **L4** | **Same Function (UI-CLI Parity)** | Frontend `instanceService.ts` issues Tauri IPC commands (`create_instance`, `copy_instance`, `switch_account_to_instance`, `close_instance`) that route to identical functions in `src-tauri/src/modules/instance.rs`. Zero shell calls to `agm.exe` exist in the UI. | **VERIFIED** |
| **L5** | **Queued Prompts Preservation** | `prompt_status_after_restore` preserves `"queued"` status for queued prompts. `resend_running_commands_for_instance` in `repo_db.rs` explicitly filters for `running` or `dispatched` prompts and skips queued ones. | **VERIFIED** |
| **L6** | **Notify Before Return** | Notification delivery is fully awaited prior to process return in `switch_account_to_instance`. Standardized lines `[Notify] email: OK/FAIL` and `[Notify] telegram: OK/FAIL` are logged. Errors force capture `std::backtrace::Backtrace::force_capture()` via `logger::log_error`. CLI initializes logger immediately at entry. | **VERIFIED** |
| **L7** | **Supabase Read-Back & Table Integrity** | `push_and_read_instance_email` upserts node and profile, then reads back email from PostgREST. `postgrest_error_message` was enhanced to detect error codes (`PGRST205` / missing relations), returning structured `Err(AppError::Network(...))` rather than masking errors as success. | **VERIFIED** |
| **L8** | **Cleanup Scope** | `cmd_test_instance_flow` cleanup Step 1 explicitly filters for `test-cli-flow*` and `test-diag*` IDs, refusing to delete `default` or any other user instance. Active prompts and running projects in `repo_db` are cleaned strictly for deleted instance IDs. | **VERIFIED** |

---

## 🧹 Developer Hygiene & Script Hardening

1. **`scripts/dev-tool-clear.ps1`**:
   - Refactored Cargo cache cleanup to target `src-tauri/target/debug/incremental` and `src-tauri/target/release/incremental`, avoiding recursive deletion of the entire `target/` directory. This preserves precompiled binaries (such as `agm.exe`) across cleanup runs.
   - Guarded demo artifact removal with `git ls-files --error-unmatch` to ensure no git-tracked directories or files can ever be deleted.
2. **Pre-flight & Formatting**:
   - Rust formatting verified via `cargo fmt -- --check` (clean, 0 drift).
   - Targeted unit tests in `src-tauri/src/modules/supabase_sync.rs` executed and passed (4/4 tests green).

---

## 📦 Verified Deliverables & Artifacts

- `src-tauri/src/modules/supabase_sync.rs`: Modularized `push_and_read_instance_email` into discrete sub-functions under 15 lines (`resolve_sync_target`, `build_sync_payloads`, `sync_node_and_profile`, `read_back_profile_email`), enhanced PostgREST error parsing, and added unit tests.
- `scripts/dev-tool-clear.ps1`: Hardened Cargo incremental cleanup and git-tracking protection.
- `.agents/skills/execute-parent-task-with-n-steps-v6/skill.md`: Installed native Antigravity skill for autonomous multi-agent parent task orchestration.
