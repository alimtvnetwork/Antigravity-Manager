# Parent Plan — 193 Implementing Next Five Tasks

Status: COMPLETED
Phase: 3 (finalized & committed)
Owner: Agent-193-Lead
Specs: `02-spec/21-app/193-implementing-next-five-tasks/spec.md`
E2E Master Plan: `02-spec/21-app/193-implementing-next-five-tasks/e2e_master_plan.md`
Subtasks: `.ai-memory/plans/subtasks/193-implementing-next-five-tasks/`
Backlog Delta: 75 steps -> 70 steps remaining (-5 steps executed)

## 1. Selected Five Tasks (Executed & Completed)

| Task # | Slug | Module Target | Status | Implementation Details |
|---|---|---|---|---|
| 01 | `profile-duplication-parity` | `src-tauri/src/modules/release_gate/profile_duplication_parity_guard.rs` | `COMPLETED` | Enforced GUI/CLI parity via unified `copy_instance_with_options` route, preserving `workspaceStorage`. |
| 02 | `settings-deep-merge` | `src-tauri/src/modules/release_gate/settings_deep_merge_synchronizer.rs` | `COMPLETED` | Recursive JSON settings deep-merge preserving target instance identity tokens. |
| 03 | `job-breakaway-fallback` | `src-tauri/src/modules/release_gate/job_breakaway_fallback_ladder.rs` | `COMPLETED` | 2-stage breakaway spawn fallback stepping down on Win32 error 5 (Access is denied). |
| 04 | `batch-file-spawn-guard` | `src-tauri/src/modules/release_gate/batch_file_spawn_guard.rs` | `COMPLETED` | Intercepted raw `.bat`/`.cmd` files from direct `Command::new` execution, wrapping via `cmd.exe /c start`. |
| 05 | `transcript-deep-inspection` | `src-tauri/src/modules/release_gate/transcript_deep_inspection_scanner.rs` | `COMPLETED` | Enforced `transcript_full.jsonl` precedence and reverse-scan >= 10 lines bypassing telemetry records. |

## 2. Invariants & Rules Upheld
- **Zero-Build & Zero-Test Invariant**: Strictly preserved; zero compilers invoked.
- **Repository-Relative Paths**: 100% compliant.
- **Anti-Collision**: Initial `IN_PROGRESS` commitment was registered on `origin/main` before code modifications.
