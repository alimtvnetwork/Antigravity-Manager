# Sequence 208 Specification: Implementing Final Five Tasks (Steps 251–255 of 255)

## Overview
This specification details the design, constraints, and verification requirements for Batch 208 (Steps 251–255 of 255) in Antigravity-Manager.
This batch represents the final milestone completing the entire 255-step improvement backlog.
All tasks originate directly from the non-negotiable architectural invariants codified in `AGENTS.md`.

### Tasks Summary
1. **Task 1 (`Imp-665`)**: `src-tauri/src/modules/instance/settings_deep_merge_validator.rs` & `tests_settings_deep_merge_validator.rs`
   - Enforce AGENTS.md Headless & CLI Parity & Settings Deep-Merge Invariant: validate JSON settings synchronization trees, enforcing recursive deep-merge without overwriting orthogonal keys, preserving developer workspace states, and rejecting shallow overwrite shortcuts.
2. **Task 2 (`Imp-666`)**: `src-tauri/src/proxy/handlers/common/thinking_block_backfill_pipeline.rs` & `tests_thinking_block_backfill_pipeline.rs`
   - Enforce AGENTS.md Pipeline First & Thinking Block Backfilling: verify that streaming and buffered Gemini payloads from reasoning models (Claude 3.7 Sonnet, Gemini 2.0 Flash Thinking, o1/o3) have their `<thought>` / `thought: true` blocks backfilled and correctly positioned before payload delivery.
3. **Task 3 (`Imp-667`)**: `src-tauri/src/modules/release_gate/coauthor_trailer_alignment_guard.rs` & `tests_coauthor_trailer_alignment_guard.rs`
   - Enforce AGENTS.md Contributor Respect & Attribution: audit Git commit messages and PR merge commits to verify formatting of `Co-authored-by: Name <email>` trailers, ensure email validity, and verify no disallowed handles leak into trailers while preserving contributor authorship credit.
4. **Task 4 (`Imp-668`)**: `src-tauri/src/modules/db_orm/fifo_restore_transaction_coordinator.rs` & `tests_fifo_restore_transaction_coordinator.rs`
   - Enforce AGENTS.md Strict FIFO Queue Invariant in Transactions: coordinate multi-table SQLite FIFO restore transactions across `active_prompts`, `prompt_backups`, and `repo_prompts`, ensuring strict transactional atomicity and rollback on any detected out-of-order sequence ID or inverted timestamp.
5. **Task 5 (`Imp-669`)**: `src-tauri/src/modules/release_gate/unverified_paths_disclosure_guard.rs` & `tests_unverified_paths_disclosure_guard.rs`
   - Enforce AGENTS.md Contributor Respect & Disclosure: audit PR descriptions, release notes, and commit scopes to guarantee that non-trivial changesets explicitly enumerate affected existing behaviors and unverified test paths, intercepting silent omissions before merge.

## Non-Negotiable Invariants
- **Zero-Build & Zero-Test Invariant**: Strictly no `cargo build`, `npm run build`, `cargo test`, or `npm test` during implementation.
- **Strictly Relative Paths**: Maintain 100% relative paths (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`).
- **No Silent Errors Invariant**: All fallible paths propagate via `AppError` or use `record_ignored` with a `// Justification:` comment.
- **Exclusively GitMap for Search**: Strictly `gitmap aum search`, `gitmap find`, and `gitmap cat`. Total ban on rg, ripgrep, grep, git grep, Select-String.
- **FIFO Ordering Invariant**: All prompt restore and queue queries must enforce `ORDER BY created_at ASC, id ASC`.
