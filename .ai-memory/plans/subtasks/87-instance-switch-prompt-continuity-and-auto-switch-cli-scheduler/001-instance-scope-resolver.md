---
plan: 87-instance-switch-prompt-continuity-and-auto-switch-cli-scheduler
subtask: "001"
title: One InstanceScope resolver (PIDs, data dir, home dir, storage layouts)
domain: backend-rust
depends_on: none
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#t07--instance-switch
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/50-instance-switch-loses-running-prompt-rca.md
  ambiguity: none
target_files:
  - src-tauri/src/modules/instance.rs — `find_pids_for_data_dir` (~L504-641), `switch_account_to_instance` (~L2693-2770), `close_instance` (~L1965)
  - src-tauri/src/modules/instance_scope.rs (new)
status: pending
---

# 001 — One InstanceScope resolver (PIDs, data dir, home dir, storage layouts)

## 1. Context
Plan 75 invariants list three state locations per instance: `<data_dir>/User/globalStorage`, `<data_dir>/AppData/Roaming/Antigravity/User/globalStorage`, `<home_dir>/AppData/Roaming/Antigravity/User/globalStorage`. PID to instance mapping exists but each caller rebuilds scope differently.

## 2. Target files and symbols
- src-tauri/src/modules/instance.rs — `find_pids_for_data_dir` (~L504-641), `switch_account_to_instance` (~L2693-2770), `close_instance` (~L1965)
- src-tauri/src/modules/instance_scope.rs (new)

## 3. Steps
1. Create `instance_scope.rs` with `InstanceScope { instance_id, data_dir, home_dir, pids, storage_dirs }` and `resolve_instance_scope(instance_id) -> Result<InstanceScope, ScopeError>`.
2. `pids` come from `find_pids_for_data_dir(data_dir)` only (no name matching). `storage_dirs` includes all three layouts that exist on disk.
3. For the default instance, scope is the default user dirs; flag it with `is_default`.
4. Unit tests with temporary directories: two instances resolve to disjoint scopes; missing dirs are reported, not guessed.

## 4. Constraints
- US English spelling; boolean names use `is`/`has`; never write `== true`.
- Function and file size follow `02-spec/02-coding-guidelines/02-canonical-size-tier.md`; new logic goes in its own module or function, not appended to the large files.
- Relative paths only; cross-platform (Windows, macOS, Linux) path and process handling.
- No silent failure: results are typed and propagated; never `let _ =` on a backup, close, or inject result.
- Match processes only by `--user-data-dir` of the target instance, never by executable name.

## 5. Out of scope
- Version bump, changelog, release notes, readme version pin (release fires only when the whole plan is complete and the user commands it).
- Any file not listed in section 2 unless a compile error forces a minimal adjacent edit (record it in the plan status).

## 6. Verify
```bash
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features
cd src-tauri && cargo test modules::instance_scope
```

## 7. Done When
- [ ] Resolver exists and is covered by tests.
- [ ] No caller changed yet (scaffold step).

## 8. Ambiguities and interim defaults
- None open for this step.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
