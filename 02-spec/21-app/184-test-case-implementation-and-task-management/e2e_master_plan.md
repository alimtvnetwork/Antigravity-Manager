# Master End-to-End Test Plan & Deferred Integration Registry (Sequence 184)

- **Slug**: `184-test-case-implementation-and-task-management`
- **Sequence**: 184
- **Status**: Enqueued for Deferred Execution (Triggered only upon explicit user command)
- **Execution Invariant**: Zero test runs during code authoring; full automated verification reserved for end-to-end testing phase.

---

## 1. Executive Summary

This catalog defines the master integration and end-to-end test scenarios covering the 10 steps implemented during Sequence 184. Any autonomous agent executing end-to-end verification must follow the step-by-step procedures outlined below, respecting transient sandboxing, non-interference with developer IDE instances, and clean state recovery.

---

## 2. Test Cases Specification

### E2E-184-01: Multi-Instance Ephemeral Sandbox Cleanup & Recursive Reclamation
- **Category**: Multi-Instance Process Hygiene & Sandboxing (Track B Step 25)
- **Pre-Conditions**: Temporary instance sandbox folder containing directories with prefixes `.antigravity-sandbox-`, `.test-profile-`, and `agm-test-`.
- **Stimulus**:
  1. Invoke `purge_ephemeral_sandboxes(&sandbox_dir)`.
- **Expected Outcome**:
  - All matching ephemeral sandbox folders are recursively purged.
  - Reclaimed byte count equals total physical size of purged directories.
  - `deleted_paths` in report accurately reflects the purged directories.

### E2E-184-02: Multi-Instance Regular Profile Protection Invariant
- **Category**: Multi-Instance Sandboxing Safety
- **Pre-Conditions**: Instance directory containing regular production folders (`production_profile`, `user-data`, `default_workspace`) alongside ephemeral sandboxes.
- **Stimulus**:
  1. Invoke `purge_ephemeral_sandboxes(&instance_root)`.
- **Expected Outcome**:
  - Production and non-ephemeral folders are completely preserved and untouched.
  - Zero false-positive directory deletions.

### E2E-184-03: UI Truncated Context Callout Keyboard Navigation & ARIA Inspection
- **Category**: UI Accessibility & WCAG Contrast (Track C Step 1)
- **Pre-Conditions**: Rendered Prompt Tree containing omitted transcript context.
- **Stimulus**:
  1. Focus the callout banner using keyboard `Tab`.
  2. Press `Enter` and `Space`.
- **Expected Outcome**:
  - `role="button"` and `tabIndex={0}` receive active focus outline with clear visibility.
  - `onKeyDown` triggers full context expansion callback without mouse interaction.
  - `aria-label` announces omitted bytes/lines to screen readers.

### E2E-184-04: UI 4-Tier Origin Badge Contrast & Accessibility Verification
- **Category**: UI Accessibility & Design System (Track C Step 2)
- **Pre-Conditions**: Prompt Tree containing nodes of origin tiers `USER_PROMPT`, `SUBAGENT_INSTRUCTION`, `SYSTEM_MESSAGE`, and `TOOL_OUTPUT`.
- **Stimulus**:
  1. Inspect contrast ratios across both Light and Dark theme palettes.
  2. Query `role="status"` and `aria-label` elements.
- **Expected Outcome**:
  - Contrast ratio meets or exceeds WCAG 2.1 AA (>= 4.5:1) in both themes.
  - User badges render sky, subagent purple, system zinc, and tool amber.

### E2E-184-05: UI Expandable Code Block Banner Keyboard Control & Copy Interaction
- **Category**: UI Accessibility (Track C Step 3)
- **Pre-Conditions**: Code block with truncated content mounted in inspection panel.
- **Stimulus**:
  1. Navigate to header via `Tab` key and press `Enter`.
  2. Press `Tab` to copy button and press `Space`.
- **Expected Outcome**:
  - Header toggles expanded state (`aria-expanded="true"` / `"false"`).
  - Code is copied to clipboard and button provides visual `"Copied"` confirmation.

### E2E-184-06: UI Truncated Text Tooltip Focus State & High-Contrast Bubble Inspection
- **Category**: UI Accessibility (Track C Step 4)
- **Pre-Conditions**: Long identifier or hash rendered using `TruncatedTextTooltip`.
- **Stimulus**:
  1. Focus copy button using keyboard navigation.
  2. Inspect tooltip visibility and styling.
- **Expected Outcome**:
  - Tooltip bubble becomes visible on both focus and mouse hover.
  - High-contrast text is selectable and readable against dark/light backdrop.

### E2E-184-07: Upstream Client Request Size Guard & Multimodal Ceiling Enforcement
- **Category**: Upstream Protocol & Memory Guardrails (Track E Imp-569)
- **Pre-Conditions**: Client sending payloads with text and multimodal contents.
- **Stimulus**:
  1. Pass 12 MB text payload to `evaluate_request_size`.
  2. Pass 35 MB multimodal payload to `evaluate_request_size`.
- **Expected Outcome**:
  - 12 MB text payload is rejected with `RejectedTooLarge` (> 10 MB ceiling).
  - 35 MB multimodal payload is accepted (<= 50 MB ceiling).

### E2E-184-08: Upstream HTTP/2 Window Stall Recovery & Protocol Downgrade Ladder
- **Category**: Upstream Network Flow Control (Track E Imp-570)
- **Pre-Conditions**: HTTP/2 upstream connection recording repeated flow-control stalls.
- **Stimulus**:
  1. Record 3 consecutive stall events via `H2WindowRecoveryTracker`.
  2. Query `should_downgrade_to_http1()`.
- **Expected Outcome**:
  - Tracker hits threshold and flags `should_downgrade_to_http1 == true`.
  - Stream success clears counter back to zero.

### E2E-184-09: Auto-Switcher Stepped Cooldown Decay & Half-Life Clearance
- **Category**: Auto-Switcher Pool Recovery (Track E Imp-571)
- **Pre-Conditions**: Account in cooldown with clean idle time.
- **Stimulus**:
  1. Calculate decay after 1 half-life (300s).
  2. Calculate decay when residual falls under 5s.
- **Expected Outcome**:
  - 1 half-life scales residual linear duration by 50%.
  - Residual under 5 seconds clears cooldown completely to `Duration::ZERO`.

### E2E-184-10: Split-SQLite Incremental Vacuum Freelist Stepping
- **Category**: Split-SQLite Storage Maintenance (Track E Imp-572)
- **Pre-Conditions**: SQLite database configured with `auto_vacuum = INCREMENTAL` and fragmented freelist pages.
- **Stimulus**:
  1. Call `FreePageStepper::step_vacuum(&conn, threshold_pages, max_step_pages)`.
- **Expected Outcome**:
  - Reclaims physical storage in bounded increments without locking queries.
  - Returns accurate initial and final freelist counts.

---

## 3. Backlog Inventory Tracking (Sequence 184 Post-Completion)

With Sequence 184 completed, the remaining steps across all tracks are:

- **Track A (Boolean Modernization & Guard Inversion)**: **0 steps remaining** (COMPLETED)
- **Track B (Multi-Instance Sandboxing & Test Hardening)**: **0 steps remaining** (COMPLETED: All 25 steps done)
- **Track C (UI Accessibility, Contrast & Audit Modernization)**: **16 steps remaining** (decreased from 20)
- **Track D (Release Gates, Telemetry & Pipeline Guardrails)**: **15 steps remaining**
- **Track E (Continuous Improvements 573–720)**: **124 steps remaining** (Imp-573 to Imp-720)

**Total Remaining Backlog**: **155 steps** (decreased by 10 steps from 165).
