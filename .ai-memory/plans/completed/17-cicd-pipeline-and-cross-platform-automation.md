# Master Plan: CI/CD Pipeline Architecture & Cross-Platform Automation

> **Plan File:** `.ai-memory/plans/completed/17-cicd-pipeline-and-cross-platform-automation.md`
> **Status:** Completed
> **Version:** 1.0.0
> **Scope:** Grounding the local CI runner (`06-cicd-local-runner.py`), updating the quality gate matrix, fixing documentation link integrity, cleaning stale pending tasks, and verifying green local gates (`exit 0`).

---

## 1. Problem Statement & Background

The repository includes a multi-worker local CI/CD runner (`03-ai-scripts/06-cicd-local-runner.py`) powered by `03-ai-scripts/02-shared-engine.py`. However, its default `CI_JOBS_MATRIX` was populated from a Go-based meta-template containing 36 quality gates. 29 of those quality gates point to non-existent directories (`linter-scripts/`, `04-code/golang/`, `linters-cicd/`, and missing `.mjs` files), causing repository-wide quality checks to fail with 29 errors.

Furthermore, `03-ai-scripts/21-sequence-integrity-linter.py` detected 18 broken markdown links across plans and standards pointing to obsolete external scripts. Stale remnants of completed Plan 14 (`14-multi-instance`) also remain in `.ai-memory/plans/pending/` and `.ai-memory/plans/subtasks/`.

---

## 2. Violation Ledger (29 Identified Gate Failures & Gaps)

| # | Quality Gate Name | Configured Command | Root Cause | Target Fix |
|---|---|---|---|---|
| 1 | `Relative Path Check` | `python linter-scripts/check-relative-paths.py` | Missing `linter-scripts/` directory | Point to `03-ai-scripts/07-relative-path-fixer.py --check` |
| 2 | `Prompts Loaded Check` | `python linter-scripts/check-prompts-loaded.py` | Stale external script | Point to `03-ai-scripts/15-sequence-and-title-auditor.py` |
| 3 | `Readme Install Section Check` | `python linter-scripts/check-readme-install-section.py` | Stale external script | Point to `03-ai-scripts/16-installer-smoke-tester.py` |
| 4 | `Forbidden Strings Check` | `python linter-scripts/check-forbidden-strings.py` | Stale external script | Replaced with native repo scanner |
| 5 | `Newline Styling Check` | `python linter-scripts/check-newline-styling.py` | Stale external script | Point to `03-ai-scripts/04-newline-fixer.py --check` |
| 6 | `Bundle Installer Generation` | `node scripts/generate-bundle-installers.mjs` | Missing `scripts/` script | Replace with `install.ps1` & `install.sh` verification |
| 7 | `Spec Tree Sync` | `node scripts/sync-spec-tree.mjs` | Missing script | Replace with `02-spec/21-app/` verification |
| 8 | `Codegen Determinism Check` | `python linters-cicd/codegen/...` | Missing directory | Remove external check |
| 9 | `Spec Verification Coverage` | `node scripts/spec-verification/...` | Missing directory | Replace with spec auditor check |
| 10 | `Validate Version JSON` | `node scripts/validate-version-json.mjs` | Missing script | Replaced with `03-ai-scripts/14-version-sync-checker.py` |
| 11 | `Doc Links Check` | `node scripts/docs/check-doc-links.mjs` | Missing script | Point to `03-ai-scripts/21-sequence-integrity-linter.py` |
| 12 | `Check File Sizes Baseline` | `python linter-scripts/check-file-sizes.py` | Missing directory | Point to `03-ai-scripts/13-file-size-guard.py` |
| 13 | `Newline Styling MJS Check` | `node linter-scripts/...` | Missing script | Remove duplicate node check |
| 14 | `Spec Folder References Check` | `python linter-scripts/...` | Missing directory | Point to `03-ai-scripts/21-sequence-integrity-linter.py` |
| 15 | `Sequence Integrity Check` | `python linter-scripts/...` | Missing directory | Point to `03-ai-scripts/21-sequence-integrity-linter.py` |
| 16 | `Prompt & Spec Path Integrity Check`| `python linter-scripts/...` | Missing directory | Point to `03-ai-scripts/24-spec-path-migrator.py --check` |
| 17 | `Linters CI/CD Test Suite` | `python linters-cicd/tests/run.py` | Missing directory | Replace with Frontend TypeScript Check (`npx tsc --noEmit`) |
| 18 | `Interface Naming Check` | `python linter-scripts/...` | Missing directory | Point to `03-ai-scripts/08-naming-autofixer.py --check` |
| 19 | `Go Base Test Suite` | `go test -C 04-code/golang ./...` | No Go codebase | Replace with Frontend Build Check (`npm run build`) |
| 20 | `Axios Version Security Check`| `python linter-scripts/...` | Missing directory | Remove non-existent check |
| 21 | `Forbidden Spec Paths Check` | `python linter-scripts/...` | Missing directory | Replace with spec structure audit |
| 22 | `Placeholder Comments Check` | `python linter-scripts/...` | Missing directory | Point to code scanner |
| 23 | `Tunable Constants Check` | `python linter-scripts/...` | Missing directory | Replace with version audit |
| 24 | `Runner Dispatch Guard Check` | `python linter-scripts/...` | Missing directory | Replace with runner self-test |
| 25 | `Lint CI Drift Self-Test` | `node scripts/tests/...` | Missing directory | Replace with `03-ai-scripts/09-cli-help-auditor.py` |
| 26 | `Required Checks Self-Test` | `node scripts/tests/...` | Missing directory | Point to local runner self-test |
| 27 | `Sync Guidelines Self-Test` | `node scripts/tests/...` | Missing directory | Replace with guidelines sync verification |
| 28 | `File Sizes Baseline Self-Test`| `python linter-scripts/tests/...` | Missing directory | Point to `03-ai-scripts/13-file-size-guard.py` |
| 29 | `Sequence Integrity (Broken Links)` | 18 broken links across docs | Target files absent | Fix markdown links across plans & standards |

---

## 3. Acceptance Criteria

1. **Grounded Quality Gate Matrix**: `03-ai-scripts/02-shared-engine.py` defines only quality gates that correspond to real files, linters, compilers, and tools in `Antigravity-Manager`.
2. **Local CI Runner Passes (Exit 0)**: `python 03-ai-scripts/06-cicd-local-runner.py` executes across a 3-worker pool with all gates passing (`✔ All passed. exit 0`).
3. **Link Integrity Restored**: `03-ai-scripts/21-sequence-integrity-linter.py` reports 0 broken links.
4. **Clean Workspace**: Stale pending Plan 14 and subtasks archived/removed.
5. **No Release Bumps**: Preserves current version `v4.9.0` without triggering unrequested releases.

---

## 4. Subtask Decomposition

- [x] `.ai-memory/plans/subtasks/17-cicd-automation/01-clean-stale-tasks-and-links.md`: Purge stale Plan 14 leftovers and fix broken markdown links.
- [x] `.ai-memory/plans/subtasks/17-cicd-automation/02-align-quality-gates-matrix.md`: Align `02-shared-engine.py` and `06-cicd-local-runner.py` to native Tauri + React + Python gates.
- [x] `.ai-memory/plans/subtasks/17-cicd-automation/03-verify-local-runner-and-git-commit.md`: Execute full `06-cicd-local-runner.py` until 100% green and commit.
