# Subtask 02: Align Quality Gates Matrix in Shared Engine & Local Runner

> **Parent Plan:** `.lovable/plans/completed/17-cicd-pipeline-and-cross-platform-automation.md`
> **Status:** Completed

## Objectives
1. Edit `03-ai-scripts/02-shared-engine.py`:
   - Replace the 36 external Go/template gate dictionary in `CI_JOBS_MATRIX` with the grounded quality gates for `Antigravity-Manager`:
     - Frontend TypeScript Verification: `npx tsc --noEmit`
     - Frontend Production Build: `npm run build`
     - Version Consistency Check: `python 03-ai-scripts/14-version-sync-checker.py`
     - File Size Guard: `python 03-ai-scripts/13-file-size-guard.py`
     - Relative Path Guard: `python 03-ai-scripts/07-relative-path-fixer.py`
     - Fast File Scanner: `python 03-ai-scripts/11-fast-file-scanner.py --check`
     - Sequence & Title Auditor: `python 03-ai-scripts/15-sequence-and-title-auditor.py`
     - Sequence Integrity Linter: `python 03-ai-scripts/21-sequence-integrity-linter.py`
     - Misspell Auditor: `python 03-ai-scripts/27-misspell-auditor.py`
     - Installer Smoke Tester: `python 03-ai-scripts/16-installer-smoke-tester.py`
     - CLI Help Auditor: `python 03-ai-scripts/09-cli-help-auditor.py`
2. Ensure `03-ai-scripts/06-cicd-local-runner.py` correctly imports and runs the updated matrix concurrently.
