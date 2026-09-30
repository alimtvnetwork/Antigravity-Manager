# CI/CD RCA 41: Test Inventory Generator Crashes on Synced Manifest Schema

## 1. Symptoms

The change-recording step that every N-step execution prompt mandates failed:

```text
python 03-ai-scripts/33-test-inventory-generator.py --record <files...>
  File "03-ai-scripts/33-test-inventory-generator.py", line 489, in record_recent_changes
    for tid, tmeta in inventory_tests.items():
AttributeError: 'list' object has no attribute 'items'
```

`--check-age` failed silently inside its `try` and reported `"error": "'int' object has no attribute 'replace'"` instead of an age.

## 2. Root Cause

`.ai-memory/test-inventory.json` was written by a synced template, not by this generator, and uses a different schema:

| Field | Generator writes | Committed manifest has |
|-------|------------------|------------------------|
| `tests` | dict keyed by test id | `[]` (list) |
| `updated_at` | ISO-8601 string | Unix epoch integer (`1789669500`) |
| `version` | `1` | `"1.0.0"` |

Both readers (`record_recent_changes`, `check_inventory_age`) assumed the generator's own schema. Nothing validated the manifest shape, so the first agent to run `--record` against the synced file hit the crash, and agents skipped the step instead of fixing it.

## 3. Fix

- Added `normalize_inventory_tests()`: accepts `tests` as a dict or a list (keyed by `id`, then `test_file`, then index) and drops non-dict entries. Both readers use it.
- `check_inventory_age` accepts `updated_at` as an epoch number or an ISO string.

Verified: `--record` exit 0 (11 files recorded); `--check-age --json` returns `age_days: 13.21`, `is_fresh: false` (exit 1 means stale by design, not a crash); `python -m py_compile` exit 0.

## 4. Prevention

- Readers of shared JSON manifests must tolerate every schema that the sync pipeline can write. Normalize at load time; never index by assumed type.
- A mandated step that crashes is a bug to fix and record, not a step to skip.
- `.ai-memory/temp/recent-file-changes.json` is tracked but is a per-run cache. Do not stage it in feature commits (prompt rule R15).
