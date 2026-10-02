# 59 — Account switch waits on quota fetch and the mailbox

**Status:** Fixed in v4.122.0. The switch command returns after the IDE credentials are written and the process is relaunched.

## Observation

Clicking Switch kept the accounts row spinning long after the click. The IDE close and reopen was only part of that wait.

## Cause

`account::switch_account` did not return until three extra jobs finished:

1. `fetch_quota_with_retry` ran before the IDE was closed. Since v4.120.0 that fetch always calls `loadCodeAssist`, so every switch waited on Google even when the account row already had a quota.
2. After the IDE was back, `select_candidate_profiles` called `fetch_recent_cross_vm_switched_accounts`, which polls the mailbox (`poll_unread_messages`) only to fill "predicted next" on the notification. That call is synchronous, so it blocked the async switch.
3. `notify_account_switched_details(...).await` then sent email and Telegram before the command returned, so the toast waited on SMTP.

`on_account_switch` also always called `wait_for_instance_prompt_channel`, which sleeps 1.5s after the process exists, even when no prompt was backed up.

## Fix

- The stored quota is used for the switch. Quota refresh, the mailbox poll, and email/Telegram run in a spawned task after the command returns.
- `wait_for_instance_prompt_channel` runs only when `count_backed_up_prompts` is greater than zero. A switch with nothing to re-push does not sleep.
- A running prompt is still backed up before close, and still re-pushed once after the process is up. Queued prompts stay queued.

## Where

- `src-tauri/src/modules/account.rs` `switch_account`
- `src-tauri/src/modules/instance.rs` `switch_account_to_instance`
- `src-tauri/src/modules/integration.rs` `DesktopIntegration::on_account_switch`
- `src-tauri/src/modules/repo_db.rs` `count_backed_up_prompts`, `needs_prompt_channel_wait`

Do not put `fetch_quota_with_retry` or `select_candidate_profiles` back in front of the `Ok(())` of a switch.
