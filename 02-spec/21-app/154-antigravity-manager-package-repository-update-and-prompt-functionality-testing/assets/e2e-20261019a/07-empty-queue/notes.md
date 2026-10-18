# E2E-154-07 — empty queue state (run 20261019a)

- Prompt Ids: none queued (a running prompt may exist; planned)
- Evidence Id: `E2E154-20261019a-<instance>-07`
- Verdict: **INCOMPLETE** — tree-view rendering needs a running app with display. Static contract verified below.
- Verification class: CI-verified-by-review (static, below) + machine-verified (pending).

## Statically verified here (CI-verified-by-review)

- Empty-state contract (spec §1.1): the panel shows only when the tree view is
  open AND at least one prompt is running or queued; otherwise it hides itself
  (no empty chrome).
- Backend returns the empty shape cleanly: `query_now_running` maps
  `QueryReturnedNoRows` → `Ok(None)` (`src-tauri/src/commands/cmd_running_up_next.rs:88–91`),
  `query_up_next` returns an empty `Vec`, `query_up_next_total` returns 0 —
  so the hook receives `{ now_running: null, up_next: [], up_next_total: 0 }`
  and renders no rows; no error path is hit for "no rows".
- Hook error handling (`useRunningAndUpNext.ts:153–176`): an empty result is
  NOT an error — `hasError` stays false, no toast, no stuck skeleton.

## Machine-verification steps (exact commands, run on a real machine)

```sh
# pre: build agm, agm instances test-create --json -> <instance>; ensure zero queued prompts
# open the prompt tree view
# expected: panel hidden; no stuck skeleton; no error toast; tree renders normally
# screenshot: screenshots/07-tree-no-panel.png
# run the DB query in db-queries.md; paste output (expect 0 queued)
```

## Bugs found

None (static review found no defect in the empty-state path).
