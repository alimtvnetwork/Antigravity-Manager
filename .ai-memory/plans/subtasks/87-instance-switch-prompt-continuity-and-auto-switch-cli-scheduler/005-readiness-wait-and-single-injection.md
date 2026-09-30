---
plan: 87-instance-switch-prompt-continuity-and-auto-switch-cli-scheduler
subtask: "005"
title: Wait for IDE readiness, inject once
domain: backend-rust
depends_on: 004-close-only-target-instance.md
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#t07--instance-switch
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/50-instance-switch-loses-running-prompt-rca.md
  ambiguity: ../../../ambiguous-questions/01-new-ambiguity/03-native-ide-prompt-detection-scope-and-acceptance-signal.md
target_files:
  - src-tauri/src/modules/repo_db.rs — resend path (~L2113-2145)
  - src-tauri/src/modules/instance.rs (~L2763 resend call)
  - src-tauri/src/modules/auto_switcher.rs (~L1335 resend call)
  - src-tauri/src/bin/agm.rs (~L9718 fast-forward resend call)
status: pending
---

# 005 — Wait for IDE readiness, inject once

## 1. Context
Three callers may resend the same prompts, and none waits for the relaunched IDE to be ready.

## 2. Target files and symbols
- src-tauri/src/modules/repo_db.rs — resend path (~L2113-2145)
- src-tauri/src/modules/instance.rs (~L2763 resend call)
- src-tauri/src/modules/auto_switcher.rs (~L1335 resend call)
- src-tauri/src/bin/agm.rs (~L9718 fast-forward resend call)

## 3. Steps
1. Add `wait_until_ready(scope, timeout)` that polls for the instance's new PID plus the agy bridge/heartbeat readiness signal; return `Ready | TimedOut`.
2. Make one function `inject_backed_up_prompts(scope)` the only injector; it marks each prompt `dispatched` exactly once (idempotent on retry).
3. Replace the three call sites with that function; delete the duplicated resend logic.
4. On `TimedOut`, keep prompts `backed_up` (not lost) and return a typed `NotReady` result for the caller to report.

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
cd src-tauri && cargo test modules::repo_db
```

## 7. Done When
- [ ] Exactly one injector function remains.
- [ ] A timeout leaves prompts recoverable.

## 8. Ambiguities and interim defaults
- Ambiguity 03 default signal: DB `dispatched` plus heartbeat within timeout, else typed unverified.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
