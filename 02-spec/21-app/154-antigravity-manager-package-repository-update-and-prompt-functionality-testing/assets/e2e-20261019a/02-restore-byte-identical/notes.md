# E2E-154-02 — restore rehydrates byte-identical images (run 20261019a)

- Prompt Id: `#P001` (restored from the E2E-154-01 backup)
- Evidence Id: `E2E154-20261019a-<instance>-02`
- Verdict: **INCOMPLETE** — no display, no Rust toolchain, no prebuilt `agm` binary on this VM. Nothing executed here.
- Verification class: CI-verified-by-review (static, below) + machine-verified (pending).

## Statically verified here (CI-verified-by-review)

- Restore reads enforce FIFO — `src-tauri/src/modules/backup_prompts_db/restore.rs:58,69,80`:
  `ORDER BY created_at ASC, sequence_id ASC` on the restore queries.
- `VwUnrestoredBackups` (`src-tauri/src/modules/db_orm/generated/views.rs:184`)
  selects `pb.HasImages`, so the restore path can rehydrate the image flag.
- Side-channel contract (spec §2, 01-architecture-spec.md §5.2): images ride
  out-of-band via `.antigravity_resume_task.json`; a broken side-channel must
  be a loud error, never a silent text-only downgrade.

## Machine-verification steps (exact commands, run on a real machine)

```sh
# pre: backup from E2E-154-01 exists for #P001 on <instance>
# restore via the app (account-switch backup restore path per spec 50, or CLI restore)
# open #P001 in the tree view; confirm images render (no broken placeholders)
# sha256-compare restored image bytes vs the originals; fill payloads/sha256-manifest.md
# screenshot: restored #P001 with images -> screenshots/02-restored-with-images.png
```

## Bugs found

None (static review found no defect in the restore path touched by this case).
