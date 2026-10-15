# 01 — Prompt tree and CLI (v4.197.1, then v4.198.0)

Source of the recorded results: `.ai-memory/plans/subtasks/antigravity-manager-and-gitmap-testing/02-e2e-cases-for-ai.md` and section 11 of `.ai-memory/memory/learned/27-e2e-instance-autoswitch-prompt-playbook.md`.

Why these cases exist: opening Prompt Tree crashed WebView2 (`0xe0000008`) because `loadTree` rebuilt on every render. Restore hung because the detached IDE inherited the CLI pipes. Send looked successful while `agy` rejected `--resume`.

## Prepare

- [ ] `git log -1 --oneline`
- [ ] Build `agm` with `-j 1` if the installed binary is older than that commit
- [ ] `agm instances ls` — expect `default` and `default-copy-5597`. Do not create a third instance if both exist
- [ ] Note `agm auto-switch status --json` so you can restore 15.0

## Cases

- [ ] **Tree cold.** `agm prompts tree -i default`. Pass: exit 0, project list, log says `prompt_tree_cache kept` on a fixed binary. Record duration.
  - Observed on local 4.197.1: first load 1.54s, cache kept. Installed 4.197.0 still wiped the cache in 1.97s (fail for that binary).
  - Observed on the v4.198.0 fixed build: about 2402 ms cold.
- [ ] **Tree warm.** Run the same command again immediately. Pass: faster than cold.
  - Observed: 0.23s on 4.197.1. About 183 ms on the fixed build.
- [ ] **Reset.** `agm prompts reset-tree -i default`, then tree once more. Pass: stdout says the cache was cleared for default, and the next load is slow again.
  - Observed: next load 1.46s after reset.
- [ ] **Send.** Pick a repo path already bound to the instance. `agm prompts send -i default -r "<that path>" "Review the latest change in this repo."` Pass: prompt id printed, status running, manager stays open.
  - Observed: `p-b5834e4b` running on default. `Antigravity.exe` started. Later that prompt was completed.
- [ ] **Enqueue.** `agm prompts queue -i default -r "<that path>" "Queue a second review."` Pass: status `queued`, `fifo_ordered true`, `queue_position` 1.
  - Observed: `queued-f669d8d6`, then later `queued-25cf7b9a` on binary 4.197.1.
- [ ] **Backup.** `agm prompts backup -i default --json`. Pass: exit 0 and a path or a count within a few seconds.
  - Observed: PASS for `default` and `default-copy-5597`.
- [ ] **Restore.** `agm prompts restore -i default --json`. Pass: exit 0 in seconds, no `FOREIGN KEY constraint failed`.
  - First binary: FAIL, no JSON for 78 seconds while Default was running (IDE inherited stdout).
  - After `Stdio::null` on detached spawn: PASS in 162 ms.
- [ ] **Auto-switch dry run.** `agm auto-switch threshold 98`, status read-back is 98.0, `agm auto-switch test 98` returns, then `agm auto-switch threshold 15`. Pass: final status is 15.0.
  - Observed: restored to 15.0.
- [ ] **Window.** `agm instances open-ui`. Open Instances. Open Prompt Tree on Default. Click Reset once. Pass: both cards visible, Auto-Switch ON, tree paints, threads stay near 50, window stays open.
  - Observed on the fixed build: All Projects (48), threads 51–54. Reset and Copy worked.
  - Earlier screenshot `assets/screenshots/antigravity-manager-and-gitmap-testing-01.png` was the desktop during CLI, not the tree header. Do not call that a tree pass.
- [ ] **GitMap.** `gitmap lf readme.md` prints `readme.md`. A paging-file or TLS panic is a host GitMap failure, not a prompt-tree failure.
- [ ] **Supabase.** `agm supabase status` lists the two real hosts only. DNS `NXDOMAIN` is `CLOUD PAUSED`. Do not create a project.
- [ ] **Email.** `agm email ls` shows one account and one recipient. `agm email status` prints `[SUCCESS]` when the user asked for full end-to-end.

## Stop

- [ ] Threshold is 15.0
- [ ] Result block has PASS or FAIL on every line above
