# E2E-154-02 — DB evidence queries

Evidence Id: `E2E154-20261019a-<instance>-02`

## Query 1 — restored row for #P001 (run on the machine)

```sql
SELECT id, original_prompt_id, has_images, is_restored
FROM prompt_backups
WHERE original_prompt_id = '#P001';
```

Expected: `is_restored = 1`, `has_images = 1`.

**Output:** pending machine execution.

## Query 2 — restore read order (static reference, from `src-tauri/src/modules/backup_prompts_db/restore.rs:58`)

```sql
-- restore queries read ORDER BY created_at ASC, sequence_id ASC (FIFO)
SELECT id, sequence_id, has_images
FROM prompt_backups
WHERE backup_batch_id = ?
ORDER BY created_at ASC, sequence_id ASC;
```

FIFO ordering verified by code review (CI-verified-by-review). Live output
pending machine execution.
