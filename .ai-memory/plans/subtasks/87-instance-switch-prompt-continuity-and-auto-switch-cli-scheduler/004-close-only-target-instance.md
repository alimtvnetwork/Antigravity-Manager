---
plan: 87-instance-switch-prompt-continuity-and-auto-switch-cli-scheduler
subtask: "004"
title: Close only the target instance's processes after a verified backup
domain: backend-rust
depends_on: 003-backup-result-typed-and-cli.md
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#t07--instance-switch
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/50-instance-switch-loses-running-prompt-rca.md
  ambiguity: none
target_files:
  - src-tauri/src/modules/instance.rs — `close_instance` (~L1965), `switch_account_to_instance`
  - src-tauri/src/modules/integration.rs (~L315-391) — default-switch path
status: pending
---

# 004 — Close only the target instance's processes after a verified backup

## 1. Context
Order must be: backup, then close, then switch, then relaunch. Closing must never touch another instance.

## 2. Target files and symbols
- src-tauri/src/modules/instance.rs — `close_instance` (~L1965), `switch_account_to_instance`
- src-tauri/src/modules/integration.rs (~L315-391) — default-switch path

## 3. Steps
1. In the switch function, call the backup for the instance's scope first; abort the switch (no close) if backup reports failures that are not explicitly ignorable, and return a typed error.
2. Close using `scope.pids` only; wait (bounded, 10 s) for exit; escalate to kill only those PIDs.
3. Return the list of closed PIDs for logging; an empty list on an instance that was running is an error state to report.
4. Test with two spawned dummy processes carrying different `--user-data-dir` arguments: only the target exits.

## 4. Constraints
- US English spelling; boolean names use `is`/`has`; never write `== true`.
- Function and file size follow `02-spec/02-coding-guidelines/02-canonical-size-tier.md`; new logic goes in its own module or function, not appended to the large files.
- Relative paths only; cross-platform (Windows, macOS, Linux) path and process handling.
- No silent failure: results are typed and propagated; never `let _ =` on a backup, close, or inject result.
- Match processes only by `--user-data-dir` of the target instance, never by executable name.
- Bound every wait with a timeout; fail before the point of no return (validate backup, then close).

## 5. Out of scope
- Version bump, changelog, release notes, readme version pin (release fires only when the whole plan is complete and the user commands it).
- Any file not listed in section 2 unless a compile error forces a minimal adjacent edit (record it in the plan status).

## 6. Verify
```bash
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features
cd src-tauri && cargo test modules::instance
```

## 7. Done When
- [ ] Another running instance survives a switch of this one.
- [ ] No close happens when backup failed.

## 8. Ambiguities and interim defaults
- None open for this step.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
