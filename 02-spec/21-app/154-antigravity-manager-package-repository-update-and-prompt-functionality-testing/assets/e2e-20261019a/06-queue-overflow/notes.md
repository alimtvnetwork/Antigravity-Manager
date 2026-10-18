# E2E-154-06 — queue overflow (>4) truncation (run 20261019a)

- Prompt Ids: `#P020`–`#P027` (8 queued, known FIFO order; planned, assigned at execution)
- Evidence Id: `E2E154-20261019a-<instance>-06`
- Verdict: **INCOMPLETE** — panel rendering needs a running app with display. Static contract verified below.
- Verification class: CI-verified-by-review (static, below) + machine-verified (pending).

## Statically verified here (CI-verified-by-review)

- Display truncation is count-only: `query_up_next` uses `LIMIT 4`
  (`src-tauri/src/commands/cmd_running_up_next.rs:98`) while
  `query_up_next_total` counts ALL queued (`cmd_running_up_next.rs:122`)
  — `up_next_total` is the full count ("four or whatever the queue holds"),
  so `+N more` has a truthful N and every queued prompt stays reachable.
- The hook exposes `upNextTotal` from the backend (`useRunningAndUpNext.ts`,
  `setUpNextTotal`, lines 145–149) for the `+N more` affordance.
- FIFO for the 8 seeded prompts comes from `ORDER BY created_at ASC, id ASC`
  (SV-2); expanding `+N more` must reveal all 8 in that order.

## Machine-verification steps (exact commands, run on a real machine)

```sh
# pre: build agm, agm instances test-create --json -> <instance>
# seed 8 queued prompts #P020..#P027 in known FIFO order on <instance>
# open the tree view; read the panel
# expected: 4 rows (#P020..#P023) + `+4 more`; expand -> all 8 in FIFO order; none unreachable
# screenshots: screenshots/06-collapsed.png (4 + `+4 more`), screenshots/06-expanded.png (all 8)
# run the DB count query in db-queries.md; paste output
```

## Bugs found

None (static review found no defect in the truncation/count path).
