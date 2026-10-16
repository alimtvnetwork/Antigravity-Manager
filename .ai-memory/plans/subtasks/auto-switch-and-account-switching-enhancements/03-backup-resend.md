# Task-03: Prompt backup + CLI/UI re-send

## Status
PENDING (needs Task-01)

## Objective
Durably back up running + enqueued prompts; allow re-sending all enqueued
prompts later via CLI and UI.

## Requirements
- Backup captures: prompt text, destination (instance/conversation),
  image/attachment references, enqueue order (FIFO preserved).
- New CLI command (e.g. `agm prompts resend-queued`) that re-sends all
  enqueued prompts in FIFO order.
- UI button in the prompts tree / queue view: "Re-send queued prompts".
- Deduplication: re-send must not duplicate prompts already dispatched
  (honor RCA-15 dedup rules).
- No silent errors; real feedback on counts (sent/skipped/failed).

## Acceptance
- [ ] Backup contains all fields; FIFO order preserved.
- [ ] CLI re-send works; UI button works.
- [ ] `cargo fmt` + `clippy` clean; `npm run build` passes.

## Constraints
- GitMap search only. Relative paths only.
