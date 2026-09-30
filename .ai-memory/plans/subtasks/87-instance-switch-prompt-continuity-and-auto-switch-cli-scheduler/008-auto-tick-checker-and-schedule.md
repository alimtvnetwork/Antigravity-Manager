---
plan: 87-instance-switch-prompt-continuity-and-auto-switch-cli-scheduler
subtask: "008"
title: `agm auto tick` and `agm auto schedule`
domain: backend-rust
depends_on: 007-switch-actor-cli-with-lock.md
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#cli-surface-added-or-changed
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/51-auto-switch-fallback-and-dual-loop-rca.md
  ambiguity: none
target_files:
  - src-tauri/src/bin/agm.rs — new `auto` subcommand group
  - src-tauri/src/modules/auto_tick.rs (new, pure decision function plus thin runner)
  - src-tauri/src/modules/auto_schedule.rs (new; platform installers)
status: pending
---

# 008 — `agm auto tick` and `agm auto schedule`

## 1. Context
The user wants a small separate checker that runs about every minute and decides how soon a switch is needed, giving a second trigger.

## 2. Target files and symbols
- src-tauri/src/bin/agm.rs — new `auto` subcommand group
- src-tauri/src/modules/auto_tick.rs (new, pure decision function plus thin runner)
- src-tauri/src/modules/auto_schedule.rs (new; platform installers)

## 3. Steps
1. Write the pure `decide_tick(state, config, now) -> TickDecision { due_now, eta_seconds, reason }` using current quota, threshold, and recent burn rate (quota delta over the last samples); unit test it with fixtures.
2. `agm auto tick [--explain] [--json]` loads state, calls `decide_tick`, and invokes the actor (subtask 007) when `due_now`; `--explain` prints the reason for switching or not switching, including why no strict 100% candidate was available.
3. `agm auto schedule install|remove|status` registers the 1-minute tick: Windows Task Scheduler (`schtasks`), Linux systemd user timer with cron fallback, macOS launchd. Interval comes from a config field with an env override.
4. `install` is idempotent and verifies registration by reading it back; failure exits non-zero.

## 4. Constraints
- US English spelling; boolean names use `is`/`has`; never write `== true`.
- Function and file size follow `02-spec/02-coding-guidelines/02-canonical-size-tier.md`; new logic goes in its own module or function, not appended to the large files.
- Relative paths only; cross-platform (Windows, macOS, Linux) path and process handling.
- No silent failure: results are typed and propagated; never `let _ =` on a backup, close, or inject result.
- Match processes only by `--user-data-dir` of the target instance, never by executable name.
- Cross-platform: each platform implementation compiled behind `cfg`; shared logic has no platform code.

## 5. Out of scope
- Version bump, changelog, release notes, readme version pin (release fires only when the whole plan is complete and the user commands it).
- Any file not listed in section 2 unless a compile error forces a minimal adjacent edit (record it in the plan status).

## 6. Verify
```bash
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features
cd src-tauri && cargo test modules::auto_tick
cd src-tauri && cargo run --bin agm -- auto tick --explain --json
```

## 7. Done When
- [ ] `--explain` states a reason in every case.
- [ ] `schedule status` reflects install and remove.

## 8. Ambiguities and interim defaults
- None open for this step.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
