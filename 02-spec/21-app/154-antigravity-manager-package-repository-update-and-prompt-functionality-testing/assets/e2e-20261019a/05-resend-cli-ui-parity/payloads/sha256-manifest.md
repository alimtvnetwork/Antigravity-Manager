# E2E-154-05 — sha256 manifest: UI vs CLI resend payloads

Evidence Id: `E2E154-20261019a-<instance>-05`. Prompt Id: `#P003`.

| Field | UI path (ResendAsModal, target_mode='new') | CLI path (`agm prompts resend-queued`) | Match |
|-------|---------------------------------------------|----------------------------------------|-------|
| text sha256 | _pending_ | _pending_ | _pending_ |
| image_payload sha256 | _pending_ | _pending_ | _pending_ |

Expected: text verbatim-identical, `image_payload` sha256-identical across
both paths. No raw base64 stored in evidence (hashes only).

Fill on the machine by hashing the decoded text and image bytes of each
dispatched payload.
