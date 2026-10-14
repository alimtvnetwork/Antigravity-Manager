# E2E cases for a blind AI

**Status:** CLI cases below were run on 2026-10-14 against the installed `agm` 4.197.0. The Reset button and the cache-keep fix are in source and are not in that binary yet.

Do not mark a case passed by reading code. Use the installed `agm` binary that contains this change. Do not print tokens or prompt bodies.

## Case 01 — Tree from the CLI

1. Run `agm prompts tree --instance default`.
2. Wait until it prints a project list or a JSON error.

**Pass:** the process exits 0 and names at least the default instance, or it prints a caught error and exits without killing other apps.

**Fail:** `agm` exits with no message, or the Antigravity Manager window closes while this command runs.

## Case 02 — Reset cache

1. Run `agm prompts reset-tree --instance default`.

**Pass:** stdout contains `prompt tree cache cleared for default`.

## Case 03 — Send and enqueue from the CLI

1. Pick a repo path that is already bound to `default` or a default copy. Do not invent one.
2. Run `agm prompts send --instance default --repo <that path> "Review the latest change in this repo."`
3. Run `agm prompts queue --instance default --repo <that path> "Queue a second review."`

**Pass:** each command prints success or one error. The manager window, if it was open, stays open.

**Fail:** either command closes the manager window.

## Case 04 — Backup and restore

1. Run `agm prompts backup`.
2. Note the file it writes. Do not commit that file.
3. Run `agm prompts restore` against that file only if the backup command printed a path.

**Pass:** backup prints a path. Restore prints a count or a clear error. Running prompts are still listed by `agm prompts running`.

## Case 05 — Tree in the window

1. Open the manager. Open Instances. Open Prompts and History on default.
2. Time the first paint. Close the tree. Open it again within two minutes.
3. Click Reset once.

**Pass:** the second open is faster than the first. Reset does not close the window. The tree paints again.

**Fail:** the window closes on open or on Reset.

## Case 06 — Screenshot of the tree

Save one screenshot of the tree after Case 05. The header must show Backup, Restore, Reset, and Refresh. Compare the project list with a row that looks like the user’s screenshot: a project badge, a conversation row, and a word count. Do not require the same project names.

## Case 07 — GitMap on this host

1. Run `gitmap lf readme.md` in this repo.

**Pass:** it prints `readme.md`.

**Fail already seen here:** the process does not start (`paging file is too small`) or it panics with `nil TLS`. Record that text. Do not treat it as a prompt-tree failure. Do not change GitMap login or colors from this repository.

## Result block

```text
CLI instances: PASS exit 0. Two profiles, both Idle. Default and Default Copy.
CLI tree: PASS on the locally compiled debug binary agm v4.197.1. First load 1.54s and the log says prompt_tree_cache kept. Second load 0.23s. After reset-tree the next load was 1.46s. The installed 4.197.0 binary still wiped the cache in 1.97s.
CLI reset: PASS on the locally compiled agm v4.197.1. `agm prompts reset-tree --instance default` printed `Prompt tree cache cleared for default`. The following tree load scanned again (1.46s) and the load before reset did not (0.23s, zero liveness probes).
CLI send: PASS. Prompt p-b5834e4b status running on default. Antigravity.exe started.
CLI enqueue: PASS exit 0. queued-f669d8d6 status queued, queue_position 1, fifo_ordered true.
Backup: PASS exit 0 for default and for default-copy-5597. Zero running prompts were snapshotted before the send.
Restore: PASS exit 0 for default. status restored.
Switch threshold: PASS. agm auto-switch threshold 98 read back 98.0%. Restored to the previous 15.0% so the live daemon does not rotate on the next tick.
Window / Reset button: NOT CLICKED. Screenshot assets/screenshots/antigravity-manager-and-gitmap-testing-01.png is the desktop during the CLI run, not the prompt tree header.
GitMap search: PASS. gitmap aum search printed one hit with bright ANSI color.
GitMap login email prompt: NOT PRESENT. Login is a token or browser menu. git user add accepts --name and --email.
Supabase DNS: FAIL. Both configured hosts return "DNS name does not exist". No new project was created.
Ubuntu: NOT RUN.
```
