# 30 — Running Prompt Resumption, Same-Conversation Continuity, and PID Recovery RCA

## Executive Summary
Prior to release 4.124.0, account switching, fast-forward actions, and IDE restarts frequently failed to restore running prompts or link them back into their active conversation sessions. Furthermore, the audit log entries lacked prompt snapshots, and process monitoring generated high CPU overhead. This RCA documents the comprehensive failure modes, architectural root causes, and definitive safeguards.

---

## 1. What Broke
1. **Prompt Drop on Switch**: When performing account rotation or fast-forward switching, the active in-flight prompt was not backed up into `active_prompts`, causing the switch process to treat the instance as idle and skip re-injection.
2. **Loss of Conversation Context**: Even when a prompt was dispatched to `.antigravity_resume_task.json`, the IDE reopened into a new blank chat session rather than resuming the existing conversation thread.
3. **Empty Audit Records**: In the Audit page, switch and update records displayed empty or masked prompts instead of the active task payload.
4. **PID Desynchronization & High CPU**: Process monitoring repeatedly traversed the entire OS process table every few seconds. When wrapper scripts (e.g. macOS `open`) exited after launching the child IDE, the saved PID became invalid, causing false negatives in instance status.

---

## 2. Root Cause Analysis (4-Part Deep Dive)

### Cause 1: Overly Restrictive Discovery Filter
In `discover_running_prompts_from_antigravity`, the parser required a line with `"type": "USER_INPUT"` inside `transcript.jsonl`. When an interaction was in preview mode, or initiated via subagent/assistant workflow where only previews or model outputs were logged, the entire transcript was discarded. As a result, `active_prompts` remained empty, `count_backed_up_prompts` returned 0, and the backup channel skipped waiting.

### Cause 2: Schema Incompatibility in SQLite Summaries
The `conversation_summaries` table schema varied between Antigravity versions. When AGM issued queries selecting `status` or other columns that were absent in older database schemas, the SQL query threw an unhandled error, aborting discovery for that entire instance database.

### Cause 3: Path Mapping Disconnect in Named Instances
Named instances configured with an empty `.gemini` home directory fell back to reading the global `.gemini` directory. Discovered conversations were then cross-referenced against `detect_running_projects`. Because the global workspace path did not match the instance's registered projects, the valid session was dropped.

### Cause 4: Truncated Resume Task Document
During dispatch, `dispatch_running_prompts` serialized `.antigravity_resume_task.json` using a minimal schema that omitted `session_id` and `conversation_id`. When Antigravity restarted and read the file, it lacked the conversation identifier required to reconnect to the existing conversation, forcing a blank session. Furthermore, if `agy` had an existing daemon worker alive, `spawn_prompt_via_agy` prematurely considered the prompt sent without re-injecting the task.

### Cause 5: Audit Snapshot Query Inversion
The query populating the audit log ordered prompts by `created_at DESC` with a limit of 50. Older prompts that were recently updated and running were excluded from the snapshot, resulting in empty prompt text in the audit view.

---

## 3. The Definitive Fix
1. **Preview Fallback (`live_prompt_text`)**:
   - If `transcript.jsonl` contains no `USER_INPUT` line, the preview string from `conversation_summaries` is retained and stored as the active prompt content.
2. **Schema Resilience (`read_conversation_summary_rows`)**:
   - Query attempts full column selection first; if columns are missing, it falls back to basic `conversation_id, preview, workspace_uris` projection without crashing.
3. **Session Preservation in Resume Task (`resume_task_document`)**:
   - Both `session_id` and `conversation_id` are explicitly persisted in `.antigravity_resume_task.json` on disk. Antigravity IDE and CLI use this ID to resume the exact thread.
4. **Audit Snapshot Precision (`switch_prompt_snapshot`)**:
   - Prompts are queried by `updated_at DESC` specific to the active `instance_id`, capturing the latest active conversation ID and content.
5. **PID Cache & Process Identity Verification (`process_identity_matches`)**:
   - The saved PID is verified directly against executable path identity before falling back to table scans. A floor of 120 seconds is enforced on quota checks to eliminate CPU thrashing.

---

## 4. Verification and Future-Proofing
- **Unit Tests**:
  - `live_prompt_uses_preview_when_the_transcript_has_no_user_input`
  - `resume_document_keeps_the_same_conversation_id`
  - `summary_rows_fall_back_when_status_columns_are_missing`
  - `saved_pid_identity_and_refresh_floor`
- **Future Guidelines**:
  - Never drop a conversation solely because explicit user message markers are absent in the log.
  - Never rewrite `.antigravity_resume_task.json` without preserving `conversation_id` and `session_id`.
  - Always verify SQLite schemas defensively with column fallbacks.
