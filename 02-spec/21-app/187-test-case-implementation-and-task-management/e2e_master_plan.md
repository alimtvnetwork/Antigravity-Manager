# Master End-to-End Test Plan & Deferred Integration Registry (Sequence 187)

- **Slug**: `187-test-case-implementation-and-task-management`
- **Sequence**: 187
- **Status**: Enqueued for Deferred Execution (Triggered only upon explicit user command)
- **Execution Invariant**: Zero test runs during code authoring; full automated verification reserved for end-to-end testing phase.

---

## 1. Executive Summary

This catalog defines the master integration and end-to-end test scenarios covering the 10 steps implemented during Sequence 187. Any autonomous agent executing end-to-end verification must follow the step-by-step procedures outlined below, respecting transient sandboxing, non-interference with developer IDE instances, and clean state recovery.

---

## 2. Test Cases Specification

### E2E-187-01: UI Dropdown Menu Keyboard Opening, Navigation & Dark-Glass Styling
- **Category**: UI Accessibility & WAI-ARIA Menu (Track C Step 13)
- **Pre-Conditions**: `DropdownMenu` mounted in toolbar or detail header with interactive actions.
- **Stimulus**:
  1. Focus the trigger element via keyboard `Tab`.
  2. Press `Enter`, `Space`, or `ArrowDown` to toggle open menu.
  3. Verify `aria-expanded="true"` and `aria-haspopup="true"`.
  4. Press `Escape` or click outside to dismiss.
- **Expected Outcome**:
  - Menu opens smoothly with dark-glass backdrop blur styling.
  - Interactive items receive `role="menuitem"` with high-contrast focus rings.
  - Pressing `Escape` closes menu and restores clean state.

### E2E-187-02: UI Accordion Section WAI-ARIA Region Linkage & Keyboard Toggling
- **Category**: UI Accessibility & Disclosure Widgets (Track C Step 14)
- **Pre-Conditions**: `AccordionSection` mounted with title, subtitle, icon, and expandable body.
- **Stimulus**:
  1. Inspect header button attributes (`id`, `aria-expanded`, `aria-controls`).
  2. Inspect content container attributes (`id`, `role="region"`, `aria-labelledby`).
  3. Focus header and press `Enter` / `Space` to toggle expansion state.
- **Expected Outcome**:
  - `aria-controls` matches content container `id`.
  - `aria-labelledby` matches header button `id`.
  - Chevron rotates 180 degrees smoothly on toggle.
  - High-contrast focus-visible outline is rendered.

### E2E-187-03: UI Account Tier Badge Semantic Status & Contrast Verification
- **Category**: UI Accessibility & Semantic Status (Track C Step 15)
- **Pre-Conditions**: `TierBadge` rendered across Ultra, Pro, Free, and unknown account tiers.
- **Stimulus**:
  1. Inspect badge DOM elements for `role="status"` and `aria-label`.
  2. Verify icons have `aria-hidden="true"`.
  3. Audit contrast ratio in light and dark themes.
- **Expected Outcome**:
  - Screen readers announce `"Tier: Ultra"`, `"Tier: Pro"`, `"Tier: Free"`, or `"Tier: Unknown"`.
  - Gradient and solid badge backgrounds meet WCAG 2.1 AA text contrast requirements.

### E2E-187-04: UI Segmented Pill Switch Radiogroup Semantics & Arrow Key Navigation
- **Category**: UI Accessibility & Radiogroup Controls (Track C Step 16)
- **Pre-Conditions**: `SegmentedPillSwitch` mounted with multiple options in dialog or header.
- **Stimulus**:
  1. Inspect container attributes (`role="radiogroup"`, `aria-label`).
  2. Navigate across radio options using `ArrowRight` / `ArrowLeft` / `ArrowDown` / `ArrowUp`.
  3. Verify roving `tabIndex` (`tabIndex={0}` on selected, `tabIndex={-1}` on unselected).
- **Expected Outcome**:
  - Active selection updates on arrow key navigation.
  - Selection pill renders with dark-glass contrast and sky-400 focus rings.

### E2E-187-05: Proxy Request Header Sanitization & Hop-by-Hop Stripping
- **Category**: Reverse Proxy Security & Header Filtering (Track E Imp-583)
- **Pre-Conditions**: Inbound HTTP request received with mixed hop-by-hop, AGM telemetry, and payload headers.
- **Stimulus**:
  1. Execute `RequestHeaderSanitizer::sanitize(&headers)`.
- **Expected Outcome**:
  - Hop-by-hop headers (`connection`, `upgrade`, `keep-alive`, `transfer-encoding`, `host`, `content-length`) are completely removed.
  - Internal telemetry headers (`x-agm-*`) and client identity headers (`cookie`, `x-real-ip`, `x-forwarded-for`) are stripped.
  - Safe payload headers (`authorization`, `content-type`, `accept`) are preserved intact.

### E2E-187-06: Proxy Request Content-Length Bounds & Oversize Rejection
- **Category**: Reverse Proxy Security & Payload Bounds (Track E Imp-584)
- **Pre-Conditions**: Proxy receives requests with various `Content-Length` headers.
- **Stimulus**:
  1. Validate absent or chunked content-length (`validate_content_length(&headers, None)`).
  2. Validate normal content-length (4KB).
  3. Validate oversize payload exceeding ceiling (> 32MB).
  4. Validate malformed / non-numeric string values.
- **Expected Outcome**:
  - Absent header yields `ContentLengthValidation::ChunkedOrAbsent`.
  - Valid lengths yield `ContentLengthValidation::Valid(bytes)`.
  - Oversize payloads yield `ContentLengthValidation::ExceedsLimit`.
  - Malformed headers yield `ContentLengthValidation::Malformed`.

### E2E-187-07: Auto-Switcher Weekly Headroom Ratio & Multiplier Scaling
- **Category**: Auto-Switcher Routing & Headroom Economics (Track E Imp-585)
- **Pre-Conditions**: Auto-switcher evaluating account quota consumption pace over 168-hour rolling week.
- **Stimulus**:
  1. Calculate ratio on-track (quota percentage matching elapsed time slope).
  2. Calculate ratio under surplus (> 1.0) and deficit (< 1.0).
  3. Evaluate score multiplier clamping to [0.5, 1.5].
- **Expected Outcome**:
  - Linear ideal burn matches ~0.595% per hour (100% / 168h).
  - Surplus accounts receive favorable score multipliers up to 1.5.
  - Deficit accounts receive penalized score multipliers down to 0.5.

### E2E-187-08: Split-SQLite Index Cardinality Selectivity Audit
- **Category**: Database Architecture & Query Optimization (Track E Imp-586)
- **Pre-Conditions**: In-memory SQLite tables with varying column value distributions.
- **Stimulus**:
  1. Evaluate column cardinality on unique column (selectivity = 1.0).
  2. Evaluate column cardinality on low-cardinality status column with > 100 rows and < 5% distinct values.
  3. Evaluate column cardinality on empty table.
- **Expected Outcome**:
  - High-selectivity column yields `is_low_selectivity = false`.
  - Low-selectivity column with > 100 rows yields `is_low_selectivity = true`.
  - Empty table defaults to selectivity 1.0 without false positives.

### E2E-187-09: Split-SQLite Query Latency Histogram Bucketization & Reset
- **Category**: Database Telemetry & Performance Monitoring (Track E Imp-587)
- **Pre-Conditions**: `QueryLatencyHistogram` initialized in database connection pool.
- **Stimulus**:
  1. Record queries across 4 latency tiers (<1ms, 1-5ms, 5-20ms, >20ms).
  2. Capture atomic `snapshot()`.
  3. Call `reset()` and verify all counters return to 0.
- **Expected Outcome**:
  - Latencies bucketized into appropriate tier counters.
  - `total_queries` strictly matches sum of individual tier buckets.
  - Zero heap allocation occurs during recording.

### E2E-187-10: Sequence 187 Task Management & Backlog Audit Synchronization
- **Category**: Orchestration & Traceability Gate
- **Pre-Conditions**: Sequences 181–187 completed across Tracks A–E.
- **Stimulus**:
  1. Verify all 10 steps of Sequence 187 are documented and executed.
  2. Validate that zero builds and zero tests were run during authoring.
  3. Verify `.ai-memory/` and `02-spec/` indices reflect updated step counts.
- **Expected Outcome**:
  - Remaining backlog decrements from 135 to 125 steps.
  - All test artifacts and UI updates are committed and pushed cleanly to `main`.
