# Application Specification: Sequence 213 (Steps 276–280)

## 1. Executive Summary
Sequence 213 implements and verifies 5 critical architectural, UI, database, caller protection, and release governance modules in `Antigravity-Manager` (`Imp-690` through `Imp-694`).
This specification satisfies the non-negotiable architectural mandates outlined in `AGENTS.md`.

---

## 2. Targeted Improvements & Step Mapping

| Step | Improvement ID | Category / Subsystem | Target File | Unit Test File | Description |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Step 276** | `Imp-690` | `proxy::handlers::common` | `src-tauri/src/proxy/handlers/common/streaming_event_frame_parser.rs` | `src-tauri/src/proxy/handlers/common/tests_streaming_event_frame_parser.rs` | Parses and validates raw SSE / HTTP event streams into discrete Gemini protocol chunks, verifying boundary frames, multi-line `data: ` blocks according to W3C SSE standard, and guaranteeing zero event loss during proxy streaming. |
| **Step 277** | `Imp-691` | `modules::instance` | `src-tauri/src/modules/instance/inline_select_dropdown_styler.rs` | `src-tauri/src/modules/instance/tests_inline_select_dropdown_styler.rs` | Enforces AGENTS.md rule: "Any inline `<select>` dropdowns mounted within segmented capsules MUST use transparent borderless styling (`bg-transparent border-0 focus:ring-0 focus:outline-none`) to eliminate nested white pill-in-pill visual defects." |
| **Step 278** | `Imp-692` | `modules::db_orm` | `src-tauri/src/modules/db_orm/fifo_queue_compaction_guard.rs` | `src-tauri/src/modules/db_orm/tests_fifo_queue_compaction_guard.rs` | Coordinates SQLite database queue compaction and tombstone cleanup (`active_prompts`, `prompt_backups`), ensuring that compaction passes NEVER re-order rows, strictly maintain monotonically increasing `created_at ASC, id ASC`, and safely purge acknowledged rows. |
| **Step 279** | `Imp-693` | `modules::instance` | `src-tauri/src/modules/instance/caller_ide_exit_code_enforcer.rs` | `src-tauri/src/modules/instance/tests_caller_ide_exit_code_enforcer.rs` | Enforces AGENTS.md rule: "Commands targeting the caller instance (`agm instances stop`, `rm`, `switch`) must fail-fast with exit code 2 unless `--force` is explicitly provided." |
| **Step 280** | `Imp-694` | `modules::release_gate` | `src-tauri/src/modules/release_gate/transient_test_sandbox_lifecycle_guard.rs` | `src-tauri/src/modules/release_gate/tests_transient_test_sandbox_lifecycle_guard.rs` | Enforces AGENTS.md rule: "Integration tests must always generate transient, isolated test sandboxes (`agm instances test-create`) and clean them up completely (`agm instances test-clean`), leaving zero orphan processes or filesystem remnants." |

---

## 3. Non-Negotiable Invariants Enforced
- **Zero-Build & Zero-Test Invariant**: Do NOT run `cargo build`, `npm run build`, `cargo test`, or `npm test` during implementation. All verification test commands are cataloged in `e2e_master_plan.md` for deferred execution.
- **Strictly Relative Git Paths**: Enforce 100% repository-relative Git paths (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`) across all code, tests, docs, and git commit messages.
- **Exclusively GitMap for Search**: Strictly use `gitmap aum search`, `gitmap find`, and `gitmap cat`. Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
- **No Silent Errors Rule**: Every fallible path routes into `AppError` or uses `record_ignored` with an explicit `// Justification:` comment.
- **Strict FIFO Ordering Invariant**: Queue compaction and drains must strictly enforce `ORDER BY created_at ASC, id ASC`.
