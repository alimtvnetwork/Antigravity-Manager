# E2E-154-03 — DB evidence queries

Evidence Id: `E2E154-20261019a-<instance>-03`

## Query 1 — queue order for the seeded prompts (run on the machine)

```sql
SELECT id, status, created_at
FROM active_prompts
WHERE instance_id = '<instance>'
  AND status IN ('running', 'queued', 'pending', 'dispatched')
ORDER BY created_at ASC, id ASC;
```

Expected: row 1 = `#P010` (`running`), rows 2–7 = `#P011`…`#P016` (queued, FIFO).
Panel positions 1–4 must equal rows 2–5 of this query; `up_next_total` = 6.

**Output:** pending machine execution.

## Query 2 — panel source queries (static reference, from `src-tauri/src/commands/cmd_running_up_next.rs:79,98`)

```sql
-- now_running
SELECT id, instance_id, status, created_at, image_payload IS NOT NULL
FROM active_prompts
WHERE status = 'running' AND instance_id = ?
ORDER BY created_at ASC, id ASC LIMIT 1;

-- up_next
SELECT id, instance_id, status, created_at, image_payload IS NOT NULL
FROM active_prompts
WHERE status IN ('queued', 'pending', 'dispatched') AND instance_id = ?
ORDER BY created_at ASC, id ASC LIMIT 4;
```

FIFO ordering verified by code review (CI-verified-by-review). Live output
pending machine execution.
