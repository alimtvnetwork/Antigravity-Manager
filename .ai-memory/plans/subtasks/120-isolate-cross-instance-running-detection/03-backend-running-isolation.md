---
plan: 120-isolate-cross-instance-running-detection
subtask: "03"
title: Backend Running Detection and Instance Process Isolation
domain: backend-rust
depends_on: "02"
citations:
  component_spec: ../../../../02-spec/21-app/120-isolate-cross-instance-running-detection/02-component-spec.md
  architecture_spec: ../../../../02-spec/21-app/120-isolate-cross-instance-running-detection/01-architecture-spec.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src-tauri/src/modules/instance.rs
  - src-tauri/src/modules/repo_db.rs
  - src-tauri/src/modules/logger.rs
status: pending
---

# 03 — Backend Running Detection and Instance Process Isolation

## 1. Context & Objectives
In multi-profile Antigravity installations, secondary instances (such as `default-copy-8159`) frequently mirror projects or inherit false `[RUNNING]` badges from the default profile. This subtask implements definitive backend isolation remedies:
1. Enable deterministic suffix matching in `resolve_instance_id` so specifiers like `"8159"` resolve accurately to `"default-copy-8159"`.
2. Strengthen `detect_running_projects` to strictly isolate workspace discovery to the target instance directory and gate project running status by active host PIDs.
3. Guarantee that `is_prompt_running_for_project` enforces strict instance-scoped AGY worker lookup (`{instance_id}:`), strictly queries instance-tagged SQLite prompt entries, and enforces the Idle Supremacy Rule over historical records.
4. Ensure `compute_project_conversation_tree` binds conversation nodes exclusively to matching `(instance_id, repo_path)` pairs and forces all entities to idle when the host instance process is dead.
5. Provide structured audit logging via `log_instance_prompt_audit` for observability.

## 2. Target Files & Symbols
- **`src-tauri/src/modules/instance.rs`**:
  - `resolve_instance_id` (~L3821)
- **`src-tauri/src/modules/repo_db.rs`**:
  - `detect_running_projects` (~L467)
  - `is_prompt_running_for_project` (~L1486)
  - `compute_project_conversation_tree` (~L3502)
  - `get_project_conversation_tree_cached` (~L3440)
- **`src-tauri/src/modules/logger.rs`**:
  - `log_instance_prompt_audit` (~L231)

## 3. Implementation Steps

### Step 3.1: Enhance `resolve_instance_id` with Suffix Matching (`instance.rs`)
1. In `src-tauri/src/modules/instance.rs`, update `resolve_instance_id`:
   - Keep clean specifier trimming, active/default checks, and exact name/id checks.
   - If clean specifier is numeric (e.g. `"8159"`), first check `seq_num == Some(num)`.
   - If no exact match, perform suffix matching against all registered instance IDs:
     - Match condition: `i.id.eq_ignore_ascii_case(clean) || i.id.ends_with(&format!("-{}", clean)) || i.id.ends_with(clean) || i.name.ends_with(clean)`.
     - If exactly one match is found, return `Ok(matched.id.clone())`.
     - If multiple matches occur, prefer the exact hyphen-delimited suffix match (`ends_with(&format!("-{}", clean))`).
2. Add comprehensive unit tests in `instance.rs` (`test_resolve_instance_id_resolution`):
   - Assert `"8159"` resolves to `"default-copy-8159"`.
   - Assert `"-8159"` resolves to `"default-copy-8159"`.
   - Assert exact `"default-copy-8159"` resolves to `"default-copy-8159"`.
   - Assert `"default"` resolves to `"inst-default"`.

### Step 3.2: Strictly Scope Project Discovery in `detect_running_projects` (`repo_db.rs`)
1. Normalize `instance_id`: Treat `""` or `"__default__"` as `"default"`.
2. Retrieve instance configuration via `registry.instances.iter().find(...)`. Return clear descriptive error if not found.
3. Discover PIDs via `find_pids_for_data_dir(&instance.data_dir, instance.is_default)`. Compute `is_instance_active = !pids.is_empty()`.
4. Scan `storage_dir = PathBuf::from(&instance.data_dir).join("User").join("workspaceStorage")`.
5. For each workspace entry, compute:
   `let is_project_active = is_instance_active && is_prompt_running_for_project(&raw_path, target_id);`
6. Emit structured audit log:
   `crate::modules::logger::log_instance_prompt_audit(target_id, &repo_name, &raw_path, is_instance_active, is_project_active, ...)`
7. Push discovered `RunningProject` with explicit `instance_id: target_id.to_string()`.

### Step 3.3: Enforce Strict Worker & DB Scoping in `is_prompt_running_for_project` (`repo_db.rs`)
1. **Host Process Gate**:
   If host instance process is dead, immediately log `"INSTANCE_PROCESS_DEAD"` and return `false`.
2. **Terminal Status Guard**:
   Check if the latest prompt record in memory or SQLite is `"completed"` or `"failed"`. If terminal, skip active execution marks.
3. **Active Prompts Memory Map**:
   Ensure `norm_inst` comparison strictly differentiates `"default"` from secondary instances. Cloned instances must strictly match `p.instance_id == norm_inst`.
4. **Active AGY Workers Map Isolation**:
   Format prefix `expected_prefix = format!("{}:", norm_inst)`. Secondary instances must match keys strictly starting with `expected_prefix`. Cross-instance worker matches are forbidden.
5. **SQLite `conversation_summaries.db` Verification**:
   - Query candidate directories strictly via `gemini_dirs_for_instance(norm_inst)`.
   - Apply strict Idle Supremacy: If `not_fully_idle == 0` or status contains `"IDLE"`, `"COMPLETED"`, `"FAILED"`, conversation is strictly idle.

### Step 3.4: Partition Conversation Tree Computation & Cache (`repo_db.rs`)
1. In `compute_project_conversation_tree`:
   - Key conversation map by `(norm_owning_inst, norm_path)`.
   - Prevent cross-attaching conversations across different instances even if their workspace paths match.
   - For all nodes, if `!is_inst_alive`, force `is_running = false` and `status = "IDLE"`.
2. In `get_project_conversation_tree_cached`:
   - Store cache under partitioned key `tree:{inst_key}:{max_words}:{only_running}`.
   - Save to SQLite table `prompt_tree_cache` with column `instance_id`.

## 4. Constraints & Conventions
- Strictly positive booleans (`is_running`, `is_instance_active`, `is_alive`).
- UNIX LF (`\n`) line endings.
- Strict relative paths; cross-platform path separators normalized.
- TOTAL BAN on git commands (no `git commit`, `git status`, `git add`).
- Symbol search using GitMap exclusively (`gitmap aum search`).

## 5. Verification Commands
```bash
# Code formatting check
cd src-tauri && cargo fmt -- --check

# Comprehensive compiler and clippy gate
cd src-tauri && cargo clippy --all-targets --all-features

# Targeted unit tests
cd src-tauri && cargo test modules::instance::tests::test_resolve_instance_id_resolution
cd src-tauri && cargo test modules::repo_db
```

## 6. Done When
- [ ] `resolve_instance_id` correctly resolves suffix specifiers like `"8159"` to `"default-copy-8159"`.
- [ ] `detect_running_projects` and `is_prompt_running_for_project` verify host process liveness and isolate worker keys.
- [ ] Conversation tree computation partitions nodes strictly by `(instance_id, path)`.
- [ ] Structured audit logs are emitted for all liveness evaluations.
- [ ] All Rust checks (`cargo fmt`, `cargo clippy`, unit tests) pass with exit code 0.
