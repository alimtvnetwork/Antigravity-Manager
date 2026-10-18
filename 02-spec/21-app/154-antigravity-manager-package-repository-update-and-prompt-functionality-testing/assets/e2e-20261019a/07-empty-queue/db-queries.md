# E2E-154-07 — DB evidence queries

Evidence Id: `E2E154-20261019a-<instance>-07`

## Query 1 — zero queued (run on the machine)

```sql
SELECT COUNT(*) AS queued_total
FROM active_prompts
WHERE instance_id = '<instance>'
  AND status IN ('queued', 'pending', 'dispatched');
```

Expected: `queued_total = 0`.

**Output:** pending machine execution.
