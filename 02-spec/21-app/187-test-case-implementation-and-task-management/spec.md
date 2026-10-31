# Application Specification: Test Case Implementation and Task Management (Sequence 187)

- **Task**: Test Case Implementation and Task Management
- **Slug**: `187-test-case-implementation-and-task-management`
- **Sequence**: 187
- **Backlog Steps Remaining**: 125 steps (decrements from 135 to 125 upon completion)
- **Status**: [IN_PROGRESS]
- **Operational Constraint**: Zero testing and zero build checking during implementation. Never run `cargo test`, `cargo build`, `npm test`, or `npm run build`. Enqueue comprehensive E2E test plan for future AI execution upon command.

---

## 1. Executive Summary & Strategic Intent

Sequence 187 implements the next 10 consecutive steps across the Antigravity-Manager backlog:
1. **Track C (UI Accessibility & WCAG Contrast)**:
   - Modernize `src/components/common/DropdownMenu.tsx` with WAI-ARIA `aria-haspopup="true"`, `aria-expanded`, keyboard trigger activation (`Enter`/`Space`), and focus-visible rings (Step 1, Track C Step 13).
   - Modernize `src/components/common/AccordionSection.tsx` with `aria-labelledby`, region linkage, and high-contrast focus rings (Step 2, Track C Step 14).
   - Modernize `src/components/common/TierBadge.tsx` with semantic `role="status"` and accessible `aria-label` (Step 3, Track C Step 15).
   - Modernize `src/components/common/SegmentedPillSwitch.tsx` with accessible `aria-label="Options"`, role="radiogroup", and roving focus refinement (Step 4, Track C Step 16).
2. **Track E (Continuous Improvements Test Suites)**:
   - Author unit tests for request header sanitizer in `src-tauri/src/proxy/handlers/common/tests_request_header_sanitizer.rs` (Step 5, Imp-583).
   - Author unit tests for request content length bounds validator in `src-tauri/src/proxy/handlers/common/tests_request_content_length_validator.rs` (Step 6, Imp-584).
   - Author unit tests for weekly quota headroom ratio in `src-tauri/src/modules/auto_switcher/tests_weekly_headroom_ratio.rs` (Step 7, Imp-585).
   - Author unit tests for index cardinality analyzer in `src-tauri/src/modules/db_orm/tests_index_cardinality_analyzer.rs` (Step 8, Imp-586).
   - Author unit tests for query latency histogram in `src-tauri/src/modules/db_orm/tests_query_latency_histogram.rs` (Step 9, Imp-587).
3. **Orchestration**:
   - Author deferred Master End-to-End Test Plan (`02-spec/21-app/187-test-case-implementation-and-task-management/e2e_master_plan.md` covering `E2E-187-01` to `E2E-187-10`) (Step 10).

---

## 2. User Request (Verbatim)

```text
# Test Case Implementation and Task Management: high priority instruction, non-negotiable task

A lot of serious issues recently, which I have checked. Can you please first write all the test cases that you have recently implemented? All the pending tasks that we have, how many steps do you need to complete that? And can you please start doing 10 steps at a time as I will give you this prompt, and I repeat that, you will just complete and list out the completed task and also list out the pending task at the end. Can you please do that for me?

Also, do not build code, do not run any tests until at the end I will give a separate prompt. So in this case, you are going to implement the code and you are going to keep end-to-end testing as the last step. When I only ask for end-to-end test to run, this is when we are going to run. So when you complete any task, make sure that you enqueue or write as a plan for end-to-end testing with test cases so that any other AI can test it later on. Do you understand? Can you please follow through?

## task: Test Case Implementation and Task Management
## slug: test-case-implementation-and-task-management
```

---

## 3. Disjoint Subtask Allocation

- **Subtask 01 (Steps 1 & 2)**: UI Accessibility Modernization for DropdownMenu & AccordionSection (`src/components/common/DropdownMenu.tsx`, `src/components/common/AccordionSection.tsx`).
- **Subtask 02 (Steps 3 & 4)**: UI Accessibility Modernization for TierBadge & SegmentedPillSwitch (`src/components/common/TierBadge.tsx`, `src/components/common/SegmentedPillSwitch.tsx`).
- **Subtask 03 (Steps 5 & 6)**: Proxy Common Request Header Sanitizer & Content Length Validator Tests (`src-tauri/src/proxy/handlers/common/tests_request_header_sanitizer.rs`, `src-tauri/src/proxy/handlers/common/tests_request_content_length_validator.rs`).
- **Subtask 04 (Steps 7, 8 & 9)**: Weekly Headroom Ratio, Index Cardinality & Query Latency Histogram Tests (`src-tauri/src/modules/auto_switcher/tests_weekly_headroom_ratio.rs`, `src-tauri/src/modules/db_orm/tests_index_cardinality_analyzer.rs`, `src-tauri/src/modules/db_orm/tests_query_latency_histogram.rs`).
- **Subtask 05 (Step 10)**: Master Deferred E2E Catalog & Task Management Backlog Decrement.
