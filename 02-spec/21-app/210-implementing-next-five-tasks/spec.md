# Sequence 210 Specification: Implementing Next Five Tasks (Steps 261–265)

## Overview
This specification details the design, constraints, and verification requirements for Batch 210 (Steps 261–265) in Antigravity-Manager.
All tasks originate directly from the non-negotiable architectural invariants codified in `AGENTS.md`.

### Tasks Summary
1. **Task 1 (`Imp-675`)**: `src-tauri/src/proxy/handlers/common/unified_context_alignment_guard.rs` & `tests_unified_context_alignment_guard.rs`
   - Enforce AGENTS.md Pipeline First & Unified Context Structural Alignment: inspect and validate multi-turn Gemini conversation payloads to ensure that role alternation strictly follows `user` -> `model` -> `user` sequence without adjacent identical roles, backfilling empty model turns or merging contiguous user turns into aligned payloads.
2. **Task 2 (`Imp-676`)**: `src-tauri/src/modules/instance/capsule_segmented_control_validator.rs` & `tests_capsule_segmented_control_validator.rs`
   - Enforce AGENTS.md Minimalist & Contextual UI: audit and validate component markup for toolbar and window controls, ensuring contiguous capsule container classes (`rounded-full`, shared border, subtle divider lines) and detecting nested pill-in-pill dropdown styling violations.
3. **Task 3 (`Imp-677`)**: `src-tauri/src/modules/release_gate/pr_problem_classification_linter.rs` & `tests_pr_problem_classification_linter.rs`
   - Enforce AGENTS.md PR Scope & Grouping: audit PR titles, descriptions, and file changesets to verify single-problem scope cohesion, detecting and flagging multi-concern PRs where unrelated functional domains (e.g. gateway proxy + release tooling + UI theming) are entangled.
4. **Task 4 (`Imp-678`)**: `src-tauri/src/modules/db_orm/fifo_prompt_restore_concurrency_limiter.rs` & `tests_fifo_prompt_restore_concurrency_limiter.rs`
   - Enforce AGENTS.md Strict FIFO Queue Invariant in Concurrent Restores: enforce concurrency limits and mutual exclusion locks during FIFO queue restore operations across multiple worker tasks or instance restarts, preventing race conditions or duplicate restores.
5. **Task 5 (`Imp-679`)**: `src-tauri/src/modules/release_gate/verified_fallback_path_linter.rs` & `tests_verified_fallback_path_linter.rs`
   - Enforce AGENTS.md Risky Path Replacement & Verified Fallback Invariant: audit replacement paths in critical operational modules to ensure every speculative optimization or accelerated path has an explicit fallback to the verified baseline path rather than an unconditional no-op.

## Non-Negotiable Invariants
- **Zero-Build & Zero-Test Invariant**: Strictly no `cargo build`, `npm run build`, `cargo test`, or `npm test` during implementation.
- **Strictly Relative Paths**: Maintain 100% relative paths (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`).
- **No Silent Errors Invariant**: All fallible paths propagate via `AppError` or use `record_ignored` with a `// Justification:` comment.
- **Exclusively GitMap for Search**: Strictly `gitmap aum search`, `gitmap find`, and `gitmap cat`. Total ban on rg, ripgrep, grep, git grep, Select-String.
- **FIFO Ordering Invariant**: All prompt queries must enforce `ORDER BY created_at ASC, id ASC`.
