# Sequence 206 Specification: Implementing Next Five Tasks (Steps 241–245)

## Overview
This specification details the design, constraints, and verification requirements for Batch 206 (Steps 241–245 of 255) in Antigravity-Manager.
All tasks originate directly from the non-negotiable architectural invariants codified in `AGENTS.md`.

### Tasks Summary
1. **Task 1 (`Imp-655`)**: `src-tauri/src/modules/instance/side_effect_order_enforcer.rs` & `tests_side_effect_order_enforcer.rs`
   - Enforce AGENTS.md Risky Path Replacement & Side Effects Ordering: Order side effects to fail before point of no return: validate credentials before killing process, validate destination before deleting source, enforce timeout boundaries on all operations.
2. **Task 2 (`Imp-656`)**: `src-tauri/src/modules/release_gate/branch_release_channel_gate.rs` & `tests_branch_release_channel_gate.rs`
   - Enforce AGENTS.md Release Channel Separation: intercept cross-branch misplacement, ensure stable releases (`vX.Y.Z`) originate exclusively on `main`, pre-releases (`vX.Y.Z-beta.N`) on `beta`, and tags match `CHANGELOG.md` headings character-for-character.
3. **Task 3 (`Imp-657`)**: `src-tauri/src/modules/release_gate/release_notes_relative_paths_linter.rs` & `tests_release_notes_relative_paths_linter.rs`
   - Enforce AGENTS.md Strictly Relative Git Paths Invariant: audit release notes, changelogs, and specs to intercept absolute filesystem paths (`D:\...`, `C:\...`, `/home/...`) or `file:///` URIs.
4. **Task 4 (`Imp-658`)**: `src-tauri/src/proxy/handlers/common/thinking_budget_proportional_allocator.rs` & `tests_thinking_budget_proportional_allocator.rs`
   - Enforce AGENTS.md Pipeline First & Thinking Budget Allocation: evaluate model-specific token ceilings (e.g. 64k vs 128k vs 8k context) and dynamically calculate proportional thinking budget vs max output tokens allocation.
5. **Task 5 (`Imp-659`)**: `src-tauri/src/modules/db_orm/fifo_prompt_restore_transaction_guard.rs` & `tests_fifo_prompt_restore_transaction_guard.rs`
   - Enforce AGENTS.md Strict FIFO Queue Invariant in Transactions: transaction-safe query wrapper ensuring prompt restore and queue batch dispatches always bundle `ORDER BY created_at ASC, id ASC` inside transactional rollbacks on sequence violations.

## Non-Negotiable Invariants
- **Zero-Build & Zero-Test Invariant**: Strictly no `cargo build`, `npm run build`, `cargo test`, or `npm test` during implementation.
- **Strictly Relative Paths**: Maintain 100% relative paths (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`).
- **No Silent Errors Invariant**: All fallible paths propagate via `AppError` or use `record_ignored` with a `// Justification:` comment.
- **Exclusively GitMap for Search**: Strictly `gitmap aum search`, `gitmap find`, and `gitmap cat`. Total ban on rg, ripgrep, grep, git grep, Select-String.
- **FIFO Ordering Invariant**: All prompt queries and transforms must enforce `ORDER BY created_at ASC, id ASC`.
