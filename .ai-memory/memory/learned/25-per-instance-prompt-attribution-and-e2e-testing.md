# 25 - Per-Instance Prompt Attribution, What Was Coded, What Went Wrong, How Not to Break It, and Every E2E Test Discussed

- Written: 2026-10-06
- Status: permanent memory. Read this before touching prompts, instances, PIDs, the queue, backup, restore, the switch paths, or the prompt CLI and IPC.
- Parent plan (AI instruction, follow in order): `.ai-memory/plans/140-per-instance-prompt-attribution-backup-queue.md`
- Architecture spec (bug table B01..B33, decisions D1..D10, AC-01..AC-32): `02-spec/21-app/140-per-instance-prompt-attribution-backup-queue/01-architecture-spec.md`
- Component and E2E spec (planned path, written by another agent; it was not on disk when this file was written, so section 8 below uses the catalog in the parent plan section 6): `02-spec/21-app/140-per-instance-prompt-attribution-backup-queue/02-component-and-e2e-spec.md`
- RCA: `02-spec/22-app-issues/34-per-instance-prompt-identity-lost-rca.md` (status: open, no code changed yet)
- Earlier session memory: `.ai-memory/memory/learned/23-switch-audit-default-key-and-wrapper-pid-recovery.md`
- Mandatory edit skill: `.ai-memory/memory/learned/24-code-failure-decoding-and-fast-codebase-learning-skill.md` (pointer skill `.agents/skills/agm-code-failure-decoding/skill.md`)
- Earlier RCAs: `.ai-memory/issues/60-switch-drops-live-conversation-rca.md`, `.ai-memory/issues/61-running-prompt-resumption-and-conversation-continuity-rca.md`, `.ai-memory/issues/62-switch-audit-wrong-key-and-macos-wrapper-pid-rca.md`
- Earlier specs: `02-spec/21-app/100-resume-same-conversation-and-pid-cache.md`, `02-spec/21-app/92-cli-instance-switch-e2e.md`, `02-spec/21-app/93-e2e-cli-commands-ai-instruction.md`, `02-spec/21-app/94-blind-ai-instance-test-and-release.md`
- Subtasks: `.ai-memory/plans/subtasks/140-per-instance-prompt-attribution-backup-queue/` (index `index.md`; files 01, 02, 03 exist; 04 and 05 are planned)
- Design and research sources: `.ai-memory/temp-agents/95-per-instance-prompt-attribution-backup-queue/research-findings.md`, `target-design.md`, `acceptance-criteria.md`
- Read order: `.ai-memory/what-to-read.md`. Task register: `.ai-memory/plans/readme.md`.

---

## 1. One-paragraph summary for a reader with no context

AGM runs several Antigravity IDE instances (the `default` instance plus named instances such as `cli-switch-proof-6857`). Each instance has running prompts, switch backups, queued prompts, and restores after an account switch. In v4.124.0, v4.125.0, and commit `366d84e5` (shipped in v4.126.0) we fixed the live conversation being dropped on switch, the empty audit snapshot, the macOS wrapper PID false stop, and a switch killing other instances' PIDs. On 2026-10-06 the user reported that with multiple instances (or even one instance) the running prompt, the backup, and the enqueue are still wrong from the CLI and in the code. Code reading found 33 bug sites (B01..B33) that all share one root cause: no canonical instance key is captured once and carried unchanged, and the resume hand-off file is per repo instead of per instance. No application code was written for that yet. The fix is planned in plan 140 and spec 140, and it starts with a live feasibility gate (E2E-01).

---

## 2. User directives (verbatim, binding)

### 2.1 2026-10-06, task 140 (prompt identity across instances)

> "You have learned how to do the end-to-end testing, and you know I have given you the prompt to test the anti-gravity manager based on the profile, based on the switch, auto-switch, how it's going to do. Now the problem is, when we have multiple instances, or even same instances, the current prompt running, taking the backup or the enqueue prompt, all of these are not right from the CLI methods and how the code is. 1st thing that I want you to do is observe the code where the bug is, 2nd, I want you to create an AI instruction in the plans folder specifying everything that AI needs to do in order to make this thing successful. All the test cases, all the end-to-end test cases that AI would execute in the machine based on the multiple instances. The things would be a little bit complicated because every time AI needs to know the instance name now in order to know which prompts is coming to which. 1st, you need to confirm that is it possible using anti-gravity, and you need to craft in there the CLI methods and also inside the UI how it would reuse the functionalities to make sure that it is working. I don't want you to code, I want you to think, and I want you to write that plan and let me know finally where you have written that plan. Also, so far what you have coded and learned, make sure that these are everything written inside the AI memory so that anytime I throw this conversation, you or any AI would read it and understand what went wrong and how you did these codes, what are the best practices and how to make sure the code does not break, and what are the end-to-end tests that we have discussed. So everything I've won inside the AI memory properly, and inject that in the what to read file. Make sure of that, your memories are safe in there, and make sure you do a git commit and push."

The memory request inside it, quoted on its own because this file exists to satisfy it:

> "so far what you have coded and learned, make sure that these are everything written inside the AI memory so that anytime I throw this conversation, you or any AI would read it and understand what went wrong and how you did these codes, what are the best practices and how to make sure the code does not break, and what are the end-to-end tests that we have discussed."

Actionable items attached to that request (non-negotiable, from spec 140 "User Request (Verbatim)"):

1. Write spec under `02-spec/21-app/<slug>/` and enqueue plan task in `.ai-memory/plans/<slug>.md` (subtasks in `.ai-memory/plans/subtasks/<slug>/`) first.
2. Search codebase exclusively via GitMap (`gitmap aum search`, `gitmap find`, `gitmap cat`, `gitmap ps`, `gitmap py`, `gitmap llm train`); TOTAL BAN on rg, ripgrep, grep, git grep, Select-String.
3. Strictly use relative Git paths; never add the absolute path, also on the release page and in release notes.
4. Observe the code where the bug is.
5. Create an AI instruction in the plans folder specifying everything that AI needs to do for success.
6. Confirm if it is possible using anti-gravity and craft CLI methods and UI functionalities.
7. Write the plan and inform where it is written.
8. Ensure all coded and learned information is written inside the AI memory.
9. Inject the information into the what to read file.
10. Ensure memories are safe and perform a git commit and push.

Also: follow the skill `execute-parent-task-with-n-steps-v6` (`.agents/skills/execute-parent-task-with-n-steps-v6`), plan first to reduce credits, learn from the `gitmap` skill.

### 2.2 2026-10-02 session (from learned 23, still binding)

1. "make this UI better in the instance section and make a minor bump and release please"
2. "respect the other PIDs when changing the account don't close other instances PID do e2e test before submitting the release please, then reelase with a minor bump please"
3. v4.124.0 request (summarized in learned 23 from the transcript; the verbatim form is in spec 100): fix the running prompt not being backed up, not appearing in the audit, and not resuming in the same conversation, for both Switch and fast-forward. Make the audit detail an "i" button that opens a scrollable modal with Copy and Cancel, like the email JSON view. Quota checks no less than 2 minutes; the PID search runs only when credit is under the threshold. Trust the PID saved at launch when its path matches. PID cache refresh every 10 minutes by default, minimum 3 minutes. Follow `.cursor/skills/execute-parent-task-with-n-steps-v6`, then a minor bump and release.
4. Spec 100 verbatim: "When you switch the account, with fast forward or switch account, the running prompts are not shown as backed up in the audit logs, and they are not resumed into the same conversation. The audit detail control should be an info button that opens a scrollable table, with copy and cancel, like the email JSON view. Google quota checks are never faster than 2 minutes. A process scan happens only when credit is under the threshold, or on the PID cache refresh (default 10 minutes, never under 3 minutes, never over 20). A PID saved at launch is trusted when that process path still matches."
5. Standing rule: "After finishing any change, always stage only the files you changed (explicit paths, never `git add -A`), write a concise commit message, commit, and push to the current branch. Never ask for permission to commit or push. If a push is rejected because the remote is ahead, rebase onto the upstream and push again. Never force-push, never use --no-verify, and never commit unrelated files, generated caches, or secrets."

### 2.3 Learned 24 request

> "Can this also include your learning so far in decoding how a code can fail, how AI can learn from it, how AI should not make mistakes in the syntax writing, and code-based understanding, where you want to emphasize upon AI to learn and write it as a, like, instruction skill so that AI must follow this and learn the code base very quickly."

---

## 3. Timeline of what was coded

| When | Commit / release | Files | Functions and change | Why |
|---|---|---|---|---|
| 2026-10-02 | v4.124.0 | `src-tauri/src/modules/repo_db.rs` | `live_prompt_text` keeps the summary preview when `transcript.jsonl` has no `USER_INPUT` line; `read_conversation_summary_rows` falls back to `conversation_id, preview, workspace_uris` when `status` columns are missing; `resume_task_document` always writes `session_id` and `conversation_id` into `.antigravity_resume_task.json`; `switch_prompt_snapshot` orders by `updated_at DESC` for the instance id (was `created_at DESC LIMIT 50`); an already-alive `agy` worker no longer counts as a successful re-push in `spawn_prompt_via_agy`; named instances read only their own home `.gemini` and keep a discovered conversation even if its path is not yet in `running_projects` | Issues 60 and 61: the live conversation was dropped, the audit was empty, and the IDE reopened a blank chat |
| 2026-10-02 | v4.124.0 | `src/pages/Audit.tsx` | Audit "i" info button opens a scrollable modal with Copy and Cancel, loaded only on click; list query omits `payload_json` | User UI request |
| 2026-10-02 | v4.124.0 | `src-tauri/src/modules/auto_switcher.rs` | `MIN_GOOGLE_QUOTA_CHECK_SECONDS` = 120 s floor; old default 300 treated as 120; custom values above 120 kept (`calculate_next_interval_seconds`) | Critical quota scheduled the next Google check in 40 s |
| 2026-10-02 | v4.124.0 | `src-tauri/src/modules/instance.rs`, `src-tauri/src/models/config.rs` | Saved PID trusted when `process_identity_matches` (executable path) holds; `refresh_pid_cache_if_due` rebuilds the process table every `pid_refresh_seconds` (default 600, clamp 180..=1200); new config field `pid_refresh_seconds` | Every quota tick scanned the whole process table (CPU) |
| 2026-10-02 | `a3fa0ee1` | `readme.md`, `readme_en.md` | Release notes synced to 4.124.0 | Git tracks these files in lowercase; staging `README.md` staged nothing |
| 2026-10-02 | v4.125.0 | `src-tauri/src/modules/instance.rs`, `src-tauri/src/modules/process.rs`, `src-tauri/src/modules/integration.rs`, `src/pages/Instances.tsx` | `should_spare_pid` and `other_instance_protection(except_id)`; `close_instance` and `process::get_antigravity_pids` skip other instances' PIDs; single-instance switch no longer calls `close_antigravity(None)`; `integration.rs` default switch only calls `close_instance("default")`; Instances page cards one column until `xl`, wrapping paths and action rows | User: "respect the other PIDs when changing the account don't close other instances PID" |
| 2026-10-02 | `366d84e5` (shipped in v4.126.0, `9b976bc9`) | `src-tauri/src/modules/account.rs` | Default-switch audit snapshot reads `crate::modules::repo_db::switch_prompt_snapshot("default")` instead of `switch_prompt_snapshot(target_ide)` (now at `account.rs:1743`) | Issue 62 symptom 1: prompts are keyed by instance id, not IDE flavor |
| 2026-10-02 | `366d84e5` | `src-tauri/src/modules/instance.rs` | `is_instance_running(instance_id, data_dir, config_pid)` (now at `instance.rs:2584`) keeps the saved-PID identity check first; when it fails it runs `find_pids_for_data_dir(data_dir, is_default)` once, saves the first match with `record_instance_pid`, and returns true; no match returns false | Issue 62 symptom 2: macOS saves the short-lived `open` wrapper PID |

Verification of `366d84e5`: `cargo fmt -- --check` clean; `cargo clippy --all-targets --all-features` zero errors and no warnings on touched lines; targeted tests passed: `saved_pid_identity_and_refresh_floor`, `switch_spares_another_instances_pid`, `switch_payload_keeps_from_to_reason_and_reinject`. Unit tests from v4.124.0: `live_prompt_uses_preview_when_the_transcript_has_no_user_input`, `resume_document_keeps_the_same_conversation_id`, `summary_rows_fall_back_when_status_columns_are_missing`, `saved_pid_identity_and_refresh_floor`. The live account switch e2e was NOT run, to protect the user's running IDE. A live prompt reinject was never watched.

### 3.1 What later commits changed (state on 2026-10-06)

- HEAD at the learned-23 audit was `ccf336b1` (v4.158.0 line); the plan 140 code reading used HEAD `93b9c825`. 119 commits landed after `366d84e5`. Both fixes are still present: `account.rs:1743` reads `switch_prompt_snapshot("default")`; `is_instance_running` at `instance.rs:2584` still recovers and records the real PID.
- `a0c5cc18` and `3581207d` (v4.155.0): the saved-PID branch of `is_instance_running` now also calls `find_pids_for_data_dir` and requires the saved PID to be in that list. This removed the "trust the saved PID without a scan" fast path the user asked for. Logged as open ambiguity `.ai-memory/ambiguous-questions/01-new-ambiguity/02-is-instance-running-saved-pid-fast-path.md`. Do not revert it without the user's answer.
- 12 of the last 30 commits (v4.151.0 to v4.158.0) only fixed rustfmt or compile errors after a feature commit (`ccf336b1`, `6228e437`, `5bb32c51`, `01b48a06`, `4c28ecf8`, `d56a384e`, `5de41444`, `a19644a7`, `76493525`, `65800177`, `e0ce4f7c`, `8ce647c7`).

### 3.2 What task 140 produced (2026-10-06, documentation only)

No application code. Files written: parent plan 140, spec 140 `01-architecture-spec.md`, RCA 34, subtasks 01..03 plus `index.md`, `02-spec/21-app/readme.md`, `02-spec/22-app-issues/readme.md`, `.ai-memory/plans/readme.md`, and this memory file. Bug sites were confirmed by reading code and re-checking line numbers with `gitmap aum search`.

---

## 4. What went wrong (root causes, one sentence each)

| Problem | Root cause | Fixed in |
|---|---|---|
| Dropped live conversation on switch and fast-forward | `discover_running_prompts_from_antigravity` required a `USER_INPUT` line and skipped a whole summaries DB when `status` columns were missing, so `active_prompts` stayed empty, `count_backed_up_prompts` was 0, re-inject was skipped, and dispatch rewrote `.antigravity_resume_task.json` without `session_id` so the IDE opened a blank chat. | v4.124.0 |
| Empty audit snapshot | `switch_account` in `account.rs` passed `target_ide` (an IDE flavor such as `"ide"` or `"agy"`) to `switch_prompt_snapshot`, but prompts are keyed by instance id, so the default instance must be read with `"default"`. | `366d84e5` |
| Wrapper PID false stop (macOS) | The launcher records the PID of the short-lived `open` wrapper, which exits after spawning the IDE, so a saved-PID-only check returned false for a live instance. | `366d84e5` |
| Other-instance PID kills | A single-instance switch called `close_antigravity(None)`, which matched every Antigravity process, so switching one instance killed the others. | v4.125.0 (`should_spare_pid`, `other_instance_protection`) |
| Per-instance prompt identity loss (open) | There is no canonical instance key that is set once when a prompt is captured and carried unchanged to dispatch: every layer re-derives it (`"default"`, `"__default__"`, `""`, the active instance, fuzzy `resolve_instance_id`, substring or `LIKE` matches, path-only joins), and `.antigravity_resume_task.json` is per repo instead of per (instance, repo, conversation), so identity is lost whenever two instances share a repo or a key fails to normalize. | Planned, plan 140 |

### 4.1 The 33 bug sites grouped by flow (full table in spec 140 section 4)

- Key (B01, B02, B03, B23): `resolve_instance_id` maps empty or `"active"` to the active instance and does fuzzy suffix matches; `get_instance_home_dir` creates `instances/__default__/home` for an unknown id; desktop switch and Telegram hardcode `"default"` and run three restore paths. `account.rs:1743` is a real default-only path and is correct; never change it to `target_ide`.
- Running detection (B04, B05, B11, B12, B14, B26, B32): `"all"` treated as default and stamped as an id; first summaries row always "running" (`|| prompts.is_empty()`); global `DELETE ... instr(id,'__')=0` wipes other writers' rows; Gate 4 folder-prefix match (`app` matches `app-v2`); substring instance filter (`default` matches `x__default-copy-8159`); `agm wpr` path-only join; `scan_conversations` collapses the same conversation id across instances.
- Backup (B06, B08, B09, B10, B18, B19, B20): Layer 1 upsert never updates `instance_id`; Layer 2 scans every project `LIKE %chat%/%task%/%prompt%`; no instance filter and `LIKE %repo_name%`; project id stored as `session_id`; empty `instance_id` rows join every backup; dedupe re-stamps `instance_id = ?3` (B steals A's backup); conversation matched by "URIs contain repo".
- Queue (B13, B24, B25, B28): scheduler consumes `backed_up` as queue items and marks `dispatched` when `sent = false`; `agm prompt -n` is `--node`; `agm prompt` without `-i` uses the active instance and marks dispatched before the spawn result; UI calls IPC `enqueue_prompt`, which does not exist.
- Restore (B07, B15, B16, B17, B21, B22, B27, B29, B30, B31, B33): per-repo resume file written by eight writers and read by six readers with no instance check; `spawn_prompt_via_agy` keys workers by raw id and `-p` starts a new chat; resend re-sends `running` rows and drops `session_id`; auto resume `LIKE %repo_name%`; auto switcher `"default"` target becomes `None`; cross-instance backup under source but dispatch under target; `rrc` backs up the active instance and re-sends all; relaunch runs restore, resend, and dispatch; `restore_running_prompts_for_instance` already resends at `backup_prompts_db.rs:614`; `rrp` resends before restoring.

### 4.2 Top 9 sites (lead-verified with `gitmap aum search`, HEAD `93b9c825`)

| Flow | Site | Problem |
|---|---|---|
| Key | `src-tauri/src/modules/instance.rs:4376` (`resolve_instance_id`, fn at `:4372`) | Empty or `"active"` id resolves to whichever instance is active |
| Backup | `src-tauri/src/modules/backup_prompts_db.rs:249`, `:252` | Rows with empty `instance_id` match every instance |
| Backup | `src-tauri/src/modules/backup_prompts_db.rs:353`, `:371` | `UPDATE prompt_backups SET ... instance_id = ?3` re-stamps another instance's backup |
| Queue | `src/components/instances/PromptTreeViewModal.tsx:1335` | UI calls IPC `enqueue_prompt`, which does not exist in Rust (0 hits in `src-tauri/src`) |
| Queue | `src-tauri/src/modules/repo_db.rs:2054` `check_and_dispatch_enqueued_prompts` | Consumes switch backups as queue items, marks dispatched even when not sent |
| Restore | `src-tauri/src/modules/repo_db.rs:1224`, `1283`, `1382`, `1603`, `2175`, `3376`, `3534`, `3595` | Resume file joined to `repo_path`: two instances on one repo overwrite each other |
| Restore | `src-tauri/src/modules/auto_switcher.rs:1691` | A `"default"` target becomes `None` (re-send every instance) |
| CLI | `src-tauri/src/bin/agm.rs:2926` (help), `:3008` (parse) | `-n` means `--node` (remote SSH), not instance; only `-i/--instance` targets an instance |
| CLI | `src-tauri/src/bin/agm.rs` `cmd_prompt_dispatch`, `cmd_which_prompts_running` (`:1640`), `cmd_resend_running_commands` (`:3372` to `:3376`) | Default to the active instance, join by path only, back up one instance and re-send all |

### 4.3 Why ten earlier fixes did not close it

Specs 100, 113, 118, 120, 121, 122, 123, 126, 128, 132 each fixed running detection keys or timeouts in one flow and treated the instance id as already correct. None made the instance id a single captured key carried across backup, queue, and dispatch, and none touched the per-repo resume file (RCA 34 section 4).

---

## 5. Feasibility: how Antigravity separates instances

- AGM launches each named instance with `USERPROFILE`, `APPDATA`, `LOCALAPPDATA` (and `HOME` on Unix) set to `<instances>/<id>/home` at `src-tauri/src/modules/instance.rs:3594-3618` (the env block starts at `:3586`; `--user-data-dir` is `<instances>/<id>/data`, `:3586` to `:3588`).
- Antigravity then writes that instance's conversations under `<instances>/<id>/home/.gemini/{antigravity,antigravity-ide,antigravity-cli}`, which `gemini_dirs_for_instance` reads (`src-tauri/src/modules/repo_db.rs:824` to `:865`). Discovery tags each result with the scanned id (`repo_db.rs:1084`).
- The default instance owns the global `%USERPROFILE%\.gemini` store (`repo_db.rs:841` to `:862`), plus `<instances>/default/home/.gemini` if present. There is no positive marker; default owns what is left.
- Antigravity writes no instance tag of its own. `app_data_dir` in `conversation_summaries.db` holds only product names (`antigravity`, `antigravity-cli`). The instance link is (a) the folder the store lives in and (b) the `instance_id` AGM stamps at capture.
- Per instance: `conversation_summaries.db`, `brain/<cid>/.system_generated/logs/transcript.jsonl`, `conversations/`, IDE data dir, `workspaceStorage` (`<data_dir>/User/workspaceStorage`).
- Shared (keyed by an `instance_id` column or by id): `instances.json` and `instances.db` (`instance_processes`, `active_instance_selection`) under `%USERPROFILE%\.antigravity_tools\instances\`; `repo_prompts.db` (`running_projects`, `active_prompts`, `agm_project_sequences`, `agm_conversation_sequences`, tree cache) under `%USERPROFILE%\.antigravity_tools\`; `backup-prompts.db` (`backup_batches`, `prompt_backups`) under `%USERPROFILE%\.antigravity_tools\backup-prompts\`. Shared per repo and colliding: `<repo_path>/.antigravity_resume_task.json`.
- Clones copy summaries DBs (`instance.rs:1470` to `:1545`), so the same conversation id can exist in two instances. `agm_conversation_sequences` PK is `conversation_id` alone and row ids are `prompt-{cid}` (`repo_db.rs:1080`), so a clone collides today.
- Not proven: that Antigravity honours the overridden profile for every file. On 2026-10-06 there were 5 registered instances (active `default`); `cli-switch-proof-6857` (pid 10952) and `test-cli-flow-1743` (pid 13104) had 20 KB per-instance `conversation_summaries.db` files with 0 rows; `clone-from-default-4424` and `new-empty-instance-4425` had none. The global `antigravity` summaries DB had 183 rows and 185 brain dirs; `antigravity-cli` had 187 rows; none with `not_fully_idle` set. E2E-01 must prove the per-instance store before any other result is trusted.
- macOS caveat: the macOS launch goes through `open` (`src-tauri/src/modules/instance.rs:3440`; env set around `:3391` to `:3402`, `:3470`). `open -a` may start the app with the login environment and drop the per-instance `HOME`. Treat macOS as unverified. The `open` wrapper is also why a saved PID can be dead while the IDE runs.

Verdict: possible for named instances; partially possible for backed-up and queued prompts until spec 140 is implemented; not yet proven live.

---

## 6. Best practices and anti-patterns

### 6.1 Do

- Instance identity: use one storage key from `canonical_instance_id(raw)` (planned, next to `resolve_instance_id` at `instance.rs:4372`): exact registry match; `""`, `"default"`, `"__default__"` map to the default id; unknown is `Err` with the known list. Resolve fuzzy user text (`#2`, `8159`, name, `active`) only in `resolve_instance_input` at the CLI or UI input layer, failing on ambiguity with candidates.
- Stamp the instance id once, at capture, from the `(instance_id, dir)` pair that produced the row. List functions take `InstanceScope::One(id)` or `InstanceScope::All`, never the magic string `"all"`.
- Keep the IDE flavor (`target_ide`: `"ide"`, `"agy"`, `"instance:<id>"`) separate from the instance id. Per-instance data (prompts, PIDs, leases, backups) is keyed by instance id.
- Prompt identity is the triple `(instance_id, conversation_id, repo_path)`; row id `prompt-{instance_id}-{conversation_id}`, `queued-{instance_id}-{uuid}`, `cli-{instance_id}-{uuid}`; `session_id` holds the conversation id only.
- Compare paths with one normalizer, `normalize_path_for_compare` (`repo_db.rs:2473`): lowercase on Windows, forward slashes, no trailing slash.
- Running means all four: own gemini dirs, `not_fully_idle > 0 AND status RUNNING`, last turn within the 120 s TTL (spec 132), and the instance PID alive.
- PIDs: a saved PID is a hint. Check identity first, then one bounded `find_pids_for_data_dir` (`instance.rs:621`) and rewrite with `record_instance_pid` (`instance.rs:83`). Refresh the PID cache every `pid_refresh_seconds` (default 600, clamp 180..=1200). Google quota checks never faster than 120 s.
- Process safety: close one instance only through `close_instance(id)` (`instance.rs:3922`) with `should_spare_pid`; protect Cursor and `agm-alim`; protect any process at `%APPDATA%\Antigravity` or without `--user-data-dir` (spec 93).
- Queue: a status state machine `running`, `backed_up`, `queued`, `dispatching`, `dispatched`, `failed` (with `status_reason`), `orphaned`. The scheduler consumes only `queued`. Claim atomically (`UPDATE ... SET status='dispatching', attempts=attempts+1 WHERE id=? AND status=?`, proceed only when `changes() == 1`). A `dispatching` row older than 120 s becomes `failed` with `dispatch_timeout`.
- Backup: dedupe by `(instance_id, prompt_id)`, then `(instance_id, normalized project_path, trimmed prompt_text)`; keep the stored owner on reactivation.
- Restore: one function, `restore_backed_up_prompts_for_instance(instance_id, keep_backup, custom_file)` in `backup_prompts_db.rs`; bounded wait with `wait_for_instance_prompt_channel` (`instance.rs:2987`); rows stay `backed_up` with `skipped_reason = "instance_not_running"` if the PID is not alive. Saved `queued` stays `queued` (`prompt_status_after_restore`, `backup_prompts_db.rs:622`).
- Resume hand-off: always build it with `resume_task_document` (`repo_db.rs:780`) carrying `instance_id`, `session_id`, `conversation_id`. Find every reader before renaming the file. Keep the legacy file only when exactly one instance owns the repo, and log the skip otherwise.
- Migration: copy the DB to `<db>.pre-v140.bak` first; one transaction; assign a legacy row only on exactly one owner, else `orphaned`; guard with `agm_schema_migrations` row `v140_instance_ids`.
- CLI: every prompt command takes `-i/--instance` and `--json`; every row prints `instance_id` and `instance_name`; action commands refuse without `-i` when more than one instance runs (exit code 2).
- IPC and UI: every IPC command in `src-tauri/src/commands/instance.rs` is a thin wrapper over the same Rust function the CLI calls (spec 94 law L4); register new commands in `src-tauri/src/lib.rs` next to `commands::list_backed_up_prompts` (`:1104`); every UI prompt row shows the instance badge `#seq name`.
- Order side effects to fail before the point of no return: validate the instance id before any backup or close; write credentials before killing a process; validate before deleting. Bound every wait with a timeout. Detect broadly, act narrowly.

### 6.2 Never

- Never call `resolve_instance_id` or `get_active_instance_id` inside storage code (`repo_db.rs`, `backup_prompts_db.rs`, `agy_cleaner.rs`, switch paths).
- Never `UPDATE instance_id` on an existing prompt or backup row.
- Never join prompts to projects or instances with `LIKE %repo_name%`, substring `contains`, folder-name prefix, or path only.
- Never stamp `"all"`, `""`, or `"__default__"` as an instance id. Never let `get_instance_home_dir` create a folder for an unknown id.
- Never treat the first summaries row as running. Never back up idle conversations as running.
- Never re-send a row whose status is `running`. Never mark `dispatched` when the send failed. Never let the scheduler consume `backed_up`.
- Never run more than one restore path for one switch (today three run back to back: B23, B29, B30, B31).
- Never turn a `"default"` target into `None` (all instances). Never dispatch the source instance's prompts under the target id.
- Never change `account.rs:1743` from `"default"` to `target_ide`.
- Never call `close_antigravity(None)` on a single-instance path. Never `taskkill` by image name. Never close another instance's PID.
- Never trust a launcher wrapper PID alone. Never remove the bounded fallback search.
- Never rewrite `.antigravity_resume_task.json` without `session_id` and `conversation_id`. Never write it from the UI.
- Never shell out to `agm` from the UI. Never default to `'default'` in `PromptTreeViewModal.tsx` when the instance is unknown (`:1241`, `:1260`, `:1338`, `:1350`).
- Never read `-n` as instance on `agm prompt`; it is `--node`.
- Never invent an `agy` session flag; `agy -p` starts a new prompt and does not resume a conversation.
- Never revert the v4.155.0 `is_instance_running` scan without the user's answer to ambiguity 02.

---

## 7. How to make sure the code does not break

### 7.1 Local gate before every push (learned 24 Part 5)

Rust edits, from `src-tauri`, with `CARGO_TARGET_DIR` set to the repo's `src-tauri/target` and `CARGO_BUILD_JOBS=1` on this host:

1. `cargo fmt` then `cargo fmt -- --check`.
2. `cargo clippy --all-targets --all-features`, zero errors, no new warnings on your lines.
3. Targeted tests: `cargo test --lib -- --test-threads=1 <test_name>`.

Frontend edits: `npm run build`. CI compiles test targets without running them; rely on CI for `tauri build` and the full suite. Four releases (v4.145.0 to v4.148.0) were lost to one compile error each because clippy was not run locally.

### 7.2 Existing tests to run before and after editing these areas

`saved_pid_identity_and_refresh_floor`, `switch_spares_another_instances_pid`, `switch_payload_keeps_from_to_reason_and_reinject`, `live_prompt_uses_preview_when_the_transcript_has_no_user_input`, `resume_document_keeps_the_same_conversation_id`, `summary_rows_fall_back_when_status_columns_are_missing`, `test_find_pids_target_path_normalization` (check it still has its `#[test]`), `test_instance_binding_stale_or_exhausted`. Integration tests: `src-tauri/tests/per_instance_prompt_liveness_test.rs`, `auto_switcher_e2e_test.rs`, `instance_cloning_and_sync_test.rs`. Local-only E2E tests use `#[ignore]` plus `RUN_TEMP_E2E=1` (spec 72).

### 7.3 Targeted tests to add with plan 140 (one per prevention rule in RCA 34 section 7)

- `canonical_instance_id`: `""`, `"default"`, `"__default__"` give the same id; unknown returns `Err` and creates no folder; `"all"` and `"active"` are errors (AC-02, AC-03).
- `resolve_instance_input`: a suffix shared by two instances returns an ambiguity error with candidates (AC-04).
- Backup isolation: backing up B leaves every row owned by A unchanged; no `UPDATE instance_id` (AC-09, AC-10).
- Exact filter: `default` never matches `default-copy-*`; `app` never matches `app-v2` (AC-22).
- First summaries row idle is not running (AC-19).
- Scheduler consumes only `queued`; `backed_up` untouched (AC-14).
- Failed spawn writes `failed` plus `status_reason` (AC-15).
- Single restore: every restored row `attempts <= 1`; no row of B changed (AC-17, AC-18).
- Same conversation id in two instances gives two rows and two sequence rows (AC-07).
- Migration on a DB copy: single owner assigned, others `orphaned` and never dispatched (AC-05, AC-08).
- Hand-off document keeps `instance_id`, `session_id`, `conversation_id`; readers ignore another instance's document (AC-12, AC-13).
- Auto switcher: `"default"` target restores only default rows; cross-instance never dispatches source rows under target (AC-23, AC-24).

For logic errors (class J in learned 24) a test that fails without the fix is mandatory.

### 7.4 Evidence rules

- Every claim needs a command, its exit code, a JSON excerpt, and an SQL row that includes `instance_id`.
- Read databases only read-only (`file:<path>?mode=ro`), always with `instance_id` in the `WHERE` clause.
- State what was not verified. Never claim a live reinject was watched unless it was. Never print emails or tokens in full.

### 7.5 Release rules

- Tag only after CI for that exact SHA is green: `gh run view <id> -R alimtvnetwork/Antigravity-Manager --json conclusion,headSha` shows `success`. A cancelled macOS job is rerun with `gh run rerun <id> --failed`, never tagged over. `gh` defaults to upstream `lbjlaq/Antigravity-Manager` in this clone; pass `-R alimtvnetwork/Antigravity-Manager`.
- Release only when the user asks. Changelog attribution is only `(Thanks to @aukgit)`. Relative paths only in commits, changelogs, and release notes.
- Staging by exact case: stage explicit paths with the exact case git tracks (check with `git ls-files`). `readme.md` and `readme_en.md` are lowercase in git; `git add README.md` staged nothing (`a3fa0ee1`). Never `git add -A`, never `--no-verify`, never force-push. PowerShell here-docs fail in this shell; use `git commit -m "title" -m "body"`.

### 7.6 Edit rules from learned 24 (summary; read the full file)

- Orient first: `git log -n 30 --oneline`, read `.ai-memory/what-to-read.md`, `.ai-memory/cicd-index.md` "Recurring Failure Classes", `.ai-memory/strictly-avoid.md`. Find code by symbol, read callers and types, run the nearby test first.
- Large files (`instance.rs`, `repo_db.rs`, `auto_switcher.rs`, `agm.rs` about 13.5k lines): read with offset and limit, at least 30 lines above and below the edit.
- Read the signature you call: `get_antigravity_executable_path` returns `PathBuf`; `AuditTask` implements `Drop` so move fields with `.clone()`; check `async`.
- Anchor replacements on a unique line such as the `fn` signature; never replace a range that starts or ends mid-block. Inside `mod tests` keep exactly one `#[test]` per test fn (`01b48a06` lost a header; a duplicate `#[test]` once silenced `test_find_pids_target_path_normalization`).
- Deleting or moving a `let`: search every use (`4c28ecf8` deleted `has_target` while still used). Adding a struct field: update every initializer, including tests (`SwitchFacts.steps`).
- rusqlite: drop `Statement` before reusing the connection (`5bb32c51`); `StepResult` is `#[non_exhaustive]`, add `_ =>`; backup API needs the `backup` feature.
- Inside Tauri commands use `tauri::async_runtime`, never `tokio::runtime::Runtime::new()` (`2bbcba13` white screen).
- Lines under 100 columns; break long conditions and chains the way rustfmt does. JSX: count `<td>`/`</td>` and `<div>`/`</div>` after editing (`8ce647c7`).
- Learn-from-failure loop: copy exact error, classify (A..K), one-sentence root cause, fix the class across siblings, add a failing test, run the full gate, record an RCA and a ban.

---

## 8. Every E2E test discussed

### 8.1 Earlier E2E discussions

- Spec 92 (2026-10-01), CLI switch on a copy without closing the open IDE: record the open IDE PID and folder; create a full IDE copy; create one running and several queued prompts on it; back them up; close only the PID whose command line matches that instance folder; switch the account (switch and fast-forward); reopen; restore the running prompt and requeue the queued ones; prove with a CLI query; confirm the account in Settings by PID-to-folder match; switch back. Code fixes in that pass: `prompt_status_after_restore` keeps `queued`; `resend_running_commands_for_instance` skips `queued`; `agm restore` prints the saved status; `close_instance` drops Cursor and AGM PIDs. Status: not run on a built CLI (installed `agm-alim.exe` was 4.109). `scripts/test-instance-e2e.ps1` creates two instances and switches one but does not verify restore status.
- Spec 93, zero-code CLI manual: Step 0 protect operator PIDs (any Antigravity process at `%APPDATA%\Antigravity` or without `--user-data-dir`), Step 1 hygiene, Step 2 create two instances, Step 3 launch and verify PID with `agm instances observe <id> --json`, Step 4 seed a prompt with `scripts/test_prompt_helper.py seed` and start `scripts/prompt_heartbeat_runner.py`, Step 5 `agm backup-running-prompts --instance <id>`, stop, `agm switch <id> <email> --json`, Step 6 relaunch and `test_prompt_helper.py check`, Step 7 `agm ff <id> --json`, Step 8 `agm agy cache-clear --keep 10 -y`, Step 9 teardown. Its Step 1 deletes every `test-*` instance; that is broader than law L8 and must not be used as written while protected sandboxes exist.
- Spec 94, blind-AI laws (stop on any break): L1 host process PIDs (the IDE you started in and `agm-alim.exe`) unchanged at the end; L2 the sandbox PID command line contains that instance `data` dir and not the default dir; L3 `--from default` copies `User/globalStorage/state.vscdb`, `--new` does not; L4 UI `invoke('create_instance')` and `invoke('switch_account_to_instance')` land in `instance.rs`, no `Command::new("agm")`; L5 after switch `queued` stays `queued` and the running prompt is `dispatched` or `running`, not deleted; L6 `[Notify] email:` and `[Notify] telegram:` lines before exit, `FAIL` with the error; L7 `agm supabase confirm <id>` reads back the bound email, `PGRST205` means missing table; L8 delete only ids starting with `test-cli-flow` or `test-diag`. Tests A (clone from default, `agm test-instance-flow --from default`), B (new empty, `agm test-instance-flow --new`; `--new` with `--from` must error), C (switch, prompts, notify), D (Supabase), E (UI uses the same functions). Release only after A..E pass.
- v4.125.0 directive "do e2e test before submitting the release": the proof was the unit test `switch_spares_another_instances_pid` (other instances' PIDs are spared). The live account switch e2e for `366d84e5` was NOT run, to protect the user's running IDE. A live prompt reinject into the same conversation has never been watched; it is pending in plan 139 on a sandbox instance only.

### 8.2 Test-instance naming, prompt marker, evidence

- Test instances: `test-cli-flow-a-<rand4>`, `test-cli-flow-b-<rand4>`, and the clone `test-cli-flow-c-<rand4>` (law L8). Delete only instances this run created.
- Prompt marker in every test prompt: `E2E140-<runid>-<instance>-<case>`, so the prompt is found in any store by text.
- Evidence read-only (`file:<path>?mode=ro`) from `repo_prompts.db` (`%USERPROFILE%\.antigravity_tools\repo_prompts.db`), `backup-prompts.db` (`%USERPROFILE%\.antigravity_tools\backup-prompts\backup-prompts.db`), and each instance's `conversation_summaries.db`, always with `instance_id` in the `WHERE` clause. Every prompt CLI call passes `-i <instance>`.
- After step 5 of plan 140 ships, `agm prompts trace -i <instance>` (NEW) prints the identity chain.

### 8.3 Safety rules (non-negotiable, every run)

- Never kill Cursor or `agm-alim`.
- Never switch or close the default instance.
- Never delete the protected sandboxes `cli-switch-proof-6857`, `test-cli-flow-1743`, `clone-from-default-4424`, `new-empty-instance-4425`.
- Do not click the Google data-collection checkbox.
- Do not launch a replacement AGM GUI.
- Never claim a reinject was watched unless it was. Never print emails or tokens in full.
- Record host PIDs first (`Get-Process Cursor, agm-alim, Antigravity`); abort if a command would stop one.

### 8.4 New catalog E2E-00..E2E-19 (plan 140 section 6)

"Today" means it can run against current code as a baseline (expected to expose the bug where one exists). "After fix" means it needs code from plan 140 steps 3 to 5. The AC mapping below is derived from spec 140 section 8; the component and E2E spec owns the final step-by-step text.

| ID | Purpose | AC IDs | When |
|---|---|---|---|
| E2E-00 | Safety snapshot: host PIDs, `agm instances --json` PIDs of default and protected sandboxes, DB row counts | AC-32 | Today, first |
| E2E-01 | Feasibility gate: a marker prompt typed in `<A>` lands in `<instances>/<A>/home/.gemini/<flavor>/conversation_summaries.db` and not in the global store. If it fails, STOP and record an ambiguity | AC-01 | Today, second |
| E2E-02 | Canonical key resolution: `default`, `__default__`, empty agree; unknown id refused with candidates and no folder; ambiguous suffix refused | AC-02, AC-03, AC-04 | After fix |
| E2E-03 | Running detection, single instance: only the live, non-idle, within-120 s conversation is running; idle first row is not | AC-19, AC-20 | Today as baseline, pass after fix |
| E2E-04 | Two instances, different repos: each prompt attributed only to its instance | AC-21, AC-22, AC-27 | Today as baseline, pass after fix |
| E2E-05 | Two instances, same repo: hand-off collision; neither overwrites the other; legacy file skipped and reported | AC-11, AC-12, AC-13 | After fix |
| E2E-06 | Same conversation id in two instances (clone `test-cli-flow-c-<rand4>`): two rows and two sequence rows | AC-06, AC-07 | After fix |
| E2E-07 | Backup isolation and idempotence: `agm brp -i <B>` leaves `<A>` rows unchanged; repeat backup adds no duplicates | AC-09, AC-10 | Today as baseline, pass after fix |
| E2E-08 | Enqueue via CLI: NEW `agm prompt enqueue` writes exactly one `queued` row | AC-28 | After fix |
| E2E-09 | Enqueue via UI: same row shape with `source_dir = "ui"`, no resume file written by the UI | AC-29 | After fix |
| E2E-10 | Queue vs backup separation: `agm qs -i <A> --once` consumes only `queued`, `backed_up` untouched | AC-14 | After fix |
| E2E-11 | Instance account switch keeps the prompt in the same conversation; other instance untouched (sandbox only) | AC-17, AC-18, AC-32 | After fix |
| E2E-12 | Auto-switch cross-instance: source backed up once, restored only into source, nothing dispatched under target | AC-23, AC-24 | After fix |
| E2E-13 | Default instance paths by unit test only (never switch default live) | AC-02, AC-23, AC-25 | After fix, unit tests |
| E2E-14 | Legacy empty `instance_id` migration on a database copy: single owner assigned, others `orphaned` and never dispatched | AC-05, AC-08 | After fix |
| E2E-15 | Restart mid-flow without duplicate resend | AC-16, AC-17 | After fix |
| E2E-16 | Failed spawn is not marked dispatched (`failed`, `repo_missing`) | AC-15 | After fix |
| E2E-17 | Refusal without `-i` when two instances run: exit code 2, nothing changed | AC-26 | After fix |
| E2E-18 | Trace identity chain: `agm prompts trace -i <A> --conversation <cid> --json` keys present; tree scoped by instance | AC-30, AC-31 | After fix |
| E2E-19 | Cleanup and protected PID check: delete only this run's `test-cli-flow-*` instances; default and protected sandbox PIDs unchanged | AC-32 | Today, last |

Report one line per case: PASS with evidence, or BLOCKED with the reason.

### 8.5 Existing E2E infrastructure

`scripts/test-instance-e2e.ps1` (also reads the resume file at `:439`), `scripts/test-instance-isolation.ps1`, `scripts/e2e_test_instance.ps1`, `scripts/e2e-instance-commands-test.ps1`, `scripts/test_prompt_helper.py`, `scripts/prompt_heartbeat_runner.py`, `.ai-memory/temp-agents/e2e-counts.py`, `e2e-inspect.py`, `e2e-restore-proof.py`. Gaps: no test for the same conversation id in two instances, no instance-filtered list CLI, no backup or restore IPC, no resume file collision test.

---

## 9. CLI and UI reuse summary

### 9.1 Existing verified commands (`src-tauri/src/bin/agm.rs` dispatch table `:51` to `:145`)

`agm instances`/`ls` (`:53`), `agm status` (`:52`), `agm history`/`audit` (`:111`), `agm wpr` (`:122`, no instance arg, scans all), `agm prompts` (`:123`), `agm prompt -i` (`:124`, `-i/--instance` parsed at `:2995` to `:3003`; `-n` is `--node`), `agm rerun` (`:125`), `agm rrc`/`resend` (`:128`), `agm brp -i` (`:131`), `agm qs [instance] [--once]` (`:134`), `agm rrp -i` (`:137`), `agm running-prompts [ls|backup|restore|export|import]` (`:140`), `agm running-projects` (`:143`), `agm tree [all]`, `agm observe` (`:99`), `agm tif` / `test-instance-flow` (`:102`, `--from`, `--new`). `--json` already exists on `wpr`, `prompts`, `rrc`, `qs`, `brp`, `rrp`, `running-prompts`, `running-projects`, `tree`.

### 9.2 Target and NEW commands (proposed)

- NEW `-i` on `wpr`, `prompts`, `running-prompts`, `running-projects`, `tree`, `rrc`; `qs` also accepts `-i`. Every row includes `instance_id` and `instance_name`. List commands without `-i` show all instances grouped. Action commands (`brp`, `rrp`, `rrc`, `prompt`, `prompt enqueue`) refuse without `-i` when more than one instance runs: exit 2, message `instance required: <n> instances running (<#seq name>, ...); pass -i <id|#seq|name>`; with one running instance they print `[instance] using <#seq name> (only running instance)`.
- NEW `agm prompt enqueue -i <instance> --repo <path> [--conversation <cid>] "<text>" [--json]` through `enqueue_prompt_for_instance`.
- NEW `agm prompts queue ls -i <instance> [--json]` (lists `queued`, `dispatching`, `failed`).
- NEW `agm prompts trace -i <instance> [--conversation <cid>] [--json]`.
- `agm rrp -i` calls only `restore_backed_up_prompts_for_instance`; `agm rrc -i` backs up and resends the same instance only.

### 9.3 IPC

- Existing (`src-tauri/src/commands/instance.rs`): `list_instances` (`:5`), `fast_forward_instance` (`:50`), `switch_account_to_instance` (`:140`), `list_running_projects` (`:207`, no instance arg), `list_backed_up_prompts` (`:212`, no instance arg), `resume_recent_project_prompts` (`:222`), `get_project_conversation_tree` (`:312`, `src/pages/Instances.tsx:281` passes no instance), `trigger_manual_profile_rotation`. No backup or restore IPC exists.
- Missing today: `enqueue_prompt`. The UI calls it at `src/components/instances/PromptTreeViewModal.tsx:1335`, it has 0 hits in `src-tauri/src`, and the catch falls back to writing the shared resume file (`:1344` to `:1363`), so the user sees no error and no `queued` row exists.
- NEW: `enqueue_prompt(instance_id, repo_path, prompt_content, conversation_id?, project_id?)`, `backup_running_prompts_for_instance(instance_id)`, `restore_running_prompts_for_instance(instance_id, keep_backup?)`, `send_prompt_now(...)` (replaces the UI `save_text_file` at `:1250` and `resume_recent_project_prompts` at `:1262`), `trace_prompts(...)`, optional `list_queued_prompts(instance_id)`. Optional `instance_id` added to `list_backed_up_prompts` and `list_running_projects`. `Instances.tsx:281` passes the instance id.
- Spec 94 law L4: the UI calls the same Rust functions as the CLI; no IPC shells out to `agm`; no UI code writes prompt state to disk.

---

## 10. Open items

- Plan 139 (`.ai-memory/plans/pending/139-switch-followups-pid-refresh-parity-and-live-e2e.md`): `pid_refresh_seconds` has no Settings UI and no CLI flag (headless and CLI parity); passing `session_id` to `agy` (no known flag, do not invent one); live account switch e2e that watches the reinject on a sandbox only; answer to ambiguity 02.
- Plan 140 steps (in order): 1 read; 2 E2E-00 and E2E-01 (stop and report if E2E-01 fails); 3 subtask 01 (D1 canonical key, D2 identity triple and migration, D3 resume hand-off, first find who reads the file); 4 subtask 02 (D4 state machine and one restore path, D5 running detection, D6 switch flows); 5 subtask 03 (D7 CLI, D8 IPC and UI reuse); 6 subtasks 04 and 05 (run E2E-00..E2E-19); 7 update this memory, RCA 34 status, `.ai-memory/what-to-read.md`, move the plan to `completed/`. Done when AC-01..AC-32 each have PASS or BLOCKED, and RCA 34 is `solved` with SHAs.
- Ambiguity 02: `.ai-memory/ambiguous-questions/01-new-ambiguity/02-is-instance-running-saved-pid-fast-path.md` (v4.155.0 scan vs the saved-PID fast path).
- Spec 140 open questions: (1) who reads `.antigravity_resume_task.json` (AGM readers at `repo_db.rs:1283`, `:3534`, `notification_hub.rs:570`, `agy_cleaner.rs:377`, `instance.rs:4941`, `agm.rs:7935`, `:8346`, `:11786`, `:14620`, `:14796`, `scripts/test-instance-e2e.ps1:439`; unknown whether the IDE, an extension, or `agy` reads it); (2) whether Antigravity honours `USERPROFILE`, `APPDATA`, `LOCALAPPDATA`, `HOME` for every file (AC-01); (3) whether macOS `open -a` passes the environment (`instance.rs:3440`); (4) whether the default registry id is literally `"default"` (code accepts `is_default` or `id == "default"` at `instance.rs:4382`, `integration.rs:384`).
- Orphan folder `test-e2e-alpha-5232` exists on disk but not in the registry; ask the user before deleting it.

## 11. E2E safety findings from the runbook author (added after Spec 02 reported)

Source: `02-spec/21-app/140-per-instance-prompt-attribution-backup-queue/02-component-and-e2e-spec.md`, `.ai-memory/plans/subtasks/140-per-instance-prompt-attribution-backup-queue/05-e2e-runbook-and-evidence-capture.md`.

- BANNED: `agm tif` / `agm test-instance-flow`. Its cleanup deletes every `test-cli-flow*` instance, which includes the protected `test-cli-flow-1743`.
- BANNED: `agm instances rm-all` (removes all sandboxes), and any command that acts on all instances or on default without `-i`.
- `agm instances create` appends `-<now%10000>` to the slug (`src-tauri/src/modules/instance.rs:838`). The real id is `test-cli-flow-a-<rand4>-<ts4>`; read it from `--json`. A and B share `<rand4>`, which gives a ready-made ambiguous suffix for AC-04.
- Scratch repos must have no `.git` folder, because `agm prompt` runs `git pull` when the current folder has one.
- Sandbox mode: `ABV_DATA_DIR` is read first by `account::get_data_dir` (`src-tauri/src/modules/account.rs:836`), so one `agm` process can run against a copy of the data folder. Use it for E2E-14 (migration), E2E-17 (refusal), and today's runs of `rrp`, `rrc`, `qs`.
- Today's `rrp`, `rrc`, `qs`, and the account switch can dispatch prompts that belong to other instances, including default. Those cases run live only after subtasks 01 and 02 ship.
- Today's `brp` pulls empty-`instance_id` rows into the target instance; run it live only if the live databases have no such rows.
- On a build with subtask 01, the first live command migrates the real databases, so E2E-14 runs on a copy right after E2E-00.
- `agm prompt -i <inst>` sends through the Antigravity CLI with `USERPROFILE` set to the instance home (`src-tauri/src/modules/repo_db.rs:3164`), so E2E-01 path 1 proves the CLI flavor only; the IDE flavor needs a human or a UI automation tool.
- A clone may not copy home conversations; if so, E2E-06's live part is BLOCKED and the integration test carries AC-07.
