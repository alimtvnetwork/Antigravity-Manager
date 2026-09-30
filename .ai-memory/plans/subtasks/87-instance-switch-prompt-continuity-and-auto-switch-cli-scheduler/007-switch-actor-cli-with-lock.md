---
plan: 87-instance-switch-prompt-continuity-and-auto-switch-cli-scheduler
subtask: "007"
title: `agm switch-if-low-credit` as the single instance-aware actor
domain: backend-rust
depends_on: 006-per-instance-verification.md
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#decisions-confirmed-by-the-user-message-2
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/51-auto-switch-fallback-and-dual-loop-rca.md
  ambiguity: none
target_files:
  - src-tauri/src/bin/agm.rs — `swlc` (~L7438-7449)
  - src-tauri/src/modules/auto_switcher.rs — entry used by the CLI
  - src-tauri/src/modules/switch_lock.rs (new)
status: pending
---

# 007 — `agm switch-if-low-credit` as the single instance-aware actor

## 1. Context
The user chose the CLI as the only actor. Two triggers will call it, so it must be idempotent and safe under overlap.

## 2. Target files and symbols
- src-tauri/src/bin/agm.rs — `swlc` (~L7438-7449)
- src-tauri/src/modules/auto_switcher.rs — entry used by the CLI
- src-tauri/src/modules/switch_lock.rs (new)

## 3. Steps
1. Add `--instance <id>` and `--all` (default: every instance whose account is at or below the threshold) plus `--json`.
2. Implement `switch_lock.rs`: a lock file in the data dir with PID and timestamp, stale after a bounded time, released on drop; a second concurrent call exits 0 with `skipped: locked`.
3. Honor `cooldown_seconds` from config through the existing `scheduler::check_cooldown`.
4. The actor runs the full pipeline from subtasks 001-006 per instance and reports each instance's result.

## 4. Constraints
- US English spelling; boolean names use `is`/`has`; never write `== true`.
- Function and file size follow `02-spec/02-coding-guidelines/02-canonical-size-tier.md`; new logic goes in its own module or function, not appended to the large files.
- Relative paths only; cross-platform (Windows, macOS, Linux) path and process handling.
- No silent failure: results are typed and propagated; never `let _ =` on a backup, close, or inject result.
- Match processes only by `--user-data-dir` of the target instance, never by executable name.
- Headless parity: no GUI dependency.

## 5. Out of scope
- Version bump, changelog, release notes, readme version pin (release fires only when the whole plan is complete and the user commands it).
- Any file not listed in section 2 unless a compile error forces a minimal adjacent edit (record it in the plan status).

## 6. Verify
```bash
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features
cd src-tauri && cargo test modules::auto_switcher
cd src-tauri && cargo run --bin agm -- swlc --all --json
```

## 7. Done When
- [ ] Two concurrent invocations do not double-switch.
- [ ] Cooldown is honored.

## 8. Ambiguities and interim defaults
- None open for this step.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
