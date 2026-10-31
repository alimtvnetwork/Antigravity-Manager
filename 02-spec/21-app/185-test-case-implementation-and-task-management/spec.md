# Application Specification: Test Case Implementation and Task Management (Sequence 185)

- **Slug**: `185-test-case-implementation-and-task-management`
- **Sequence**: 185
- **Status**: [ACTIVE]
- **Operational Constraint**: Zero testing and zero build checking during implementation. Write test cases and code improvements without running cargo test or cargo build. Enqueue comprehensive E2E test plan for future AI execution upon command.

---

## 1. Executive Summary

Sequence 185 advances **Track C** (UI Accessibility, Contrast & Audit Modernization) by modernizing `SegmentedPillGroup`, `StatusBadge`, `CopyActionButton`, and `DarkGlassPopover`, and implements 5 continuous improvement test suites in **Track E** (Imp-573 to Imp-577), concluding with master deferred E2E catalog generation.

---

## 2. 10-Step Execution Scope

### Subtask 01: UI Segmented Pill Group & Status Badge Modernization (Track C Steps 5 & 6)
1. **Step 1 (Track C - Step 5)**: Modernize `src/components/common/SegmentedPillGroup.tsx`.
   - Add keyboard arrow key navigation (ArrowLeft / ArrowRight) for WAI-ARIA tablist accessibility.
   - Enforce proper `tabIndex` (`tabIndex={isActive ? 0 : -1}`) and focus ring styling.
2. **Step 2 (Track C - Step 6)**: Modernize `src/components/common/StatusBadge.tsx`.
   - Add semantic `role="status"` and accessible `aria-label` attributes.
   - Enhance text contrast across all 6 status variants (`running`, `queued`, `idle`, `error`, `success`, `cooldown`) for WCAG 2.1 AA compliance in both Light and Dark themes.

### Subtask 02: UI Copy Action Button & Dark-Glass Popover Modernization (Track C Steps 7 & 8)
3. **Step 3 (Track C - Step 7)**: Modernize `src/components/common/CopyActionButton.tsx`.
   - Add keyboard accessibility (Enter / Space trigger handlers).
   - Add `focus-visible:ring-2 focus-visible:ring-sky-500/60` and explicit `aria-label`.
4. **Step 4 (Track C - Step 8)**: Modernize `src/components/common/DarkGlassPopover.tsx`.
   - Add focus management: `tabIndex={-1}` on popover container and auto-focus on open.
   - Add `aria-label="Dialog popover"`.

### Subtask 03: Upstream Response Chunk Assembler & Connection Age Sentinel Tests (Track E Steps 573 & 574)
5. **Step 5 (Track E - Imp-573)**: Author unit tests for response chunk assembler in `src-tauri/src/proxy/upstream/tests_response_chunk_assembler.rs`.
   - Test sequential chunk accumulation and total length.
   - Test buffer ceiling enforcement returning Err when max_bytes is exceeded.
   - Test `finish()` consuming assembler and returning byte vector.
   - Register module in `src-tauri/src/proxy/upstream/mod.rs`.
6. **Step 6 (Track E - Imp-574)**: Author unit tests for connection age sentinel in `src-tauri/src/proxy/upstream/tests_connection_age_sentinel.rs`.
   - Test connection registration and initial state.
   - Test touch activity updating request count and last_used_at.
   - Test max age expiration returning true on touch.
   - Test `sweep_stale_connections()` removing expired entries.
   - Register module in `src-tauri/src/proxy/upstream/mod.rs`.

### Subtask 04: Auto-Switcher Tier Filter & Split-SQLite Column Validators Tests (Track E Steps 575, 576 & 577)
7. **Step 7 (Track E - Imp-575)**: Author unit tests for account tier filter in `src-tauri/src/modules/auto_switcher/tests_account_tier_filter.rs`.
   - Test inactive account exclusion.
   - Test thinking and vision capability requirements filtering.
   - Test model name pattern matching.
   - Test min tier requirement gating.
   - Register module in `src-tauri/src/modules/auto_switcher/mod.rs`.
8. **Step 8 (Track E - Imp-576)**: Author unit tests for column nullability validator in `src-tauri/src/modules/db_orm/tests_column_nullability_validator.rs`.
   - Test table inspection via `PRAGMA table_info`.
   - Test detection of missing NOT NULL constraints on required columns.
   - Test primary keys exempted from nullability violations.
   - Register module in `src-tauri/src/modules/db_orm/mod.rs`.
9. **Step 9 (Track E - Imp-577)**: Author unit tests for column type affinity validator in `src-tauri/src/modules/db_orm/tests_column_type_affinity_validator.rs`.
   - Test SQLite type affinity mapping (`INTEGER`, `TEXT`, `BLOB`, `REAL`, `NUMERIC`).
   - Test table inspection extracting column metadata and affinity.
   - Register module in `src-tauri/src/modules/db_orm/mod.rs`.

### Subtask 05: Deferred E2E Master Plan & Backlog Enqueueing
10. **Step 10**: Author `02-spec/21-app/185-test-case-implementation-and-task-management/e2e_master_plan.md` defining tests `E2E-185-01` to `E2E-185-10`.
    - Recalculate remaining backlog to **145 steps**.
    - Finalize plan to `.ai-memory/plans/completed/185-test-case-implementation-and-task-management.md`.
    - Update plan index in `.ai-memory/plans/readme.md`.
