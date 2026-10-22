# Application Specification: Test Case Implementation and Task Management (Sequence 184)

- **Slug**: `184-test-case-implementation-and-task-management`
- **Sequence**: 184
- **Status**: [ACTIVE]
- **Operational Constraint**: Zero testing and zero build checking during implementation. Write test cases and code improvements without running cargo test or cargo build. Enqueue comprehensive E2E test plan for future AI execution upon command.

---

## 1. Executive Summary

Sequence 184 completes the final remaining step of **Track B** (Multi-Instance Sandboxing & Test Hardening), executes 4 core steps of **Track C** (UI Accessibility, Contrast & Audit Modernization), and implements 4 continuous improvement test suites in **Track E** (Imp-569 to Imp-572), concluding with master deferred E2E catalog generation.

---

## 2. 10-Step Execution Scope

### Subtask 01: Multi-Instance Ephemeral Sandbox Purger Test Suite (Track B Final Step)
1. **Step 1 (Track B - Step 25)**: Author comprehensive unit tests for ephemeral sandbox profile purger in `src-tauri/src/modules/instance/tests_sandbox_purger.rs`.
   - Test directory discovery matching `.antigravity-sandbox-`, `.test-profile-`, and `agm-test-` prefixes.
   - Test recursive byte size calculation for reclaimed storage.
   - Test exclusion of non-ephemeral regular profile directories.
   - Test graceful error handling on file lock collisions or missing root folders.
   - Register module in `src-tauri/src/modules/instance.rs`.
   - **Track B Completion**: Mark Track B 100% complete (25/25 steps).

### Subtask 02: UI Accessibility, Dark-Glass Contrast & Keyboard Navigation Modernization (Track C Steps 1–4)
2. **Step 2 (Track C - Step 1)**: Modernize `src/components/instances/PromptTreeView/TruncatedContextCallout.tsx`.
   - Enhance keyboard navigation with `tabIndex={0}`, `role="button"`, and Enter/Space keyboard handlers.
   - Modernize dark-glass styling to ensure high-contrast WCAG 2.1 AA compliant amber/slate contrast tokens.
   - Add accessible `aria-expanded` and semantic label hints.
3. **Step 3 (Track C - Step 2)**: Modernize `src/components/common/OriginTierBadgePill.tsx`.
   - Ensure high WCAG contrast across all 4 origin tiers (`USER_PROMPT`, `SUBAGENT_INSTRUCTION`, `SYSTEM_MESSAGE`, `TOOL_OUTPUT`).
   - Standardize border luminosity and dark-glass backdrop blur tokens.
4. **Step 4 (Track C - Step 3)**: Modernize `src/components/common/ExpandableCodeBlockBanner.tsx`.
   - Add keyboard accessibility to collapse header and copy action button (`role="button"`, `tabIndex={0}`, onKeyDown).
   - Ensure contrast of inline code text and gradient fade overlay.
5. **Step 5 (Track C - Step 4)**: Modernize `src/components/common/TruncatedTextTooltip.tsx`.
   - Ensure tooltip bubble contrast, interactive focus states, and screen reader announcements.

### Subtask 03: Upstream Request Size Guard & HTTP/2 Window Recovery Tests (Track E Steps 569 & 570)
6. **Step 6 (Track E - Imp-569)**: Author unit tests for request size guard in `src-tauri/src/proxy/handlers/common/tests_request_size_guard.rs`.
   - Test payload acceptance within default text ceiling (10 MB).
   - Test payload acceptance within multimodal ceiling (50 MB).
   - Test rejection on oversized payloads (`RejectedTooLarge`).
   - Test model name multimodal capability detection (`is_multimodal_model_name`).
   - Register module in `src-tauri/src/proxy/handlers/common/mod.rs`.
7. **Step 7 (Track E - Imp-570)**: Author unit tests for HTTP/2 window recovery monitor in `src-tauri/src/proxy/upstream/tests_h2_window_recovery.rs`.
   - Test consecutive flow-control stall recording.
   - Test threshold evaluation triggering HTTP/1.1 downgrade (`should_downgrade_to_http1`).
   - Test stall counter reset on stream success.
   - Register module in `src-tauri/src/proxy/upstream/mod.rs`.

### Subtask 04: Auto-Switcher Cooldown Decay Stepper & Split-SQLite Freelist Stepper Tests (Track E Steps 571 & 572)
8. **Step 8 (Track E - Imp-571)**: Author unit tests for stepped cooldown decay in `src-tauri/src/modules/auto_switcher/tests_cooldown_decay_stepper.rs`.
   - Test linear reduction within first half-life interval (< 5 minutes).
   - Test exponential 50% decay step after complete half-life.
   - Test full cooldown clearance when residual falls below floor threshold (5 seconds).
   - Register module in `src-tauri/src/modules/auto_switcher/mod.rs`.
9. **Step 9 (Track E - Imp-572)**: Author unit tests for split-SQLite freelist page stepper in `src-tauri/src/modules/db_orm/tests_free_page_stepper.rs`.
   - Test no-op when freelist is below threshold.
   - Test incremental vacuum stepping capped at `max_step_pages`.
   - Test database connection error propagation.
   - Register module in `src-tauri/src/modules/db_orm/mod.rs`.

### Subtask 05: Deferred E2E Master Plan & Backlog Enqueueing
10. **Step 10**: Author `02-spec/21-app/184-test-case-implementation-and-task-management/e2e_master_plan.md` defining tests `E2E-184-01` to `E2E-184-10`.
    - Recalculate remaining backlog to **155 steps**.
    - Finalize plan to `.ai-memory/plans/completed/184-test-case-implementation-and-task-management.md`.
    - Update plan index in `.ai-memory/plans/readme.md`.
