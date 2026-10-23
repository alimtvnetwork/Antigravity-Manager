# Sequence 211 Specification: Implementing Next Five Tasks (Steps 266–270)

## Overview
This specification details the design, constraints, and verification requirements for Batch 211 (Steps 266–270) in Antigravity-Manager.
All tasks originate directly from the non-negotiable architectural invariants codified in `AGENTS.md`.

### Tasks Summary
1. **Task 1 (`Imp-680`)**: `src-tauri/src/modules/instance/windows_shell_dispatch_argument_escaper.rs` & `tests_windows_shell_dispatch_argument_escaper.rs`
   - Enforce AGENTS.md Process Spawning & Job Breakaway Invariant: safely escape and sanitize command arguments when dispatching via `cmd.exe /c start ""` fallback ladder.
2. **Task 2 (`Imp-681`)**: `src-tauri/src/modules/db_orm/fifo_queue_drain_monotonicity_verifier.rs` & `tests_fifo_queue_drain_monotonicity_verifier.rs`
   - Enforce AGENTS.md FIFO Invariant: verify that successive drained batches from SQLite queue tables maintain strictly non-decreasing `(created_at, id)` sequence.
3. **Task 3 (`Imp-682`)**: `src-tauri/src/proxy/handlers/common/streaming_chunk_boundary_guard.rs` & `tests_streaming_chunk_boundary_guard.rs`
   - Enforce AGENTS.md Pipeline First: enforce maximum chunk boundary capacity (1MB cap) and frame delimiter validation across SSE response streams.
4. **Task 4 (`Imp-683`)**: `src-tauri/src/modules/instance/truncation_omission_banner_formatter.rs` & `tests_truncation_omission_banner_formatter.rs`
   - Enforce AGENTS.md Minimalist UI: format raw `<truncated N bytes>` and `<truncated N lines>` tokens into rich interactive dark-glass UI omission banners.
5. **Task 5 (`Imp-684`)**: `src-tauri/src/modules/release_gate/individually_revertable_commit_guard.rs` & `tests_individually_revertable_commit_guard.rs`
   - Enforce AGENTS.md PR Scope & Grouping: audit commit series to verify each commit forms an independent, cohesive, and individually revertable unit.

## Non-Negotiable Invariants
- **Zero-Build & Zero-Test Invariant**: Strictly no `cargo build`, `npm run build`, `cargo test`, or `npm test` during implementation.
- **Strictly Relative Paths**: Maintain 100% relative paths (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`).
- **No Silent Errors Invariant**: All fallible paths propagate via `AppError` or use `record_ignored` with a `// Justification:` comment.
- **Exclusively GitMap for Search**: Strictly `gitmap aum search`, `gitmap find`, and `gitmap cat`. Total ban on rg, ripgrep, grep, git grep, Select-String.
- **FIFO Ordering Invariant**: All prompt queries must enforce `ORDER BY created_at ASC, id ASC`.
