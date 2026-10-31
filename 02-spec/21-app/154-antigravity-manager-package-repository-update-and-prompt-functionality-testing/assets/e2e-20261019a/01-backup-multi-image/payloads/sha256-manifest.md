# E2E-154-01 — sha256 manifest: stored image blobs vs source files

Evidence Id: `E2E154-20261019a-<instance>-01`. Prompt Id: `#P001`.

| # | Source file (relative) | sha256(source) | Blob sha256 (from backup) | Match |
|---|------------------------|----------------|---------------------------|-------|
| 1 | _pending_ | _pending_ | _pending_ | _pending_ |
| 2 | _pending_ | _pending_ | _pending_ | _pending_ |

Expected: every stored blob sha256-matches its source file byte-for-byte.

Fill on the machine with:

```sh
sha256sum <source-image-1> <source-image-2>
# compare against blob bytes extracted from backup_prompts.db for #P001
```
