# Master End-to-End Test Plan & Deferred Integration Registry (Sequence 186)

- **Slug**: `186-test-case-implementation-and-task-management`
- **Sequence**: 186
- **Status**: Enqueued for Deferred Execution (Triggered only upon explicit user command)
- **Execution Invariant**: Zero test runs during code authoring; full automated verification reserved for end-to-end testing phase.

---

## 1. Executive Summary

This catalog defines the master integration and end-to-end test scenarios covering the 10 steps implemented during Sequence 186. Any autonomous agent executing end-to-end verification must follow the step-by-step procedures outlined below, respecting transient sandboxing, non-interference with developer IDE instances, and clean state recovery.

---

## 2. Test Cases Specification

### E2E-186-01: UI Linear Progress Bar Accessibility & Indeterminate State Announcement
- **Category**: UI Accessibility & WAI-ARIA Progressbar (Track C Step 9)
- **Pre-Conditions**: `LinearProgressBar` component mounted with determinate percentage and indeterminate loading mode.
- **Stimulus**:
  1. Inspect DOM attributes of the progress container in determinate mode.
  2. Inspect DOM attributes in indeterminate mode (`indeterminate={true}`).
  3. Verify keyboard focusability (`tabIndex={0}`) and focus ring styling.
- **Expected Outcome**:
  - `role="progressbar"` is present with `aria-valuenow`, `aria-valuemin={0}`, and `aria-valuemax={100}`.
  - `aria-valuetext` formats progress string accurately (e.g. `"45% complete"` or `"Loading..."`).
  - High-contrast background track (`bg-white/10 dark:bg-white/10`) and progress fill ensure clear visual perception.

### E2E-186-02: UI Empty State Placeholder Semantic Status & Action Keyboard Activation
- **Category**: UI Accessibility & Live Regions (Track C Step 10)
- **Pre-Conditions**: `EmptyStatePlaceholder` mounted inside empty list or filtered table view.
- **Stimulus**:
  1. Query component root element attributes.
  2. Focus the optional action button using `Tab` and press `Enter`.
- **Expected Outcome**:
  - Component root has `role="status"` and `aria-live="polite"`.
  - Icon has `aria-hidden="true"`.
  - Action button triggers assigned callback handler and displays visible `focus-visible:ring-2 focus-visible:ring-sky-500/60` outline.

### E2E-186-03: UI Number Stepper Keyboard Interaction & Screen Reader Labels
- **Category**: UI Accessibility & Spinbutton Controls (Track C Step 11)
- **Pre-Conditions**: `NumberStepper` mounted with min/max bounds and step increment.
- **Stimulus**:
  1. Navigate to decrement button, input field, and increment button via keyboard `Tab`.
  2. Press `Enter` or `Space` on increment button.
  3. Enter custom number into text input.
- **Expected Outcome**:
  - Buttons have explicit `aria-label="Decrease value"` and `aria-label="Increase value"`.
  - Input field has explicit `aria-label="Number input"`.
  - All interactive elements exhibit focus-visible rings with dark-glass tokens.

### E2E-186-04: UI Confirm Modal Dialog Landmarks & Accessible Name Resolution
- **Category**: UI Accessibility & Modal Dialogs (Track C Step 12)
- **Pre-Conditions**: `ConfirmModal` opened via user action.
- **Stimulus**:
  1. Inspect dialog root element attributes.
  2. Verify modal title element `id="confirm-modal-title"`.
  3. Press `Escape` key or click Close button.
- **Expected Outcome**:
  - Modal container has `role="dialog"`, `aria-modal="true"`, and `aria-labelledby="confirm-modal-title"`.
  - Close button has explicit `aria-label="Close dialog"`.
  - Action buttons render high-contrast focus rings and execute `onConfirm` / `onClose`.

### E2E-186-05: Upstream Dynamic DNS Resolver In-Memory TTL Caching & Sweep Verification
- **Category**: Upstream Network & DNS Resilience (Track E Imp-578)
- **Pre-Conditions**: Proxy upstream configured with `DynamicDnsResolver`.
- **Stimulus**:
  1. Resolve host `"127.0.0.1"` on port `8080`.
  2. Immediately resolve same host and port.
  3. Advance clock past TTL and call `sweep_expired()`.
- **Expected Outcome**:
  - Second lookup returns cached `SocketAddr` without performing redundant DNS queries.
  - Expired cache entries are successfully swept from internal HashMap.
  - Case-insensitive hostnames (`"LOCALHOST"` vs `"localhost"`) resolve to the same cached entry.

### E2E-186-06: Upstream Adaptive Retry Decelerator Delay & Failure Penalty Scaling
- **Category**: Upstream Network & Error Backoff (Track E Imp-579)
- **Pre-Conditions**: Multiple retry attempts simulated against failing upstream endpoint.
- **Stimulus**:
  1. Compute delay for attempt 0 with 0 failures.
  2. Compute delay for attempt 2 with 5 rolling failures.
  3. Compute delay for high attempt count exceeding `max_delay`.
- **Expected Outcome**:
  - Delay scales exponentially based on attempt index.
  - Each rolling endpoint failure increases delay by 20% pressure factor.
  - Delay is clamped by `max_delay` before bounded pseudo-random jitter is applied.

### E2E-186-07: Auto-Switcher Instance Binding Affinity Bonus Calculation
- **Category**: Auto-Switcher Routing & Sticky Sessions (Track E Imp-580)
- **Pre-Conditions**: Candidate selection evaluating active instance bindings.
- **Stimulus**:
  1. Evaluate affinity bonus where candidate account ID matches instance current account.
  2. Evaluate affinity bonus where candidate account ID differs.
  3. Provide custom bonus override score.
- **Expected Outcome**:
  - Matching account receives `15.0` default affinity bonus score.
  - Mismatched candidate receives `0.0` bonus.
  - Custom affinity bonus overrides default score accurately.

### E2E-186-08: Split-SQLite Table Column Default Constraint Audit
- **Category**: Database Architecture & Schema Integrity (Track E Imp-581)
- **Pre-Conditions**: In-memory or on-disk SQLite database containing tables with varying column constraints.
- **Stimulus**:
  1. Execute `ColumnDefaultValidator::audit_table(&conn, "users")`.
- **Expected Outcome**:
  - Correctly enumerates total columns and metadata (`cid`, `name`, `data_type`, `not_null`, `default_value`).
  - Identifies non-primary-key columns that are marked `NOT NULL` but lack `DEFAULT` specifications.
  - Returns empty list for properly constrained tables.

### E2E-186-09: Split-SQLite Composite Index Coverage Verification
- **Category**: Database Architecture & Query Performance (Track E Imp-582)
- **Pre-Conditions**: Database tables containing single-column and multi-column indexes.
- **Stimulus**:
  1. Inspect indices using `inspect_table_indices(&conn, "sessions")`.
  2. Evaluate `is_query_covered` against leading column filters and secondary column filters.
- **Expected Outcome**:
  - Successfully parses `PRAGMA index_list` and `PRAGMA index_info`.
  - Accurately reports `is_unique` and ordered column names.
  - Queries filtered by leading index column return `true`; queries missing leading index column return `false`.

### E2E-186-10: Sequence 186 Task Management & Backlog Audit Synchronization
- **Category**: Orchestration & Traceability Gate
- **Pre-Conditions**: Sequences 181–186 completed across Tracks A–E.
- **Stimulus**:
  1. Verify all 10 steps of Sequence 186 are documented and executed.
  2. Validate that zero builds and zero tests were run during authoring.
  3. Verify `.ai-memory/` and `02-spec/` indices reflect updated step counts.
- **Expected Outcome**:
  - Remaining backlog decrements from 145 to 135 steps.
  - All test artifacts and UI updates are committed and pushed cleanly to `main`.
