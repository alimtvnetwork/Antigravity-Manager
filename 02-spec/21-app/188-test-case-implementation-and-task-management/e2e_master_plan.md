# Master End-to-End Test Plan & Deferred Integration Registry (Sequence 188)

- **Slug**: `188-test-case-implementation-and-task-management`
- **Sequence**: 188
- **Status**: Enqueued for Deferred Execution (Triggered only upon explicit user command)
- **Execution Invariant**: Zero test runs during code authoring; full automated verification reserved for end-to-end testing phase.

---

## 1. Executive Summary

This catalog defines the master integration and end-to-end test scenarios covering the 10 steps implemented during Sequence 188. Any autonomous agent executing end-to-end verification must follow the step-by-step procedures outlined below, respecting transient sandboxing, non-interference with developer IDE instances, and clean state recovery.

---

## 2. Test Cases Specification

### E2E-188-01: UI Segmented Tab Group Roving WAI-ARIA Focus & Home/End Navigation
- **Category**: UI Accessibility & Tablist Navigation (Track C Step 17)
- **Pre-Conditions**: `SegmentedTabGroup` mounted in workspace navigation with 4 tabs.
- **Stimulus**:
  1. Focus active tab and press `ArrowRight` / `ArrowLeft`.
  2. Press `Home` key to jump to the first tab.
  3. Press `End` key to jump to the last tab.
- **Expected Outcome**:
  - `role="tablist"` container has descriptive `aria-label`.
  - Active tab receives `aria-selected="true"`, `tabIndex={0}`, and `aria-controls="tabpanel-<id>"`.
  - Inactive tabs receive `aria-selected="false"` and `tabIndex={-1}`.
  - Arrow, Home, and End keys update active tab cleanly with focus-visible outline.

### E2E-188-02: UI Segmented Capsule Radio Arrow Key Traversal & Focus Rings
- **Category**: UI Accessibility & Capsule Radios (Track C Step 18)
- **Pre-Conditions**: `SegmentedCapsuleRadio` mounted with multiple options in settings panel.
- **Stimulus**:
  1. Inspect container attributes (`role="radiogroup"`, `aria-label`).
  2. Navigate between radio buttons using arrow keys (`ArrowRight`/`ArrowLeft`/`ArrowDown`/`ArrowUp`).
  3. Press `Home` and `End` keys to navigate boundaries.
- **Expected Outcome**:
  - Active selection switches cleanly, skipping disabled options.
  - Selected capsule button receives `aria-checked="true"` and `tabIndex={0}`.
  - High-contrast sky-400 focus-visible outline is rendered.

### E2E-188-03: UI Modal Dialog Landmarks & Title Association
- **Category**: UI Accessibility & Accessible Dialogs (Track C Step 19)
- **Pre-Conditions**: `ModalDialog` opened via confirmation or action trigger.
- **Stimulus**:
  1. Inspect dialog root element for `role="dialog"`, `aria-modal="true"`, and `aria-labelledby="modal-dialog-title"`.
  2. Verify heading element has matching `id="modal-dialog-title"`.
  3. Verify Close, Cancel, and Confirm buttons render high-contrast focus rings.
- **Expected Outcome**:
  - Screen readers identify and announce modal title cleanly on mount.
  - Keyboard focus is trapped within modal until dismissed.

### E2E-188-04: UI Tooltip Bubble Programmatic Linkage & High-Contrast Tokens
- **Category**: UI Accessibility & Tooltip Linkage (Track C Step 20)
- **Pre-Conditions**: `TooltipBubble` wrapped around action button or status pill.
- **Stimulus**:
  1. Hover or focus the wrapped element.
  2. Inspect `aria-describedby` attribute on container matching the tooltip's `id`.
  3. Verify contrast tokens in light and dark themes.
- **Expected Outcome**:
  - Tooltip container mounts with `role="tooltip"` and matching unique ID.
  - High-contrast text (`text-slate-100` on `bg-slate-900/95`) ensures > 12:1 contrast ratio.

### E2E-188-05: Proxy Response Content-Type Classification & Standard Normalization
- **Category**: Reverse Proxy Protocols & Header Standards (Track E Imp-588)
- **Pre-Conditions**: Proxy receives upstream responses with varying Content-Type headers.
- **Stimulus**:
  1. Classify SSE, JSON, vendor JSON (`problem+json`), plain text, and binary headers.
  2. Normalize outgoing response headers (`normalize_content_type`).
- **Expected Outcome**:
  - Stream responses normalize to `"text/event-stream; charset=utf-8"`.
  - JSON responses normalize to `"application/json; charset=utf-8"`.
  - Missing headers default safely to specified canonical fallback.

### E2E-188-06: Proxy Compression Negotiation & SSE Stream Bypass
- **Category**: Reverse Proxy Compression & Latency Optimization (Track E Imp-589)
- **Pre-Conditions**: Inbound request with `Accept-Encoding: gzip, deflate, br`.
- **Stimulus**:
  1. Negotiate for real-time SSE stream (`is_streaming = true`).
  2. Negotiate for small payload (< 1KB).
  3. Negotiate for large payload (> 1KB) with Brotli and Gzip client support.
- **Expected Outcome**:
  - SSE streams always negotiate `Identity` encoding (`should_compress = false`) to eliminate buffering lag.
  - Sub-1KB payloads bypass compression.
  - Large payloads prioritize Brotli over Gzip over Deflate.

### E2E-188-07: Auto-Switcher Candidate Deduplication & Score Sorting
- **Category**: Auto-Switcher Routing & Deduplication (Track E Imp-590)
- **Pre-Conditions**: Auto-switcher candidate pool containing duplicate account bindings and aliases.
- **Stimulus**:
  1. Deduplicate records sharing identical `account_id` with differing scores.
  2. Deduplicate records sharing identical email (case-insensitive) under different IDs.
- **Expected Outcome**:
  - Retains the single candidate instance with the highest composite score.
  - Sorts resulting candidate slice in strictly descending composite score order.

### E2E-188-08: Split-SQLite Transaction Commit Latency Aggregation & Thresholds
- **Category**: Database Architecture & Fsync Latency Monitoring (Track E Imp-591)
- **Pre-Conditions**: `CommitLatencyObserver` initialized in SQLite transaction pipeline.
- **Stimulus**:
  1. Record standard commits (< 50ms).
  2. Record slow commits (> 50ms).
  3. Inspect aggregated stats (`total_commits_tracked`, `avg_commit_duration_ms`, `max_commit_duration_ms`, `slow_commits_gt_50ms`).
- **Expected Outcome**:
  - Aggregates count, average, and peak commit durations accurately.
  - Accurately tracks slow commits exceeding threshold without performance penalty.

### E2E-188-09: Split-SQLite Database File Size Watchdog & Threshold Auditing
- **Category**: Database Storage & WAL Explosive Growth Guard (Track E Imp-592)
- **Pre-Conditions**: Database storage directory with main database and associated `-wal` files.
- **Stimulus**:
  1. Inspect file size under threshold.
  2. Inspect file size exceeding threshold.
- **Expected Outcome**:
  - Sums main database bytes and WAL file bytes into `total_bytes`.
  - Evaluates `is_over_threshold` accurately against configured ceiling.

### E2E-188-10: Sequence 188 Task Management & Backlog Audit Synchronization
- **Category**: Orchestration & Traceability Gate
- **Pre-Conditions**: Sequences 181–188 completed across Tracks A–E.
- **Stimulus**:
  1. Verify all 10 steps of Sequence 188 are documented and executed.
  2. Validate that zero builds and zero tests were run during authoring.
  3. Verify `.ai-memory/` and `02-spec/` indices reflect updated step counts.
- **Expected Outcome**:
  - Remaining backlog decrements from 125 to 115 steps.
  - All test artifacts and UI updates are committed and pushed cleanly to `main`.
