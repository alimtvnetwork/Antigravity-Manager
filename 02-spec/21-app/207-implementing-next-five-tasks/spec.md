# Sequence 207 Specification: Implementing Next Five Tasks (Steps 246–250)

## Overview
This specification details the design, constraints, and verification requirements for Batch 207 (Steps 246–250 of 255) in Antigravity-Manager.
All tasks originate directly from the non-negotiable architectural invariants codified in `AGENTS.md`.

### Tasks Summary
1. **Task 1 (`Imp-660`)**: `src-tauri/src/proxy/handlers/common/agent_prompt_sanitization_regex_engine.rs` & `tests_agent_prompt_sanitization_regex_engine.rs`
   - Enforce AGENTS.md regex-based prompt sanitization: replaces static keyword replacements with comprehensive regex pattern matching to sanitize prompt injections and instruction leaks while strictly preserving pipeline system prompts and user queries.
2. **Task 2 (`Imp-661`)**: `src-tauri/src/modules/instance/caller_ide_heartbeat_watchdog.rs` & `tests_caller_ide_heartbeat_watchdog.rs`
   - Enforce AGENTS.md caller IDE self-preservation invariant: tracks caller IDE process vitality, verifies heartbeats, and guarantees that auto-switcher daemons never evict, terminate, or starve active developer working sessions.
3. **Task 3 (`Imp-662`)**: `src-tauri/src/modules/release_gate/changelog_tag_monotonic_gate.rs` & `tests_changelog_tag_monotonic_gate.rs`
   - Enforce AGENTS.md release workflow & tag heading invariant: verifies that release tags match `CHANGELOG.md` headings character-for-character and monotonically progress over previous tags, preventing backwards versioning or cross-branch misplacement.
4. **Task 4 (`Imp-663`)**: `src-tauri/src/modules/db_orm/fifo_queue_index_analyzer.rs` & `tests_fifo_queue_index_analyzer.rs`
   - Enforce AGENTS.md strict First-In-First-Out database indexing: inspects table schema definitions across `active_prompts`, `prompt_backups`, and `repo_prompts.db`, verifying the presence of composite index `(created_at ASC, id ASC)` to prevent full-table sort overhead.
5. **Task 5 (`Imp-664`)**: `src-tauri/src/modules/release_gate/bounded_matcher_timeout_verifier.rs` & `tests_bounded_matcher_timeout_verifier.rs`
   - Enforce AGENTS.md risky path & bounded matcher invariant: audits pattern matchers to detect broadly and act narrowly, ensuring regex matchers do not cross line breaks or tag boundaries unconstrained, and bounding all wait loops with a timeout.

## Non-Negotiable Invariants
- **Zero-Build & Zero-Test Invariant**: Strictly no `cargo build`, `npm run build`, `cargo test`, or `npm test` during implementation.
- **Strictly Relative Paths**: Maintain 100% relative paths (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`).
- **No Silent Errors Invariant**: All fallible paths propagate via `AppError` or use `record_ignored` with a `// Justification:` comment.
- **Exclusively GitMap for Search**: Strictly `gitmap aum search`, `gitmap find`, and `gitmap cat`. Total ban on rg, ripgrep, grep, git grep, Select-String.
- **FIFO Ordering Invariant**: All prompt restore queries must enforce `ORDER BY created_at ASC, id ASC`.
