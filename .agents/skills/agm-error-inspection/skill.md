---
name: agm-error-inspection
description: >-
  Inspect, diagnose, filter, and heal application errors from the SQLite speed error database (errors.db) using AGM CLI commands.
---

# AGM Error Inspection & Self-Healing Skill (`agm-error-inspection`)

This skill provides autonomous CLI diagnostics, querying, filtering, and self-healing for errors tracked in the Antigravity Manager SQLite speed error database (`errors.db`).

## 1. Primary CLI Commands & Usage

All error commands are accessed via the `agm errors` domain (aliases: `agm error`, `agm err`).

### List Recent Errors
```bash
# View the most recent 20 errors in formatted table output
agm errors

# View recent errors with a custom limit
agm errors list --limit 50

# Filter errors by severity level ('error', 'warn', 'info')
agm errors list --severity error

# Search errors by message keyword or error code
agm errors list "database locked"
agm errors list E9001

# Output errors in machine-readable JSON format for AI parsing
agm errors list --json
agm errors list --severity error --limit 10 --json
```

### Inspect Specific Error Record by ID
```bash
# View full error record with stack trace and metadata by ID
agm errors inspect e3b0c442-98fc-1c14-9afb-4c8996fb9242

# View inspection details in machine-readable JSON
agm errors inspect e3b0c442-98fc-1c14-9afb-4c8996fb9242 --json
```

### Display Error Metrics
```bash
# Display total count of recorded errors grouped by severity
agm errors count
```

### Automated Self-Healing & Diagnostics
```bash
# Inspect system anomalies without applying changes (dry run)
agm errors fix --dry-run

# Run automated self-healing to resolve stale file locks, WAL anomalies, and permissions
agm errors fix

# Run self-healing and return structured JSON diagnostic report
agm errors fix --json
```

### Purge Error Database
```bash
# Clear recorded error history from errors.db
agm errors clear

# Clear history with JSON confirmation
agm errors clear --json
```

---

## 2. JSON Response Contract

When querying with `--json`, the output conforms to `Vec<ErrorRecord>`:

```json
[
  {
    "id": "e3b0c442-98fc-1c14-9afb-4c8996fb9242",
    "error_code": "E9001",
    "severity": "error",
    "message": "Failed to parse conversation transcript",
    "stack_trace": "Error: Failed to parse\n    at parseTranscript (src/modules/repo_db/transcript.rs:420)",
    "source": "transcript_reader",
    "trigger_action": "read_prompt_turn",
    "metadata_json": "{\"cid\":\"conv-123\",\"path\":\"transcript.jsonl\"}",
    "created_at": 1729780800000
  }
]
```

---

## 3. Best Practices for AI Agents

1. **Check Errors on Failures:** When an operation in the GUI or background task reports an anomaly, run `agm errors list --limit 5 --severity error --json` to diagnose the root cause without guessing.
2. **Run Self-Healing Before Retrying:** If an error indicates a locked SQLite database or stale process lockfile, run `agm errors fix` to trigger the automated repair ladder before re-attempting the command.
3. **Purge After Verification:** After verifying a series of surgical fixes, clear the error database via `agm errors clear` so subsequent test runs start with a clean diagnostic baseline.
