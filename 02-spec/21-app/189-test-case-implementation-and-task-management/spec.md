# Application Specification: Sequence 189 - Test Case Implementation & Task Management

## 1. Context & Objectives
This specification covers Sequence 189 of the master implementation roadmap for Antigravity-Manager. Following strict Zero-Build / Zero-Test disciplines, this sequence executes exactly 10 atomic steps, decrementing the overall task backlog from 115 steps to 105 steps.

## 2. Track Architecture & Backlog Accounting
- **Total Initial Backlog**: 255 steps across 5 functional tracks:
  - Track A: Boolean Logic Modernization & Guard Clause Inversion (Steps 1–25) - **100% Completed** (Sequence 183)
  - Track B: Multi-Instance Profile Sandboxing & Lifecycle Test Suites (Steps 1–25) - **100% Completed** (Sequence 184)
  - Track C: Desktop UI Accessibility, WCAG Contrast & Keyboard Navigation (Steps 1–25) - Steps 1–20 Completed; **Steps 21–24 executed in Sequence 189** (1 step remaining: Step 25)
  - Track D: Release Management & Security Gates (Steps 1–25) - 25 steps remaining
  - Track E: Continuous Improvements Test Automation (Imp-561–680, 120 steps) - Imp-561–592 Completed; **Imp-593–597 executed in Sequence 189** (80 steps remaining: Imp-598–680)
- **Current Turn**: Sequence 189 (10 steps)
- **Remaining After Sequence 189**: 105 steps (Track C: 1 step, Track D: 25 steps, Track E: 80 steps)

## 3. Detailed Scope of Sequence 189

### 3.1 UI Accessibility & Keyboard Navigation (Track C: Steps 21–24)
1. **Step 1 (Track C Step 21)**: `src/components/common/SearchInput.tsx`
   - Add landmark `role="search"` and `aria-label="Search"`.
   - Add accessible label `aria-label="Clear search input"` to clear button.
   - Support `Escape` key handling to clear active query when focused.
   - Refine focus-visible ring styles for dark-glass contrast (`focus-within:ring-2 focus-within:ring-sky-500/30`).
2. **Step 2 (Track C Step 22)**: `src/components/common/Pagination.tsx`
   - Add `aria-label="Pagination"` to navigation containers.
   - Add `aria-current="page"` on current active page item.
   - Add explicit `aria-label="Previous page"` and `aria-label="Next page"` with `aria-disabled` on boundary buttons.
   - Add accessible labels for page size `<select>` and hide ellipses from assistive technology via `aria-hidden="true"`.
3. **Step 3 (Track C Step 23)**: `src/components/common/Toast.tsx`
   - Add dynamic `role={type === 'error' ? 'alert' : 'status'}` and `aria-live={type === 'error' ? 'assertive' : 'polite'}`.
   - Add `aria-atomic="true"` for atomic screen reader announcements.
   - Add accessible label `aria-label="Dismiss notification"` on the close button with focus-visible styling.
4. **Step 4 (Track C Step 24)**: `src/components/common/ShortcutLegendCard.tsx`
   - Add `role="region"` with `aria-label="Keyboard Shortcuts"`.
   - Add semantic `<kbd>` containers with screen reader hidden helpers (`<span className="sr-only">Shortcut keys: ...</span>`).
   - Add accessible close button `aria-label="Close keyboard shortcuts legend"`.

### 3.2 Continuous Improvements Test Automation (Track E: Imp-593–597)
5. **Step 5 (Track E Imp-593)**: Unit tests for SSE stream flusher in `src-tauri/src/proxy/handlers/common/tests_stream_flusher.rs`.
   - Verify default 15s keepalive interval and custom interval override.
   - Verify `record_data_sent()` resets idle timer and clears `should_emit_keepalive()`.
   - Verify `get_keepalive_frame()` yields `: keepalive\n\n` byte literal and resets timer.
6. **Step 6 (Track E Imp-594)**: Unit tests for stream early abort sentinel in `src-tauri/src/proxy/handlers/common/tests_stream_early_abort_sentinel.rs`.
   - Verify initial state (`is_aborted() == false`).
   - Verify `trigger_abort()` sets atomic flag to true and cancels `CancellationToken`.
   - Verify cloned instances share abortion state and cancellation propagation.
7. **Step 7 (Track E Imp-595)**: Unit tests for auto-switcher rotation frequency brake in `src-tauri/src/modules/auto_switcher/tests_rotation_brake.rs`.
   - Verify default 5 switches/hour ceiling.
   - Verify recent switches within 3600-second window are accurately counted while older switches are ignored.
   - Verify `can_rotate == false` when switch count reaches or exceeds limit.
8. **Step 8 (Track E Imp-596)**: Unit tests for SQLite concurrency deadlock detector in `src-tauri/src/modules/db_orm/tests_deadlock_detector.rs`.
   - Verify initial `LockSessionTracker` state with custom and default configurations.
   - Verify `record_busy_retry()` increments retry counter and signals danger once retry threshold is reached.
   - Verify `check_contention()` threshold duration check.
9. **Step 9 (Track E Imp-597)**: Unit tests for SQLite compile options introspection in `src-tauri/src/modules/db_orm/tests_compile_options.rs`.
   - Verify in-memory SQLite connection execution of `inspect_sqlite_capabilities`.
   - Verify capabilities struct fields: compile options list, threadsafe detection, JSON1/FTS5 detection.

### 3.3 End-to-End Master Catalog Enqueueing
10. **Step 10**: Enqueue master deferred E2E catalog in `02-spec/21-app/189-test-case-implementation-and-task-management/e2e_master_plan.md` defining tests `E2E-189-01` through `E2E-189-10`. Finalize plan in `.ai-memory/plans/completed/189-test-case-implementation-and-task-management.md`.

## 4. Invariants & Guardrails
- **Zero Builds / Zero Tests**: No running of `cargo build`, `cargo test`, `npm run build`, or `npm test`.
- **Search Exclusively via GitMap**: Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
- **Strictly Relative Paths**: All path references must remain relative to repository root.
- **Atomic 10-Step Batching**: Complete all 10 steps within the current turn and commit cleanly.
