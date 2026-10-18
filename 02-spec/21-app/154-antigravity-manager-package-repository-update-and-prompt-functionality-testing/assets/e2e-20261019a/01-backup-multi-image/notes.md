# E2E-154-01 — backup captures multi-image prompt (run 20261019a)

- Prompt Id: `#P001` (planned; assigned at execution)
- Evidence Id: `E2E154-20261019a-<instance>-01`
- Verdict: **INCOMPLETE** — no display, no Rust toolchain, no prebuilt `agm` binary on this VM. Nothing executed here.
- Verification class: CI-verified-by-review (static, below) + machine-verified (pending).

## Statically verified here (CI-verified-by-review)

- Backup queue reads enforce the house FIFO invariant — `src-tauri/src/modules/backup_prompts_db/backup.rs:285,290,294`:
  `... FROM active_prompts WHERE status = 'queued' ORDER BY created_at ASC, id ASC`. No inverted LIFO.
- `prompt_backups` carries `HasImages` (`src-tauri/src/modules/db_orm/generated/views.rs:142`, `VwQueuedPromptsFifo` selects `p.HasImages`; `VwUnrestoredBackups` at `views.rs:184` selects `pb.HasImages`) — the `has_images=true` assertion in spec §4 has a backing column.
- Backup queue source is `active_prompts` (same table the panel queries), so backup and panel share queue truth.

## Machine-verification steps (exact commands, run on a real machine)

```sh
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features
cargo build --bin agm
agm instances test-create --json            # transient sandbox; note the instance slug -> <instance>
# enqueue prompt #P001 with >=2 images on <instance> (images ride image_payload)
agm prompts backup-queued --json
sqlite3 backup_prompts.db "SELECT id, has_images FROM prompt_backups WHERE original_prompt_id = '#P001';"
# sha256-compare stored image blobs vs source files; fill payloads/sha256-manifest.md
# screenshot: prompt #P001 with images in the tree view -> screenshots/01-tree-with-images.png
agm instances test-clean
```

## Bugs found

None (static review found no defect in the backup path touched by this case).
