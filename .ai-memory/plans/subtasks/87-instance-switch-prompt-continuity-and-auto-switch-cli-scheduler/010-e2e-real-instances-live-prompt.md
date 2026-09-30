---
plan: 87-instance-switch-prompt-continuity-and-auto-switch-cli-scheduler
subtask: "010"
title: Local E2E: two real instances, live prompt, two switches, teardown
domain: e2e-scripts
depends_on: 009-strict-candidate-and-unified-triggers.md
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#acceptance-criteria
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/50-instance-switch-loses-running-prompt-rca.md
  ambiguity: ../../../ambiguous-questions/02-ambiguity-resolved/03-native-ide-prompt-detection-scope-and-acceptance-signal.md
target_files:
  - scripts/test-instance-e2e.ps1
  - scripts/e2e_test_instance.ps1
  - scripts/test-instance-isolation.ps1
status: pending
---

# 010 — Local E2E: two real instances, live prompt, two switches, teardown

## 1. Context
Earlier fixes were claimed without a real prompt in a second real instance. This step proves the pipeline. It is local only and opt-in (see spec 72).

## 2. Target files and symbols
- scripts/test-instance-e2e.ps1
- scripts/e2e_test_instance.ps1
- scripts/test-instance-isolation.ps1

## 3. Steps
1. Create `test-inst-alpha` and `test-inst-beta` with distinct accounts; launch both.
2. Start a live prompt in alpha; run `agm swlc --instance test-inst-alpha --json` twice with a different strict candidate each time.
3. Assert: backup outcome shows the prompt; only alpha's PIDs closed, beta untouched; alpha's credentials changed; prompt injected once; verification result is `Verified`; heartbeat iteration advanced.
4. Teardown in `finally`: stop processes, `agm instances rm <id> --force`, then confirm no leftover dirs or lock files.
5. Fail fast with a clear message when accounts or the IDE are unavailable; never leave the run half-done silently.

## 4. Constraints
- US English spelling; boolean names use `is`/`has`; never write `== true`.
- Function and file size follow `02-spec/02-coding-guidelines/02-canonical-size-tier.md`; new logic goes in its own module or function, not appended to the large files.
- Relative paths only; cross-platform (Windows, macOS, Linux) path and process handling.
- No silent failure: results are typed and propagated; never `let _ =` on a backup, close, or inject result.
- Match processes only by `--user-data-dir` of the target instance, never by executable name.
- Script checks every exit code; teardown always runs.

## 5. Out of scope
- Version bump, changelog, release notes, readme version pin (release fires only when the whole plan is complete and the user commands it).
- Any file not listed in section 2 unless a compile error forces a minimal adjacent edit (record it in the plan status).

## 6. Verify
```bash
pwsh -NoProfile -File ./scripts/test-instance-e2e.ps1
```

## 7. Done When
- [ ] Both switches verified on alpha; beta never interrupted.
- [ ] Zero leftover instances.

## 8. Ambiguities and interim defaults
- Launching a real IDE is part of the user's request (real test instances); stop and report if the machine has no IDE installed.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
