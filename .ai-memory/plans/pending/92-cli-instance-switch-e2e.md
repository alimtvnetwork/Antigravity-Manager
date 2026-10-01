# Plan 92: CLI instance switch on a copy

> **Status:** IN PROGRESS
> **Spec:** [02-spec/21-app/92-cli-instance-switch-e2e.md](../../../02-spec/21-app/92-cli-instance-switch-e2e.md)

## Done this pass
- [x] Pull `main` (already up to date). CI green on `78f4a589`.
- [x] Record protected PIDs. Do not close Cursor or `agm-alim.exe` 10584.
- [x] Restore keeps queued prompts queued and re-sends the prompt that was running.
- [x] `close_instance` refuses Cursor and AGM processes.
- [x] Instance list: a refresh that started before a click cannot overwrite the instance the user just chose.
- [x] Accounts Focus keeps the white background and dark text, and scrolls for up to 4 seconds.
- [x] Email dark-mode links and code chips use light text instead of cyan on navy.

## Proved with the installed CLI (agm 4.109.0) on 2026-10-01
- Protected process `agm-alim.exe` PID 10584 stayed alive through create, launch, switch, fast-forward, and switch-back.
- Created copy `cli-switch-proof-6857`. Folder: `C:\Users\Administrator\.antigravity_tools\instances\cli-switch-proof-6857\data`. Exe: `Antigravity-cli-switch-proof-6857.exe`.
- Seeded Gitmap prompts: `proof-running-1` running, `proof-queued-1` and `proof-queued-2` queued. Backup batch `batch_7b56d16fe8c246168ff5166e3a95226b` stored all three.
- Launch PID 9116, then after switch PID 11364, then after switch-back PID 8588. Each main process command line contained that data folder.
- `instances switch` to `erfan.office.n@gmail.com` (from `rokixshohag1@gmail.com`). Settings DB `state.vscdb` and `app_storage.json` both read `erfan.office.n@gmail.com`. Default instance stayed `wuberujija38@gmail.com` and was not running.
- `ff cli-switch-proof-6857` moved `erfan.office.n@gmail.com` to `emersondaina25@gmail.com`.
- Switch-back to `rokixshohag1@gmail.com`. Settings DB confirmed that address. The copy is still running as PID 8588.
- The installed CLI marked the two queued prompts `dispatched` as well as the running one. The source fix for that is not in this binary yet.

## Still open
- [ ] Rebuild `agm` and re-run restore on this copy so queued prompts stay `queued`. The installed 4.109.0 CLI does not contain that fix.
- [ ] Telegram message colors (email templates are updated; Telegram is HTML text, not these CSS rules).
- [x] Conversation prune is `agm prune` and already skips running and queued prompts. `agm clean` / `agm purge` only delete build artifacts.
