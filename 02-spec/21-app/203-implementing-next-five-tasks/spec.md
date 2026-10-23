# Sequence 203 Specification: Implementing Next Five Tasks (Steps 226–230)

## Overview
This specification details the design, constraints, and verification requirements for Batch 203 (Steps 226–230 of 255) in Antigravity-Manager.
All tasks originate directly from the non-negotiable architectural invariants codified in `AGENTS.md`.

### Tasks Summary
1. **Task 1 (`Imp-640`)**: `src-tauri/src/modules/release_gate/preflight_checks_gate.rs` & `tests_preflight_checks_gate.rs`
   - Enforce AGENTS.md pre-flight gate protocol: Verifies `cargo fmt -- --check`, `cargo clippy --all-targets --all-features`, and `npm run build` checks before release tag generation, ensuring zero warnings and enforcing compile safety prior to tagging.
2. **Task 2 (`Imp-641`)**: `src-tauri/src/modules/instance/caller_ide_exit_code_enforcer.rs` & `tests_caller_ide_exit_code_enforcer.rs`
   - Enforce AGENTS.md caller IDE self-preservation invariant: Commands targeting the caller instance (`agm instances stop`, `rm`, `switch`) must fail-fast with exit code 2 unless `--force` is explicitly provided, preventing developer workflow disruption.
3. **Task 3 (`Imp-642`)**: `src-tauri/src/modules/release_gate/pr_template_compliance_guard.rs` & `tests_pr_template_compliance_guard.rs`
   - Enforce AGENTS.md review & template alignment: Verifies `.github/pull_request_template.md` completeness, validating problem classification, behavior alterations, unverified paths, and rollback strategy sections.
4. **Task 4 (`Imp-643`)**: `src-tauri/src/proxy/handlers/common/thinking_budget_filter_pipeline.rs` & `tests_thinking_budget_filter_pipeline.rs`
   - Enforce AGENTS.md pipeline thinking budget backfilling and filtering: Uniformly handles converted Gemini payloads, filtering unsupported models from thinking allocation and dynamically backfilling default thinking budgets when requested by clients.
5. **Task 5 (`Imp-644`)**: `src-tauri/src/modules/db_orm/fifo_prompt_restore_validator.rs` & `tests_fifo_prompt_restore_validator.rs`
   - Enforce AGENTS.md strict First-In-First-Out invariant: Inspects all prompt restore and query builder expressions across `active_prompts`, `prompt_backups`, and `repo_prompts.db`, verifying strict `ORDER BY created_at ASC, id ASC` and intercepting any inverted LIFO queries (`ORDER BY updated_at DESC`).

## Non-Negotiable Invariants
- **Zero-Build & Zero-Test Invariant**: Strictly no `cargo build`, `npm run build`, `cargo test`, or `npm test` during implementation.
- **Strictly Relative Paths**: Maintain 100% relative paths (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`).
- **No Silent Errors Invariant**: All fallible paths propagate via `AppError` or use `record_ignored` with a `// Justification:` comment.
- **Exclusively GitMap for Search**: Strictly `gitmap aum search`, `gitmap find`, and `gitmap cat`. Total ban on rg, ripgrep, grep, git grep, Select-String.
- **FIFO Ordering Invariant**: All queries must use `ORDER BY created_at ASC, id ASC`.
