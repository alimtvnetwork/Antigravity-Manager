# E2E-154-06 — payload manifest

Evidence Id: `E2E154-20261019a-<instance>-06`.

No image bytes in this case. The operative payload is the `get_running_and_up_next`
response for 8 queued prompts:

```json
{
  "now_running": null,
  "up_next": [ { "id": "#P020" }, { "id": "#P021" }, { "id": "#P022" }, { "id": "#P023" } ],
  "up_next_total": 8,
  "instance_id": "<instance>",
  "as_of_ms": "<backend timestamp>"
}
```

Expected: `up_next` = first 4 FIFO, `up_next_total` = 8 (display truncation only).

Actual payload: pending machine execution.
