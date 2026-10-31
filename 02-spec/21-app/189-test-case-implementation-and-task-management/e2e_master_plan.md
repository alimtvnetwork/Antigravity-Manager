# Master End-to-End Test Plan & Deferred Integration Registry (Sequence 189)

- **Slug**: `189-test-case-implementation-and-task-management`
- **Sequence**: 189
- **Status**: Enqueued for Deferred Execution (Triggered only upon explicit user command)
- **Execution Invariant**: Zero test runs during code authoring; full automated verification reserved for end-to-end testing phase.

---

## 1. Executive Summary

This catalog defines the master integration and end-to-end test scenarios covering the 10 steps implemented during Sequence 189. Any autonomous agent executing end-to-end verification must follow the step-by-step procedures outlined below, respecting transient sandboxing, non-interference with developer IDE instances, and clean state recovery.

---

## 2. Test Cases Specification

### E2E-189-01: UI SearchInput Search Landmark, Accessible Clear & Escape Dismissal
- **Category**: UI Accessibility & Search Landmarks (Track C Step 21)
- **Pre-Conditions**: `SearchInput` mounted in workspace search or prompt filter bar.
- **Stimulus**:
  1. Inspect root container for `role="search"` and `aria-label="Search"`.
  2. Focus the input field and type a search term.
  3. Verify clear button appears with `aria-label="Clear search input"`.
  4. Press `Escape` key while focused on the text input.
- **Expected Outcome**:
  - `role="search"` landmark is detected by assistive technologies.
  - Pressing `Escape` instantly clears the text query and retains focus.
  - Clear button provides visible focus-ring and accessible label.

### E2E-189-02: UI Pagination Navigation Landmark, Page Status & Bound Controls
- **Category**: UI Accessibility & Navigation (Track C Step 22)
- **Pre-Conditions**: `Pagination` mounted with total pages > 1 and page size selector.
- **Stimulus**:
  1. Inspect container elements for `role="region"` and `aria-label="Pagination Navigation"`.
  2. Inspect mobile navigation for `<nav aria-label="Mobile Pagination">`.
  3. Check current page button for `aria-current="page"`.
  4. Navigate to boundary pages (page 1 and page total) and check `aria-disabled` on Prev/Next buttons.
  5. Inspect ellipsis elements to ensure `aria-hidden="true"` is present.
- **Expected Outcome**:
  - Mobile and desktop viewports both provide accessible navigation landmarks.
  - Active page is identified with `aria-current="page"`.
  - Ellipses are hidden from screen readers.
  - Page size select has accessible label.

### E2E-189-03: UI Toast Notification Dynamic Live Regions & Dismiss Labels
- **Category**: UI Accessibility & Status Alerts (Track C Step 23)
- **Pre-Conditions**: `Toast` component triggered with `type="error"` and `type="info"`.
- **Stimulus**:
  1. Trigger error notification: inspect root element for `role="alert"` and `aria-live="assertive"`.
  2. Trigger info notification: inspect root element for `role="status"` and `aria-live="polite"`.
  3. Inspect close button for `aria-label="Dismiss notification"`.
- **Expected Outcome**:
  - Screen readers immediately vocalize error alerts and politely announce info updates.
  - Dismiss button has accessible name and high-contrast dark-glass focus outline.

### E2E-189-04: UI ShortcutLegendCard Semantic Region, List Items & Phonetic Shortcuts
- **Category**: UI Accessibility & Keyboard Shortcuts (Track C Step 24)
- **Pre-Conditions**: `ShortcutLegendCard` mounted via shortcut overlay or settings view.
- **Stimulus**:
  1. Inspect container element for `role="region"` and `aria-label="Keyboard Shortcuts"`.
  2. Inspect shortcuts container for `role="list"` and individual items for `role="listitem"`.
  3. Verify screen-reader text (`<span className="sr-only">`) expands symbols like `⌘` to "Command" and `↵` to "Enter".
  4. Verify visual keys have `aria-hidden="true"`.
- **Expected Outcome**:
  - Screen readers identify and count available shortcuts accurately.
  - Modifier symbols are vocalized with natural language words without unintelligible symbol chatter.

### E2E-189-05: Proxy SSE Stream Keepalive & Flusher Inactivity Reset
- **Category**: Proxy Streaming Protocols (Track E Imp-593)
- **Pre-Conditions**: `StreamFlusher` configured with 15s keepalive threshold.
- **Stimulus**:
  1. Initialize `StreamFlusher` and probe `should_emit_keepalive()` immediately.
  2. Simulate elapsed idle period exceeding threshold.
  3. Invoke `record_data_sent()` and verify idle timer resets.
  4. Call `get_keepalive_frame()` and verify `: keepalive\n\n` byte literal output.
- **Expected Outcome**:
  - Stream flusher emits comments only after the configured silence duration.
  - Keepalive emission resets the timer and maintains active reverse proxy connections.

### E2E-189-06: Proxy Stream Early Abort Sentinel & Tokio Token Propagation
- **Category**: Stream Cancellation Coordination (Track E Imp-594)
- **Pre-Conditions**: `StreamEarlyAbortSentinel` instantiated in streaming handler.
- **Stimulus**:
  1. Verify initial un-aborted status (`is_aborted() == false`).
  2. Trigger abort via `trigger_abort()` or direct cancellation token.
  3. Verify `is_aborted()` reflects `true` across multiple thread clones.
  4. Pass child token to `tokio::select!` block and verify immediate cancellation branch execution.
- **Expected Outcome**:
  - Immediate upstream cancellation stops LLM generation and saves token quota.
  - Cross-thread synchronization is guaranteed via `SeqCst` ordering.

### E2E-189-07: Auto-Switcher Rotation Frequency Brake & Rolling Window Ceilings
- **Category**: Account Rotation & Oscillation Protection (Track E Imp-595)
- **Pre-Conditions**: `check_rotation_frequency` configured with default ceiling (5/hour).
- **Stimulus**:
  1. Supply empty history: verify `can_rotate == true` and `switches_in_past_hour == 0`.
  2. Supply 5 switches within the past 3600 seconds: verify `can_rotate == false`.
  3. Supply timestamps older than 3600 seconds: verify they are filtered out cleanly.
  4. Override with custom threshold: verify dynamic limit enforcement.
- **Expected Outcome**:
  - Account switching is throttled when rapid oscillation is detected.
  - Brake automatically disengages once older switch records age past 1 hour.

### E2E-189-08: Split-SQLite Concurrency Deadlock Detector & Busy Retry Cascade
- **Category**: Database Concurrency & Lock Contention (Track E Imp-596)
- **Pre-Conditions**: `LockSessionTracker` initialized for database transaction.
- **Stimulus**:
  1. Check initial state: verify 0 retries and `check_contention() == None`.
  2. Invoke `record_busy_retry()` 14 times: verify returned value is `false`.
  3. Invoke 15th retry: verify returned value flips to `true` (danger threshold).
  4. Configure immediate warning threshold and verify `check_contention()` returns elapsed duration.
- **Expected Outcome**:
  - Lock contention warnings are surfaced before reaching the 5,000ms SQLite busy timeout ceiling.
  - Retry cascade triggers victim selection and backoff routines.

### E2E-189-09: Split-SQLite Runtime Compile Options Introspection Scanner
- **Category**: Database Engine Diagnostics (Track E Imp-597)
- **Pre-Conditions**: SQLite in-memory or file database connection open.
- **Stimulus**:
  1. Invoke `inspect_sqlite_capabilities(&conn)`.
  2. Inspect returned `SqliteEngineCapabilities` struct.
  3. Verify JSON serialization round-trip.
- **Expected Outcome**:
  - `compile_options` list contains SQLite feature flags.
  - `is_threadsafe`, `has_json1`, and `has_fts5` flags accurately reflect the engine build.

### E2E-189-10: Sequence 189 Regression Gate & Task Backlog Invariant
- **Category**: Verification & Backlog Gate
- **Pre-Conditions**: Sequence 189 implementation complete across all 10 steps.
- **Stimulus**:
  1. Verify zero build commands (`cargo build`, `npm run build`) and zero test runners (`cargo test`, `npm test`) were executed.
  2. Verify all path references use repository-relative Git paths.
  3. Verify task backlog is decremented from 115 to 105 steps remaining.
- **Expected Outcome**:
  - 100% adherence to zero-build / zero-test authoring discipline.
  - Clean git status ready for atomic commit and push.
