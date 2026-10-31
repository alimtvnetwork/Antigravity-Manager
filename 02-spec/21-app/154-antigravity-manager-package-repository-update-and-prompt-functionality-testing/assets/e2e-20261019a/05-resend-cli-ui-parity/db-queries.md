# E2E-154-05 — DB evidence queries

Evidence Id: `E2E154-20261019a-<instance>-05`

## Query 1 — resend rows for #P003 (run on the machine)

```sql
SELECT id, status, image_payload IS NOT NULL AS has_image_payload, created_at
FROM active_prompts
WHERE id LIKE '#P003%'
ORDER BY created_at ASC, id ASC;
```

Expected: the UI-resent and CLI-resent rows both present with
`has_image_payload = 1`.

**Output:** pending machine execution.

## Query 2 — CLI resend command output (run on the machine)

```sh
agm prompts resend-queued --json
```

Capture the JSON output (redact image bytes to sha256) into
`payloads/sha256-manifest.md`.

**Output:** pending machine execution.
