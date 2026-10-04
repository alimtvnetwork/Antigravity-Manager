---
plan: 118-instance-prompt-execution-status-fix
subtask: "02"
title: Frontend status logic remediation and per-instance E2E test verification
domain: frontend/test
target_files:
  - src/pages/Instances.tsx
  - src/components/instances/PromptTreeViewModal.tsx
  - src-tauri/tests/per_instance_prompt_liveness_test.rs
status: pending
---

# 02 — Frontend Status Logic Remediation & Per-Instance E2E Test Verification

## 1. Overview & Objectives

In Task 118, user testing highlighted that prompt running states falsely leak across instances and projects:
- **Default profile** running *only* `Antigravity-Manager` displayed running indicators on dormant projects `SpecBuilder` and `coding-guidelines`.
- **Instance 8159 (`default-copy-8159`)** running *only* `coding-guidelines` displayed running indicators on dormant projects `Antigravity-Manager` and `SpecBuilder`.

The primary frontend driver of this false reporting is the fallback clause `|| c.status === 'RUNNING'`, which bypassed backend real-time process verification whenever historical SQLite records retained `status = 'RUNNING'`.

This subtask executes:
1. **Frontend Condition Remediation**: Purge all instances of `|| c.status === 'RUNNING'` across `src/pages/Instances.tsx` and `src/components/instances/PromptTreeViewModal.tsx`. Liveness must strictly derive from the `is_running` boolean.
2. **Comprehensive E2E Integration Test Suite**: Expand `src-tauri/tests/per_instance_prompt_liveness_test.rs` to thoroughly verify per-instance project isolation, dead process forcing, and zero worker bleeding.

---

## 2. Step-by-Step Implementation Plan

### Part 1: Modify `src/pages/Instances.tsx`

1. **`fetchRunningTasks` Filter (~line 250)**:
   - Change:
     ```typescript
     // BEFORE:
     const running = data.filter(
         (node) => Boolean(node.is_running) || Boolean(node.conversations?.some((c) => c.is_running || c.status === 'RUNNING'))
     );
     // AFTER:
     const running = data.filter(
         (node) => Boolean(node.is_running) || Boolean(node.conversations?.some((c) => Boolean(c.is_running)))
     );
     ```

2. **Instance Card `hasActiveTask` Detection (~line 1019)**:
   - Change:
     ```typescript
     // BEFORE:
     const isNodeRunning = Boolean(node.is_running) || Boolean(node.conversations?.some((c) => c.is_running || c.status === 'RUNNING'));
     // AFTER:
     const isNodeRunning = Boolean(node.is_running) || Boolean(node.conversations?.some((c) => Boolean(c.is_running)));
     ```

3. **Instance Card Project Sorting `sortedProjects` (~lines 1373–1374)**:
   - Change:
     ```typescript
     // BEFORE:
     const aRunning = Boolean(inst.is_running) && Boolean(a.is_running || a.conversations?.some((c) => c.is_running || c.status === 'RUNNING'));
     const bRunning = Boolean(inst.is_running) && Boolean(b.is_running || b.conversations?.some((c) => c.is_running || c.status === 'RUNNING'));
     // AFTER:
     const aRunning = Boolean(inst.is_running) && Boolean(a.is_running || a.conversations?.some((c) => Boolean(c.is_running)));
     const bRunning = Boolean(inst.is_running) && Boolean(b.is_running || b.conversations?.some((c) => Boolean(c.is_running)));
     ```

4. **Project Pill Running Badge `isProjRunning` (~line 1400)**:
   - Change:
     ```typescript
     // BEFORE:
     const isProjRunning = Boolean(inst.is_running) && Boolean(proj.is_running || proj.conversations?.some((c) => c.is_running || c.status === 'RUNNING'));
     // AFTER:
     const isProjRunning = Boolean(inst.is_running) && Boolean(proj.is_running || proj.conversations?.some((c) => Boolean(c.is_running)));
     ```

---

### Part 2: Modify `src/components/instances/PromptTreeViewModal.tsx`

Eliminate `|| c.status === 'RUNNING'` across all 9 locations in `PromptTreeViewModal.tsx`:
1. Line 107: `if (conv.is_running === true || conv.status === 'RUNNING')` -> `if (Boolean(conv.is_running))`
2. Line 656: `c.is_running === true || c.status === 'RUNNING'` -> `Boolean(c.is_running)`
3. Lines 694–695: `a.conversations.some((c) => c.is_running || c.status === 'RUNNING')` -> `a.conversations.some((c) => Boolean(c.is_running))`
4. Line 714: `if (conv.is_running === true || conv.status === 'RUNNING')` -> `if (Boolean(conv.is_running))`
5. Line 1158: `p.conversations.some((c) => c.is_running || c.status === 'RUNNING')` -> `p.conversations.some((c) => Boolean(c.is_running))`
6. Lines 1176–1177: `a.conversations.some((c) => c.is_running || c.status === 'RUNNING')` -> `a.conversations.some((c) => Boolean(c.is_running))`
7. Line 1208: `c.is_running || c.status === 'RUNNING'` -> `Boolean(c.is_running)`
8. Lines 1211–1212: `a.is_running || a.status === 'RUNNING'` -> `Boolean(a.is_running)`
9. Line 1234: `conv.is_running || conv.status === 'RUNNING'` -> `Boolean(conv.is_running)`

---

### Part 3: Expand `src-tauri/tests/per_instance_prompt_liveness_test.rs`

Implement four comprehensive test scenarios:

1. **`test_default_profile_running_antigravity_manager_only`**:
   - Construct Default instance mock storage with 3 projects: `Antigravity-Manager`, `SpecBuilder`, `coding-guidelines`.
   - Dispatch active execution for `Antigravity-Manager` with `is_running = true`.
   - Verify `SpecBuilder` and `coding-guidelines` evaluate to `is_running == false`.
   - Verify only `Antigravity-Manager` has affirmative running status.

2. **`test_instance_8159_running_coding_guidelines_only`**:
   - Construct Instance 8159 (`default-copy-8159`) mock storage with 3 projects: `coding-guidelines`, `Antigravity-Manager`, `SpecBuilder`.
   - Dispatch active execution for `coding-guidelines` with `is_running = true`.
   - Verify `Antigravity-Manager` and `SpecBuilder` evaluate to `is_running == false`.
   - Verify zero leakage from default profile.

3. **`test_stopped_instance_projects_forced_idle_instance_process_dead`**:
   - Construct instance records with historical `status = "RUNNING"` and `not_fully_idle = 1`.
   - Set instance process state as dead/stopped (`is_instance_alive == false`).
   - Verify that all projects and conversations are forced to `is_running == false`.
   - Verify that rationale contains `"INSTANCE_PROCESS_DEAD"`.

4. **`test_worker_checking_cannot_bleed_across_instances`**:
   - Register an active worker under `"default:d:/work/Antigravity-Manager"`.
   - Query `is_prompt_running_for_project("Antigravity-Manager", "default-copy-8159")`.
   - Verify it returns `false` due to instance key isolation.

---

## 3. Test Execution Commands & Verification Protocol

Run the following commands in order:

```bash
# 1. Run per-instance prompt liveness integration tests
cargo test --test per_instance_prompt_liveness_test -- --nocapture

# 2. Verify frontend TypeScript compilation and bundling
npm run build

# 3. Rust formatting check
cd src-tauri && cargo fmt -- --check

# 4. Comprehensive Rust Clippy gate
cd src-tauri && cargo clippy --all-targets --all-features
```

---

## 4. Acceptance Criteria Checklist

- [ ] **AC-1**: `src/pages/Instances.tsx` has zero references to `c.status === 'RUNNING'`.
- [ ] **AC-2**: `src/components/instances/PromptTreeViewModal.tsx` has zero references to `c.status === 'RUNNING'`.
- [ ] **AC-3**: Default profile running `Antigravity-Manager` displays a running badge strictly on `Antigravity-Manager`; `SpecBuilder` and `coding-guidelines` display as idle.
- [ ] **AC-4**: Instance 8159 running `coding-guidelines` displays a running badge strictly on `coding-guidelines`; `Antigravity-Manager` and `SpecBuilder` display as idle.
- [ ] **AC-5**: Any stopped instance (`is_running == false`) immediately and unconditionally evaluates all projects to idle with `INSTANCE_PROCESS_DEAD` rationale.
- [ ] **AC-6**: Active AGY workers in one instance never bleed into running status of another instance.
- [ ] **AC-7**: All integration tests in `src-tauri/tests/per_instance_prompt_liveness_test.rs` pass cleanly.
