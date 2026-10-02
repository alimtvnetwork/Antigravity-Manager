# 60 — Switch and fast-forward drop the live conversation

## What broke

Account switch and fast-forward did not show the running prompt in the audit row, and they did not put that prompt back into the same conversation.

## Why

1. Discovery required a `USER_INPUT` line in `transcript.jsonl`. A conversation that only had a preview was discarded, so `active_prompts` stayed empty, the prompt-channel wait was skipped, and the audit snapshot was blank.
2. If `conversation_summaries` had no `status` column, the query failed and that database was skipped.
3. A named instance with an empty home `.gemini` tree searched the global tree, then dropped those rows when the workspace path was not already listed as a running project.
4. Dispatch rewrote `.antigravity_resume_task.json` without `session_id`. The IDE uses that id to reopen the same conversation. `agy -p` does not. An already-alive `agy` worker was reported as success without sending anything.
5. The audit snapshot listed prompts by `created_at` with a limit of 50, so a just-updated older row could be missed.
6. Every running check scanned all processes. Quota checks could run again in under 2 minutes.

## Fix

Preview text is stored when the transcript has no `USER_INPUT`. The summaries read falls back when status columns are missing. The resume file keeps `session_id`. The audit payload stores that conversation id and the prompt text. Google checks wait at least 2 minutes. The saved PID is trusted until the 10-minute cache refresh, or until credit is under the threshold and that PID no longer matches.

## Not claimed

A live IDE was not switched, and a live prompt was not watched being injected. The proof is the unit tests for preview fallback, the resume document, the summaries fallback, the audit payload, the 120-second floor, and the PID refresh clamp.
