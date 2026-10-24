---
name: agm-error-manager-cli
description: Read and clear Antigravity Manager Error Manager history from the CLI via agm errors, backed by errors.db.
---

# AGM Error Manager CLI

The dashboard Error Manager History drawer reads the in-memory Zustand store (`src/stores/error-store.ts`). The same failures are written to `errors.db` through `log_client_error`. When a message has no `[E####]` tag, the stored code is E9001.

## Commands

- `agm errors list` — recent rows (`--limit`, `--severity`, `--json`)
- `agm errors count`
- `agm errors clear`
- `agm errors fix` — repair pass (`--dry-run`)

`agm failed-commands` is a different table. It records unknown CLI input, not the Error Manager drawer.

## When to use this

Use `agm errors list --json` before changing dashboard error handling. Do not treat an empty drawer as an empty database. The drawer keeps the latest 50 session errors. The database keeps the persisted rows.
