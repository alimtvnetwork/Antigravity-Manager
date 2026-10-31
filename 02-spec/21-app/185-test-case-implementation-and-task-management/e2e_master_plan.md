# Master End-to-End Test Plan & Deferred Integration Registry (Sequence 185)

- **Slug**: `185-test-case-implementation-and-task-management`
- **Sequence**: 185
- **Status**: Enqueued for Deferred Execution (Triggered only upon explicit user command)
- **Execution Invariant**: Zero test runs during code authoring; full automated verification reserved for end-to-end testing phase.

---

## 1. Executive Summary

This catalog defines the master integration and end-to-end test scenarios covering the 10 steps implemented during Sequence 185. Any autonomous agent executing end-to-end verification must follow the step-by-step procedures outlined below, respecting transient sandboxing, non-interference with developer IDE instances, and clean state recovery.

---

## 2. Test Cases Specification

### E2E-185-01: UI Segmented Pill Group Keyboard Navigation & Selection Sync
- **Category**: UI Accessibility & WAI-ARIA Tablist (Track C Step 5)
- **Pre-Conditions**: Segmented pill group mounted with multiple options in dialog or navigation bar.
- **Stimulus**:
  1. Focus the active tab and press `ArrowRight` / `ArrowDown`.
  2. Press `ArrowLeft` / `ArrowUp` to navigate in reverse.
- **Expected Outcome**:
  - Selection updates cleanly across enabled options, wrapping around boundaries.
  - Active tab receives `tabIndex={0}` while inactive tabs receive `tabIndex={-1}`.
  - Focus ring renders with high contrast.

### E2E-185-02: UI Status Badge Semantic Roles & WCAG Contrast Evaluation
- **Category**: UI Accessibility & Contrast Tokens (Track C Step 6)
- **Pre-Conditions**: Badges rendered across all 6 status types (`running`, `queued`, `idle`, `error`, `success`, `cooldown`).
- **Stimulus**:
  1. Inspect contrast ratios across Dark and Light themes.
  2. Query `role="status"` and `aria-label` attributes.
- **Expected Outcome**:
  - Text contrast meets or exceeds WCAG 2.1 AA (4.5:1 minimum).
  - Screen reader announces status label clearly.

### E2E-185-03: UI Copy Action Button Keyboard Activation & Focus Ring Inspection
- **Category**: UI Accessibility & Keyboard Control (Track C Step 7)
- **Pre-Conditions**: CopyActionButton mounted across icon, minimal, and capsule variants.
- **Stimulus**:
  1. Focus button via `Tab` key and press `Enter` or `Space`.
- **Expected Outcome**:
  - Clipboard writes payload text cleanly.
  - Visual state transitions to `"Copied!"` with green check icon.
  - Focus ring renders with `sky-500/60` outline.

### E2E-185-04: UI Dark-Glass Popover Focus Containment & Auto-Focus Restoration
- **Category**: UI Accessibility & Focus Management (Track C Step 8)
- **Pre-Conditions**: Trigger button associated with DarkGlassPopover.
- **Stimulus**:
  1. Open popover and inspect document activeElement.
  2. Press `Escape` key.
- **Expected Outcome**:
  - Popover dialog receives focus on mount (`tabIndex={-1}`).
  - Escape closes popover and restores focus cleanly.

### E2E-185-05: Upstream Response Chunk Assembler Memory Guardrails & Stream Termination
- **Category**: Upstream Communication & Memory Bounds (Track E Imp-573)
- **Pre-Conditions**: Upstream response chunks arriving over streaming channel.
- **Stimulus**:
  1. Feed chunks into `ResponseChunkAssembler`.
  2. Test appending chunk exceeding configured max_bytes.
- **Expected Outcome**:
  - Chunks accumulate into continuous byte buffer.
  - Exceeding buffer limit rejects with error without consuming memory.
  - `finish()` returns collected bytes cleanly.

### E2E-185-06: Upstream Connection Max Age Retirement & Idle Sweeping
- **Category**: Upstream Connection Hygiene (Track E Imp-574)
- **Pre-Conditions**: ConnectionAgeSentinel tracking active gateway connections.
- **Stimulus**:
  1. Register connection and allow age to exceed max_age.
  2. Invoke `touch_connection` and `sweep_stale_connections`.
- **Expected Outcome**:
  - Touching connection past max_age returns `true` (retirement needed).
  - Sweeper removes expired connection IDs and logs sweep event.

### E2E-185-07: Auto-Switcher Account Capability Tier Filtering & Model Matching
- **Category**: Auto-Switcher Candidate Qualification (Track E Imp-575)
- **Pre-Conditions**: Array of candidate accounts with diverse capabilities and active states.
- **Stimulus**:
  1. Filter candidates for request requiring `gemini-2.0-flash` with thinking support.
- **Expected Outcome**:
  - Inactive accounts are excluded immediately.
  - Accounts lacking thinking or model support are filtered out.
  - Returns only fully qualified accounts.

### E2E-185-08: Split-SQLite Column Nullability Constraint Validation
- **Category**: Split-SQLite Schema Integrity (Track E Imp-576)
- **Pre-Conditions**: SQLite database with defined tables.
- **Stimulus**:
  1. Execute `validate_required_not_null_columns` for required columns.
- **Expected Outcome**:
  - Correctly flags columns lacking `NOT NULL` constraints.
  - Exempts primary key columns.
  - Returns structured `NullabilityViolation` records.

### E2E-185-09: Split-SQLite Column Type Affinity & Spec Section 3.1 Mapping
- **Category**: Split-SQLite Schema Standards (Track E Imp-577)
- **Pre-Conditions**: SQLite table with varied data types (`VARCHAR`, `INT`, `REAL`, `BLOB`, `BOOLEAN`).
- **Stimulus**:
  1. Call `inspect_table_columns(&conn, table_name)`.
- **Expected Outcome**:
  - Accurately maps declared types to `SqliteAffinity` enum values according to SQLite Section 3.1 rules.
  - Returns column metadata including primary key and nullability flags.

### E2E-185-10: End-to-End UI & Split-SQLite Concurrency Integration Verification
- **Category**: End-to-End Subsystem Cohesion
- **Pre-Conditions**: Complete application running in test mode with SQLite split databases.
- **Stimulus**:
  1. Verify schema validations pass across accounts and instances databases.
  2. Verify UI components mount without console warnings.
- **Expected Outcome**:
  - Clean execution with zero database schema violations or accessibility regressions.

---

## 3. Backlog Inventory Tracking (Sequence 185 Post-Completion)

With Sequence 185 completed, the remaining steps across all tracks are:

- **Track A (Boolean Modernization & Guard Inversion)**: **0 steps remaining** (COMPLETED)
- **Track B (Multi-Instance Sandboxing & Test Hardening)**: **0 steps remaining** (COMPLETED)
- **Track C (UI Accessibility, Contrast & Audit Modernization)**: **12 steps remaining** (decreased from 16)
- **Track D (Release Gates, Telemetry & Pipeline Guardrails)**: **15 steps remaining**
- **Track E (Continuous Improvements 578–720)**: **118 steps remaining** (Imp-578 to Imp-720)

**Total Remaining Backlog**: **145 steps** (decreased by 10 steps from 155).
