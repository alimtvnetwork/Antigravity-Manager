# Sequence 205 Specification: Implementing Next Five Tasks (Steps 236–240)

## Overview
This specification details the design, constraints, and verification requirements for Batch 205 (Steps 236–240 of 255) in Antigravity-Manager.
All tasks originate directly from the non-negotiable architectural invariants codified in `AGENTS.md`.

### Tasks Summary
1. **Task 1 (`Imp-650`)**: `src-tauri/src/proxy/handlers/common/thinking_budget_headroom_evaluator.rs` & `tests_thinking_budget_headroom_evaluator.rs`
   - Enforce AGENTS.md thinking budget dynamic allocation and headroom protection: ensures that allocated `thinking_budget` never starves text generation headroom, guaranteeing at least minimum text headroom (512 tokens or proportional ceiling) to prevent upstream 400 Bad Request / 422 Unprocessable Entity errors.
2. **Task 2 (`Imp-651`)**: `src-tauri/src/modules/release_gate/preflight_clippy_gate.rs` & `tests_preflight_clippy_gate.rs`
   - Enforce AGENTS.md pre-flight checks gate: audits `cargo clippy --all-targets --all-features` execution results and diagnostic outputs, intercepting compiler warnings, clippy linter rejections, and flag regressions prior to release tagging.
3. **Task 3 (`Imp-652`)**: `src-tauri/src/modules/release_gate/pr_unverified_paths_linter.rs` & `tests_pr_unverified_paths_linter.rs`
   - Enforce AGENTS.md contributor cost disclosure & PR template alignment: audits PR descriptions for non-empty, substantive disclosure in the "Unverified Paths" and "Affected Existing Behaviors" sections, especially when high-risk subsystems (reverse proxy streaming, split SQLite ORM, process lifecycle) are touched.
4. **Task 4 (`Imp-653`)**: `src-tauri/src/modules/instance/multi_stage_launcher_engine.rs` & `tests_multi_stage_launcher_engine.rs`
   - Enforce AGENTS.md process spawning & job breakaway execution ladder: executes multi-instance launches using target binary, base binary, or Windows Shell dispatch (`cmd.exe /c start`), supervising child process tracking and handling Windows Job Object access denial (`os error 5`).
5. **Task 5 (`Imp-654`)**: `src-tauri/src/modules/db_orm/fifo_prompt_order_enforcer.rs` & `tests_fifo_prompt_order_enforcer.rs`
   - Enforce AGENTS.md strict First-In-First-Out invariant: intercepts raw SQLite queries destined for `active_prompts`, `prompt_backups`, and `repo_prompts.db`, detecting and transforming any inverted LIFO ordering into canonical `ORDER BY created_at ASC, id ASC`.

## Non-Negotiable Invariants
- **Zero-Build & Zero-Test Invariant**: Strictly no `cargo build`, `npm run build`, `cargo test`, or `npm test` during implementation.
- **Strictly Relative Paths**: Maintain 100% relative paths (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`).
- **No Silent Errors Invariant**: All fallible paths propagate via `AppError` or use `record_ignored` with a `// Justification:` comment.
- **Exclusively GitMap for Search**: Strictly `gitmap aum search`, `gitmap find`, and `gitmap cat`. Total ban on rg, ripgrep, grep, git grep, Select-String.
- **FIFO Ordering Invariant**: All prompt queries and transforms must enforce `ORDER BY created_at ASC, id ASC`.
