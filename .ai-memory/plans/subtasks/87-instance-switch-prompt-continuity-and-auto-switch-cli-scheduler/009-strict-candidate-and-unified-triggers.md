---
plan: 87-instance-switch-prompt-continuity-and-auto-switch-cli-scheduler
subtask: "009"
title: Strict 100% candidate, one trigger path, consistent model
domain: backend-rust+frontend
depends_on: 007-switch-actor-cli-with-lock.md, 008-auto-tick-checker-and-schedule.md
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#t10--auto-switch
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/51-auto-switch-fallback-and-dual-loop-rca.md
  ambiguity: none
target_files:
  - src-tauri/src/modules/auto_switcher.rs — `select_candidate_profiles` (~L813; sub-100% fallback ~L932-957 and ~L1102-1127), target-model resolution (~L1291-1364, ~L1409-1411, ~L2039-2109)
  - src/components/common/BackgroundTaskRunner.tsx (~L146-148 early return on non-Tauri)
  - src-tauri/src/models/config.rs — auto-switch config (~L298-329)
status: pending
---

# 009 — Strict 100% candidate, one trigger path, consistent model

## 1. Context
User decision: strict candidate only. The GUI loop duplicates the Rust daemon and exits early outside Tauri.

## 2. Target files and symbols
- src-tauri/src/modules/auto_switcher.rs — `select_candidate_profiles` (~L813; sub-100% fallback ~L932-957 and ~L1102-1127), target-model resolution (~L1291-1364, ~L1409-1411, ~L2039-2109)
- src/components/common/BackgroundTaskRunner.tsx (~L146-148 early return on non-Tauri)
- src-tauri/src/models/config.rs — auto-switch config (~L298-329)

## 3. Steps
1. Remove every sub-100% fallback; when no 100% candidate exists, the actor reports `no_strict_candidate` and does not switch.
2. Resolve the target model through one function used by the actor, the daemon, and the UI.
3. Make the reactive trigger (quota refresh at or below threshold, and UI threshold change) call the same actor asynchronously; delete the separate React switching logic, keeping only status display.
4. Expose the tick interval and threshold in `gui_config.json` with environment overrides; document them in `agm` help.
5. Update spec 01 and spec 22-app-issues/12 cross-references only if wording conflicts (append a dated note; do not rewrite).

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
cd src-tauri && cargo test modules::auto_switcher
npm run build
```

## 7. Done When
- [ ] No sub-100% fallback remains.
- [ ] GUI and headless share one decision and one actor.

## 8. Ambiguities and interim defaults
- None open for this step.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
