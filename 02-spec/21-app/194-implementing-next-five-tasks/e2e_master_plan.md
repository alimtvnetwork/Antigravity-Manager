# Master End-to-End Test Plan: Sequence 194 (Release Gate & Architectural Invariants)

## Overview
This document contains the deferred end-to-end test cases for Sequence 194 tasks.
Per the project's zero-build & zero-test invariant, compilation gates and execution of these test cases are strictly deferred until the user explicitly requests an end-to-end verification run via a dedicated prompt.

---

### Test Case E2E-194-01: FIFO Queue Query Ordering Verification
- **Objective**: Verify that prompt restore, dispatch, and queue processing queries strictly enforce `ORDER BY created_at ASC, id ASC` and that inverted LIFO patterns (`ORDER BY updated_at DESC`, `ORDER BY created_at DESC`) are rejected.
- **Pre-Conditions**: SQLite databases `active_prompts`, `prompt_backups`, and `repo_prompts.db` loaded with pending and backed up records.
- **Execution Steps**:
  1. Invoke `validate_fifo_dispatch_query("SELECT id FROM active_prompts WHERE status = 'backed_up' ORDER BY created_at ASC, id ASC")`.
  2. Confirm return is `Ok(FifoValidationReport)` with `is_valid: true`.
  3. Attempt validating an inverted query: `SELECT id FROM active_prompts WHERE status = 'queued' ORDER BY updated_at DESC`.
  4. Confirm validation fails with `FifoQueueViolation::ForbiddenLIFOOrdering`.
  5. Attempt validating a query missing secondary `id ASC` sort key.
  6. Confirm validation fails with `FifoQueueViolation::MissingDeterministicOrdering`.
- **Expected Outcome**: Only strictly monotonic FIFO queries pass validation; all LIFO queries fail fast.

---

### Test Case E2E-194-02: Segmented Capsule & Borderless Select UI Verification
- **Objective**: Verify that desktop window controls and toolbar action groups are rendered inside segmented pill capsules and that inline `<select>` elements use borderless transparent styling without pill-in-pill defects.
- **Pre-Conditions**: UI component rendering `TitleBar` or `NavSettings`.
- **Execution Steps**:
  1. Execute `validate_capsule_and_select_styling` on standard window controls markup containing `rounded-full` and `divide-x`.
  2. Confirm report returns `is_compliant: true` and `has_segmented_capsule: true`.
  3. Test loose un-encapsulated buttons (e.g. `<div className="flex gap-2"><button id="btn-window-minimize">...`).
  4. Confirm report fails with compliance violation explaining capsule grouping requirement.
  5. Test nested `<select>` element with non-transparent border/background (e.g. `bg-white border`).
  6. Confirm report flags `has_pill_in_pill_defect: true`.
- **Expected Outcome**: UI layout enforces contiguous segmented styling and eliminates pill-in-pill visual defects.

---

### Test Case E2E-194-03: 4-Tier Prompt Origin Hierarchy Verification
- **Objective**: Verify prompt origin classification across 4 tiers (`USER_PROMPT`, `SUBAGENT_INSTRUCTION`, `SYSTEM_MESSAGE`, `TOOL_OUTPUT`) and validate branch glyph nesting for subagents.
- **Pre-Conditions**: A conversation tree containing human directives, worker subagents, system preambles, and tool outputs.
- **Execution Steps**:
  1. Classify raw record types: `"USER_INPUT"` -> `OriginTier::UserPrompt`, `"SUBAGENT_INSTRUCTION"` -> `OriginTier::SubagentInstruction`, `"SYSTEM_MESSAGE"` -> `OriginTier::SystemMessage`, `"TOOL_CALLS"` -> `OriginTier::ToolOutput`.
  2. Verify tier metadata tokens: `UserPrompt` -> sky/user, `SubagentInstruction` -> purple/bot, `SystemMessage` -> zinc/terminal, `ToolOutput` -> amber/wrench.
  3. Format tree lines: depth 0 root node renders without prefix; depth 1 subagent renders with `├── ↳` or `└── ↳`.
  4. Validate hierarchy tree inputs: subagents lacking parent node or referencing missing parent IDs fail with `unparented_subagents` violation.
- **Expected Outcome**: Tree visualizer discriminates all 4 tiers with distinct tokens and branch glyphs.

---

### Test Case E2E-194-04: Interactive Truncation Callout Parser Verification
- **Objective**: Verify that raw `<truncated N bytes>` and `<truncated N lines>` markers are parsed and replaced with interactive dark-glass omission banners.
- **Pre-Conditions**: Raw transcript or response text containing inline truncation tags.
- **Execution Steps**:
  1. Pass text containing `<truncated 500 bytes>` to `parse_truncation_markers`.
  2. Verify callout record is extracted with `marker_type: TruncationType::Bytes` and `count: 500`.
  3. Verify transformed text replaces tag with `⚡ [Omitted 500 bytes of context - Click to inspect/expand]`.
  4. Pass text containing `<truncated 2048 bytes>`.
  5. Verify human-readable KB format: `⚡ [Omitted 2.0 KB of context - Click to inspect/expand]`.
  6. Pass text containing `<truncated 45 lines>`.
  7. Verify lines format: `⚡ [Omitted 45 lines of context - Click to inspect/expand]`.
  8. Confirm `contains_raw_markers_after_transform` is `false`.
- **Expected Outcome**: Zero raw truncation tags rendered; all markers become interactive dark-glass omission banners.

---

### Test Case E2E-194-05: No Silent Errors Linter Verification
- **Objective**: Verify that all fallible operations propagate errors or use `record_ignored` with mandatory `// Justification:` comments, and TypeScript error catches route through the error store.
- **Pre-Conditions**: Source code snippets in Rust and TypeScript.
- **Execution Steps**:
  1. Lint Rust snippet with bare `let _ = std::fs::remove_file(path)`.
  2. Verify violation `BareLetUnderscoreOnFallible` is flagged.
  3. Lint Rust snippet with `record_ignored(result, "context")` accompanied by preceding `// Justification: ...`.
  4. Verify report returns `is_compliant: true` and counts `justified_best_efforts_count: 1`.
  5. Lint Rust snippet with `record_ignored` lacking justification comment.
  6. Verify violation `MissingJustificationComment` is flagged.
  7. Lint TypeScript snippet with bare `catch {}`.
  8. Verify violation `BareCatchBlock` is flagged.
  9. Lint TypeScript snippet with `catch (e) { useErrorStore.getState().trackWarning(e, ...); }`.
  10. Verify report returns `is_compliant: true`.
- **Expected Outcome**: No silent errors swallowed; all fallibles logged or propagated with justification.
