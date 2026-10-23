# Application Specification: Sequence 190 - Test Case Implementation & Task Management

## 1. Context & Objectives
This specification covers Sequence 190 of the master implementation roadmap for Antigravity-Manager. Following strict Zero-Build / Zero-Test disciplines, this sequence executes exactly 10 atomic steps, decrementing the overall task backlog from 105 steps to 95 steps.

## 2. Track Architecture & Backlog Accounting
- **Total Initial Backlog**: 255 steps across 5 functional tracks:
  - Track A: Boolean Logic Modernization & Guard Clause Inversion (Steps 1–25) - **100% Completed** (Sequence 183)
  - Track B: Multi-Instance Profile Sandboxing & Lifecycle Test Suites (Steps 1–25) - **100% Completed** (Sequence 184)
  - Track C: Desktop UI Accessibility, WCAG Contrast & Keyboard Navigation (Steps 1–25) - Steps 1–24 Completed; **Step 25 executed in Sequence 190 (Track C 100% COMPLETED)**
  - Track D: Release Management, Security Gates & CI/CD Compliance (Steps 1–25) - **Steps 1–4 executed in Sequence 190** (21 steps remaining: Steps 5–25)
  - Track E: Continuous Improvements Test Automation (Imp-561–680, 120 steps) - Imp-561–597 Completed; **Imp-598–601 executed in Sequence 190** (79 steps remaining: Imp-602–680)
- **Current Turn**: Sequence 190 (10 steps)
- **Remaining After Sequence 190**: 95 steps (Track C: 0 steps, Track D: 21 steps, Track E: 79 steps)

## 3. Detailed Scope of Sequence 190

### 3.1 UI Accessibility Modernization (Track C: Step 25 - Final Step)
1. **Step 1 (Track C Step 25)**: `src/components/common/MicroToast.tsx`
   - Add dynamic `role={variant === 'error' ? 'alert' : 'status'}`.
   - Add `aria-live={variant === 'error' ? 'assertive' : 'polite'}` and `aria-atomic="true"`.
   - Add accessible label `aria-label="Dismiss notification"` on dismiss button with visible focus outline.
   - Support hover and focus timer pausing to satisfy WCAG 2.2.1 Timing Adjustable.
   - Add `aria-hidden="true"` to decorative Lucide icons.

### 3.2 Release Management & Compliance Gates (Track D: Steps 1–4)
2. **Step 2 (Track D Step 1)**: Automated test suite for version bump manifest synchronization across `package.json`, `Cargo.toml`, and `tauri.conf.json` in `src-tauri/src/modules/release_gate/tests_version_manifest_sync.rs`.
3. **Step 3 (Track D Step 2)**: Automated test suite for release channel branch gating (`main` for stable releases, `beta` for pre-releases) in `src-tauri/src/modules/release_gate/tests_release_channel_gate.rs`.
4. **Step 4 (Track D Step 3)**: Automated test suite for author attribution compliance (strictly attributing `@aukgit` in changelog and release notes) in `src-tauri/src/modules/release_gate/tests_author_attribution.rs`.
5. **Step 5 (Track D Step 4)**: Automated test suite for repository-relative Git path hygiene (detecting and barring absolute filesystem paths like `D:\...`, `C:\...`, `/home/...`) in `src-tauri/src/modules/release_gate/tests_relative_path_linter.rs`.

### 3.3 Continuous Improvements Test Automation (Track E: Imp-598–601)
6. **Step 6 (Track E Imp-598)**: Unit tests for SSE chunk coalescer in `src-tauri/src/proxy/handlers/common/tests_sse_chunk_coalescer.rs`.
   - Verify chunk buffering across partial SSE event delimiters (`\n\n`, `\r\n\r\n`).
   - Verify `push_chunk`, `finish`, `buffered_len`, and lifetime `stats`.
7. **Step 7 (Track E Imp-599)**: Unit tests for SSE comment filter in `src-tauri/src/proxy/handlers/common/tests_sse_comment_filter.rs`.
   - Verify stripping of internal telemetry comments (`: keepalive\n\n`, `: ping\n`) while preserving `data:` and `event:` frames.
   - Verify `contains_sse_comment` detection logic.
8. **Step 8 (Track E Imp-600)**: Unit tests for auto-switcher settling timer in `src-tauri/src/modules/auto_switcher/tests_switch_settling_timer.rs`.
   - Verify default 10-second settling duration (`DEFAULT_SETTLING_DELAY_SECS`).
   - Verify custom duration initialization and `await_settling` timing.
9. **Step 9 (Track E Imp-601)**: Unit tests for split-SQLite incremental vacuum stepper in `src-tauri/src/modules/db_orm/tests_vacuum_stepper.rs`.
   - Verify zero freelist fast path.
   - Verify bounded chunk execution (`DEFAULT_VACUUM_CHUNK_SIZE = 100`) and report metrics.

### 3.4 End-to-End Master Catalog Enqueueing
10. **Step 10**: Enqueue master deferred E2E catalog `02-spec/21-app/190-test-case-implementation-and-task-management/e2e_master_plan.md` (`E2E-190-01` to `E2E-190-10`). Finalize plan in `.ai-memory/plans/completed/190-test-case-implementation-and-task-management.md`.

## 4. Invariants & Guardrails
- **Zero Builds / Zero Tests**: No running of `cargo build`, `cargo test`, `npm run build`, or `npm test`.
- **Search Exclusively via GitMap**: Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
- **Strictly Relative Paths**: All path references must remain relative to repository root.
- **Atomic 10-Step Batching**: Complete all 10 steps within the current turn and commit cleanly.
