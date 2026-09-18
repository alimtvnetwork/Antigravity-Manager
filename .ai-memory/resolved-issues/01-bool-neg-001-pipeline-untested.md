# BOOL-NEG-001 Not Yet Validated In `run-all.sh` Pipeline

## Description

The new `BOOL-NEG-001` SQL check passes its standalone smoke test (4 forbidden names flagged, 10 allow-listed names pass), but it has not yet been exercised through the full `linters-cicd/run-all.sh` orchestrator. We do not know whether:

- Its findings are correctly merged into the combined SARIF.
- Its registry entry is loaded (it sits at the bottom of `checks/registry.json`).
- The synthetic STYLE-099 + the new BOOL-NEG-001 coexist without rule-id collisions.
- `--check-timeout` and `--jobs` parallelization correctly handle the new check.

## Root Cause

Under investigation. The check has only been exercised directly via `python3 checks/boolean-column-negative/sql.py --path /tmp/bn-fix`.

## Steps to Reproduce

1. Place a SQL file with both forbidden and allow-listed boolean column names in a clean repo.
2. Run `linters-cicd/run-all.sh --jobs auto --check-timeout 20`.
3. Inspect the merged SARIF for `ruleId: "BOOL-NEG-001"`.
4. Expected: 4 findings with file/line metadata. Actual: not yet verified.

## Attempted Solutions

- [x] Standalone smoke test — passed (4 findings, exit 1, allow-list silently passed).
- [x] Pipeline integration test — `tests/pipeline/check-bool-neg-001-pipeline.sh` (CI-blocking, v4.25.0).
- [x] Parallel run determinism check — exercised via `--jobs auto` in the same pipeline test.

## Priority

Resolved (v4.25.0).

## Blocked By

Nothing — closed by `tests/pipeline/check-bool-neg-001-pipeline.sh` and CI step "BOOL-NEG-001 pipeline smoke test (run-all.sh end-to-end)".
