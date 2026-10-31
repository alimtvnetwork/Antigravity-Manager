# Application Specification: Test Case Implementation and Task Management (Sequence 186)

- **Slug**: `186-test-case-implementation-and-task-management`
- **Sequence**: 186
- **Status**: [ACTIVE]
- **Operational Constraint**: Zero testing and zero build checking during implementation. Write test cases and code improvements without running cargo test or cargo build. Enqueue comprehensive E2E test plan for future AI execution upon command.

---

## 1. Executive Summary

Sequence 186 executes 4 additional modernization steps in **Track C** (UI Accessibility, Contrast & Audit Modernization) covering `LinearProgressBar`, `EmptyStatePlaceholder`, `NumberStepper`, and `ConfirmModal`, and implements 5 continuous improvement test suites in **Track E** (Imp-578 to Imp-582), concluding with master deferred E2E catalog generation.

---

## 2. 10-Step Execution Scope

### Subtask 01: UI Linear Progress Bar & Empty State Placeholder Modernization (Track C Steps 9 & 10)
1. **Step 1 (Track C - Step 9)**: Modernize `src/components/common/LinearProgressBar.tsx`.
   - Add accessible `aria-valuetext`, `aria-label="Progress"`, and `tabIndex={0}` focusable keyboard inspection.
   - Refine gradient and border luminosity tokens across Light and Dark themes.
2. **Step 2 (Track C - Step 10)**: Modernize `src/components/common/EmptyStatePlaceholder.tsx`.
   - Add semantic `role="status"` and `aria-live="polite"` landmarks.
   - Enhance contrast of icon container, title, description, and action button.

### Subtask 02: UI Number Stepper & Confirm Modal Modernization (Track C Steps 11 & 12)
3. **Step 3 (Track C - Step 11)**: Modernize `src/components/common/NumberStepper.tsx`.
   - Add explicit `aria-label="Decrease value"` and `aria-label="Increase value"` on buttons.
   - Add `focus-visible:ring-2 focus-visible:ring-sky-500/60` on stepper input and action buttons.
4. **Step 4 (Track C - Step 12)**: Modernize `src/components/common/ConfirmModal.tsx`.
   - Add `aria-labelledby` linking modal dialog shell to title header.
   - Add focus-visible rings on Cancel, Confirm, and Close buttons.

### Subtask 03: Upstream Dynamic DNS Resolver & Retry Decelerator Tests (Track E Steps 578 & 579)
5. **Step 5 (Track E - Imp-578)**: Author unit tests for dynamic DNS resolver in `src-tauri/src/proxy/upstream/tests_dynamic_dns_resolver.rs`.
   - Test address resolution and in-memory TTL caching.
   - Test cache sweep removing expired IP entries.
   - Register module in `src-tauri/src/proxy/upstream/mod.rs`.
6. **Step 6 (Track E - Imp-579)**: Author unit tests for retry decelerator in `src-tauri/src/proxy/upstream/tests_retry_decelerator.rs`.
   - Test exponential backoff calculation across retry attempts.
   - Test failure pressure penalty scaling with rolling failures.
   - Test max delay clamping and bounded jitter.
   - Register module in `src-tauri/src/proxy/upstream/mod.rs`.

### Subtask 04: Auto-Switcher Binding Affinity & Split-SQLite Schema Audits Tests (Track E Steps 580, 581 & 582)
7. **Step 7 (Track E - Imp-580)**: Author unit tests for account binding affinity in `src-tauri/src/modules/auto_switcher/tests_binding_affinity.rs`.
   - Test affinity bonus applied to active account on instance.
   - Test zero affinity bonus for distinct candidate accounts.
   - Test custom bonus score overrides.
   - Register module in `src-tauri/src/modules/auto_switcher/mod.rs`.
8. **Step 8 (Track E - Imp-581)**: Author unit tests for column default validator in `src-tauri/src/modules/db_orm/tests_column_default_validator.rs`.
   - Test audit of table column metadata via `PRAGMA table_info`.
   - Test detection of non-null columns missing default values (excluding PKs).
   - Register module in `src-tauri/src/modules/db_orm/mod.rs`.
9. **Step 9 (Track E - Imp-582)**: Author unit tests for index coverage verifier in `src-tauri/src/modules/db_orm/tests_index_coverage_verifier.rs`.
   - Test PRAGMA index_list and index_info parsing.
   - Test leading column query coverage evaluation (`is_query_covered`).
   - Register module in `src-tauri/src/modules/db_orm/mod.rs`.

### Subtask 05: Deferred E2E Master Plan & Backlog Enqueueing
10. **Step 10**: Author `02-spec/21-app/186-test-case-implementation-and-task-management/e2e_master_plan.md` defining tests `E2E-186-01` to `E2E-186-10`.
    - Recalculate remaining backlog to **135 steps**.
    - Finalize plan to `.ai-memory/plans/completed/186-test-case-implementation-and-task-management.md`.
    - Update plan index in `.ai-memory/plans/readme.md`.
