# E2E-154-07 — payload manifest

Evidence Id: `E2E154-20261019a-<instance>-07`.

No image bytes in this case. Expected `get_running_and_up_next` response:

```json
{
  "now_running": null,
  "up_next": [],
  "up_next_total": 0,
  "instance_id": "<instance>",
  "as_of_ms": "<backend timestamp>"
}
```

Panel must hide itself on this response (spec §1.1).

Actual payload: pending machine execution.
