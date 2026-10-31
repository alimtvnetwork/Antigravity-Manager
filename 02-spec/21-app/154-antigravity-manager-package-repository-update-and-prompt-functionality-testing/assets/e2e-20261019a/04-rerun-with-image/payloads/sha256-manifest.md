# E2E-154-04 — sha256 manifest: rerun image_payload vs original

Evidence Id: `E2E154-20261019a-<instance>-04`. Prompt Ids: `#P002` → `#P002-R1`.

| Artifact | sha256 | Match |
|----------|--------|-------|
| Original image (source file) | _pending_ | — |
| `image_payload` in dispatched payload (`#P002-R1`) | _pending_ | _pending_ |
| `image_payload` in `.antigravity_resume_task.json` (`#P002-R1`) | _pending_ | _pending_ |

Expected: all three sha256-identical. No raw base64 stored in evidence
(redacted to hashes only).

Fill on the machine with:

```sh
sha256sum <original-image>
# extract image_payload for #P002-R1 from the dispatched payload and the
# resume-task JSON, hash the decoded bytes, compare
```
