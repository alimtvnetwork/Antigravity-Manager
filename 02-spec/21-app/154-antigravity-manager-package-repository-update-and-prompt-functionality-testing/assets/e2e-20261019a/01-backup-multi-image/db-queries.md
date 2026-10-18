# E2E-154-01 — DB evidence queries

Evidence Id: `E2E154-20261019a-<instance>-01`

## Query 1 — backup row for #P001 (run on the machine)

```sql
SELECT id, original_prompt_id, has_images, created_at
FROM prompt_backups
WHERE original_prompt_id = '#P001';
```

Expected: exactly one row, `has_images = 1`.

**Output:** pending machine execution.

## Query 2 — source queue order (static reference, from `src-tauri/src/modules/backup_prompts_db/backup.rs:285`)

```sql
SELECT id, status, created_at
FROM active_prompts
WHERE status = 'queued'
ORDER BY created_at ASC, id ASC;
```

This is the query the backup path executes; FIFO ordering verified by code
review (CI-verified-by-review). Live output pending machine execution.
