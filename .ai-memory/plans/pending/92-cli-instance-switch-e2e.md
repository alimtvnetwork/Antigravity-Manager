# Plan 92: CLI instance switch on a copy

> **Status:** IN PROGRESS
> **Spec:** [02-spec/21-app/92-cli-instance-switch-e2e.md](../../../02-spec/21-app/92-cli-instance-switch-e2e.md)

## Done this pass
- [x] Pull `main` (already up to date). CI green on `78f4a589`.
- [x] Record protected PIDs. Do not close Cursor or `agm-alim.exe` 10584.
- [x] Restore keeps queued prompts queued and re-sends the prompt that was running.
- [x] `close_instance` refuses Cursor and AGM processes.

## Remaining
- [ ] Build is required before the scenario can be run. The installed app is still 4.109.
- [ ] Scenario, on a new instance only: create copy, seed running + queued prompts, backup, fast-forward, switch, reopen, CLI-verify statuses, Settings check, switch back, delete the copy.
- [ ] UI: selector stuck on worker-alpha, Focus contrast and scroll, Instances/Parts close buttons.
- [ ] Prune/clear must keep running and queued prompts (confirm in code, then fix gaps).
- [ ] Email and Telegram: one project name each, prompt text, HTML, `/update`.
