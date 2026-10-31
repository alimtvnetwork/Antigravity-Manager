# Sequence 212 Specification: Implementing Next Five Tasks (Steps 271–275)

## Overview
This specification details the design, constraints, and verification requirements for Batch 212 (Steps 271–275) in Antigravity-Manager.
All tasks originate directly from the non-negotiable architectural invariants codified in `AGENTS.md`.

### Tasks Summary
1. **Task 1 (`Imp-685`)**: `src-tauri/src/proxy/handlers/common/pipeline_response_header_sanitizer.rs` & `tests_pipeline_response_header_sanitizer.rs`
   - Enforce AGENTS.md Pipeline First & Header Sanitization: strips hop-by-hop and internal upstream response headers (`server`, `alt-svc`, `x-goog-safety-ratings`, `x-goog-ratelimit-remaining`, `set-cookie`) and normalizes standard Antigravity response headers (`content-type: text/event-stream; charset=utf-8`, `cache-control: no-cache`, `connection: keep-alive`).
2. **Task 2 (`Imp-686`)**: `src-tauri/src/modules/instance/window_control_capsule_formatter.rs` & `tests_window_control_capsule_formatter.rs`
   - Enforce AGENTS.md Minimalist & Contextual UI: wraps adjacent desktop window control actions (Minimize, Maximize, Close) into contiguous segmented pill capsules (`rounded-full`, shared border, subtle dividers, dark-glass styling) eliminating loose, disjointed circular buttons.
3. **Task 3 (`Imp-687`)**: `src-tauri/src/modules/db_orm/fifo_dead_letter_queue_spillover_guard.rs` & `tests_fifo_dead_letter_queue_spillover_guard.rs`
   - Enforce AGENTS.md FIFO Queue Invariant in Dead-Letter Handling: safely transitions unrecoverable prompts into dead-letter storage while strictly maintaining `ORDER BY created_at ASC, id ASC` ordering and preserving chronological lineage.
4. **Task 4 (`Imp-688`)**: `src-tauri/src/modules/instance/caller_process_ancestry_tree_scanner.rs` & `tests_caller_process_ancestry_tree_scanner.rs`
   - Enforce AGENTS.md Caller IDE Self-Preservation: traverses process PPID hierarchy upward to identify caller IDE process origin and protects the working developer session from termination or auto-switcher eviction.
5. **Task 5 (`Imp-689`)**: `src-tauri/src/modules/release_gate/github_release_notes_parity_guard.rs` & `tests_github_release_notes_parity_guard.rs`
   - Enforce AGENTS.md Documentation & Attribution and Strictly Relative Paths: verifies GitHub release notes strictly attribute `@aukgit`, intercept unauthorized third-party handles, reject absolute filesystem paths, and match corresponding `CHANGELOG.md` tag entries.

## Non-Negotiable Invariants
- **Zero-Build & Zero-Test Invariant**: Strictly no `cargo build`, `npm run build`, `cargo test`, or `npm test` during implementation.
- **Strictly Relative Paths**: Maintain 100% relative paths (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`).
- **No Silent Errors Invariant**: All fallible paths propagate via `AppError` or use `record_ignored` with a `// Justification:` comment.
- **Exclusively GitMap for Search**: Strictly `gitmap aum search`, `gitmap find`, and `gitmap cat`. Total ban on rg, ripgrep, grep, git grep, Select-String.
- **FIFO Ordering Invariant**: All prompt queries must enforce `ORDER BY created_at ASC, id ASC`.
