# E2E-154-04 — DB evidence queries

Evidence Id: `E2E154-20261019a-<instance>-04`

## Query 1 — rerun row carries the image (run on the machine)

```sql
SELECT id, status, image_payload IS NOT NULL AS has_image_payload, created_at
FROM active_prompts
WHERE id IN ('#P002', '#P002-R1')
ORDER BY created_at ASC, id ASC;
```

Expected: `#P002-R1` row present with `has_image_payload = 1`.

**Output:** pending machine execution.

## Query 2 — side-channel file (run on the machine)

Inspect `.antigravity_resume_task.json` for the rerun: assert the
`image_payload` (base64) entry exists for `#P002-R1` and its sha256 matches the
original image. Record the excerpt (redacted to sha256 only — no raw base64 in
evidence) in `payloads/sha256-manifest.md`.

**Output:** pending machine execution.
