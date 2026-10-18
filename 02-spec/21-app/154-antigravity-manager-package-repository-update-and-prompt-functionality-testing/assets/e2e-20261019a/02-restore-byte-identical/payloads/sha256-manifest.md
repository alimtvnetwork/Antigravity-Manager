# E2E-154-02 — sha256 manifest: restored images vs originals

Evidence Id: `E2E154-20261019a-<instance>-02`. Prompt Id: `#P001`.

| # | Original sha256 | Restored sha256 | Match |
|---|-----------------|-----------------|-------|
| 1 | _pending_ | _pending_ | _pending_ |
| 2 | _pending_ | _pending_ | _pending_ |

Expected: byte-identical; images render in the UI (no broken placeholders).

Fill on the machine with:

```sh
sha256sum <original-image-1> <original-image-2>
# compare against the restored image bytes for #P001
```
