# Same-conversation resume, audit info modal, and PID cache

## User Request (Verbatim)

When you switch the account, with fast forward or switch account, the running prompts are not shown as backed up in the audit logs, and they are not resumed into the same conversation. The audit detail control should be an info button that opens a scrollable table, with copy and cancel, like the email JSON view. Google quota checks are never faster than 2 minutes. A process scan happens only when credit is under the threshold, or on the PID cache refresh (default 10 minutes, never under 3 minutes, never over 20). A PID saved at launch is trusted when that process path still matches.

## Root cause

`discover_running_prompts_from_antigravity` dropped a live conversation when the transcript had no `USER_INPUT` line, and it skipped the whole database when the status columns were missing. Named instances also fell back onto the global `.gemini` tree and then threw the row away when the workspace path was not in `detect_running_projects`. With nothing stored, `count_backed_up_prompts` was 0, the switch skipped re-inject, and `switch_prompt_snapshot` read only the newest 50 prompts by `created_at`, so the audit payload had an empty prompt.

`dispatch_running_prompts` then rewrote `.antigravity_resume_task.json` without `session_id`. `spawn_prompt_via_agy` treated an already-alive `agy` worker as success and did not send the prompt. `agy -p` starts a new CLI prompt. The same IDE conversation is the `session_id` on that resume file.

`is_instance_running` called `find_pids_for_data_dir` before the saved PID, so every quota tick scanned the process table. Critical quota could schedule the next Google check in 40 seconds.

## Behavior

- A missing `USER_INPUT` line still stores the conversation preview. A summaries table without status columns still returns the newest rows.
- Named instances read only that instance's home `.gemini` tree. A discovered conversation is stored even when the workspace path is not already in `running_projects`.
- The resume file always includes `session_id` and `conversation_id`. An already-alive `agy` worker is not treated as a successful re-push. The file with a conversation id still counts as the same-conversation resume.
- The audit payload records that conversation id and the prompt text from the newest `updated_at` row for that instance. The list query still omits `payload_json`.
- The audit row uses an info button. The detail is a modal: scrollable table, Copy, Cancel. It loads only on click.
- Google quota checks wait at least 120 seconds. The old default of 300 seconds is treated as 120. A custom value above 120 is kept.
- `is_instance_running` checks the saved PID and its executable path only. The full process table is rebuilt on `pid_refresh_seconds` (default 600, clamped to 180..=1200). A scan to locate the instance runs only when quota is under the threshold and the saved PID does not match.

## Conflicts

Research agents were already spawned for this task. The switch path shares `repo_db.rs`, `instance.rs`, and `auto_switcher.rs`, so the lead implemented the change instead of a second overlapping edit. The user asked for a minor release, which outranks the skill rule that forbids a version bump.
