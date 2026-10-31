# Application Specification: Sequence 214 (Steps 281–285)

## 1. Executive Summary
Sequence 214 implements and verifies 5 critical architectural, proxy streaming, UI capsule, database pagination, process execution, and contributor governance modules in `Antigravity-Manager` (`Imp-695` through `Imp-699`).
This specification satisfies the non-negotiable architectural mandates outlined in `AGENTS.md`.

---

## 2. Targeted Improvements & Step Mapping

| Step | Improvement ID | Category / Subsystem | Target File | Unit Test File | Description |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Step 281** | `Imp-695` | `proxy::handlers::common` | `src-tauri/src/proxy/handlers/common/streaming_payload_accumulator.rs` | `src-tauri/src/proxy/handlers/common/tests_streaming_payload_accumulator.rs` | Accumulates and validates streaming response chunks into cohesive Gemini candidate turns, tracking token boundaries, thinking block delimiters (`<thought>...</thought>`), and tool invocation fragments, bounding maximum stream buffer size to prevent memory bloat while ensuring complete turn delivery. |
| **Step 282** | `Imp-696` | `modules::instance` | `src-tauri/src/modules/instance/capsule_segmented_button_group_styler.rs` | `src-tauri/src/modules/instance/tests_capsule_segmented_button_group_styler.rs` | Enforces AGENTS.md rule: "When rendering adjacent toolbar actions or desktop window controls (minimize, maximize, close), always wrap them into contiguous segmented pill capsules (`rounded-full`, shared border, subtle divider lines, and dark-glass styling) rather than loose, disjointed circular buttons." |
| **Step 283** | `Imp-697` | `modules::db_orm` | `src-tauri/src/modules/db_orm/fifo_prompt_restore_cursor_stepper.rs` | `src-tauri/src/modules/db_orm/tests_fifo_prompt_restore_cursor_stepper.rs` | Implements a cursor-driven FIFO prompt restore query stepper (`WHERE (created_at > :cursor_time) OR (created_at = :cursor_time AND id > :cursor_id) ORDER BY created_at ASC, id ASC LIMIT :batch_size`) that enables deterministic paginated drains of prompt queues without unbounded memory usage or sequence inversions. |
| **Step 284** | `Imp-698` | `modules::instance` | `src-tauri/src/modules/instance/batch_file_execution_interceptor.rs` | `src-tauri/src/modules/instance/tests_batch_file_execution_interceptor.rs` | Enforces AGENTS.md rule: "Never pass `.cmd` or `.bat` batch files directly as the executable path in `std::process::Command::new(...)`. Multi-instance launchers must implement multi-stage fallback: target binary → detected base binary → Windows Shell dispatch (`cmd.exe /c start`)." |
| **Step 285** | `Imp-699` | `modules::release_gate` | `src-tauri/src/modules/release_gate/contributor_authorship_preservation_guard.rs` | `src-tauri/src/modules/release_gate/tests_contributor_authorship_preservation_guard.rs` | Enforces AGENTS.md rule: "Preserve authorship by preferring the contributor's own PR for squash commits, or attaching explicit `Co-authored-by:` trailers on merge commits and proxy PRs. Disclose costs before merging: highlight affected existing behaviors and unverified paths alongside improvements." |

---

## 3. Non-Negotiable Invariants Enforced
- **Zero-Build & Zero-Test Invariant**: Do NOT run `cargo build`, `npm run build`, `cargo test`, or `npm test` during implementation. All verification test commands are cataloged in `e2e_master_plan.md` for deferred execution.
- **Strictly Relative Git Paths**: Enforce 100% repository-relative Git paths (`02-spec/...`, `.ai-memory/...`, `src/...`, `src-tauri/...`) across all code, tests, docs, and git commit messages.
- **Exclusively GitMap for Search**: Strictly use `gitmap aum search`, `gitmap find`, and `gitmap cat`. Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
- **No Silent Errors Rule**: Every fallible path routes into `AppError` or uses `record_ignored` with an explicit `// Justification:` comment.
- **Strict FIFO Ordering Invariant**: Queue queries and cursor stepping must strictly enforce `ORDER BY created_at ASC, id ASC`.
