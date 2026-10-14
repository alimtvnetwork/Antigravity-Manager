# E2E cases for a blind AI

**Status:** `PENDING`. These steps were not executed on a live window.

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
CLI tree:
CLI reset:
CLI send:
CLI enqueue:
Backup:
Restore:
Window first open:
Window second open:
Reset button:
GitMap:
```
