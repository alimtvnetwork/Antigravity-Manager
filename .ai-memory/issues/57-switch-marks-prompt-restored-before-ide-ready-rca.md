# Issue 57: Switch marks the running prompt restored before the IDE can accept it

Status: fixed in code on `87135787`, released in v4.119.0. A live switch that watches one inject in `app.log` was not run.
Raised: 2026-10-02
Supersedes the observed failure in [50](./50-instance-switch-loses-running-prompt-rca.md)
Spec: [02-spec/21-app/95-accounts-columns-reinject-and-zip-update.md](../../02-spec/21-app/95-accounts-columns-reinject-and-zip-update.md)
Learning note: [learned/23](../learned/23-accounts-quota-reinject-and-how-to-find-it.md)

## 1. Symptom
Email and Telegram already reported success (`[Notify] email: OK`, `[Notify] telegram: OK`). The prompt that was running before the switch did not come back in the IDE. Specs 39, 72, and 92 said restore worked.

## 2. Trigger
Any account switch that launches the instance and then tries to restore: CLI, the Accounts switch, and auto-switch. Those three are not separate implementations. A non-default instance goes through `instance::switch_account_to_instance`. The default instance goes through `account::switch_account` and `integration::on_account_switch`. Auto-switch calls one of those two.

## 3. Root cause
`launch_instance` called `restore_running_prompts`, `resend_running_commands_for_instance`, and `dispatch_running_prompts` immediately after `cmd.spawn()`. That first pass set `prompt_backups.is_restored = 1` and `active_prompts.status = 'dispatched'` before the IDE process could accept a prompt. The later restore in the switch function selected `is_restored = 0` and `status = 'backed_up'` and found no row. A successful notify line next to a zero restore count is the signature.

`dispatch_running_prompts` only selects `status = 'backed_up'`. `resend_running_commands_for_instance` skips `queued`. A row marked `dispatched` before `spawn_prompt_via_agy` returns cannot be sent again.

## 4. Why it escaped
The restore function did run, so a log that only says "restore was called" looks like success. Earlier specs treated that call as proof. Issue 50 named a different cause (discovery reading the global state, and up to three resend callers). That earlier note stayed "open, not fixed", so a later reader could chase the wrong function. No live switch on a non-default instance was watched for the order `[Instance] Prompt channel ready` then `[RepoDB] Dispatched agy execution`.

## 5. Fix
`launch_instance` still reinjects, because a normal launch is not a switch. A switch calls `launch_instance_without_prompt_reinject`, which is the same launch with the reinject flag off. `wait_for_instance_prompt_channel` polls until the instance process exists (up to 12s), sleeps 1.5s, then the switch restores once. `spawn_prompt_via_agy` must return true before the row becomes `dispatched`. An already-running agy worker counts as success. A missing binary or a failed spawn leaves the row `backed_up` and logs the prompt id (`[RepoDB] Prompt '...' was not re-pushed`). Queued prompts stay queued. Auto-switch does not add a second restore; it only retries rows that are still `backed_up`.

## 6. Prevention
Count every call to `restore_running_prompts`, `resend_running_commands_for_instance`, and `dispatch_running_prompts` from close through notify. If launch also calls them, launch is the first inject and the later step is a second inject against an already consumed backup. Prove order from `app.log`. Do not add a third restore in the auto-switcher.

## 7. Regression check
On a non-default instance, with a prompt whose status is `backed_up`, one switch log must contain `[Instance] Prompt channel ready` before `[RepoDB] Dispatched agy execution`, and only one of those dispatch lines. A forced spawn failure must leave that prompt id `backed_up`. This live check was not run for v4.119.0.
