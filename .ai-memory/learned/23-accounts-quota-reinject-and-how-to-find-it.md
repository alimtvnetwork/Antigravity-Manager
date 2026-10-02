# How the accounts, prompt re-push, and update-zip fix was found

Read this before changing the Accounts table, account switch, or the GitMap update zip. The symptom and the first log line are not the same thing.

Formal write-ups: [issue 57](../issues/57-switch-marks-prompt-restored-before-ide-ready-rca.md) is the prompt root cause. [issue 58](../issues/58-pro-badge-raw-tier-id-rca.md) is the raw tier id. [issue 47](../issues/47-pro-badge-missing-when-tier-not-fetched-rca.md) is still the cached-project skip. [issue 50](../issues/50-instance-switch-loses-running-prompt-rca.md) is an older hypothesis; do not treat it as the current cause.

## What looked done and was not

Email and Telegram already left `[Notify] email: OK` and `[Notify] telegram: OK`. Specs 39, 72, and 92 said the running prompt was restored. The restore function did run. It marked `prompt_backups.is_restored = 1` and `active_prompts.status = 'dispatched'` inside `launch_instance`, immediately after `spawn`, before the IDE could accept a prompt. The later step in `switch_account_to_instance` then queried `is_restored = 0` and `status = 'backed_up'` and found nothing. A zero restore count next to a successful email is the signature of this bug.

## How to find that class of bug

1. Read the switch function from the close step through the notify step. Count every call to `restore_running_prompts`, `resend_running_commands_for_instance`, and `dispatch_running_prompts`.
2. If a launch helper also calls those functions, the launch is the first inject. The later "step 5" is a second inject against an already consumed backup.
3. `dispatch_running_prompts` only selects `status = 'backed_up'`. `resend_running_commands_for_instance` skips `queued`. A prompt marked `dispatched` before `spawn_prompt_via_agy` returns cannot be sent again.
4. CLI, the Accounts switch, and auto-switch are not three implementations. Non-default goes through `instance::switch_account_to_instance`. Default goes through `account::switch_account` and `integration::on_account_switch`. Auto-switch calls one of those two. Fix the shared function. Do not add a third restore in the auto-switcher that marks the backup restored again.
5. Prove order from `app.log`, not from a hand-written success table. One line `[Instance] Prompt channel ready` must appear before `[RepoDB] Dispatched agy execution`. A failed spawn must leave the row `backed_up` and must log the prompt id.

## What the code does now

- `launch_instance_without_prompt_reinject` is what a switch calls. A normal launch still reinjects.
- `wait_for_instance_prompt_channel` waits until the instance process exists, then waits 1.5s, then the switch restores once.
- `spawn_prompt_via_agy` must return true before the row becomes `dispatched`. An already-running agy worker counts as success. A missing binary or a failed spawn does not.
- Queued prompts stay queued.

## Update zip

`agm update export-zip` downloads the current platform's GitHub release zip into `~/.antigravity_tools/update-export/agm-update.zip` and does not install it. GitMap `createUpdatePackageZip` for package `agm` sends that zip. The remote command stays PowerShell `Expand-Archive` on Windows and `unzip` on Linux and macOS.

## Files

- Quota columns and the More menu: `src/components/accounts/AccountTable.tsx`
- Tier normalization on fetch: `src-tauri/src/modules/quota.rs`
- One inject after ready: `src-tauri/src/modules/instance.rs`, `integration.rs`, `repo_db.rs`
- Zip export: `src-tauri/src/bin/agm.rs`
- Fleet zip: `D:\work\gitmap\cli\cmdupdate\update_fleet.go`
- Spec: `02-spec/21-app/95-accounts-columns-reinject-and-zip-update.md`
