---
plan: 87-instance-switch-prompt-continuity-and-auto-switch-cli-scheduler
subtask: "002"
title: Discover running prompts inside the instance's own directories
domain: backend-rust
depends_on: 001-instance-scope-resolver.md
citations:
  app_spec: ../../../../02-spec/21-app/85-instance-switch-update-integrity-ui-fixes-and-supabase-vault.md#t07--instance-switch
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
  issue_rca: ../../../issues/50-instance-switch-loses-running-prompt-rca.md
  ambiguity: ../../../ambiguous-questions/01-new-ambiguity/03-native-ide-prompt-detection-scope-and-acceptance-signal.md
target_files:
  - src-tauri/src/modules/repo_db.rs — `discover_running_prompts_from_antigravity` (~L572-745), `detect_running_projects` (~L423-519)
status: pending
---

# 002 — Discover running prompts inside the instance's own directories

## 1. Context
Discovery reads the default/global Gemini/Antigravity state, so a cloned instance's running prompt is invisible.

## 2. Target files and symbols
- src-tauri/src/modules/repo_db.rs — `discover_running_prompts_from_antigravity` (~L572-745), `detect_running_projects` (~L423-519)

## 3. Steps
1. Change both functions to take `&InstanceScope` and read only `scope.home_dir`/`scope.storage_dirs` for that instance.
2. Allow the global location only when `scope.is_default` (ambiguity 03 default B).
3. Return `Vec<RunningPrompt>` with the instance id attached; log counts per location at debug level without prompt text.
4. Tests with fixture conversation directories for two instances: each sees only its own prompt.

## 4. Constraints
- US English spelling; boolean names use `is`/`has`; never write `== true`.
- Function and file size follow `02-spec/02-coding-guidelines/02-canonical-size-tier.md`; new logic goes in its own module or function, not appended to the large files.
- Relative paths only; cross-platform (Windows, macOS, Linux) path and process handling.
- No silent failure: results are typed and propagated; never `let _ =` on a backup, close, or inject result.
- Match processes only by `--user-data-dir` of the target instance, never by executable name.
- Do not log prompt bodies or emails.

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
- [ ] A prompt in instance B is detected when scoped to B and not when scoped to A.
- [ ] Tests pass.

## 8. Ambiguities and interim defaults
- Ambiguity 03 default: per-instance dirs; global only for the default instance.

---

Execution: one step per run. Self-loop after Verify passes. Max 2 agents, max 3 threads per agent.
This task is standalone — read it plus its cited files, nothing else is assumed.
