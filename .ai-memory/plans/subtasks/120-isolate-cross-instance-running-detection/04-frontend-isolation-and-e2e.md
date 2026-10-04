---
plan: 120-isolate-cross-instance-running-detection
subtask: "04"
title: Frontend Isolation and E2E Integration Testing
domain: frontend-and-testing
depends_on: "03"
citations:
  component_spec: ../../../../02-spec/21-app/120-isolate-cross-instance-running-detection/02-component-spec.md
  architecture_spec: ../../../../02-spec/21-app/120-isolate-cross-instance-running-detection/01-architecture-spec.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src/pages/Instances.tsx
  - src-tauri/tests/test_instance_prompt_running_isolation.rs
status: pending
---

# 04 — Frontend Isolation and E2E Integration Testing

## 1. Context & Objectives
To prevent visual state bleeding and eliminate false-positive `[RUNNING]` badges, frontend components must evaluate task and project execution states strictly from verified backend boolean properties (`is_running`), gated by the host instance process status (`inst.is_running`). Furthermore, a deterministic Rust integration test suite (`test_instance_prompt_running_isolation.rs`) must be implemented to test multi-instance running isolation, dead-process gating, and suffix resolution using authentic SQLite databases.

## 2. Target Files & Symbols
- **`src/pages/Instances.tsx`**:
  - `fetchRunningTasks` (~L240)
  - `hasActiveTask` (~L1015)
  - `sortedProjects` (~L1372)
  - `isProjRunning` (~L1406)
- **`src-tauri/tests/test_instance_prompt_running_isolation.rs`** (New E2E Test Suite):
  - `test_default_profile_running_antigravity_manager_only`
  - `test_instance_8159_running_coding_guidelines_only`
  - `test_stopped_instance_process_dead_forces_all_idle`
  - `test_resolve_instance_id_suffix_matching`

## 3. Implementation Steps

### Step 3.1: Frontend State Hygiene & Process Gating (`Instances.tsx`)
1. **Audit `fetchRunningTasks`**:
   - Ensure the filtering of `runningTreeNodes` evaluates `Boolean(node.is_running) || Boolean(node.conversations?.some((c) => Boolean(c.is_running)))`.
   - Verify zero occurrences of `c.status === 'RUNNING'`.
2. **Audit `hasActiveTask`**:
   - Verify that the card header active task indicator is strictly gated:
     ```typescript
     const hasActiveTask = Boolean(inst.is_running) && runningTreeNodes.some((node) => {
         const isInstanceMatch = inst.config.is_default
             ? (node.instance_id === 'default' || node.instance_id === '__default__' || !node.instance_id || node.instance_id === inst.config.id)
             : node.instance_id === inst.config.id;
         const isNodeRunning = Boolean(node.is_running) || Boolean(node.conversations?.some((c) => Boolean(c.is_running)));
         return isInstanceMatch && isNodeRunning;
     });
     ```
   - Confirm that secondary instances (e.g. 8159) match strictly on `node.instance_id === inst.config.id` with no loose name matching.
3. **Audit Project Sorting & Running Badge**:
   - `sortedProjects`: Prioritize active projects based on `Boolean(inst.is_running) && (Boolean(p.is_running) || Boolean(p.conversations?.some((c) => Boolean(c.is_running))))`.
   - `isProjRunning`: Ensure the badge is rendered only if `inst.is_running` is true and `proj.is_running` or child conversations are true.

### Step 3.2: Implement Integration Test Suite (`test_instance_prompt_running_isolation.rs`)
Create `src-tauri/tests/test_instance_prompt_running_isolation.rs` using `tempfile::tempdir()` and authentic SQLite databases:
1. **Harness Helpers**:
   - `init_test_summaries_db(path)`: Creates table `conversation_summaries` matching the Antigravity SQLite schema.
   - `write_workspace_json(folder, repo_path)`: Creates `workspace.json` pointing to test project locations.
   - `insert_conversation_record(conn, cid, title, status, not_fully_idle, ws_path)`: Populates database records.
2. **Test Case 1: Default Profile Running Antigravity-Manager Only**:
   - Create 3 workspaces: `Antigravity-Manager`, `SpecBuilder`, `coding-guidelines`.
   - Seed `conversation_summaries.db` with active turn for `Antigravity-Manager` (`status = "RUNNING"`, `not_fully_idle = 1`) and idle historical records for `SpecBuilder` and `coding-guidelines`.
   - Assert `Antigravity-Manager` evaluates to `is_running: true`.
   - Assert `SpecBuilder` evaluates to `is_running: false`.
   - Assert `coding-guidelines` evaluates to `is_running: false`.
3. **Test Case 2: Instance 8159 Running coding-guidelines Only**:
   - In `default-copy-8159` directory, create 3 workspaces.
   - Seed active turn for `coding-guidelines` (`status = "RUNNING"`, `not_fully_idle = 1`).
   - Seed stale record for `Antigravity-Manager` (`status = "RUNNING"`, `not_fully_idle = 0`).
   - Assert `coding-guidelines` on 8159 evaluates to `is_running: true`.
   - Assert `Antigravity-Manager` on 8159 evaluates to `is_running: false` (Zero cross-bleed from Default!).
   - Assert `SpecBuilder` on 8159 evaluates to `is_running: false`.
4. **Test Case 3: Stopped Instance Process Gating (`INSTANCE_PROCESS_DEAD`)**:
   - With host process inactive (`is_inst_alive = false`), assert all projects and conversations evaluate to `is_running: false` and `status: "IDLE"`.
   - Assert audit log contains `"INSTANCE_PROCESS_DEAD"`.
5. **Test Case 4: Suffix Resolution for Instance Specifiers**:
   - Assert `resolve_instance_id("8159")` resolves to `"default-copy-8159"`.
   - Assert `resolve_instance_id("-8159")` resolves to `"default-copy-8159"`.

## 4. Constraints & Conventions
- Strictly positive booleans throughout test and component logic.
- UNIX LF line endings.
- Strict relative paths.
- Total ban on git commands.
- Symbol search using GitMap exclusively.

## 5. Verification Commands
```bash
# Frontend build validation
npm run build

# Integration test suite execution
cd src-tauri && cargo test --test test_instance_prompt_running_isolation

# Rust linter and formatting checks
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features
```

## 6. Done When
- [ ] `src/pages/Instances.tsx` relies exclusively on verified `is_running` boolean properties gated by host process liveness.
- [ ] `src-tauri/tests/test_instance_prompt_running_isolation.rs` is fully implemented and passes all 4 test cases.
- [ ] `npm run build` succeeds without TypeScript or bundling errors.
- [ ] Rust pre-flight checks (`cargo fmt`, `cargo clippy`, `cargo test`) complete cleanly.
