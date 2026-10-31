# E2E-154-06 — DB evidence queries

Evidence Id: `E2E154-20261019a-<instance>-06`

## Query 1 — full queued count + order (run on the machine)

```sql
SELECT COUNT(*) AS queued_total
FROM active_prompts
WHERE instance_id = '<instance>'
  AND status IN ('queued', 'pending', 'dispatched');

SELECT id, status, created_at
FROM active_prompts
WHERE instance_id = '<instance>'
  AND status IN ('queued', 'pending', 'dispatched')
ORDER BY created_at ASC, id ASC;
```

Expected: `queued_total = 8`; the 8 rows in FIFO order `#P020`…`#P027`.
Panel must show rows 1–4 + `+4 more`; expanded view must match all 8 rows.

**Output:** pending machine execution.
