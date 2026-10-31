# Specification 194: Implementing Next Five Tasks (FIFO Queues, Capsule UI, Origin Tiers, Truncation & Error Visibility)

## 1. Overview & Context
This specification defines the architectural contracts and compliance verifiers for the 5 continuous compliance and reliability tasks in Sequence 194.
All tasks originate directly from the non-negotiable architectural invariants codified in `AGENTS.md`.

## 2. Five Planned Tasks Breakdown

### Task 1: FIFO Prompt Restore & Queue Invariant Guard
- **Module**: `src-tauri/src/modules/release_gate/fifo_queue_ordering_guard.rs`
- **Specification**:
  - All prompt restore, dispatch, and queue processing queries across SQLite databases (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`.
  - Inverted LIFO queries (`ORDER BY updated_at DESC`, `ORDER BY created_at DESC`) are strictly forbidden in dispatch logic.
  - Implement static analysis and SQL query validation routines checking for compliance and rejecting illegal LIFO patterns.

### Task 2: Minimalist Capsule & Transparent Dropdown Styling Guard
- **Module**: `src-tauri/src/modules/release_gate/capsule_dropdown_styling_guard.rs`
- **Specification**:
  - When rendering adjacent toolbar actions or desktop window controls (minimize, maximize, close), wrap them into contiguous segmented pill capsules (`rounded-full`, shared border, subtle divider lines, and dark-glass styling) rather than loose disjointed buttons.
  - Any inline `<select>` dropdowns mounted within segmented capsules MUST use transparent borderless styling (`bg-transparent border-0 focus:ring-0 focus:outline-none`) to eliminate nested white pill-in-pill visual defects.
  - Implement a structural JSX/HTML class validator that detects nested non-transparent `<select>` elements and verifies segmented capsule classes.

### Task 3: 4-Tier Prompt Origin Classification & Hierarchy Guard
- **Module**: `src-tauri/src/modules/release_gate/origin_tier_hierarchy_guard.rs`
- **Specification**:
  - Prompt tree views and inspectors must discriminate among 4 distinct origin tiers:
    1. `USER_PROMPT` (Sky badge, User icon)
    2. `SUBAGENT_INSTRUCTION` (Purple badge, Bot icon)
    3. `SYSTEM_MESSAGE` (Zinc badge, Terminal icon)
    4. `TOOL_OUTPUT` (Amber badge, Wrench icon)
  - Subagents must nest hierarchically beneath their parent user prompt with branch glyphs (`↳`, `└──`).
  - Implement an origin tier classifier and tree validator that guarantees correct badge tokens and tree branch formatting.

### Task 4: Interactive Truncation Callout Parser Guard
- **Module**: `src-tauri/src/modules/release_gate/truncation_callout_parser_guard.rs`
- **Specification**:
  - Never render raw `<truncated N bytes>` or `<truncated N lines>` markers as plain text.
  - Parse and render them as interactive dark-glass omission banners (`⚡ [Omitted N bytes/lines of context - Click to inspect/expand]`).
  - Implement a regex-based parser that extracts byte/line counts and transforms markers into structured callout records.

### Task 5: No Silent Errors Linter & Justification Gate
- **Module**: `src-tauri/src/modules/release_gate/no_silent_errors_linter.rs`
- **Specification**:
  - Every fallible operation must route its failure into the error module. Swallowing errors without routing is forbidden.
  - Rust: errors flow into `AppError` and are logged via `tracing`. Bare `let _ = <fallible>`, `.ok()`, or `.unwrap_or_default()` on a `Result` without routing are forbidden.
  - For genuinely best-effort ops: `crate::error::record_ignored(...)` plus a one-line `// Justification:` comment explaining safety are required.
  - TypeScript: failures must go through `captureError`/`captureException` or `trackWarning` (never a bare `catch {}`).
  - Implement a source code linter verifying error propagation and justification comments.

## 3. Invariants & Rules
- Zero builds and zero tests during implementation phase (Rule 1).
- Strictly repository-relative Git paths.
- Search exclusively via GitMap.
- All 5 tasks marked `IN PROGRESS (TAKEN)` and committed before assigning execution.
