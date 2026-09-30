# Plan 89: CI/CD Rustfmt `cmd_instances` Formatting Drift & Pipeline Recovery

> **Spec Reference:** [02-spec/21-app/89-cicd-rustfmt-cmd-instances-fix.md](../../../02-spec/21-app/89-cicd-rustfmt-cmd-instances-fix.md)  
> **Target Subsystems:** `src-tauri/src/bin/agm.rs`, `.githooks/pre-commit`, `scripts/install-git-hooks.mjs`, `package.json`, `.github/workflows/release.yml`, `.ai-memory/cicd-issues/`, `.ai-memory/cicd-index.md`, `.ai-memory/issues/`, `.ai-memory/strictly-avoid.md`, `.ai-memory/what-to-read.md`.  
> **Status:** COMPLETED except Subtask 06 (BLOCKED: issue 56, `workflow` token scope)  
> **How it started:** User reported CI breaking on `main`; every completed CI run from `db5a3787` to `2fe82117` was red. Re-invoked 2026-10-01 with the V4 N-step orchestrator prompt; that run continued from this plan (resume rule) with `SOLO_FALLBACK: invoke_subagent absent`.  
> **Steps (re-run ledger):** 34 of 300 (Phase 1: 12 of 150, Phase 2: 22 of 150).  

---

## User Request (Verbatim)

```markdown
Can you please check that if the CI/CD is working fine and if anything is breaking in the CI/CD, try to fix that. Yeah, I think things are breaking in the CI/CD. We need to fix the CI/CD first. Make sure that nothing remains behind. Right, the root cause of this failure. Okay, and based on that, your job is to fix the CI/CD, make the minor bug remedies and check until it gets green. Okay? Also, when you fix the CI/CD, you find the right and root cause properly so that these issues never repeat again. Find the CI/CD issues folder. Try to understand previous ones and new ones, why it is happening, root cause of it. Update its memory in the AI memory so that anytime any AI model reads the AI memory folder, it knows why it failed, how it failed, and it shouldn't do it again.

adn make sure issues are listed in theissues and rerrun so that mistakesa rte not repeated agian ,clear??
```

---

## Traceable Subtasks & Execution Results

- **Subtask 01**: Author RCA 40 and register in cicd-issues index -> [DONE]
  - Authored `.ai-memory/cicd-issues/40-recurring-rustfmt-drift-and-releases-shipping-on-red-ci-rca.md`.
  - Registered entry 40 in `.ai-memory/cicd-index.md` and `.ai-memory/cicd-issues/readme.md`.
- **Subtask 02**: Fix rustfmt formatting at line 8612 of `agm.rs` -> [DONE]
  - Formatted `let non_flag_args: Vec<String> = args...` multi-line into builder style across lines 8612-8616.
- **Subtask 03**: Update spec, strictly-avoid rules, and institutional memory -> [DONE]
  - Authored `02-spec/21-app/89-cicd-rustfmt-cmd-instances-fix.md` and registered in `02-spec/21-app/readme.md`.
  - Added total ban on single-line long iterator chains to `.ai-memory/strictly-avoid.md`.
  - Added `.ai-memory/temp-agents/` to `.gitignore`.
- **Subtask 04**: Stage by explicit path, commit, push, and verify green CI -> [DONE]
  - Staged with explicit paths (R8).
  - Pushed atomic commit to `origin/main`.
  - Remote CI workflow run verified green across all matrix targets (`windows-2022`, `ubuntu-latest`, `macos-latest`).
- **Subtask 05**: Enforce rustfmt locally via tracked pre-commit hook -> [DONE]
  - `.githooks/pre-commit`, `scripts/install-git-hooks.mjs`, `package.json` (`prepare`, `hooks:install`) in `09338155`.
  - Hook test: drifted staged file exit 1; clean tree exit 0.
- **Subtask 06**: Gate `release.yml` on `cargo fmt -- --check` -> [BLOCKED]
  - Committed locally as `ci(release): gate release on cargo fmt --check`; push rejected because the token lacks the `workflow` scope. Tracked in `.ai-memory/issues/56-release-fmt-gate-blocked-by-token-workflow-scope.md`.
- **Subtask 07**: Memory accuracy audit (re-run 2026-10-01) -> [DONE]
  - `cicd-index.md`: rows 35–39 added, dead row-32 link marked, RCA 40 set to partial, recurring-class table added.
  - `strictly-avoid.md`: appended "Bypassing CI Gates or Tagging on Unverified CI" ban (existing entries untouched).
  - `what-to-read.md`: corrected the false "enforced rustfmt gate in `release.yml`" claim.
  - Issue 56 opened; issue 55 moved to resolved in `.ai-memory/issues/readme.md`.
  - RCA 40: sections 5–7 (masked failures, gate status, announced CI deadlines).
  - `cicd-index.md`: seven dead `resolved-issues/01–07` links repointed to renumbered files `02–08`.
- **Subtask 08**: Fix change-recording tool crash (RCA 41) -> [DONE]
  - `03-ai-scripts/33-test-inventory-generator.py`: `normalize_inventory_tests()` for list-or-dict `tests`; epoch `updated_at` accepted.
  - `--record` exit 0; `--check-age --json` reports age instead of an error; `py_compile` exit 0.
  - RCA `.ai-memory/cicd-issues/41-test-inventory-generator-schema-mismatch-rca.md`, indexed.

---

## Verification Summary

1. **Rustfmt Compliance**:
   - `src-tauri/src/bin/agm.rs:8612` broken into multi-line method chain conforming strictly to rustfmt's 100-character line width cap.
2. **Institutional Memory**:
   - 4-part RCA captured in `.ai-memory/cicd-issues/40-recurring-rustfmt-drift-and-releases-shipping-on-red-ci-rca.md`.
   - Rule recorded in `.ai-memory/strictly-avoid.md`.
3. **CI Pipeline Status**:
   - Verified through GitHub Actions workflow run on commit pushed to `main`.
