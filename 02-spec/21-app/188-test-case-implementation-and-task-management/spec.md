# Application Specification: Test Case Implementation and Task Management (Sequence 188)

- **Task**: Test Case Implementation and Task Management
- **Slug**: `188-test-case-implementation-and-task-management`
- **Sequence**: 188
- **Backlog Steps Remaining**: 115 steps (decrements from 125 to 115 upon completion)
- **Status**: [IN_PROGRESS]
- **Operational Constraint**: Zero testing and zero build checking during implementation. Never run `cargo test`, `cargo build`, `npm test`, or `npm run build`. Enqueue comprehensive E2E test plan for future AI execution upon command.

---

## 1. Executive Summary & Strategic Intent

Sequence 188 implements the next 10 consecutive steps across the Antigravity-Manager backlog:
1. **Track C (UI Accessibility & WCAG Contrast)**:
   - Modernize `src/components/common/SegmentedTabGroup.tsx` with WAI-ARIA `role="tablist"`, `aria-label`, arrow key navigation, and dark-glass contrast tokens (Step 1, Track C Step 17).
   - Modernize `src/components/common/SegmentedCapsuleRadio.tsx` with roving `tabIndex`, `role="radiogroup"`, `aria-label`, and focus-visible rings (Step 2, Track C Step 18).
   - Modernize `src/components/common/ModalDialog.tsx` with accessible dialog landmarks, title linking (`aria-labelledby="modal-dialog-title"`), and high-contrast button rings (Step 3, Track C Step 19).
   - Modernize `src/components/common/TooltipBubble.tsx` with accessible tooltip ID linkage via `useId()` and `role="tooltip"` (Step 4, Track C Step 20).
2. **Track E (Continuous Improvements Test Suites)**:
   - Author unit tests for response content type matcher in `src-tauri/src/proxy/handlers/common/tests_response_content_type_matcher.rs` (Step 5, Imp-588).
   - Author unit tests for compression negotiator in `src-tauri/src/proxy/handlers/common/tests_compression_negotiator.rs` (Step 6, Imp-589).
   - Author unit tests for auto-switcher candidate deduplicator in `src-tauri/src/modules/auto_switcher/tests_candidate_dedup.rs` (Step 7, Imp-590).
   - Author unit tests for transaction commit latency observer in `src-tauri/src/modules/db_orm/tests_commit_latency_observer.rs` (Step 8, Imp-591).
   - Author unit tests for split-SQLite database size watchdog in `src-tauri/src/modules/db_orm/tests_db_size_watchdog.rs` (Step 9, Imp-592).
3. **Orchestration**:
   - Author deferred Master End-to-End Test Plan (`02-spec/21-app/188-test-case-implementation-and-task-management/e2e_master_plan.md` covering `E2E-188-01` to `E2E-188-10`) (Step 10).

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

- **Subtask 01 (Steps 1 & 2)**: UI Accessibility Modernization for SegmentedTabGroup & SegmentedCapsuleRadio (`src/components/common/SegmentedTabGroup.tsx`, `src/components/common/SegmentedCapsuleRadio.tsx`).
- **Subtask 02 (Steps 3 & 4)**: UI Accessibility Modernization for ModalDialog & TooltipBubble (`src/components/common/ModalDialog.tsx`, `src/components/common/TooltipBubble.tsx`).
- **Subtask 03 (Steps 5 & 6)**: Proxy Response Content Type Matcher & Compression Negotiator Tests (`src-tauri/src/proxy/handlers/common/tests_response_content_type_matcher.rs`, `src-tauri/src/proxy/handlers/common/tests_compression_negotiator.rs`).
- **Subtask 04 (Steps 7, 8 & 9)**: Candidate Dedup, Commit Latency Observer & DB Size Watchdog Tests (`src-tauri/src/modules/auto_switcher/tests_candidate_dedup.rs`, `src-tauri/src/modules/db_orm/tests_commit_latency_observer.rs`, `src-tauri/src/modules/db_orm/tests_db_size_watchdog.rs`).
- **Subtask 05 (Step 10)**: Master Deferred E2E Catalog & Task Management Backlog Decrement.
