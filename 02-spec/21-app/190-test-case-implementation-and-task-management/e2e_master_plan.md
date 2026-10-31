# Master End-to-End Test Plan & Deferred Integration Registry (Sequence 190)

- **Slug**: `190-test-case-implementation-and-task-management`
- **Sequence**: 190
- **Status**: Enqueued for Deferred Execution (Triggered only upon explicit user command)
- **Execution Invariant**: Zero test runs during code authoring; full automated verification reserved for end-to-end testing phase.

---

## 1. Executive Summary

This catalog defines the master integration and end-to-end test scenarios covering the 10 steps implemented during Sequence 190. Any autonomous agent executing end-to-end verification must follow the step-by-step procedures outlined below, respecting transient sandboxing, non-interference with developer IDE instances, and clean state recovery.

---

## 2. Test Cases Specification

### E2E-190-01: UI MicroToast Accessibility, Politeness Levels & Hover Pause
- **Category**: UI Accessibility & Live Notifications (Track C Step 25 - Final Step)
- **Pre-Conditions**: `MicroToast` mounted with variant `error` and variant `success`.
- **Stimulus**:
  1. Trigger error notification: verify `role="alert"`, `aria-live="assertive"`, and `aria-atomic="true"`.
  2. Trigger success notification: verify `role="status"`, `aria-live="polite"`.
  3. Hover or focus notification and verify countdown timer pauses.
  4. Move cursor away and verify countdown timer resumes.
  5. Inspect dismiss button for `aria-label="Dismiss notification"` and `focus-visible:ring-2 focus-visible:ring-sky-500/60`.
- **Expected Outcome**:
  - Critical errors are announced immediately, while non-error toasts announce politely.
  - Users can pause notifications on hover/focus to satisfy WCAG 2.2.1 Timing Adjustable.
  - Track C is 100% completed.

### E2E-190-02: Release Gate Version Manifest Synchronization Check
- **Category**: Release Compliance & Manifest Gating (Track D Step 1)
- **Pre-Conditions**: Manifest versions extracted from `package.json`, `Cargo.toml`, and `tauri.conf.json`.
- **Stimulus**:
  1. Execute `verify_manifest_versions` with matching version triples (`4.250.0`).
  2. Execute with mismatched version in `Cargo.toml`.
  3. Execute with mismatched version in `tauri.conf.json`.
  4. Verify missing manifest returns `is_synced: false`.
- **Expected Outcome**:
  - Synchronized manifests yield `is_synced == true`.
  - Any version drift is immediately intercepted prior to tagging.

### E2E-190-03: Release Gate Branch Channel Isolation Gate
- **Category**: Release Compliance & Branch Gating (Track D Step 2)
- **Pre-Conditions**: Git release tag and targeting branch provided.
- **Stimulus**:
  1. Validate stable tag `v4.250.0` on `main`: verify `is_valid == true` and `channel == Stable`.
  2. Validate stable tag `v4.250.0` on `beta`: verify `is_valid == false` with descriptive error.
  3. Validate preview tag `v4.250.0-beta.1` on `beta`: verify `is_valid == true` and `channel == Preview`.
  4. Validate preview tag on `main`: verify `is_valid == false`.
- **Expected Outcome**:
  - Stable releases are strictly gated to `main` branch.
  - Preview releases are strictly gated to `beta` branch.

### E2E-190-04: Release Gate Author Attribution Exclusivity Verifier
- **Category**: Release Compliance & Contributor Attribution (Track D Step 3)
- **Pre-Conditions**: Changelog text content provided for release note generation.
- **Stimulus**:
  1. Scan text containing `(Thanks to @aukgit)`: verify `is_compliant == true`.
  2. Scan text containing other contributor `@randomdev`: verify `is_compliant == false` and handle is captured.
  3. Verify case-insensitivity (`@AUKGIT` passes).
- **Expected Outcome**:
  - Only `@aukgit` is allowed as a contributor handle, preserving GitHub release page attribution hygiene.

### E2E-190-05: Release Gate Repository-Relative Path Hygiene Linter
- **Category**: Release Compliance & Path Hygiene (Track D Step 4)
- **Pre-Conditions**: Documentation or specification text provided.
- **Stimulus**:
  1. Scan text containing relative paths (`02-spec/...`, `.ai-memory/...`): verify `is_clean == true`.
  2. Scan text containing Windows drive letters (`D:\work\...`, `C:/repo/...`): verify violation reported.
  3. Scan text containing Unix user roots (`/home/...`, `/Users/...`): verify violation reported.
- **Expected Outcome**:
  - Absolute filesystem paths are intercepted and blocked across all public release documentation.

### E2E-190-06: Proxy SSE Chunk Coalescing & Delimiter Boundary Slicing
- **Category**: Proxy Streaming Protocols (Track E Imp-598)
- **Pre-Conditions**: `SseChunkCoalescer` initialized.
- **Stimulus**:
  1. Push single complete chunk ending in `\n\n`: verify 1 frame returned, 0 buffered.
  2. Push single complete chunk ending in `\r\n\r\n`: verify CRLF frame parsed cleanly.
  3. Push fragmented payload across 3 chunk pushes: verify complete frame emitted on final push.
  4. Push chunk with split delimiter across boundary: verify correct frame coalescing.
  5. Call `finish()` with residual bytes: verify trailing bytes returned.
- **Expected Outcome**:
  - Streaming byte fragments are assembled into valid RFC-compliant SSE frames without data corruption.

### E2E-190-07: Proxy SSE Telemetry Comment Stripping & Payload Preservation
- **Category**: Proxy Streaming Protocols (Track E Imp-599)
- **Pre-Conditions**: SSE chunks containing mixed data and comments.
- **Stimulus**:
  1. Filter chunk without colons: verify fast path identity return.
  2. Filter chunk with pure comments (`: keepalive`, `: ping`): verify comments stripped.
  3. Filter chunk with interleaved comments and `data:` frames: verify data frames preserved.
  4. Filter chunk containing URLs with port colons (`https://example.com:8080`): verify colons inside payload are preserved.
- **Expected Outcome**:
  - Telemetry comments are stripped cleanly while preserving payload fidelity.

### E2E-190-08: Auto-Switcher Post-Switch Settling Delay Barrier
- **Category**: Account Rotation & Lifecycle Protection (Track E Imp-600)
- **Pre-Conditions**: `SwitchSettlingTimer` initialized with default 10s delay.
- **Stimulus**:
  1. Assert `DEFAULT_SETTLING_DELAY_SECS == 10`.
  2. Initialize with custom duration (e.g. 0ms and 15ms) and verify `await_settling` completes cleanly.
  3. Verify paused virtual time advances through 10s barrier instantaneously in testing.
- **Expected Outcome**:
  - Newly launched IDE processes are granted a 10s quiet period before prompt dispatch.

### E2E-190-09: Split-SQLite Incremental Auto-Vacuum Stepping
- **Category**: Database Storage Hygiene & Freelist Maintenance (Track E Imp-601)
- **Pre-Conditions**: In-memory or file database with `PRAGMA auto_vacuum = INCREMENTAL`.
- **Stimulus**:
  1. Execute `step_incremental_vacuum` on zero-freelist database: verify fast-path 0 pages reclaimed.
  2. Populate and delete rows to create freelist pages: verify bounded chunk stepping (`DEFAULT_VACUUM_CHUNK_SIZE = 100`).
  3. Verify `IncrementalVacuumReport` accurately tracks before/after freelist counts and steps.
- **Expected Outcome**:
  - Freelist pages are reclaimed in bounded chunks without prolonged write lock contention.

### E2E-190-10: Sequence 190 Regression Gate & Backlog Integrity
- **Category**: Verification & Quality Gate
- **Pre-Conditions**: Sequence 190 implementation complete across all 10 steps.
- **Stimulus**:
  1. Confirm zero builds (`cargo build`, `npm run build`) and zero tests (`cargo test`, `npm test`) were executed.
  2. Confirm strictly relative Git paths were maintained.
  3. Confirm task backlog decremented from 105 to 95 steps remaining.
- **Expected Outcome**:
  - 100% adherence to all project governance invariants.
  - Clean git status ready for commit and push.
