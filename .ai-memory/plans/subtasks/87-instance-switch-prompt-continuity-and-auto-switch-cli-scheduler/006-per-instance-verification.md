---
plan: 87-instance-switch-prompt-continuity-and-auto-switch-cli-scheduler
subtask: "006"
title: Verify per instance and expose in CLI JSON
domain: backend-rust
depends_on: 005-readiness-wait-and-single-injection.md
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#t07--instance-switch
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/50-instance-switch-loses-running-prompt-rca.md
  ambiguity: ../../../ambiguous-questions/02-ambiguity-resolved/03-native-ide-prompt-detection-scope-and-acceptance-signal.md
target_files:
  - src-tauri/src/modules/repo_db.rs — `verify_prompts_running` (~L1081-1130)
  - src-tauri/src/bin/agm.rs — `ff` (~L9705) and switch commands output
status: pending
---

# 006 — Verify per instance and expose in CLI JSON

## 1. Context
Verification must answer 'did this instance resume its prompt?' and never report success on a timeout.

## 2. Target files and symbols
- src-tauri/src/modules/repo_db.rs — `verify_prompts_running` (~L1081-1130)
- src-tauri/src/bin/agm.rs — `ff` (~L9705) and switch commands output

## 3. Steps
1. Make `verify_prompts_running(scope)` return `Verified | Unverified(reason) | Failed(reason)` using the acceptance signal from ambiguity 03.
2. Include the result and prompt counts in the JSON of `agm ff`, `agm swlc`, and instance switch commands.
3. Exit code is non-zero for `Failed`; `Unverified` exits 0 but prints a clear warning line.

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
- [ ] JSON of the switch commands contains the verification result.

## 8. Ambiguities and interim defaults
- Resolved (ambiguity 03) applies.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
