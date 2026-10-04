---
plan: 114-debug-instance-prompt-running-detection
subtask: "02"
title: Frontend Prompt Tree Alignment, False-Positive Elimination, and Integration Test Suite
domain: frontend-and-testing
depends_on: 01-backend-detection-refactor.md
citations:
  app_spec: ../../../../02-spec/21-app/114-debug-instance-prompt-running-detection/02-component-spec.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src/pages/Instances.tsx
  - src/components/instances/PromptTreeViewModal.tsx
  - src-tauri/tests/per_instance_prompt_liveness_test.rs
status: completed
---

# 02 — Frontend Prompt Tree Alignment, False-Positive Elimination, and Integration Test Suite

## 1. Context
During multi-instance operations, the frontend erroneously displayed glowing `RUNNING` badges on dormant projects (e.g. `SpecBuilder` and `coding-guidelines` on the Default instance, and `Antigravity-Manager` and `SpecBuilder` on instance `8159`). This subtask implements the frontend alignment, scoped instance filtering, clean empty states, and a comprehensive Rust integration test suite to verify per-instance prompt liveness.

## 2. Target Files and Symbols
- `src/pages/Instances.tsx`: `fetchRunningTasks`, `hasActiveTask`, `instanceProjects`, `isProjRunning`
- `src/components/instances/PromptTreeViewModal.tsx`: `relevant`, scoped instance filtering, empty states
- `src-tauri/tests/per_instance_prompt_liveness_test.rs`: Test Cases 1 through 4

## 3. Step-by-Step Implementation Tasks

### Step 1: Align `fetchRunningTasks` and Instance Card Mapping in `src/pages/Instances.tsx`
1. Ensure `fetchRunningTasks` in `src/pages/Instances.tsx` calls `get_project_conversation_tree` and parses `AgmProjectTreeNode[]`.
2. Map `runningTreeNodes` using affirmative running checks:
   - `node.is_running === true` OR `node.conversations.some(c => c.is_running === true || c.status === 'RUNNING')`.
3. In each instance card:
   - Evaluate `hasActiveTask`: Verify `inst.is_running === true` AND at least one matching node in `runningTreeNodes` belongs specifically to this instance.
   - For Default instance: Match `node.instance_id === 'default' || node.instance_id === '__default__' || !node.instance_id || node.instance_id === inst.config.id`.
   - For Non-default instances (e.g. `default-copy-8159`): Match strictly `node.instance_id === inst.config.id`.

### Step 2: Eliminate False Positive `isProjRunning` Badges on Dormant Projects
1. In `src/pages/Instances.tsx` within the project row rendering:
   - Replace permissive conditions with affirmative guard:
     ```typescript
     const isProjRunning = Boolean(inst.is_running) && (
         Boolean(proj.is_running) || 
         Boolean(proj.conversations?.some((c) => Boolean(c.is_running)))
     );
     ```
   - If `inst.is_running` is false, `isProjRunning` must immediately evaluate to `false`.
   - Stale/dormant conversations (`status === 'IDLE'`, `not_fully_idle === 0`) must never cause `isProjRunning` to be true.
2. Ensure the glowing cyan pulse and `RUNNING` text badge appear ONLY when `isProjRunning === true`.

### Step 3: Enforce Strict `instanceId` Scoping & Clean Empty States in `PromptTreeViewModal.tsx`
1. Pass `instanceId: instanceId || undefined` when invoking `get_project_conversation_tree`.
2. Strictly filter tree nodes by `instanceId` on the client side:
   - If `instanceId === 'default' || instanceId === '__default__'`, match default nodes only.
   - If `instanceId` is non-default, match strictly `p.instance_id === instanceId`.
   - Prevent cross-profile workspace pollution by filtering both projects and nested conversation arrays.
3. Render high-clarity empty states:
   - When no projects exist for an instance (`treeData.length === 0`), display a centered card with a folder icon, title "No Projects Found in this Profile", and descriptive copy advising the user to launch the instance and open a workspace.
   - When search filters return zero results, provide a clear "No conversations match" notice with a "Clear search query" button.
   - When no conversation is selected in the detail pane, display a neutral placeholder prompting selection with `MessageSquare`.

### Step 4: Implement Rust Integration Test Suite (`per_instance_prompt_liveness_test.rs`)
Create `src-tauri/tests/per_instance_prompt_liveness_test.rs` covering four exhaustive scenarios:
1. **Test Case 1: Default Instance Running Antigravity-Manager Only**:
   - Process for Default instance is alive.
   - Default DB has `Antigravity-Manager` with `not_fully_idle = 1, status = "CASCADE_RUN_STATUS_RUNNING"`.
   - `SpecBuilder` and `coding-guidelines` have `not_fully_idle = 0, status = "CASCADE_RUN_STATUS_IDLE"`.
   - Assert `Antigravity-Manager` reports `is_running = true`.
   - Assert `SpecBuilder` and `coding-guidelines` report `is_running = false`.
2. **Test Case 2: Instance 8159 Running coding-guidelines Only**:
   - Process for Instance 8159 is alive.
   - 8159 DB has `coding-guidelines` with `not_fully_idle = 1, status = "CASCADE_RUN_STATUS_RUNNING"`.
   - `Antigravity-Manager` and `SpecBuilder` have `not_fully_idle = 0, status = "CASCADE_RUN_STATUS_IDLE"`.
   - Assert `coding-guidelines` reports `is_running = true`.
   - Assert `Antigravity-Manager` and `SpecBuilder` report `is_running = false`.
3. **Test Case 3: Copied/Dormant Workspaces Do Not Activate False `is_running`**:
   - Cloned `workspaceStorage` paths present in 8159 matching Default instance paths.
   - Default is running `Antigravity-Manager`, but 8159's record for `Antigravity-Manager` is idle.
   - Assert 8159 tree does NOT inherit the running state from Default.
4. **Test Case 4: Process Termination Forces `is_running = false`**:
   - Database contains stale `not_fully_idle = 1` or `RUNNING` records (crashed/killed session).
   - Instance process is verified dead/terminated (`is_inst_alive == false`).
   - Assert `is_running` is forced to `false` for project and conversation nodes.

### Step 5: Verify Structured Audit Logging Telemetry
Verify that every evaluation produces structured logs matching:
`[PROMPT_LIVENESS_PROBE][PROJECT] instance_id="..." project="..." is_running=... rationale="..."`

## 4. Constraints & Guidelines
- Follow all rules in `AGENTS.md` and repository coding guidelines.
- US English spelling; boolean names use positive prefixes (`is`/`has`); never write `== true` in TypeScript/Rust expressions.
- Cross-platform path handling with forward/backward slash normalization.
- TOTAL BAN on all git commands (`git`, `git commit`, `git status`, etc.).
- Never modify version numbers or release changelogs during task execution.

## 5. Out of Scope
- Modifying SQLite database schemas in user profile directories.
- Triggering automatic release ceremonies or version bumps.
- Modifying unrelated modal views outside the prompt tree and instance pages.

## 6. Verification Commands
```bash
# Frontend build check
npm run build

# Rust formatting & linter gates
cd src-tauri && cargo fmt -- --check
```

## 7. Done When
- [x] `src/pages/Instances.tsx` only renders `RUNNING` badge for affirmatively active projects on running instances.
- [x] `src/components/instances/PromptTreeViewModal.tsx` strictly filters by `instanceId` and provides clean empty states.
- [x] All 4 test cases in `src-tauri/tests/per_instance_prompt_liveness_test.rs` pass cleanly.
- [x] `npm run build` succeeds without TypeScript or bundling errors.
- [x] Cargo fmt pre-flight checks pass.

## 8. Verification Results & Evidence

### Frontend Build
- Command: `npm run build`
- Output: `✓ built in 15.68s`, exit code `0`. Zero TypeScript diagnostic errors, Vite production bundle generated.

### Rust Formatting Gate
- Command: `cd src-tauri && cargo fmt -- --check`
- Output: clean exit code `0`. All Rust files adhere strictly to standard rustfmt formatting.

### Integration Test Suite Implementation
- File: `src-tauri/tests/per_instance_prompt_liveness_test.rs`
- Coverage:
  1. `test_case_1_default_running_antigravity_manager_only`: Hermetic SQLite sandbox for Default profile, verifies AGM active running while SpecBuilder and coding-guidelines remain strictly idle.
  2. `test_case_2_instance_8159_running_coding_guidelines_only`: Hermetic SQLite sandbox for 8159 profile, verifies coding-guidelines active running while AGM and SpecBuilder remain strictly idle.
  3. `test_case_3_cross_instance_isolation_copied_dormant_workspaces`: Verifies cross-instance isolation between cloned workspaces across Default and 8159 profiles with zero running state bleed.
  4. `test_case_4_process_termination_gating_forces_idle`: Verifies that dead/crashed processes (`is_inst_alive == false`) force `is_running = false` regardless of stale DB records and log `INSTANCE_PROCESS_DEAD`.
