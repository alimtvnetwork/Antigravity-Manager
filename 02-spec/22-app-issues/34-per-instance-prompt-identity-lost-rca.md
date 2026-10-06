# 34 - Per-Instance Prompt Identity Lost Across Running, Backup, Queue, and Restore RCA

| Field | Value |
|---|---|
| Issue | 34 |
| Date | 2026-10-06 |
| Status | open (fix planned in spec 140, no code changed yet) |
| Fix spec | `02-spec/21-app/140-per-instance-prompt-attribution-backup-queue/01-architecture-spec.md`, `02-spec/21-app/140-per-instance-prompt-attribution-backup-queue/02-component-and-e2e-spec.md` |
| Parent plan | `.ai-memory/plans/140-per-instance-prompt-attribution-backup-queue.md` |
| Bug IDs | B01 to B33 (defined in section 4 of the architecture spec) |

## Executive Summary

AGM cannot reliably tell which Antigravity instance a prompt belongs to. When two or more instances run at the same time, or when one instance is relaunched by a switch, prompts show up as running in the wrong instance, switch backups move between instances, enqueued prompts never reach the queue, and restores send the same prompt two or three times. The user reported it in these words:

> "when we have multiple instances, or even same instances, the current prompt running, taking the backup or the enqueue prompt, all of these are not right from the CLI methods and how the code is."

The cause is not one bad line. Every layer works out the instance id again on its own, and the resume hand-off file is shared by every instance that opens the same repository. This RCA lists the symptoms per flow, the root cause, why ten earlier fixes did not close it, and the rules that the spec 140 fix must turn into tests.

---

## 1. Symptoms (What Breaks)

Line numbers were re-checked with GitMap on 2026-10-06.

### 1.1 Running detection

1. An empty instance id resolves to whichever instance is active, not to default: `src-tauri/src/modules/instance.rs:4375` to `:4376` (`resolve_instance_id` returns `get_active_instance_id()`). Callers that fall back to `"default"` only do so on `Err`, so the fallback never fires (B01).
2. A fuzzy suffix such as `8159` resolves to `default-copy-8159` inside storage code: `src-tauri/src/modules/instance.rs:4413` to `:4432` (B02).
3. An unknown id such as `__default__` silently creates `instances/__default__/home`: `src-tauri/src/modules/instance.rs:25` to `:33` (B03).
4. `"all"` is read as default and later stamped as an instance id; default merges the host home and `instances/default/home` under one id: `src-tauri/src/modules/repo_db.rs:824` to `:865`, stamp at `:1084` (B04).
5. The first conversation in a summaries database always counts as running, even when idle: `src-tauri/src/modules/repo_db.rs:992` (`|| prompts.is_empty()`) (B05).
6. A global `DELETE ... instr(id, '__') = 0` wipes rows written by the backup path and by `agm prompt`: `src-tauri/src/modules/repo_db.rs:626` (B11).
7. Gate 4 matches by folder name and prefix, so `app` matches `app-v2`: `src-tauri/src/modules/repo_db.rs:2008` to `:2012` (B12).
8. The instance filter is a substring test, so `default` matches `x__default-copy-8159`: `src-tauri/src/modules/repo_db.rs:2706` to `:2709` (B14).
9. The same conversation id in two instances (a clone) collapses into one untagged item: `src-tauri/src/modules/agy_cleaner.rs:95` to `:124`, `:192` to `:196` (B32).

### 1.2 Backup

1. The Layer 1 upsert on `prompt-{cid}` never updates `instance_id`, so a stale owner sticks: `src-tauri/src/modules/repo_db.rs:1195` to `:1203` (B06).
2. Layer 2 scans `state.vscdb` of every project, running or not, for any key `LIKE %chat%`, `%task%`, or `%prompt%`: `src-tauri/src/modules/repo_db.rs:1239` to `:1310` (B08).
3. Layer 2 fallback and dedupe have no instance filter and use `LIKE %repo_name%`: `src-tauri/src/modules/repo_db.rs:1316` to `:1319`, `:1337` (B09).
4. Layer 2 stores the workspace project id as `session_id` instead of a conversation id: `src-tauri/src/modules/repo_db.rs:1366`, `:1390` (B10).
5. Rows with an empty `instance_id` join every instance's backup and get re-stamped: `src-tauri/src/modules/backup_prompts_db.rs:249` to `:252`, re-stamp at `:338` (B18).
6. Dedupe ignores the instance and then reassigns the row to the current target, so backing up B steals A's backup: `src-tauri/src/modules/backup_prompts_db.rs:353` and `:371` (`UPDATE prompt_backups SET ... instance_id = ?3`) (B19).
7. The conversation id is matched by "workspace URIs contain the repo" with no instance filter: `src-tauri/src/modules/backup_prompts_db.rs:300` to `:304` (B20).

### 1.3 Enqueue and queue

1. The scheduler treats switch backups (`backed_up`) as queue items and races the restore path: `src-tauri/src/modules/repo_db.rs:2060` to `:2064` (B13).
2. The scheduler marks a row `dispatched` whether or not the send worked: `src-tauri/src/modules/repo_db.rs:2189` to `:2195` (`sent` is never checked) (B13).
3. The UI calls IPC `enqueue_prompt`, which does not exist in Rust, then falls back to writing the shared resume file; no `queued` row is ever written: `src/components/instances/PromptTreeViewModal.tsx:1335` (B28).

### 1.4 Restore and dispatch

1. `.antigravity_resume_task.json` lives in the repository folder, so two instances on one repo overwrite each other: writers at `src-tauri/src/modules/repo_db.rs:1224`, `:1382`, `:1603`, `:2175`, `:3376`, `:3595`; `src-tauri/src/bin/agm.rs:3154`; `src/components/instances/PromptTreeViewModal.tsx:1237` (B07).
2. Readers open that file without checking its `instance_id`: `src-tauri/src/modules/repo_db.rs:1283`, `:3534`; `src-tauri/src/modules/notification_hub.rs:570`; `src-tauri/src/modules/agy_cleaner.rs:377`; `src-tauri/src/bin/agm.rs:7935`, `:8346` (B33).
3. `spawn_prompt_via_agy` keys workers by the raw id, so `default` and `__default__` are two workers; `__default__` takes the named-instance HOME branch; `-p` starts a new chat: `src-tauri/src/modules/repo_db.rs:3125`, `:3164` (B15).
4. Resend includes rows still `running` (duplicates), dedupes by repo only, and drops `session_id`: `src-tauri/src/modules/repo_db.rs:3248`, `:3317`, `:3376` to `:3390` (B16).
5. Auto resume uses `LIKE %repo_name%` and then the shared resume file: `src-tauri/src/modules/repo_db.rs:3507`, `:3512`, `:3534` (B17).
6. An auto rotation with target `"default"` passes `None`, which re-sends every instance's prompts: `src-tauri/src/modules/auto_switcher.rs:1690` to `:1695` (B21).
7. A cross-instance rotation backs up the source instance but dispatches under the target id, so moved prompts are never restored: `src-tauri/src/modules/auto_switcher.rs:1431`, `:1595`, `:1715` (B22).
8. Desktop switch and Telegram hardcode `"default"` and run three restore paths back to back: `src-tauri/src/modules/integration.rs:341` to `:437`; `src-tauri/src/modules/telegram_inbound.rs:1355`, `:1385`, `:2729`, `:2788` (B23). `src-tauri/src/modules/account.rs:1743` is a real default-only path and is correct.
9. Every relaunch with reinject runs restore, then resend, then dispatch: `src-tauri/src/modules/instance.rs:3565` to `:3575`, `:3669` to `:3680` (B29).
10. `restore_running_prompts_for_instance` already resends, so every caller that resends afterwards sends twice: `src-tauri/src/modules/backup_prompts_db.rs:614` (B30).

### 1.5 CLI

1. `agm prompt -n` means `--node` (remote SSH), not instance; only `-i/--instance` targets an instance: `src-tauri/src/bin/agm.rs:3008`, help text `:2931` (B24).
2. `agm prompt` without `-i` uses the active instance, not the owner of the current folder; inserts a `running_projects` row with no `__` and an empty storage path (later deleted by B11); stores the repo slug as `session_id`; marks the row `dispatched` before the spawn result is known: `src-tauri/src/bin/agm.rs:3129` to `:3150` (B25).
3. `agm wpr` joins prompts to projects by path with no instance check: `src-tauri/src/bin/agm.rs:1640` (B26).
4. `agm rrc` backs up only the active instance and then re-sends every instance: `src-tauri/src/bin/agm.rs:3372` to `:3376` (B27).
5. `agm rrp` resends and dispatches before it restores, then the restore resends again: `src-tauri/src/bin/agm.rs:4169` to `:4171` (B31).

### 1.6 UI

1. Enqueue never writes a database row (B28, section 1.3).
2. "Send Now" writes the shared resume file directly from the UI: `src/components/instances/PromptTreeViewModal.tsx:1237` (B07).
3. The modal falls back to `'default'` when the instance is unknown (spec 140 section 6.10, UI rule 1).
4. `list_running_projects` and `list_backed_up_prompts` take no instance argument, and `src/pages/Instances.tsx:281` calls `get_project_conversation_tree` without an instance id, so the UI cannot show one instance's prompts only.

---

## 2. Root Cause

### 2.1 One sentence

There is no canonical instance key that is set once when a prompt is captured and carried unchanged to dispatch: every layer re-derives it (`"default"`, `"__default__"`, `""`, the active instance, fuzzy `resolve_instance_id`, substring or `LIKE` matches, path-only joins), and the resume hand-off file is per repo instead of per (instance, repo, conversation), so identity is lost whenever two instances share a repo or a key fails to normalize.

### 2.2 Five whys

1. **Why does a prompt from instance A get restored into B, restored twice, or not at all?** Because the row that reaches dispatch carries the wrong `instance_id`, no `instance_id` filter, or a resume file that B overwrote.
2. **Why is the `instance_id` wrong at dispatch?** Because it was rewritten after capture: the backup dedupe re-stamps it (B19), empty ids are assigned to the active instance (B01), and switch paths pass `"default"`, `None`, or the target id instead of the source id (B21, B22, B23).
3. **Why can later code rewrite or reinterpret it?** Because there is no single function that turns any input into one canonical id. Storage code calls `resolve_instance_id`, which accepts empty, `active`, and fuzzy suffixes, and each caller adds its own `"default"` fallback.
4. **Why was no canonical key introduced earlier?** Because each earlier fix targeted the visible symptom in one flow (running detection, TTL, composite project ids) and treated the instance id as an input already known to be correct, not as a value that must be captured once and frozen.
5. **Why did the per-repo resume file survive?** Because it was designed when only one instance ran per repository. The file carries `instance_id`, but no reader checks it and no writer is scoped by it, so it silently became shared state between instances.

---

## 3. Contributing Causes

### 3.1 Key normalization drift
`""`, `"default"`, `"__default__"`, `"all"`, and `"active"` are treated differently in different functions. `resolve_instance_id` maps empty to the active instance (B01), `get_instance_home_dir` creates a folder for any unknown string (B03), `gemini_dirs_for_instance` treats `"all"` as default and the result is stamped `"all"` (B04), and `spawn_prompt_via_agy` treats `"__default__"` as a named instance (B15).

### 3.2 Per-repo resume file
`.antigravity_resume_task.json` is written to `<repo_path>/` by eight writers (B07) and read by six readers with no instance check (B33). Two instances on the same repository overwrite each other's hand-off.

### 3.3 Status confusion between `backed_up` and `queued`
Discovery writes `backed_up` at capture (`src-tauri/src/modules/repo_db.rs:1089`), switch backup writes `backed_up`, and the scheduler consumes `backed_up`, `queued`, and `pending` together (B13). A switch backup is therefore treated as a user enqueue and dispatched by whichever path wins the race.

### 3.4 Three restore paths
`restore_running_prompts_for_instance` resends internally (B30), and callers then call `resend_running_commands_for_instance` and `dispatch_running_prompts` again (B23, B29, B31). One restore sends the same prompt up to three times.

### 3.5 Fuzzy joins
Prompts are joined to projects and instances with `LIKE %repo_name%` (B09, B17), substring `contains` on the instance id (B14), "URIs contain repo" (B20), folder-name prefix match (B12), and path-only joins (B26). Neighbor repositories and neighbor instances match.

### 3.6 Hardcoded `"default"`
Desktop switch, Telegram, and the auto switcher pass `"default"` or turn it into `None` (all instances) even when the effective target is a named instance (B21, B23). The CLI fallback `unwrap_or_else(|_| "default".to_string())` repeats this at `src-tauri/src/bin/agm.rs:3130` and `:3372`.

### 3.7 Missing IPC `enqueue_prompt`
The UI calls a command that is not registered in Rust (B28). The failure is caught and replaced by a shared file write, so the user sees no error and no queued row exists.

### 3.8 CLI defaults to the active instance
`agm prompt`, `agm brp`, and `agm rrc` act on the active instance when `-i` is missing (B25, B27). With two instances running, the command silently acts on the wrong one.

### 3.9 `-n` means `--node`
On `agm prompt`, `-n` routes the prompt to a remote SSH node (`src-tauri/src/bin/agm.rs:3008`). Users and agents who read `-n` as "instance name" dispatch to a node lookup instead of an instance (B24).

---

## 4. Why Earlier Fixes Did Not Close It

| Spec | What it fixed | What it left open |
|---|---|---|
| 100 | Named instances read only their own home `.gemini`; resume file keeps `session_id`; PID refresh window | Resume file stayed per repo; default still merges two stores |
| 113 | Projects come from each instance's own `workspaceStorage`; running needs PID plus active prompt | Backup and queue still re-derive the instance id |
| 118 | Liveness TTL of 300 s | TTL only; no key change |
| 120 | `running_projects.id` instance-prefixed; dedupe by `(instance_id, cid)` | Dedupe key not applied to `active_prompts`, `prompt_backups`, or sequences |
| 121 | Composite id `{base}__{instance_id}`; running means `not_fully_idle > 0 AND RUNNING` | The `prompts.is_empty()` rule (B05) and the global `DELETE` (B11) still bypass it |
| 122 | Four-gate running check | Gate 4 kept the folder-name prefix match (B12) |
| 123 | Dead process forces idle, with reason codes | Reason codes never reached dispatch (`dispatched` written on failure) |
| 126, 128 | TTL 60 s to 120 s; running only if PID alive | TTL only; no key change |
| 132 | Turns idle for more than 120 s are not running; match `instance_id` or `__{id}` suffix | Substring instance filter (B14) remained in `get_project_execution_status` |

All of these fixed running detection keys and timeouts. None of them made the instance id a single captured key carried unchanged across backup, queue, and dispatch, and none of them touched the per-repo resume file. Each fix was correct for its own flow, and identity was lost again at the next hand-over between flows.

---

## 5. Feasibility Note

Antigravity writes no instance tag of its own. The `app_data_dir` column in `conversation_summaries.db` holds only product names such as `antigravity` and `antigravity-cli`. The instance link is therefore:

1. the per-instance HOME folder: AGM launches each named instance with `USERPROFILE`, `APPDATA`, and `LOCALAPPDATA` (and `HOME` on Unix) set to `<instances>/<id>/home` (`src-tauri/src/modules/instance.rs:3586` to `:3618`), so its conversations land under `<instances>/<id>/home/.gemini/<flavor>/`; the default instance owns the global `%USERPROFILE%\.gemini` store; and
2. the `instance_id` that AGM stamps on the row at capture time.

This is unproven live. The per-instance summaries databases that exist on disk today have 0 rows, so it is not yet shown that Antigravity honors the overridden profile for every file. macOS launches through `open` (`src-tauri/src/modules/instance.rs:3440`), which may drop the environment, so macOS is unverified. E2E-01 in the companion spec must prove the per-instance store before any other test result is trusted.

---

## 6. Fix Plan

| Item | Path |
|---|---|
| Architecture spec (bug table B01 to B33, target design, AC-01 to AC-32) | `02-spec/21-app/140-per-instance-prompt-attribution-backup-queue/01-architecture-spec.md` |
| Component and end-to-end spec | `02-spec/21-app/140-per-instance-prompt-attribution-backup-queue/02-component-and-e2e-spec.md` |
| Parent plan | `.ai-memory/plans/140-per-instance-prompt-attribution-backup-queue.md` |
| Subtask 01, canonical instance key and per-instance resume file | `.ai-memory/plans/subtasks/140-per-instance-prompt-attribution-backup-queue/01-canonical-instance-key-and-per-instance-resume-file.md` |
| Subtask 02, backup, queue, and restore pipeline fixes | `.ai-memory/plans/subtasks/140-per-instance-prompt-attribution-backup-queue/02-backup-queue-restore-pipeline-fixes.md` |
| Subtask 03, CLI and IPC parity | `.ai-memory/plans/subtasks/140-per-instance-prompt-attribution-backup-queue/03-cli-and-ipc-parity.md` |
| Subtask 04, E2E multi-instance test catalog | `.ai-memory/plans/subtasks/140-per-instance-prompt-attribution-backup-queue/04-e2e-multi-instance-test-catalog.md` |
| Subtask 05, E2E runbook and evidence capture | `.ai-memory/plans/subtasks/140-per-instance-prompt-attribution-backup-queue/05-e2e-runbook-and-evidence-capture.md` |

Order: subtask 05 (E2E-00 and E2E-01 only) first to prove feasibility, then 01, 02, 03, then 04 and 05 in full.

---

## 7. Prevention Rules

Each rule becomes at least one unit or end-to-end test in spec 140.

1. **Never derive an instance id twice.** The id is stamped once at capture from the gemini directory that was scanned, through `canonical_instance_id`. Test: `""`, `"default"`, and `"__default__"` give the same id; an unknown id returns `Err` and creates no folder (AC-02, AC-03).
2. **Never `UPDATE instance_id`.** No statement may change the owner of an existing prompt or backup row. Test: backing up B leaves every row owned by A unchanged (AC-09, AC-10).
3. **Never use `LIKE`, `contains`, or a folder-name prefix to join prompts to projects or instances.** Use normalized full-path equality and exact instance equality. Test: `agm wpr -i default` returns no `default-copy-*` row; `app` does not match `app-v2` (AC-22).
4. **Never fall back to the active instance inside storage code.** Only the CLI or UI input layer may resolve `active`, and only when the user typed it. Test: action commands without `-i` refuse with exit code 2 when two instances run (AC-26).
5. **One restore path.** A switch restore calls only `restore_backed_up_prompts_for_instance`. Test: after `agm rrp -i <A>`, every restored row has `attempts <= 1` and no row of B changed (AC-17, AC-18).
6. **The queue consumes only `queued`.** Test: `backed_up` rows are unchanged after `agm qs -i <A> --once` (AC-14).
7. **`dispatched` only when sent.** A failed send writes `failed` with a `status_reason`. Test: a missing repo gives `failed`, `repo_missing` (AC-15).
8. **Per-instance hand-off.** Every hand-off carries `instance_id`, `session_id`, and `conversation_id`; readers ignore a hand-off for another instance; the legacy repo file is written only when one instance owns the repo. Test: two instances enqueue to one repo and neither hand-off overwrites the other (AC-11, AC-12, AC-13).

---

## 8. Verification Status

- Nothing is fixed yet. No application code was changed for this issue.
- No live end-to-end run has been done yet. E2E-01 (per-instance store proof) is the first required run.
- The bug sites B01 to B33 were confirmed by reading code and re-checking line numbers with GitMap on 2026-10-06.
