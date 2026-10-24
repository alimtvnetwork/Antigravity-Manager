---
name: agm-error-manager-cli
description: >-
  CLI entry for dashboard and SQLite error inspection (`agm errors list|inspect|count|fix|clear`).
  Use this skill when an agent needs to read, filter, or heal errors.db without guessing commands.
---

# AGM Error Manager CLI

Canonical inspection skill: `.agents/skills/agm-error-inspection/skill.md`.

Use these commands. Do not invent extra error domains.

```bash
agm errors list --limit 50 --json
agm errors list --severity error --json
agm errors inspect <id> --json
agm errors count
agm errors fix --dry-run --json
agm errors clear --json
```

Notes:

- `agm errors list` reads `errors.db`.
- The GUI error drawer reads the in-memory Zustand store. Clearing `errors.db` does not clear the drawer badge.
- Codes without an `[E####]` tag are stored as `E9001`.
- `failed_to_resolve_parent` is a stale history string. Current `cmd_files` reports `invalid_path: parent directory cannot be opened`.
- `NowRunningUpNext: prompt prompt-<uuid> not found in tree` is a warning when the running strip id is missing from tree data.
