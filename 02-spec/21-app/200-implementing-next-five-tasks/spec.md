# Sequence 200 Specification: Implementing Next Five Tasks (Steps 211–215)

## Overview
This specification details the design, constraints, and verification requirements for Batch 200 (Steps 211–215 of 255) in Antigravity-Manager.
All tasks originate directly from the non-negotiable architectural invariants codified in `AGENTS.md`.

### Tasks Summary
1. **Task 1 (`Imp-625`)**: `src-tauri/src/proxy/handlers/common/prefix_stability_sanitizer.rs` & `tests_prefix_stability_sanitizer.rs`
   - Enforce prefix stability for prompt caching, sanitize request header prefixes (stripping disallowed `x-goog-` and internal proxy headers), and ensure consistent prefix ordering across multi-turn sessions per AGENTS.md pipeline-first architecture ("prefix stability, and the sanitization of risky prompts and request headers").
2. **Task 2 (`Imp-626`)**: `src-tauri/src/proxy/handlers/common/unified_context_structural_aligner.rs` & `tests_unified_context_structural_aligner.rs`
   - Align multi-role turn alternating structures (user/model/user/model), merge contiguous turns of the same role (e.g. adjacent `user` messages into single turn), sanitize empty text blocks, ensure `parts` arrays are non-empty, and validate `systemInstruction` placement per AGENTS.md ("unified context structural alignment").
3. **Task 3 (`Imp-627`)**: `src-tauri/src/modules/release_gate/thinking_cache_target_version_guard.rs` & `tests_thinking_cache_target_version_guard.rs`
   - Enforce AGENTS.md thinking cache invalidation control: verify alignment of `SUGGESTION_DELETE_THINKING_STORE` boolean flag, `SUGGESTION_TARGET_VERSION`, current package version, and release changeset classification (flags when schema/architecture refactors occur without cache invalidation evaluation, or when routine release erroneously enables prompt).
4. **Task 4 (`Imp-628`)**: `src-tauri/src/modules/instance/job_breakaway_process_spawner.rs` & `tests_job_breakaway_process_spawner.rs`
   - Enforce AGENTS.md process spawning & job breakaway invariant: multi-stage launcher fallback (target binary → detected base binary → Windows Shell dispatch `cmd.exe /c start`) combined with Windows Job Object breakaway fallback ladder (`os error 5: Access is denied` retry without `CREATE_BREAKAWAY_FROM_JOB`).
5. **Task 5 (`Imp-629`)**: `src-tauri/src/modules/db_orm/lock_timeout_escalator.rs` & `tests_lock_timeout_escalator.rs`
   - Enforce AGENTS.md bounded timeout and error visibility rules: adaptive SQLite split-DB lock timeout escalation under high concurrency, escalating from 250ms to 5000ms with jitter, routing exhausted locks into structured `AppError::DatabaseLocked`.

## Non-Negotiable Invariants
- **Zero-Build & Zero-Test Invariant**: Strictly no `cargo build`, `npm run build`, `cargo test`, or `npm test` during implementation.
- **Strictly Relative Paths**: Maintain 100% relative paths (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`).
- **No Silent Errors Invariant**: All fallible paths propagate via `AppError` or use `record_ignored` with a `// Justification:` comment.
- **Exclusively GitMap for Search**: Strictly `gitmap aum search`, `gitmap find`, and `gitmap cat`. Total ban on rg, ripgrep, grep, git grep, Select-String.
- **FIFO Ordering Invariant**: All queries must use `ORDER BY created_at ASC, id ASC`.
