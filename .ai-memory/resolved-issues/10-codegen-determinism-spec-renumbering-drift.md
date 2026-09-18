# RCA: Codegen Determinism Drift After Spec Renumbering & Shell-to-Python Migration

- **Date:** 2026-08-30
- **Status:** Resolved
- **Severity:** Blocker (CI/CD Pipeline Quality Gate Failure)
- **Target Files:** `linters-cicd/codegen/fixtures/expected/*`, `linters-cicd/codegen/scripts/*`, `.github/workflows/ci.yml`

---

## 1. Why It Happened
During CI pipeline execution of `bash linters-cicd/codegen/scripts/verify-codegen-determinism.sh`, the test failed with drift detected across generated Go and PHP files:
```text
Error: Codegen drift detected for lang=go
        Expected: /home/runner/work/coding-guidelines-v24/coding-guidelines-v24/linters-cicd/codegen/fixtures/expected/User.generated.go
        Actual:   /tmp/codegen-verify-GGOrlM/User.generated.go
```

## 2. How It Happened
When spec sequences in `02-spec/04-database-conventions/` were renumbered from `01-naming-conventions.md` to `03-naming-conventions.md`, the code generation emitters (`go_emitter.py`, `php_emitter.py`, `ts_emitter.py`) were correctly updated to cite `03-naming-conventions.md` in file headers. However, the committed fixtures in `linters-cicd/codegen/fixtures/expected/` had not yet been regenerated. Furthermore, verification relied on Bash scripts (`.sh`) which lack native Windows/PowerShell support.

## 3. Root Cause
1. Committed expected fixtures in `linters-cicd/codegen/fixtures/expected/` contained outdated header comments citing `01-naming-conventions.md`.
2. Codegen verification and regeneration relied on non-portable Bash scripts (`verify-codegen-determinism.sh`, `regen-codegen-fixtures.sh`).

## 4. Code Fix & Prevention Rule
1. **Shell-to-Python Conversion:** Migrated `verify-codegen-determinism.sh` and `regen-codegen-fixtures.sh` into pure, cross-platform Python scripts:
   - `linters-cicd/codegen/scripts/verify_codegen_determinism.py`
   - `linters-cicd/codegen/scripts/regen_codegen_fixtures.py`
2. **Fixture Regeneration:** Executed `regen_codegen_fixtures.py` to synchronize all expected fixtures (`User.generated.go`, `User.generated.php`, `User.generated.ts`).
3. **CI & NPM Integration:** Updated `.github/workflows/ci.yml` and `package.json` to invoke the Python scripts directly.
4. **Local Runner Quality Gate:** Registered `Codegen Determinism Check` in `03-ai-scripts/03-cicd-local-runner.py`.
5. **Prompt Rule Hardened:** Mandated in `01-prompts/08-dry-code/01-python-dry-architecture-and-caching.md` and `17-ci-cd/04-ci-cd-fix-with-release.md` that all CI verifiers must be written in cross-platform Python rather than `.sh`.
