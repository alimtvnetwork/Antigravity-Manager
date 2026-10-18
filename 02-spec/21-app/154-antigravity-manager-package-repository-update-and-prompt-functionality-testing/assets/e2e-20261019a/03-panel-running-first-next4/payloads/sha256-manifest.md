# E2E-154-03 — payload manifest

Evidence Id: `E2E154-20261019a-<instance>-03`.

No image bytes in this case. Panel row payload (from `get_running_and_up_next`)
is covered by the DB queries in `db-queries.md`.

Expected panel payload shape (per `RunningAndUpNext`, `cmd_running_up_next.rs:32`):

```json
{
  "now_running": { "id": "#P010", "status": "running", "origin_tier": "USER_PROMPT", "has_images": false },
  "up_next": [ { "id": "#P011" }, { "id": "#P012" }, { "id": "#P013" }, { "id": "#P014" } ],
  "up_next_total": 6,
  "instance_id": "<instance>",
  "as_of_ms": "<backend timestamp>"
}
```

Actual payload: pending machine execution.
