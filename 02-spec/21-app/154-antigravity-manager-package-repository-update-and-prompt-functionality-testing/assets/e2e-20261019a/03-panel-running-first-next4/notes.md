# E2E-154-03 — panel shows running-first + next-4 in FIFO order (run 20261019a)

- Prompt Ids: `#P010` (running), `#P011`–`#P016` (queued, known FIFO order; planned, assigned at execution)
- Evidence Id: `E2E154-20261019a-<instance>-03`
- Verdict: **INCOMPLETE** — panel rendering needs a running app with display; screenshots pending. Static contract verified below.
- Verification class: CI-verified-by-review (static, below) + machine-verified (pending).

## Statically verified here (CI-verified-by-review)

### SV-1 — `get_running_and_up_next` IPC contract shape matches spec 01 §6.1

`src-tauri/src/commands/cmd_running_up_next.rs`:
- Command: `pub fn get_running_and_up_next(instance_id: Option<String>) -> Result<RunningAndUpNext, String>` (line 130) — matches `get_running_and_up_next(instance_id: string | null)`.
- Payload `RunningAndUpNext` (lines 32–44): `now_running: Option<PromptQueueRow>`, `up_next: Vec<PromptQueueRow>`, `up_next_total: i64`, `instance_id: String`, `as_of_ms: i64` — all five spec fields present.
- `PromptQueueRow` (lines 19–29): `id`, `instance_id`, `status`, `origin_tier`, `created_at`, `has_images` — the prompt Id and 4-tier origin field the user asked to see on every row.
- Registered in the Tauri invoke handler: `src-tauri/src/lib/run.rs:351` (`commands::get_running_and_up_next,`).

### SV-2 — FIFO ordering on the panel queries

- now-running: `ORDER BY created_at ASC, id ASC LIMIT 1` (`cmd_running_up_next.rs:79`).
- up-next: `ORDER BY created_at ASC, id ASC LIMIT 4` (`cmd_running_up_next.rs:98`).
- Statuses: `running` for position 0; `queued`/`pending`/`dispatched` for up-next — matches spec §6.2 (dispatched included so prompts don't vanish between dispatch and first heartbeat).
- Repo-wide audit: all other `ORDER BY ... DESC` hits are listings/history/prune paths (error-log listing, backup-batch listing, `prompt_history`, retention prune at `backup.rs:557` keeping newest N) — none in dispatch logic. No LIFO violation found.

### SV-3 — `useRunningAndUpNext.ts` polling/TTL logic matches spec

`src/components/instances/PromptTreeView/useRunningAndUpNext.ts`:
- Poll cadence 3s while mounted (`RUNNING_UP_NEXT_POLL_MS = 3_000`, line 33; effect lines 195–204).
- TTL: unseen 30s → `isStale` (`RUNNING_UP_NEXT_TTL_MS`, line 29); unseen 90s → drop (`RUNNING_UP_NEXT_DROP_MS`, line 31); never dropped on a single missed poll (lines 139–152).
- No re-sort: backend order rendered verbatim, running row index 0, queued rows keep FIFO with 1-based `queuePosition` (lines 122–136).
- Error routing: manual refresh → `captureError`, background poll → `trackWarning`; last-good items retained (lines 153–176). No silent errors.
- Immediate refresh on `prompt://dispatched` / `prompt://resumed` events (lines 207–240); staleness re-eval tick every 10s.
- Arg naming `{ instanceId }` matches the working repo convention (`get_project_conversation_tree`: frontend `instanceId`/`maxWords` at `usePromptTreeCoreActions.ts:292` ↔ Rust `instance_id`/`max_words` at `instance.rs:695`) — not a bug.
- Panel strip exists: `src/components/instances/PromptTreeView/NowRunningUpNext.tsx` (subtask 03).

## Machine-verification steps (exact commands, run on a real machine)

```sh
# pre: build agm, agm instances test-create --json -> <instance>
# seed 1 running (#P010) + 6 queued (#P011..#P016) in known FIFO order on <instance>
# open the prompt tree view; read the Now Running + Up Next strip
# expected: position 0 = #P010 (RUNNING); positions 1-4 = #P011..#P014 (FIFO);
#           `+2 more` affordance; 4-tier badges on all rows
# screenshot: screenshots/03-panel-running-first.png
# run the DB queue-order query in db-queries.md; paste output
```

## Bugs found

None (static review found no defect in the IPC contract, queries, or hook logic).
