# Spec 92: CLI instance switch on a copy, without closing the open IDE

Status: in progress
Date: 2026-10-01
Related: specs 26, 72, 73, 85. Plan: `.ai-memory/plans/pending/92-cli-instance-switch-e2e.md`

## User request (verbatim, task portion)

Focus on instance switching with CLI commands, not the IDE UI. Do not close the IDE that is already open. Record its PID and folder. Create a new instance that is a full IDE copy. On that copy, create a running prompt and queued prompts (Gitmap tests, or a CI check). Before close, back up the exact running prompt and the queued prompts. Close only the PID whose command line matches that instance folder. Switch the account (try fast-forward and switch). Reopen that instance. Push the backed-up prompt back so it is running, and put the queued prompts back on the queue. Prove it with a CLI query. Open Settings on that instance and confirm the account changed, matching PID to folder. Switch the account back. Also fix: instance selection stuck on worker-alpha, Focus scroll and contrast, Instances/Parts panels with no close button, prune/clear must keep running and queued prompts and the last conversations of active projects (UI, CLI, Telegram), duplicate project names in email and Telegram, Telegram HTML, more prompt text, `/update` in status, light text on dark, SSH/node examples. Pull before work. Fix CI first and record the root cause.

No screenshot was attached to this request.

## Protected at start

- Cursor (this chat). AGM `agm-alim.exe` PID 10584 and its WebView children.
- `close_instance` now refuses PIDs whose name or exe is Cursor or AGM.
- Switching the **default** instance still calls `close_antigravity` for non-sandbox IDE processes (`instance.rs` around the `is_default_inst` branch). The proof run must use a new copy, never the default instance.

## What already exists

From the CLI discovery on 2026-10-01: `agm instances create --from`, `instances stop` / `rm`, `instances switch`, `ff`, `backup-running-prompts`, `restore-running-prompts`, `which-prompts-running`, `running-prompts`. PID resolution is `find_pids_for_data_dir`. Copy data lives under the instance `data_dir`. `scripts/test-instance-e2e.ps1` already creates two instances and switches one of them. It does not verify restore status, and its screenshot is synthetic.

## Fix in this pass

1. `prompt_status_after_restore` in `backup_prompts_db.rs`: saved `queued` stays `queued`. Any other saved status is restored as `backed_up`, which `dispatch_running_prompts` sends again.
2. `resend_running_commands_for_instance` skips `queued`, so a resend cannot mark the queue as dispatched.
3. `agm restore` prints `queued` or `running` from the saved status, not a hardcoded `queued`.
4. `close_instance` drops Cursor and AGM PIDs before `taskkill`.

## Still open

- Run the copy-instance scenario on a built CLI (the installed `agm-alim.exe` is 4.109 and does not contain this fix yet).
- UI selector bounce, Focus contrast, panel close buttons, email/Telegram duplicates. Waiting on the UI discovery pass.
- CI at the start of this run was green (`78f4a589`).
