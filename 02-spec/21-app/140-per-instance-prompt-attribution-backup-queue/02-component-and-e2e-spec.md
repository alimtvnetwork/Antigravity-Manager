# Component and End-to-End Specification: Per-Instance Prompt Attribution

| Field | Value |
|---|---|
| Spec | 140, part 02 |
| Slug | `140-per-instance-prompt-attribution-backup-queue` |
| Created | 2026-10-06 |
| Status | planned (no application code is written in this run) |
| Architecture spec | `02-spec/21-app/140-per-instance-prompt-attribution-backup-queue/01-architecture-spec.md` (decisions, bug IDs B01 to B33, acceptance criteria AC-01 to AC-32) |
| Parent plan | `.ai-memory/plans/140-per-instance-prompt-attribution-backup-queue.md` |
| Subtasks owned by this spec | `.ai-memory/plans/subtasks/140-per-instance-prompt-attribution-backup-queue/04-e2e-multi-instance-test-catalog.md`, `.ai-memory/plans/subtasks/140-per-instance-prompt-attribution-backup-queue/05-e2e-runbook-and-evidence-capture.md` |
| Design source | `.ai-memory/temp-agents/95-per-instance-prompt-attribution-backup-queue/target-design.md` (D1 to D10) |
| Reused E2E material | `02-spec/21-app/92-cli-instance-switch-e2e.md`, `02-spec/21-app/93-e2e-cli-commands-ai-instruction.md`, `02-spec/21-app/94-blind-ai-instance-test-and-release.md`, `src-tauri/tests/per_instance_prompt_liveness_test.rs` |

This document does not repeat the root cause or the bug table. Read section 2 and section 4 of the architecture spec first. Every test below names the AC IDs it proves and the bug IDs it exercises.

## 1. How to read this spec

1. Section 2 is the contract each component must meet: which instance key it receives and which key it writes.
2. Section 3 maps every user action to the CLI command, the IPC command, and the one shared Rust function behind both.
3. Section 4 is the test environment model: how an AI knows, at every step, which prompt belongs to which instance.
4. Section 5 is the full E2E catalog, E2E-00 to E2E-19.
5. Section 6 is the coverage matrix (every AC covered at least once).
6. Section 7 lists the unit and integration tests that cover the paths that must never run live.

The step-by-step PowerShell runbook lives in subtask 05. The compact checklist and ordering live in subtask 04.

## 2. Component contract table

"Key received" is the instance key the component accepts from its caller. "Key written" is the instance key it must store or pass on. "Canonical" means the output of `canonical_instance_id` (architecture spec section 6.1): an exact registry id, with `""`, `"default"`, `"__default__"` all mapped to the default id. "Input text" means raw user text (`#2`, a suffix, a display name, `active`) that only `resolve_instance_input` may interpret.

| # | Component | Rust entry points (current file) | Inputs | Outputs | Key received | Key it must write | Bugs fixed |
|---|---|---|---|---|---|---|---|
| C1 | Canonical key resolver | NEW `canonical_instance_id`, NEW `resolve_instance_input`, changed `get_instance_home_dir` (`src-tauri/src/modules/instance.rs`) | raw id string or user input text | `Ok(canonical id)` or `Err` with a candidate list | raw string (storage callers) or input text (CLI and IPC entry only) | none; it never writes rows and never creates `instances/<unknown>/home` | B01, B02, B03 |
| C2 | Gemini dir scanner | `gemini_dirs_for_instance`, NEW `gemini_dirs_tagged(InstanceScope)` (`src-tauri/src/modules/repo_db.rs`); NEW `scan_conversations_for(InstanceScope, keep)` (`src-tauri/src/modules/agy_cleaner.rs`) | `InstanceScope::One(id)` or `InstanceScope::All` | list of `(instance_id, dir)` pairs; list of `(instance_id, ConversationItem)` | canonical id or `All` | every returned dir and conversation carries its owner id; `"all"` is never returned as an id | B04, B20, B32 |
| C3 | Running detector | `discover_running_prompts_from_antigravity`, `detect_running_projects`, `is_prompt_running_for_project`, `get_project_execution_status` (`src-tauri/src/modules/repo_db.rs`) | canonical id, the instance PID state, the summaries DB rows of that instance's dirs | `ActivePrompt` candidates with `status = running`; `running_projects` rows | canonical id | `active_prompts.instance_id` = the owner from C2, stamped once; row id `prompt-{instance_id}-{conversation_id}`; `running_projects.id` = `{base}__{instance_id}`; `source_dir` = the scanned dir | B05, B08, B11, B12, B14 |
| C4 | Backup writer | NEW `backup_prompts_for_instance` (`src-tauri/src/modules/backup_prompts_db.rs`), merging `repo_db::backup_running_prompts` and `backup_active_running_prompts_for_instance` for one instance | canonical id, optional custom backup file | `BackupReport { instance_id, batch_id, inserted, reactivated, skipped }` | canonical id | `prompt_backups.instance_id` = the target id on insert only; never `UPDATE ... SET instance_id`; dedupe key `(instance_id, prompt_id)` then `(instance_id, normalized project_path, trimmed prompt_text)` | B06, B08, B09, B10, B18, B19, B20 |
| C5 | Queue writer | NEW `enqueue_prompt_for_instance`, NEW `send_prompt_now_for_instance` (`src-tauri/src/modules/repo_db.rs`) | canonical id, repo path, text, optional conversation id, `source_dir` (`cli` or `ui`) | one `ActivePrompt` row (`queued-{instance_id}-{uuid}` or `cli-{instance_id}-{uuid}`) | canonical id (resolved from input text by the CLI or IPC wrapper before the call) | `active_prompts.instance_id` = that id, `status = queued`, `session_id` = the conversation id or empty, never a repo slug | B25, B28 |
| C6 | Queue scheduler | `check_and_dispatch_enqueued_prompts` (`src-tauri/src/modules/repo_db.rs`) | `InstanceScope` | per-row outcome (`dispatched`, `failed` with `status_reason`) | canonical id or `All` | consumes only `status = queued`; claims with the atomic `UPDATE ... WHERE id = ? AND status = 'queued'`; writes `dispatched` only when the send returned true | B13 |
| C7 | Restore dispatcher | NEW `restore_backed_up_prompts_for_instance` (`src-tauri/src/modules/backup_prompts_db.rs`); `spawn_prompt_via_agy`, `resend_running_commands_for_instance`, `auto_resume_recent_prompts`, `dispatch_running_prompts` (`src-tauri/src/modules/repo_db.rs`) | canonical id, `keep_backup`, optional custom file | `RestoreReport { instance_id, restored_from_backup, dispatched, failed, skipped }` | canonical id | reads only `prompt_backups.instance_id = ?` and `active_prompts.instance_id = ? AND status = 'backed_up'`; worker key `{canonical id}:{normalized repo}`; HOME override chosen by canonical id; `attempts` incremented once per claim; `source_dir = backup_restore` | B15, B16, B17, B29, B30, B31 |
| C8 | Resume hand-off writer and reader | NEW `write_resume_handoff` (`src-tauri/src/modules/repo_db.rs`) built on `resume_task_document`; readers in `notification_hub.rs`, `agy_cleaner.rs`, `agm.rs`, `repo_db.rs` | the `ActivePrompt` being dispatched | `ResumeHandoffOutcome { path, legacy: written \| skipped_multi_owner \| not_needed, error }` | canonical id taken from the row | document keys `instance_id`, `session_id`, `conversation_id` (session equals conversation); per-instance file `.antigravity_resume_task.<instance_id>.json` when the reader research allows it; legacy file only when exactly one instance owns the repo; readers drop documents whose `instance_id` is not theirs | B07, B33 |
| C9 | Switch orchestrators | `execute_profile_rotation_with_context` (`src-tauri/src/modules/auto_switcher.rs`), desktop switch (`src-tauri/src/modules/integration.rs`), Telegram (`src-tauri/src/modules/telegram_inbound.rs`), `switch_account_to_instance` and launch reinject (`src-tauri/src/modules/instance.rs`) | the instance the switch acts on | one backup call (C4) before close, one restore call (C7) after relaunch, one audit entry | canonical id of the instance that is closed and relaunched | the same id for backup, restore, and the audit entry; a `"default"` target stays `Some(default id)`; cross-instance rotation restores the source only into the source | B21, B22, B23, B29 |
| C10 | Legacy migration | NEW `migrate_prompt_instance_ids_v140` (twins in `repo_db.rs` and `backup_prompts_db.rs`) | the live DB file | migrated DB, `<db>.pre-v140.bak`, `agm_schema_migrations` row `v140_instance_ids` | none (reads `gemini_dirs_tagged(All)`) | the single owner id, or `instance_id = ''` with `status = orphaned` and `status_reason` | residue of B01, B18 |
| C11 | Prompt tracer | NEW `trace_prompts_for_instance` (`src-tauri/src/modules/repo_db.rs`) | canonical id, optional conversation id | `Vec<PromptTrace>` with `source_dir`, `conversation_id`, `repo_path`, `row_id`, `status`, `status_history`, `backup_row_id`, `handoff`, `dispatch_result` | canonical id | none (read-only) | supports AC-30 |
| C12 | CLI commands | `src-tauri/src/bin/agm.rs`: `wpr`, `prompts`, `prompt`, `rrc`, `brp`, `qs`, `rrp`, `running-prompts`, `running-projects`, `tree`, NEW `prompt enqueue`, NEW `prompts queue ls`, NEW `prompts trace` | argv | text or `--json`; exit 0, 1 (error), 2 (instance required) | input text from `-i/--instance` (positional on `qs`) | passes only the canonical id to C3 to C11; every printed row and JSON object has `instance_id` and `instance_name` | B24, B25, B26, B27, B31 |
| C13 | IPC commands | `src-tauri/src/commands/instance.rs`: NEW `enqueue_prompt`, NEW `send_prompt_now`, NEW `backup_running_prompts_for_instance`, NEW `restore_running_prompts_for_instance`, NEW `trace_prompts`, NEW `list_queued_prompts`; changed `list_running_projects`, `list_backed_up_prompts` (optional `instance_id`); `get_project_conversation_tree` | JSON args from the UI | serialized results with `instance_id` and `instance_name` | input text or instance id from the UI | thin wrapper: resolve input, call the same function the CLI calls (C4 to C7, C11); never shell out to `agm`, never write files | B28 |
| C14 | UI: `PromptTreeViewModal` and Instances page | `src/components/instances/PromptTreeViewModal.tsx` (`handleEnqueuePrompt`, Send Now), `src/pages/Instances.tsx` (tree load) | the modal's `instanceId`, the project row's `instance_id` | IPC calls only | the instance id of the open modal or row | passes `instanceId` to every IPC call; never defaults to `'default'` when unknown (shows an error); never writes `.antigravity_resume_task*.json`; shows the badge `#seq name` on every prompt row; Backup and Restore actions sit in the instance row's contextual panel as a segmented pill pair (AGENTS.md UI rule) | B07 (UI writer), B28 |

### 2.1 Key flow rule (applies to every row above)

1. The instance key enters the system exactly once per prompt: at capture (C3, from the dir C2 returned), at enqueue (C5, from the CLI or UI input resolved by C1), or at migration (C10).
2. From then on the key is copied, never re-derived. No component calls `get_active_instance_id` except `resolve_instance_input` when the user literally typed `active`.
3. No join or filter uses `LIKE %name%`, substring `contains`, or folder-name prefix. Path equality uses `normalize_path_for_compare` only.

## 3. CLI and UI parity table

"Today" columns are verified in `src-tauri/src/bin/agm.rs` and `src-tauri/src/commands/instance.rs` (2026-10-06). "NEW" marks a command or flag that does not exist today and ships in subtask 03 (`.ai-memory/plans/subtasks/140-per-instance-prompt-attribution-backup-queue/03-cli-and-ipc-parity.md`).

| User action | CLI command today | CLI command target | IPC command today | IPC target | Shared Rust function |
|---|---|---|---|---|---|
| List running prompts | `agm wpr --json` (no instance flag; scans all instances); `agm running-prompts ls --json` | `agm wpr -i <inst> --json` (NEW `-i`); without `-i` grouped by instance | `get_project_conversation_tree(instance_id?)` (Instances.tsx passes none); `list_running_projects()` (no instance arg) | `get_project_conversation_tree(instance_id)`; `list_running_projects(instance_id?)` | `get_project_conversation_tree_cached`, `list_running_projects(InstanceScope)` |
| List queue | none (`agm queue` / `agm queues` is the agy queue view, not `active_prompts`); today's equivalent: `agm prompts --json` filtered by `instance_id` and `status` in the runbook | NEW `agm prompts queue ls -i <inst> --json` | `list_backed_up_prompts()` (no instance arg) | NEW `list_queued_prompts(instance_id)`; `list_backed_up_prompts(instance_id?)` | NEW `list_queued_prompts_for_instance` |
| Enqueue | none (`agm prompt` dispatches immediately); today's equivalent for tests: a seeded row in a sandbox copy (runbook sandbox mode) | NEW `agm prompt enqueue -i <inst> --repo <path> [--conversation <cid>] "<text>" [--json]` | `enqueue_prompt` is called by the UI but is not registered in Rust (B28); the UI then writes the shared resume file | NEW `enqueue_prompt(instance_id, repo_path, prompt_content, conversation_id?, project_id?)` | NEW `enqueue_prompt_for_instance` |
| Send now | `agm prompt -i <inst> "<text>"` (cwd is the repo; `-n` is `--node`, not instance) | `agm prompt -i <inst> "<text>"` with the refusal rule and `failed` on spawn error | none (UI writes the file with `save_text_file`, then calls `resume_recent_project_prompts`) | NEW `send_prompt_now(instance_id, repo_path, prompt_content, conversation_id?)` | NEW `send_prompt_now_for_instance` |
| Backup | `agm brp -i <inst> --json` (no `-i` means the active instance) | `agm brp -i <inst> --json`; refuses without `-i` when 2 or more instances run | none | NEW `backup_running_prompts_for_instance(instance_id)` | NEW `backup_prompts_for_instance` |
| Restore | `agm rrp -i <inst> [--keep] --json` (resends before restoring, B31) | `agm rrp -i <inst> [--keep] --json` through the single restore path | none | NEW `restore_running_prompts_for_instance(instance_id, keep_backup?)` | NEW `restore_backed_up_prompts_for_instance` |
| Switch account | `agm instances switch <inst> <account>`; `agm switch <account> --instance <inst>` | unchanged syntax; backup and restore through C4 and C7 with the same id | `switch_account_to_instance` | unchanged | `instance::switch_account_to_instance` calling C4 then C7 |
| Auto-switch | `agm instances ff <inst>` / `agm ff <inst>` (scoped); `agm auto-switch run` and `agm auto-switch test [N]` (all instances) | unchanged syntax; rotation keeps prompts with the source instance | `fast_forward_instance`; `trigger_manual_profile_rotation(instance_id?)` | unchanged | `auto_switcher::trigger_manual_rotation_for_instance` and `execute_profile_rotation_with_context` |
| Trace | none | NEW `agm prompts trace -i <inst> [--conversation <cid>] [--json]` | none | NEW `trace_prompts(instance_id, conversation_id?)` | NEW `trace_prompts_for_instance` |
| Tree | `agm tree [all] --json` (no instance flag) | `agm tree [all] -i <inst> --json` (NEW `-i`) | `get_project_conversation_tree(instance_id?)` | same, and `src/pages/Instances.tsx` must pass `instanceId` | `get_project_conversation_tree_cached` |

Commands that are never allowed in any E2E run on this machine, because they act on every instance, on the default instance, or delete sandboxes:

| Command | Why it is banned |
|---|---|
| `agm test-instance-flow` / `agm tif` | Step 1 deletes every `test-cli-flow*` and `test-diag*` instance, which includes the protected `test-cli-flow-1743` |
| `agm instances rm-all` | Removes all non-default sandboxes |
| `agm switch <account>` without `--instance`, `agm account switch <account>` without an instance | Switches the active or default instance |
| `agm ff` without an instance, `agm instances all ff` | Rotates every running instance, including default |
| `agm auto-switch run`, `agm auto-switch test`, `agm auto-switch enable`, `disable`, `toggle` | Evaluates or changes rotation for all instances and changes the user's daemon state |
| `agm rrc`, `agm rrp`, `agm brp`, `agm qs` without an instance (today) | Today they act on the active instance or on all instances (B13, B27, B31) |
| `agm agy cache-clear`, `agm accounts export`, `scripts/dev-tool-clear.ps1 -CleanInstances` | Deletes user conversations, prints tokens, or deletes instances |
| `scripts/test-instance-e2e.ps1`, `scripts/test-instance-isolation.ps1` as they are | They hold absolute paths and create instances without the allowed prefix (law L8 of spec 94) |

## 4. Test environment model

### 4.1 The problem the model solves

Antigravity writes no instance tag (architecture spec section 3). The only ways to know which instance a prompt belongs to are the folder a store lives in and the `instance_id` AGM stamps. A test that cannot tell which instance produced a row cannot prove anything. The model below gives every test prompt three independent fingerprints: the instance it was sent to, a unique marker in its text, and the per-instance folder it must appear in.

### 4.2 Naming rule for test instances

| Role | Display name passed to `agm instances create` | Registry id (read back from `--json`) | Created by |
|---|---|---|---|
| A | `test-cli-flow-a-<rand4>` | `test-cli-flow-a-<rand4>-<ts4>` | `agm instances create test-cli-flow-a-<rand4> --data-only --json` |
| B | `test-cli-flow-b-<rand4>` | `test-cli-flow-b-<rand4>-<ts4>` | `agm instances create test-cli-flow-b-<rand4> --data-only --json` |
| C (optional clone) | `test-cli-flow-c-<rand4>` | read from `--json` | `agm instances create test-cli-flow-c-<rand4> --from <A> --data-only --json` (clone of A, to reproduce the same conversation id in two instances) |

Rules:

1. `<rand4>` is one random 4-digit number per run and is shared by A, B, and C. `create_instance_with_account` appends `-<now % 10000>` to the slug (`src-tauri/src/modules/instance.rs:838`), so the registry id is never the display name. Always read `id` from the `--json` output and store it in `run.json`.
2. Because A and B share the name suffix `-<rand4>`, the input `<rand4>` is a ready-made ambiguous suffix for AC-04.
3. The prefix `test-cli-flow-` satisfies law L8 of spec 94 and D10. Cleanup deletes only ids recorded in this run's `run.json`, never by prefix.
4. On `agm instances create`, `-f` means `--from`, not `--force`. Never pass `-f` to `create`.
5. Do not add a `.git` folder to scratch repos: `agm prompt` runs `git pull` when the cwd has `.git` (`src-tauri/src/bin/agm.rs:3103`).

### 4.3 Run id, scratch repos, and the marker string

| Item | Value |
|---|---|
| Run id | `<yyyyMMdd-HHmmss>-<rand4>` |
| Run folder (evidence) | `.ai-memory/temp-agents/140-per-instance-e2e-runs/<runid>/` (gitignored; `.gitignore` covers `.ai-memory/temp-agents/`) |
| Scratch root | `%TEMP%\agm-e2e140-<runid>\` |
| Scratch repos | `repo-a` (A only), `repo-b` (B only), `repo-shared` (A and B), `repo-gone` (renamed away in E2E-16); each holds one `README.md` with the run id |
| Marker | `E2E140-<runid>-<instance_id>-<case>` where `<case>` is the E2E ID plus a step letter, for example `E2E140-20261006-101500-4821-test-cli-flow-a-4821-3307-E2E04a` |
| Shared-text marker (B19 reproduction only) | `E2E140-<runid>-shared-<case>`; the instance is then known only from the conversation id found in that instance's summaries DB |

Every test prompt has this shape, so it is cheap, harmless, and long enough to be observed while running:

```text
<marker> This is an automated isolation test. Do not edit, create, or delete any file. List the names of the files in this folder, then count from 1 to 40 in words, one per line, then reply DONE <marker>.
```

The marker appears twice so it is found whether a store keeps the head or the tail of the text.

### 4.4 Where each fingerprint can be found

| Store | Path | Instance filter |
|---|---|---|
| `repo_prompts.db` (`active_prompts`, `running_projects`, `agm_conversation_sequences`) | `%USERPROFILE%\.antigravity_tools\repo_prompts.db` | `instance_id` column |
| Backup DB (`prompt_backups`, `backup_batches`) | `%USERPROFILE%\.antigravity_tools\backup-prompts\backup-prompts.db`, or `SQL.db` in the same folder when it exists, or `data\backup-prompts\` relative to the cwd of the `agm` process when that folder exists (`src-tauri/src/modules/backup_prompts_db.rs:93`) | `instance_id` column |
| Named instance conversations | `<instances>/<id>/home/.gemini/{antigravity,antigravity-ide,antigravity-cli}/conversation_summaries.db` and `.../brain/<cid>/.system_generated/logs/transcript.jsonl` | the folder itself (the summaries table has no instance column) |
| Default instance conversations | `%USERPROFILE%\.gemini\{antigravity,antigravity-ide,antigravity-cli}\` plus `<instances>/default/home/.gemini/...` if present | the folder itself |
| Resume hand-off | `<repo>/.antigravity_resume_task.json` (legacy) and, after subtask 01, `<repo>/.antigravity_resume_task.<instance_id>.json` | `instance_id` key in the JSON |
| Instance PIDs | `agm instances ls --json` (`config.id`, `is_running`, `pid`), `agm observe <id> --json` (`pids`, `data_dir`) | instance id |

`<instances>` is `%USERPROFILE%\.antigravity_tools\instances`.

### 4.5 Evidence queries

All reads open SQLite read-only with the URI form `file:<path>?mode=ro` (D10). Parameters are named: `:inst` (canonical instance id), `:other` (the other test instance), `:marker`, `:cid`, `:batch`, `:since` (unix seconds at case start). The runbook helper `Invoke-RoSql` (subtask 05) runs them through Python's `sqlite3` with `uri=True`.

Q01, all rows of one instance in `active_prompts` (today's columns):

```sql
SELECT id, instance_id, project_id, repo_path, session_id, status, created_at, updated_at
FROM active_prompts
WHERE instance_id = :inst
ORDER BY created_at;
```

Q01N, same after subtask 01 adds the new columns:

```sql
SELECT id, instance_id, repo_path, session_id, status, status_reason, source_dir, attempts, created_at, updated_at
FROM active_prompts
WHERE instance_id = :inst
ORDER BY created_at;
```

Q02, rows of one instance that carry the marker:

```sql
SELECT id, instance_id, session_id, status, created_at, updated_at
FROM active_prompts
WHERE instance_id = :inst AND instr(prompt_content, :marker) > 0;
```

Q03, leak check: the marker must not appear under any other instance id:

```sql
SELECT id, instance_id, status
FROM active_prompts
WHERE instance_id <> :inst AND instr(prompt_content, :marker) > 0;
```

Q04, running projects of one instance:

```sql
SELECT id, instance_id, repo_path, workspace_storage_path, is_running, last_detected_at, updated_at
FROM running_projects
WHERE instance_id = :inst
ORDER BY id;
```

Q05, composite id rule (AC-21), expected `bad = 0`:

```sql
SELECT COUNT(*) AS bad
FROM running_projects
WHERE instance_id = :inst
  AND substr(id, -length('__' || :inst)) <> '__' || :inst;
```

Q06, conversation sequence rows of one instance for one conversation:

```sql
SELECT conversation_id, seq_id, instance_id, project_key, updated_at
FROM agm_conversation_sequences
WHERE instance_id = :inst AND conversation_id = :cid;
```

Q06X, the same conversation across the two clone instances (AC-07), expected 2 rows after subtask 01:

```sql
SELECT conversation_id, seq_id, instance_id
FROM agm_conversation_sequences
WHERE conversation_id = :cid AND instance_id IN (:inst, :other);
```

Q07, backup rows of one instance (backup DB):

```sql
SELECT id, backup_batch_id, prompt_id, instance_id, project_path, conversation_id, status, is_restored, restored_at, created_at
FROM prompt_backups
WHERE instance_id = :inst
ORDER BY created_at, id;
```

Q08, rows of a backup batch that belong to another instance (AC-10), expected `foreign_rows = 0` and `own_rows >= 1`:

```sql
SELECT
  SUM(CASE WHEN instance_id = :inst THEN 1 ELSE 0 END) AS own_rows,
  SUM(CASE WHEN instance_id IS NULL OR instance_id <> :inst THEN 1 ELSE 0 END) AS foreign_rows
FROM prompt_backups
WHERE backup_batch_id = :batch;
```

Q09, the marker in one instance's summaries DB (run once per flavor file; the file path is the instance filter):

```sql
SELECT conversation_id, title, status, not_fully_idle, workspace_uris, last_modified_time
FROM conversation_summaries
WHERE instr(coalesce(preview, ''), :marker) > 0 OR instr(coalesce(title, ''), :marker) > 0;
```

When Q09 returns nothing, the runbook searches `brain/*/.system_generated/logs/transcript.jsonl` under the same flavor folder with a plain string `Contains` check (never `Select-String`).

Q10, conversation id present in a summaries DB (used against the other instance's and the global store to prove absence):

```sql
SELECT COUNT(*) AS n FROM conversation_summaries WHERE conversation_id = :cid;
```

Q11, legacy keys (AC-05; also the G-LEGACY gate in section 4.7), expected 0 after migration:

```sql
SELECT COUNT(*) AS n
FROM active_prompts
WHERE (instance_id IS NULL OR instance_id IN ('', '__default__', 'all'))
  AND status <> 'orphaned';
```

Q11B, same on the backup DB:

```sql
SELECT COUNT(*) AS n
FROM prompt_backups
WHERE (instance_id IS NULL OR instance_id IN ('', '__default__', 'all'))
  AND is_restored = 0;
```

Q12, captured row id format (AC-06), expected 0 after subtask 01:

```sql
SELECT COUNT(*) AS bad
FROM active_prompts
WHERE instance_id = :inst AND id LIKE 'prompt-%'
  AND id <> 'prompt-' || instance_id || '-' || session_id;
```

Q13, queue view of one instance (after subtask 01):

```sql
SELECT id, instance_id, repo_path, session_id, status, status_reason, source_dir, attempts, updated_at
FROM active_prompts
WHERE instance_id = :inst
  AND status IN ('queued', 'dispatching', 'dispatched', 'failed')
  AND instr(prompt_content, :marker) > 0;
```

Q14, non-test guard: rows of instances outside this run that changed during a case window (any row is a finding):

```sql
SELECT id, instance_id, status, updated_at
FROM active_prompts
WHERE updated_at >= :since
  AND instance_id NOT IN (:inst, :other)
ORDER BY updated_at;
```

Q15, backed-up rows of one instance (AC-14, AC-23):

```sql
SELECT id, status, session_id, updated_at
FROM active_prompts
WHERE instance_id = :inst AND status = 'backed_up'
ORDER BY id;
```

Q16, running rows of one instance (AC-16):

```sql
SELECT id, status, updated_at
FROM active_prompts
WHERE instance_id = :inst AND status = 'running'
ORDER BY id;
```

Q16N adds `attempts` to Q16 after subtask 01.

Q17, cross-instance dispatch check (AC-24):

```sql
SELECT COUNT(*) AS n
FROM active_prompts
WHERE instance_id = :other AND session_id = :cid;
```

Q14 needs care: the running AGM GUI (`agm-alim`) and the user may write default rows during a case. A Q14 row is a FAIL only when its `updated_at` falls inside the window of a scoped command this run issued and its status moved to `backed_up`, `dispatching`, or `dispatched`; otherwise it is recorded as background noise.

### 4.6 Identity at every step

| Step | How the AI knows the instance | Proof query |
|---|---|---|
| Instance created | `id` from `agm instances create ... --json` | `agm instances ls --json` shows `config.id` |
| Instance launched | `agm observe <id> --json` `pids`; the PID command line contains `instances\<id>\data` and not `%APPDATA%\Antigravity` | Win32_Process command line |
| Prompt sent | marker contains `<instance_id>`; sent with `-i <id>` from the repo folder | Q02 on `:inst`, Q03 empty |
| Conversation created | marker found in that instance's own summaries DB or transcript | Q09 on `<instances>/<id>/home/...`; Q10 on the other instance and global store returns 0 |
| Captured as running | `active_prompts.instance_id = <id>`, row id `prompt-<id>-<cid>` | Q02, Q12 |
| Backed up | `prompt_backups.instance_id = <id>` in a batch of that instance only | Q07, Q08 |
| Queued | `queued-<id>-<uuid>` row | Q13 |
| Hand-off written | JSON `instance_id = <id>`, `session_id = conversation_id = <cid>` | file read |
| Restored and dispatched | `attempts <= 1`, `source_dir = backup_restore`, marker in the same instance's conversation | Q01N, Q09 |

### 4.7 Run gates

| Gate | Rule | Effect when it fails |
|---|---|---|
| G-SAFETY | Protected snapshot from E2E-00 unchanged | Abort the run (section 5, E2E-00) |
| G-FEASIBLE | E2E-01 passed | Stop; report E2E-01 FAIL; mark every other case BLOCKED |
| G-BUILD-01, G-BUILD-02, G-BUILD-03 | The `agm` build contains subtask 01, 02, or 03. Detected only inside sandbox mode so the live databases are not migrated by the probe: 01 when `agm prompts --json` run with `ABV_DATA_DIR = sandbox-full` leaves the `source_dir`, `status_reason`, `attempts` columns and the `agm_schema_migrations` table in the copy; 02 when `agm rrp -i test-cli-flow-1743 --json` run with `ABV_DATA_DIR = sandbox-data` (it acts on the sanitized copy only) prints the `RestoreReport` keys (`restored_from_backup`, `dispatched`, `failed`, `skipped`); 03 when `agm prompts trace --help` prints the trace usage | Cases marked "after" for that subtask are BLOCKED, not FAIL |
| G-LEGACY | Q11 and Q11B both return 0 on the live DBs | Today-baseline steps that call `agm brp` are skipped (BLOCKED, reason `legacy_rows_present`), because today's backup pulls empty-instance rows into the test instance (B18) |
| G-AGY | `agy` resolves (AGM finds the Antigravity CLI executable) and the test instance has a bound account | Prompt-sending steps that use `agm prompt -i` are BLOCKED; IDE typing by a human is the fallback |

### 4.8 Sandbox mode

`account::get_data_dir` honors the process environment variable `ABV_DATA_DIR` first (`src-tauri/src/modules/account.rs:836`), and the instances dir, `repo_prompts.db`, and the backup DB all live under that data dir. Setting `ABV_DATA_DIR` for one `agm` process makes it read and write a copy instead of the live files. The setting is process-local (`persist_clean_env` only calls `std::env::set_var`).

Sandbox mode is used for:

1. E2E-14 (migration on a copy, never the live files).
2. Today-baseline runs of commands that dispatch (`rrp`, `rrc`, `qs`) or that would act on default without `-i` (E2E-16 baseline, E2E-17).

Two sandboxes exist, both built from copies of `repo_prompts.db`, the backup DB, and `instances/instances.json` (plus `instances.db`):

| Sandbox | Folder | Content | Allowed commands |
|---|---|---|---|
| Sanitized | `<runfolder>/sandbox-data/` | every `active_prompts`, `running_projects`, `agm_conversation_sequences`, and `prompt_backups` row whose `instance_id` is not one of this run's test ids is deleted from the copy, so a dispatch inside it can only reach the scratch repos; seeded rows use ids that start with `e2e140-<runid>-` | any command in this spec, including `qs`, `rrp`, `rrc` |
| Full | `<runfolder>/sandbox-full/` | unsanitized copy, used only to measure migration on real data | read commands only (`agm prompts --json`); never a command that dispatches |

Deleting rows from a copy is not a write to the live databases. The live files are only ever opened read-only, except for the narrow row cleanup in E2E-19.

## 5. E2E test catalog

Run mode values:

| Value | Meaning |
|---|---|
| today | Safe on today's build; the result is final |
| today-baseline | Safe on today's build; today's expected result shows the bug, the same steps give the final verdict after the named subtasks ship |
| sandbox-baseline | Today only inside sandbox mode (section 4.8) |
| after-01, after-02, after-03 | Live run only when that subtask has shipped (gate G-BUILD) |
| unit-only | Never run live; covered by the tests in section 7 |

Common to every case:

1. Before: record `:since` (unix seconds) and re-check G-SAFETY.
2. After: re-check G-SAFETY, run Q14 for the window, write `<runfolder>/<ID>.json` (format in subtask 05).
3. Bound every wait: launch 90 s, conversation appears 180 s, running observed 120 s, idle observed 150 s, any `agm` call 120 s (`qs --once`, `rrp`, `brp` 300 s).
4. Pass/fail is decided only by the listed evidence. "Watched the reinject" may be claimed only if a human or a screenshot saw it.

### E2E-00 Safety snapshot

| Field | Value |
|---|---|
| AC | AC-32 |
| Bugs | none (guard) |
| Run mode | today |
| Preconditions | `agm` resolves on PATH or at `src-tauri/target/debug/agm.exe`; Python 3 with `sqlite3` |

Steps:

1. Record main processes: every `Cursor.exe`, `agm-alim.exe`, `Antigravity.exe` whose command line has no `--type=` (PID, name, start time, command line with emails masked).
2. `agm instances ls --json`: record `config.id`, `is_running`, `pid` for `default` (the entry with `is_default` or id `default`) and for `cli-switch-proof-6857`, `test-cli-flow-1743`, `clone-from-default-4424`, `new-empty-instance-4425`.
3. Classify every `Antigravity.exe` main process: command line contains `%APPDATA%\Antigravity` or lacks `--user-data-dir` means default (protected); contains `instances\<protected id>\` means protected sandbox.
4. `agm auto-switch status --json`: record enabled state only (read-only). If enabled, mark the run `noisy_autoswitch = true`; do not disable it.
5. Record counts by status for non-test instances (`SELECT instance_id, status, COUNT(*) FROM active_prompts GROUP BY instance_id, status` read-only) as the baseline for Q14.
6. Run gates G-LEGACY and G-AGY and store their results.

Expected: snapshot written to `<runfolder>/E2E-00.json`.

Evidence: process list, instance list (emails masked), gate results.

Pass/fail: PASS when the snapshot is complete. Abort rules for the whole run: a protected PID is gone or its start time changed; the default instance PID changed; a protected sandbox PID changed; any command would need a banned command from section 3. On abort: stop only this run's test instances (`agm instances stop <id>`), write the report with `ABORTED`, tell the user. Never restart or kill anything protected.

Cleanup: none.

### E2E-01 Feasibility gate: Antigravity honors the per-instance profile

| Field | Value |
|---|---|
| AC | AC-01 |
| Bugs | none (proves D9) |
| Run mode | today |
| Preconditions | E2E-00 PASS; G-AGY for path 1; a human or UI automation for path 2 |

Steps:

1. Create A: `agm instances create test-cli-flow-a-<rand4> --data-only --json`. Record `id`, `data_dir`, `bound_email` (masked). If `bound_email` is null, path 1 is BLOCKED.
2. Bind and launch: `agm instances assign <A> <scratch>\repo-a`, then `agm instances launch <A>`. Poll `agm observe <A> --json` every 3 s up to 90 s until `is_running` is true.
3. Check law L2: the PID command line contains `instances\<A>\data` and does not contain `%APPDATA%\Antigravity`.
4. Path 1 (CLI flavor): from `<scratch>\repo-a`, run `agm prompt -i <A> "<marker E2E01a>"`. Poll every 5 s up to 180 s: Q09 on `<instances>/<A>/home/.gemini/antigravity-cli/conversation_summaries.db` (and the other two flavors), then the transcript search.
5. Path 2 (IDE flavor): a human or UI automation types `<marker E2E01b>` into the agent panel of A's IDE window (repo-a open). Poll the same way on `antigravity` and `antigravity-ide` flavors.
6. For every conversation id found, run Q10 on the three global stores `%USERPROFILE%\.gemini\<flavor>\conversation_summaries.db` and search the global `brain/<cid>` folder.

Expected: the marker is found only under `<instances>/<A>/home/.gemini/`; its conversation id is absent from the global store.

Evidence: create JSON (masked), observe JSON, PID command line, Q09 rows with the file path, Q10 counts on the global store, transcript file path.

Pass/fail: PASS when at least one path finds the marker only in A's folder and Q10 on the global store is 0 for every found id. FAIL when the marker shows up in the global store or in no store at all within 180 s. When only path 1 passes, report `PASS (cli flavor), IDE flavor BLOCKED` and continue; IDE-only cases (E2E-09) keep their own gate. If FAIL: stop the run, report, mark all other cases BLOCKED. Nothing else is trusted.

Cleanup: keep A for later cases.

### E2E-02 Canonical key resolution

| Field | Value |
|---|---|
| AC | AC-02, AC-03, AC-04 |
| Bugs | B01, B02, B03 |
| Run mode | today-baseline; final verdict after-01 and after-03 |
| Preconditions | E2E-01 PASS; B created: `agm instances create test-cli-flow-b-<rand4> --data-only --json` (not launched yet) |

Steps:

1. Today (read-only): `agm observe default --json`, `agm observe __default__ --json`; record `instance_id` from each.
2. Today: `agm observe nope-0000 --json`; record exit code and stderr; check `<instances>\nope-0000` does not exist.
3. Today: `agm observe <rand4> --json` (suffix shared by A and B names); record exit code and which instance it chose.
4. After-03: `agm prompts -i default --json` and `agm prompts -i __default__ --json`; compare the row sets.
5. After-03: NEW `agm prompt enqueue -i nope-0000 --repo <scratch>\repo-a "<marker E2E02d>"`; record exit code, stderr, and folder absence.
6. After-03: `agm brp -i <rand4> --json`; record exit code and stderr.

Expected after the fix: steps 1 and 4 give the same default id and identical rows; step 5 exits non-zero with `unknown instance id 'nope-0000'; known: ...` and no `nope-0000` folder; step 6 exits non-zero with `ambiguous instance '<rand4>': candidates <A>, <B>` and changes nothing (Q07 for A and B unchanged).

Expected today: step 3 exits 0 and picks one instance (B02). Step 1 may disagree (B01). Recorded as baseline, not as a run failure.

Evidence: exit codes, stderr, `Test-Path` results, the two JSON row sets, Q07 before and after step 6.

Pass/fail: after-fix PASS only when all three ACs hold. Today: the case result is `BASELINE` with the observed behavior.

Cleanup: none.

### E2E-03 Running detection, single instance, idle versus running, first-row rule

| Field | Value |
|---|---|
| AC | AC-19, AC-20 |
| Bugs | B05, B11 |
| Run mode | today-baseline; final verdict after-02 (and after-03 for `wpr -i`) |
| Preconditions | E2E-01 PASS; A running with repo-a; A's summaries DB has exactly the conversation(s) from E2E-01 |

Steps:

1. Wait until every A conversation is idle: Q09 rows show `not_fully_idle = 0` and `last_modified_time` older than 150 s.
2. List: after-03 `agm wpr -i <A> --json`; today `agm wpr --json` and keep only objects whose `instance_id` equals `<A>`.
3. Today-baseline only when G-LEGACY passes: `agm brp -i <A> --json`; then Q07 for A and Q02 with the E2E-01 marker.
4. Send a running prompt: from repo-a, `agm prompt -i <A> "<marker E2E03a>"`. Poll `wpr` (as in step 2) every 2 s up to 120 s until a row for that conversation shows running; record the time.
5. Wait until Q09 shows the conversation idle for 150 s. Re-run step 2.
6. `agm instances stop <A>`. Poll `agm observe <A> --json` until `is_running` is false (60 s). Re-run step 2. Then `agm instances launch <A>` again for later cases.

Expected after the fix: step 2 and step 5 show zero running rows for A (the first and only conversation is idle, AC-19); step 3 captures nothing; step 4 shows the row running; step 6 shows zero running rows (AC-20).

Expected today: step 3 backs up the idle first conversation (B05).

Evidence: Q09 idle fields, `wpr` JSON excerpts with timestamps, Q07 and Q02 rows, observe JSON before and after stop.

Pass/fail: after-fix PASS when steps 2, 5, and 6 return no running row for A and step 4 returns one. Step 4 not observed within 120 s is FAIL only if Q09 shows `not_fully_idle > 0` during that window (otherwise BLOCKED: prompt too short).

Cleanup: A relaunched.

### E2E-04 Running detection, two instances, different repos

| Field | Value |
|---|---|
| AC | AC-21, AC-22, AC-27, AC-31, AC-06 |
| Bugs | B11, B12, B14, B26 |
| Run mode | today-baseline; final verdict after-01, after-02, after-03 |
| Preconditions | A running with repo-a; B: `agm instances assign <B> <scratch>\repo-b`, `agm instances launch <B>`, L2 check as in E2E-01 |

Steps:

1. From repo-a: `agm prompt -i <A> "<marker E2E04a>"`. From repo-b: `agm prompt -i <B> "<marker E2E04b>"`.
2. While both run (poll up to 120 s): `agm wpr --json` (no `-i`). After-03 also `agm wpr -i <A> --json`, `agm wpr -i <B> --json`, `agm wpr -i default --json`.
3. Q02 and Q03 for each marker with `:inst` = its instance. Q09 on A's and B's summaries DBs for both markers.
4. Composite ids: Q05 for A and B. Record Q04 row counts.
5. Refresh by the other discovery path: `agm running-projects --json` (after-03 `agm running-projects -i <A> --json`), then (G-LEGACY) `agm brp -i <A> --json`. Re-run Q04 and Q05 for A.
6. Tree: after-03 `agm tree -i <A> --json` and `agm tree -i <B> --json`; today `agm tree all --json` filtered by instance in the runbook.
7. After-01: Q12 for A and B.

Expected after the fix: every `wpr` object has `instance_id` and `instance_name`, grouped by instance (AC-27); `wpr -i <A>` has only A's rows; `wpr -i default` has no row whose `instance_id` is not the default id (AC-22); Q03 empty for both markers; marker A only in A's summaries, marker B only in B's; Q05 `bad = 0` and the Q04 count does not drop after step 5 (AC-21); tree for A has only A's projects (AC-31); Q12 `bad = 0` (AC-06).

Expected today: step 5 may delete rows written by `agm prompt` (B11); `wpr` rows may lack instance fields (B26).

Evidence: all JSON excerpts, Q02 to Q05, Q09, Q12 rows.

Pass/fail: after-fix PASS when every expected item holds. Any cross-instance row is FAIL.

Cleanup: none.

### E2E-05 Two instances, same repo (resume hand-off collision)

| Field | Value |
|---|---|
| AC | AC-11, AC-12, AC-13 |
| Bugs | B07, B33, B25 |
| Run mode | today-baseline; final verdict after-01 and after-03 |
| Preconditions | A and B running; both assigned `<scratch>\repo-shared` (`agm instances assign <A> <scratch>\repo-shared`, same for B; relaunch each only if the workspace does not open; a human may open the folder with File, Open Folder in each window) |

Steps:

1. Today-baseline: from repo-shared, `agm prompt -i <A> "<marker E2E05a>"`; read `repo-shared\.antigravity_resume_task.json` (copy it to the run folder). Then `agm prompt -i <B> "<marker E2E05b>"`; read the file again.
2. Find A's conversation id in repo-shared: Q09 on A's summaries for marker E2E05a, giving `<cidA>`; same for B, giving `<cidB>`.
3. After-03: NEW `agm prompt enqueue -i <A> --repo <scratch>\repo-shared --conversation <cidA> "<marker E2E05c>" --json` and NEW `agm prompt enqueue -i <B> --repo <scratch>\repo-shared --conversation <cidB> "<marker E2E05d>" --json`.
4. After-03: NEW `agm prompts queue ls -i <A> --json` and `-i <B> --json`; Q13 for each marker.
5. After-03: record `LastWriteTime` of `repo-shared\.antigravity_resume_task.json` (or its absence), then `agm qs <A> --once` (after-03 also accepts `agm qs -i <A> --once`) and `agm qs <B> --once`.
6. Read `repo-shared\.antigravity_resume_task.<A>.json` and `repo-shared\.antigravity_resume_task.<B>.json` (or, when the reader research in architecture spec section 6.4.1 required the legacy name, the DB rows from Q13). Re-check the legacy file `LastWriteTime`.
7. After-03: NEW `agm prompts trace -i <A> --conversation <cidA> --json`; read `handoff`.

Expected after the fix: both queued rows exist, each with its own `instance_id` (AC-11); each hand-off has `instance_id`, `session_id`, `conversation_id` with `session_id = conversation_id` equal to `<cidA>` or `<cidB>` (AC-12); the legacy file was not written in step 5 and `handoff.legacy = "skipped_multi_owner"` (AC-13).

Expected today: after step 1 the legacy file holds only B's `instance_id` and `conversation_id` is the repo slug, not a conversation id (B07, B25).

Evidence: both copies of the legacy file from step 1, queue JSON, Q13 rows, per-instance files, `LastWriteTime` values, trace JSON.

Pass/fail: after-fix PASS when AC-11, AC-12, and AC-13 hold.

Cleanup: none (files stay in the scratch repo and are removed with it in E2E-19).

### E2E-06 Same conversation id in two instances via clone

| Field | Value |
|---|---|
| AC | AC-06, AC-07 |
| Bugs | B32, B06 |
| Run mode | today-baseline; final verdict after-01 |
| Preconditions | A has at least one conversation `<cidX>` (from E2E-01); A stopped for the copy (`agm instances stop <A>`) so files are not locked |

Steps:

1. Create C: `agm instances create test-cli-flow-c-<rand4> --from <A> --data-only --json`. Record C's id. Relaunch A.
2. Q10 with `:cid = <cidX>` on C's summaries DBs. If 0 in every flavor, the live part is BLOCKED (`clone_does_not_copy_conversations`) and AC-07 rests on the integration test `same_conversation_id_in_two_instances_yields_two_rows` (section 7).
3. Launch C (`agm instances launch <C>`), L2 check.
4. After-03: `agm tree -i <A> --json` and `agm tree -i <C> --json`; today `agm tree all --json`.
5. Q06X with `:inst = <A>`, `:other = <C>`, `:cid = <cidX>`.
6. After-01: when `<cidX>` is running in both (only possible if a human resumes it in both windows), `agm brp -i <A> --json` and `agm brp -i <C> --json`, then `SELECT instance_id, id FROM active_prompts WHERE session_id = :cid AND instance_id IN (:inst, :other)`.

Expected after the fix: Q06X returns 2 rows, one per instance, each with its own `seq_id`; step 6, when run, returns `prompt-<A>-<cidX>` and `prompt-<C>-<cidX>` (AC-06, AC-07).

Expected today: Q06X returns at most 1 row (primary key is `conversation_id` alone).

Evidence: C create JSON, Q10 on C, tree JSON, Q06X rows, step 6 rows or the reason it was skipped.

Pass/fail: after-fix PASS when Q06X has 2 rows; step 6 is optional and its absence is not a failure.

Cleanup: stop C (`agm instances stop <C>`); C is deleted in E2E-19.

### E2E-07 Backup isolation and idempotence, no steal

| Field | Value |
|---|---|
| AC | AC-09, AC-10 |
| Bugs | B06, B18, B19, B20 |
| Run mode | today-baseline (requires G-LEGACY); final verdict after-01 and after-02 |
| Preconditions | A and B running, both on repo-shared |

Steps:

1. Variant 07a (distinct text): from repo-shared, `agm prompt -i <A> "<marker E2E07a>"` and `agm prompt -i <B> "<marker E2E07b>"`. Poll until both run.
2. `agm brp -i <A> --json`; save the batch id `<batchA>`; Q07 for A (snapshot S1) and Q08 with `<batchA>`.
3. `agm brp -i <B> --json`; save `<batchB>`; Q07 for A (snapshot S2); Q08 with `<batchB>`.
4. `agm brp -i <A> --json` again; Q07 for A (snapshot S3).
5. Variant 07b (shared text, reproduces B19): wait until both idle; send the same text `<shared marker E2E07c>` to A and B from repo-shared; find `<cidA>` and `<cidB>` via Q09 on each instance's summaries; `agm brp -i <A> --json`, then `agm brp -i <B> --json`; read `SELECT id, instance_id, conversation_id FROM prompt_backups WHERE instr(prompt_text, :marker) > 0 AND instance_id IN (:inst, :other)`.

Expected after the fix: S1 equals S2 for every A row (AC-09); Q08 for `<batchA>` returns only A and for `<batchB>` only B (AC-10); S3 has no duplicate unrestored row for the same `prompt_id` (idempotent); in 07b there are two rows, one with `instance_id = <A>` and `conversation_id = <cidA>`, one with B and `<cidB>`.

Expected today: in 07b the A row is re-stamped to B (B19).

Evidence: brp JSON, S1, S2, S3, Q08 rows, 07b rows.

Pass/fail: after-fix PASS when all expected items hold. Skipped steps due to G-LEGACY give BLOCKED, not FAIL.

Cleanup: none.

### E2E-08 Enqueue via CLI to one instance, dispatch only there

| Field | Value |
|---|---|
| AC | AC-28, AC-14, AC-15 |
| Bugs | B13, B25 |
| Run mode | after-03 (needs after-01 columns); sandbox-baseline today |
| Preconditions | A and B running; A idle in repo-a; at least one `backed_up` row for A exists (from E2E-07 or E2E-03) |

Steps:

1. Q15 for A (snapshot before). Q01N for B (snapshot before).
2. NEW `agm prompt enqueue -i <A> --repo <scratch>\repo-a "<marker E2E08a>" --json`.
3. Q13 for the marker: exactly one row, `id` starts with `queued-<A>-`, `status = queued`, `repo_path` equals repo-a normalized, `source_dir = cli` (AC-28).
4. `agm qs <A> --once` (300 s bound).
5. Q13 again: `status = dispatched`, `attempts = 1`. Q15 for A: unchanged (AC-14). Q01N for B: unchanged.
6. Poll Q09 on A's summaries (180 s) for the marker; Q09 on B's summaries and on the global store must stay empty.
7. Sandbox-baseline today: in the sandbox copy, insert one `queued` row and one `backed_up` row for A (ids `e2e140-<runid>-q1`, `e2e140-<runid>-b1`, repo-a), run `agm qs <A> --once` with `ABV_DATA_DIR` set to the sandbox, and read both rows from the sandbox.

Expected after the fix: steps 3, 5, and 6 as written; `dispatched` appears only because the send succeeded (AC-15 positive half).

Expected today (step 7): the `backed_up` row is consumed and marked `dispatched` (B13).

Evidence: enqueue JSON, Q13 before and after, Q15 and Q01N snapshots, Q09 hits.

Pass/fail: after-fix PASS when one row is written, dispatched once, only A's conversations receive the marker, and A's `backed_up` rows plus all B rows are unchanged.

Cleanup: none.

### E2E-09 Enqueue via UI uses the same function and row shape

| Field | Value |
|---|---|
| AC | AC-29 |
| Bugs | B28, B07 |
| Run mode | after-03; the running AGM GUI must be a build that contains subtask 03 (never launch a replacement GUI; if it is older, BLOCKED) |
| Preconditions | E2E-08 PASS (gives the CLI row shape to compare); A running with repo-a |

Steps:

1. Record `LastWriteTime` (or absence) of `repo-a\.antigravity_resume_task.json` and `repo-a\.antigravity_resume_task.<A>.json`.
2. Human or UI automation tool (this step cannot be done by a CLI-only agent): in the running AGM window, open Instances, open instance A's row, open the prompt tree (`PromptTreeViewModal`), select project repo-a, type `<marker E2E09a>` in the enqueue box, click Enqueue. Do not click the Google data-collection checkbox.
3. NEW `agm prompts queue ls -i <A> --json`; Q13 for the marker.
4. Compare the UI row with the CLI row from E2E-08: same columns non-null, `id` prefix `queued-<A>-`, `status = queued`, `instance_id = <A>`, `source_dir = ui` (the only allowed difference besides text and time).
5. Re-check the two file timestamps from step 1.

Expected: one queued row with the CLI row shape and `source_dir = ui`; no resume file written or changed by the UI action.

Evidence: screenshot from the human or tool if available (path recorded), queue JSON, Q13 row, both CLI and UI rows side by side, timestamps.

Pass/fail: PASS when the row exists with the same shape and both timestamps are unchanged. FAIL when the row is missing (the IPC still falls back to the file) or a file was written. BLOCKED when no human or UI tool is available.

Cleanup: none (the row is dispatched or deleted in later cases or E2E-19).

### E2E-10 Queue versus backup separation

| Field | Value |
|---|---|
| AC | AC-14, AC-16 |
| Bugs | B13, B16, B27 |
| Run mode | after-02 and after-03; sandbox-baseline today |
| Preconditions | A running; A has at least one `backed_up` row; A currently has a running prompt (send `<marker E2E10a>` from repo-a and wait until it shows running) |

Steps:

1. NEW `agm prompt enqueue -i <A> --repo <scratch>\repo-a "<marker E2E10b>" --json`.
2. Q15 (backed_up) and Q16N (running, with `attempts`) for A: snapshot T1.
3. `agm qs <A> --once`. Q15 again: unchanged. Q13 for E2E10b: `dispatched` or, while the running prompt still owns the project, still `queued` (the scheduler dispatches only idle projects).
4. `agm rrc -i <A> --json`. Q16N again.
5. Sandbox-baseline today: in the sanitized sandbox, seed one `running` row (`e2e140-<runid>-r1`), one `backed_up` row (`e2e140-<runid>-b2`), and one `queued` row (`e2e140-<runid>-q2`) for A, all on repo-a. Today's `rrc` re-sends rows of every instance (B27), which is why it runs only in the sandbox, where only test rows exist. With `ABV_DATA_DIR` set, run `agm qs <A> --once`, then `agm rrc --json`, and read the three rows after each command.

Expected after the fix: Q15 unchanged after step 3 (AC-14); every running row has the same `updated_at` and `attempts` after step 4 (AC-16); `rrc` touched only `backed_up` and `failed` rows of A.

Expected today (step 5): the `running` row is re-sent (B16).

Evidence: T1, Q15 and Q16N after each step, Q13, rrc JSON.

Pass/fail: after-fix PASS when AC-14 and AC-16 hold.

Cleanup: none.

### E2E-11 Instance account switch on A: backup, close only A, relaunch, restore the same conversation, B untouched

| Field | Value |
|---|---|
| AC | AC-17, AC-18, AC-32, AC-12, AC-25 |
| Bugs | B15, B16, B29, B30, B31, B23 |
| Run mode | after-02 and after-03 (live switch today runs the triple restore that may dispatch rows of other instances, B29 and B30) |
| Preconditions | A and B running; A has a running prompt `<marker E2E11a>` in repo-a with conversation `<cidA>` (Q09); a second account id that is bound to no instance, read from `agm accounts --json` (keep only `id` and a masked email; never write the raw JSON to disk) |

Steps:

1. Snapshot: protected PIDs, A PID, B PID; Q01N for A and B; Q07 for A and B.
2. `agm instances switch <A> <account-id> --json` (300 s bound).
3. Observe: A's PID changed (closed and relaunched by the switch; if the switch does not relaunch, run `agm instances launch <A>`); B PID unchanged; every protected PID unchanged.
4. Q07 for A: a new batch with only A rows; Q07 for B unchanged.
5. If the switch did not restore by itself, run `agm rrp -i <A> --json` once.
6. Q01N for A: every row with `source_dir = backup_restore` has `attempts <= 1` (AC-17). Q01N for B: identical to step 1 (AC-18).
7. Hand-off: read `repo-a\.antigravity_resume_task.<A>.json` (or trace `handoff`): `session_id = conversation_id = <cidA>` (AC-12).
8. NEW `agm prompts trace -i <A> --conversation <cidA> --json`: one dispatch for the restored row.
9. `agm history --json`: the switch entry has `instance_id = <A>` (AC-25 for the CLI path).
10. Poll Q09 on A's summaries (180 s) for a new turn in `<cidA>` (or a new conversation carrying the marker, which is today's `-p` behavior, B15); Q09 on B and global: no marker.

Expected: steps 3 to 10 as written; only A's process was closed (AC-32).

Evidence: PID snapshots before and after, switch JSON (masked), Q07, Q01N, hand-off file, trace JSON, history entry, Q09 hits.

Pass/fail: PASS when AC-17, AC-18, AC-32, and AC-12 hold. If any protected or B PID changed: ABORT the run. A restored prompt that lands in a new conversation instead of `<cidA>` is FAIL for "same conversation" with reason `new_chat_spawn` (B15) and is reported separately from the isolation verdict.

Cleanup: switch A back only if the user asked for it; otherwise leave the test account on A (A is deleted in E2E-19).

### E2E-12 Auto-switch, cross instance

| Field | Value |
|---|---|
| AC | AC-23, AC-24 |
| Bugs | B21, B22 |
| Run mode | after-02 live for the scoped part; the cross-instance and default-target parts are covered by integration tests (section 7) |
| Preconditions | A and B running; A has `backed_up` rows; B has at least one row |

Steps:

1. Snapshot Q15 for A and B, Q01N for B, PIDs.
2. `agm instances ff <A> --json` (scoped manual rotation of A; never `agm ff` without an instance, never `agm auto-switch run`).
3. `agm history --json`: read the rotation entry; record source and target instance ids.
4. If the target instance equals A (same-instance rotation): Q15 for B unchanged; A rows were backed up and restored only under A (Q01N for A, `attempts <= 1`).
5. If the target instance is B (cross instance): for each A conversation id `<cid>` captured in step 1, Q17 with `:other = <B>` returns 0 (AC-24); A rows stay `backed_up` until A relaunches; then relaunch A (`agm instances launch <A>`) and check they are dispatched under A only.
6. If the target instance is default or a protected sandbox: ABORT (the scoped command must never act there).
7. Run the integration tests `auto_rotation_default_target_restores_only_default` and `cross_instance_rotation_restores_source_only` (section 7) and attach their result.

Expected: as in steps 4 and 5; integration tests green.

Evidence: ff JSON, history entry, Q15, Q17, Q01N, test output.

Pass/fail: PASS when the live part matches the branch taken and both integration tests pass. AC-23 is proven by the integration test only (never rotate default live).

Cleanup: none.

### E2E-13 Default instance paths

| Field | Value |
|---|---|
| AC | AC-23, AC-25, AC-02 |
| Bugs | B21, B23 |
| Run mode | unit-only (never switch, close, back up, or restore the default instance live) |
| Preconditions | subtask 02 shipped |

Steps:

1. Run the unit and integration tests in section 7 that name AC-23, AC-25, and AC-02: `auto_rotation_default_target_restores_only_default`, `desktop_switch_audits_default_instance`, `telegram_switch_audits_effective_instance`, `account_switch_default_path_keeps_default`, `canonical_default_aliases_map_to_default`.

Expected: all pass.

Evidence: test names and pass lines from the targeted `cargo test --test <file> <name>` runs.

Pass/fail: PASS when every named test passes.

Cleanup: none.

### E2E-14 Legacy empty instance_id rows migration (on a copy)

| Field | Value |
|---|---|
| AC | AC-05, AC-08 |
| Bugs | residue of B01, B18 |
| Run mode | after-01 in sandbox mode only; today: read-only counts on the live DBs (Q11, Q11B) as the baseline |
| Preconditions | E2E-00 PASS. With a build that contains subtask 01 this case runs immediately after E2E-00 and before any other `agm` command of that build opens the live databases, because the first open of the live `repo_prompts.db` by a fixed build runs the migration for real. It needs no test instance: the owners are copies of two existing sandboxes inside the sandbox folder. |

In this case `<O1>` is `test-cli-flow-1743` and `<O2>` is `cli-switch-proof-6857`. Only their copies inside the sandbox folders are touched; the live instances are never opened for writing.

Steps:

1. Build two sandboxes (section 4.8) from the same copies of `repo_prompts.db`, the backup DB, `instances\instances.json`, and `instances\instances.db`, keeping the relative layout: `<runfolder>\sandbox-full\` (unsanitized) and `<runfolder>\sandbox-data\` (sanitized). In both, copy `<O1>`'s and `<O2>`'s `home\.gemini\<flavor>\conversation_summaries.db` files into `instances\<id>\home\.gemini\<flavor>\` (create an empty summaries DB with the `conversation_summaries` schema when the live one is missing). Never open the live files for writing.
2. Full-copy baseline: Q11 and Q11B on `sandbox-full` before any `agm` call (this equals today's live baseline).
3. Seed in `sandbox-data` only. In the copied summaries DB of `<O1>`, insert `conversation_id = e2e140-<runid>-one`. In the copied summaries DBs of `<O1>` and `<O2>`, insert `conversation_id = e2e140-<runid>-dup` (same id in two instances). In the copied `repo_prompts.db`, insert three `active_prompts` rows with `instance_id = ''`, `repo_path` = `<scratch>\repo-a`, `status = backed_up`: `e2e140-<runid>-l1` with `session_id = e2e140-<runid>-one` (single owner `<O1>`), `e2e140-<runid>-l2` with `session_id = e2e140-<runid>-nocid` (no owner), `e2e140-<runid>-l3` with `session_id = e2e140-<runid>-dup` (two owners).
4. With `ABV_DATA_DIR = <runfolder>\sandbox-data`, run `agm prompts --json` (any command that opens the DB runs the migration once).
5. Read in the sandbox: rows l1, l2, l3 (`id`, `instance_id`, `status`, `status_reason`); Q11 and Q11B; `SELECT name FROM agm_schema_migrations`; check `repo_prompts.db.pre-v140.bak` exists in the sandbox.
6. With the same `ABV_DATA_DIR`: `agm qs --once` (all instances, allowed only here because the sanitized sandbox holds nothing but the seeded rows). Re-read l2 and l3.
7. With `ABV_DATA_DIR = <runfolder>\sandbox-full`, run `agm prompts --json` once (never `qs`, `rrp`, or `rrc` there: its rows point at real repos), then Q11 and Q11B on `sandbox-full`.

Expected: l1 `instance_id = <O1>`; l2 and l3 `status = orphaned` with `legacy_owner_unknown` and `legacy_owner_ambiguous`; Q11 and Q11B 0 on both copies (AC-05); the `.pre-v140.bak` file exists; after step 6 l2 and l3 are still `orphaned` (AC-08).

Evidence: copy manifest (relative names and sizes), seeded rows, rows after migration, Q11, Q11B, migration row, `.bak` presence, qs output.

Pass/fail: PASS when every expected item holds. Any write to a live DB file (`LastWriteTime` of the live `repo_prompts.db` changed by this case's commands) is FAIL. Note that the AGM GUI may write the live file at any time; compare against the GUI's normal write pattern recorded in E2E-00 and treat a change as a finding to investigate, not an automatic FAIL.

Cleanup: delete `<runfolder>\sandbox-full\` and `<runfolder>\sandbox-data\` at the end of the run (E2E-19).

### E2E-15 Restart AGM CLI mid-flow, no duplicate resend

| Field | Value |
|---|---|
| AC | AC-16, AC-17 |
| Bugs | B13, B16, B30, B31 |
| Run mode | after-02 and after-03 |
| Preconditions | A running and idle in repo-a; two queued rows for A (NEW `agm prompt enqueue` with markers `E2E15a`, `E2E15b`) and at least one `backed_up` row |

Steps:

1. Start the continuous scheduler as a child process of this run: `agm qs <A>` (no `--once`), record its PID `<qsPid>` (an `agm.exe` started by this run, never `agm-alim.exe`).
2. Poll Q13 every 1 s up to 120 s until the first marker row becomes `dispatching` or `dispatched`.
3. Stop only `<qsPid>` (Stop-Process by that PID). Verify `<qsPid>` is gone and every protected PID is unchanged.
4. Run `agm qs <A> --once`, then `agm rrp -i <A> --json`, then `agm qs <A> --once` again.
5. Q13 for both markers: each `attempts <= 1`, final status `dispatched` or `failed` (a `dispatching` row older than 120 s must become `failed` with `dispatch_timeout`). Q16N: running rows unchanged.
6. Count marker occurrences across A's transcripts: each marker appears in exactly one conversation. Q09 on B and global: no marker.

Expected: no prompt was sent twice (AC-17 for restore, no resend of running rows AC-16).

Evidence: `<qsPid>`, Q13 timeline, Q16N, transcript counts, qs and rrp outputs.

Pass/fail: PASS when every marker has one send and running rows are unchanged.

Cleanup: none (the stopped scheduler was this run's own child).

### E2E-16 Failure path: spawn fails, row not marked dispatched

| Field | Value |
|---|---|
| AC | AC-15 |
| Bugs | B13, B25 |
| Run mode | after-02 and after-03 live; sandbox-baseline today |
| Preconditions | A running; `<scratch>\repo-gone` exists |

Steps:

1. NEW `agm prompt enqueue -i <A> --repo <scratch>\repo-gone "<marker E2E16a>" --json`.
2. Rename `<scratch>\repo-gone` to `<scratch>\repo-gone-moved`.
3. `agm qs <A> --once`.
4. Q13 for the marker: `status = failed`, `status_reason = repo_missing`, `attempts = 1`.
5. Sandbox-baseline today: in the sandbox, seed `e2e140-<runid>-f1` as `queued` for A with `repo_path` = the moved-away path, run `agm qs <A> --once` with `ABV_DATA_DIR`, read the row.

Expected after the fix: step 4 as written.

Expected today (step 5): the row becomes `dispatched` although nothing was sent (B13).

Evidence: enqueue JSON, rename result, qs output, Q13 row.

Pass/fail: PASS when the row is `failed` with a reason and never `dispatched`.

Cleanup: rename the folder back or leave it for E2E-19.

### E2E-17 Refusal: action commands without -i when two instances run

| Field | Value |
|---|---|
| AC | AC-26 |
| Bugs | B25, B27, B31 |
| Run mode | after-03, in sandbox mode only (if the refusal regressed, the command acts only on the sanitized copy) |
| Preconditions | A and B running (so the copied registry plus live PIDs report two running instances); unit test `action_commands_refuse_without_instance_when_two_running` passed |

Steps:

1. Build or refresh the sanitized sandbox. Record in the sandbox `SELECT COUNT(*), MAX(updated_at) FROM active_prompts` and `SELECT COUNT(*) FROM prompt_backups`.
2. With `ABV_DATA_DIR` set: `agm brp`, `agm rrp`, `agm rrc` (each without `-i`); record exit code and stderr.
3. Re-read the two counts.
4. After-03, also with `ABV_DATA_DIR`: `agm prompt "<marker E2E17a>"` from repo-shared (owned by both A and B); record exit code.

Expected: exit code 2 for every command with the message `instance required: 2 instances running (...); pass -i <id|#seq|name>`; counts unchanged.

Evidence: exit codes, stderr, counts before and after.

Pass/fail: PASS when every command exits 2 and nothing changed.

Cleanup: none.

### E2E-18 Trace command shows the identity chain

| Field | Value |
|---|---|
| AC | AC-30 |
| Bugs | supports verification of all |
| Run mode | after-03 |
| Preconditions | E2E-11 PASS (a prompt of A went through capture, backup, and restore) |

Steps:

1. NEW `agm prompts trace -i <A> --conversation <cidA> --json`.
2. Check keys on the restored prompt: `source_dir`, `conversation_id`, `repo_path`, `row_id`, `status`, `status_history`, `backup_row_id`, `handoff`, `dispatch_result`.
3. Cross-check values: `source_dir` is under `<instances>/<A>/home/.gemini/`; `row_id` exists in Q01N; `backup_row_id` exists in Q07 for A; `handoff.path` exists on disk.
4. NEW `agm prompts trace -i <B> --conversation <cidA> --json` returns no prompt of A.

Expected: every key present with values that match the stores.

Evidence: trace JSON, cross-check query rows.

Pass/fail: PASS when all keys are present and every cross-check matches.

Cleanup: none.

### E2E-19 Cleanup

| Field | Value |
|---|---|
| AC | AC-32 |
| Bugs | none (guard) |
| Run mode | today (always runs last, also after an abort, limited to stopping test instances when aborted) |
| Preconditions | `run.json` lists this run's instance ids |

Steps:

1. For each id in `run.json` (and only those): `agm instances stop <id>`; poll `agm observe <id> --json` until stopped (60 s).
2. Optional row cleanup (one allowed write, narrow): in a write connection to the live `repo_prompts.db` and backup DB, `DELETE FROM active_prompts WHERE instance_id IN (<run ids>)`, the same for `running_projects`, `agm_conversation_sequences`, `prompt_backups`; record row counts before and after. Never delete by prefix, by `LIKE`, or by empty id.
3. For each id in `run.json`: `agm instances rm <id> --force`.
4. Delete `%TEMP%\agm-e2e140-<runid>\`, `<runfolder>\sandbox-full\`, and `<runfolder>\sandbox-data\`. Keep every evidence JSON file.
5. Final safety check: the protected snapshot from E2E-00 matches exactly (AC-32); `agm instances ls --json` still lists `default`, `cli-switch-proof-6857`, `test-cli-flow-1743`, `clone-from-default-4424`, `new-empty-instance-4425` with the same PIDs as at start.

Expected: only this run's instances and folders are gone; protected PIDs unchanged.

Evidence: stop and rm outputs, row counts, final snapshot diff.

Pass/fail: PASS when every protected PID and instance is unchanged and no run id remains in the registry.

Cleanup: this is the cleanup.

## 6. Coverage matrix

| AC | Covered by |
|---|---|
| AC-01 | E2E-01 |
| AC-02 | E2E-02, E2E-13 |
| AC-03 | E2E-02 |
| AC-04 | E2E-02 |
| AC-05 | E2E-14 |
| AC-06 | E2E-04, E2E-06 |
| AC-07 | E2E-06 |
| AC-08 | E2E-14 |
| AC-09 | E2E-07 |
| AC-10 | E2E-07 |
| AC-11 | E2E-05 |
| AC-12 | E2E-05, E2E-11 |
| AC-13 | E2E-05 |
| AC-14 | E2E-08, E2E-10 |
| AC-15 | E2E-08, E2E-16 |
| AC-16 | E2E-10, E2E-15 |
| AC-17 | E2E-11, E2E-15 |
| AC-18 | E2E-11 |
| AC-19 | E2E-03 |
| AC-20 | E2E-03 |
| AC-21 | E2E-04 |
| AC-22 | E2E-04 |
| AC-23 | E2E-12, E2E-13 |
| AC-24 | E2E-12 |
| AC-25 | E2E-11, E2E-13 |
| AC-26 | E2E-17 |
| AC-27 | E2E-04 |
| AC-28 | E2E-08 |
| AC-29 | E2E-09 |
| AC-30 | E2E-18 |
| AC-31 | E2E-04 |
| AC-32 | E2E-00, E2E-11, E2E-19 (and the G-SAFETY check around every case) |

All 32 acceptance criteria are covered at least once.

## 7. Unit and integration tests to add in src-tauri/tests/

These cover the paths that must never run live (default instance, cross-instance rotation, migration logic, races) and give each live case a fast pre-check. Every file follows the pattern already used in `src-tauri/tests/per_instance_prompt_liveness_test.rs`: a temp directory as the data dir, seeded SQLite files, mock registry. Because `ABV_DATA_DIR` is process-wide, every test in a file that sets it holds one file-level `static` mutex for its whole body. Dispatch is never a real process: subtask 02 provides a sender seam (a function parameter or trait used by `restore_backed_up_prompts_for_instance` and the scheduler) so tests count sends instead of spawning `agy`.

| File | Test | Asserts | AC | Bugs |
|---|---|---|---|---|
| `canonical_instance_key_test.rs` | `canonical_default_aliases_map_to_default` | `""`, `default`, `__default__`, `DEFAULT` all return the default registry id | AC-02 | B01 |
| same | `canonical_unknown_id_errors_and_creates_no_folder` | unknown id returns `Err` with the known ids listed; no `instances/<id>/home` created; `get_instance_home_dir` also errors | AC-03 | B03 |
| same | `resolve_input_ambiguous_suffix_lists_candidates` | two instances whose names end with the same suffix give `Err` naming both | AC-04 | B02 |
| same | `resolve_input_unique_suffix_resolves` | a suffix owned by one instance resolves to it | AC-04 | B02 |
| same | `canonical_never_returns_active_instance` | with active instance set to a named instance, `canonical_instance_id("")` still returns default | AC-02 | B01 |
| same | `gemini_dirs_tagged_all_never_stamps_all` | `InstanceScope::All` returns owner ids only, never `all` | AC-05 | B04 |
| `prompt_identity_triple_test.rs` | `captured_row_id_includes_instance` | captured id equals `prompt-{instance_id}-{conversation_id}` | AC-06 | B06 |
| same | `same_conversation_id_in_two_instances_yields_two_rows` | the same cid in A and C copies gives two `active_prompts` rows and two `agm_conversation_sequences` rows | AC-07 | B32 |
| same | `scan_conversations_for_keeps_same_cid_per_instance` | tagged scan returns the cid once per instance | AC-07 | B32 |
| same | `path_compare_has_no_prefix_match` | `app` does not match `app-v2`; Windows case and slash differences match | AC-22 | B12 |
| `prompt_backup_isolation_test.rs` | `backup_of_b_does_not_modify_rows_of_a` | A rows identical before and after backing up B, including same text and repo | AC-09 | B19 |
| same | `backup_inserts_only_target_instance` | every row of the new batch has the target id | AC-10 | B18, B20 |
| same | `backup_is_idempotent_per_instance` | two backups of A give one unrestored row per `prompt_id` | AC-10 | B06 |
| same | `backup_skips_idle_first_row` | an idle first summaries row is not backed up | AC-19 | B05 |
| `prompt_queue_state_machine_test.rs` | `scheduler_consumes_only_queued` | `backed_up` rows untouched after one scheduler cycle | AC-14 | B13 |
| same | `failed_send_sets_failed_with_reason` | sender returns false: status `failed`, `status_reason` set, never `dispatched` | AC-15 | B13, B25 |
| same | `atomic_claim_has_single_winner` | two claimers on one row: exactly one sees `changes() == 1` | AC-17 | B13 |
| same | `stale_dispatching_becomes_failed_after_timeout` | `dispatching` older than 120 s becomes `failed` with `dispatch_timeout` | AC-15 | B13 |
| same | `resend_skips_running_rows` | `resend_running_commands_for_instance` sends no `running` row | AC-16 | B16 |
| `single_restore_path_test.rs` | `restore_dispatches_each_row_once` | each `backed_up` row of A sent once, `attempts = 1` | AC-17 | B29, B30, B31 |
| same | `restore_of_a_never_touches_b` | B rows unchanged, no send for B | AC-18 | B30 |
| same | `restore_without_live_pid_keeps_backed_up` | instance not running: rows stay `backed_up`, report `instance_not_running` | AC-17 | B29 |
| same | `launch_reinject_uses_single_path` | the launch reinject block calls the restore function once and nothing else sends | AC-17 | B29 |
| `resume_handoff_test.rs` | `handoff_has_instance_session_and_conversation` | document keys present; `session_id == conversation_id` | AC-12 | B07, B16 |
| same | `two_owners_skip_legacy_file` | repo owned by A and B: legacy file not written, outcome `skipped_multi_owner`, both per-instance hand-offs present | AC-11, AC-13 | B07 |
| same | `reader_ignores_foreign_instance_document` | a reader evaluating B ignores a document with `instance_id = A` | AC-11 | B33 |
| extend `per_instance_prompt_liveness_test.rs` | `first_row_idle_is_not_running` | a summaries DB with one idle row reports zero running | AC-19 | B05 |
| same | `stopped_instance_reports_zero_running` | dead PID: zero running rows | AC-20 | B05 |
| same | `composite_ids_survive_other_discovery_path` | rows written by capture survive `detect_running_projects` and the reverse | AC-21 | B11 |
| same | `default_filter_excludes_default_copy` | `default` never matches `x__default-copy-8159` | AC-22 | B14 |
| same | update `test_case_c_suffix_alias_resolution_for_cloned_instances` | assert the unique-suffix and ambiguous-suffix rules of `resolve_instance_input` instead of first-match fuzzy resolution | AC-04 | B02 |
| `switch_flow_instance_scope_test.rs` | `auto_rotation_default_target_restores_only_default` | target `default` restores only default rows; never `None` | AC-23 | B21 |
| same | `cross_instance_rotation_restores_source_only` | A to B rotation: A backed up once, A rows dispatched only under A, none under B | AC-24 | B22 |
| same | `desktop_switch_audits_default_instance` | desktop switch audit entry has the default id and one restore call | AC-25 | B23, B29 |
| same | `telegram_switch_audits_effective_instance` | Telegram switch with `--instance <A>` audits A and restores A only | AC-25 | B23 |
| same | `account_switch_default_path_keeps_default` | `account.rs` default path keeps `"default"`, never `target_ide` | AC-25 | B23 |
| `legacy_instance_migration_test.rs` | `migration_assigns_single_owner` | empty id row whose cid exists in one instance gets that id | AC-08 | B01, B18 |
| same | `migration_orphans_unknown_and_ambiguous` | no owner or two owners: `orphaned` with the right reason | AC-08 | B18 |
| same | `migration_leaves_no_legacy_keys` | Q11 and Q11B return 0 | AC-05 | B01 |
| same | `migration_copies_db_before_transaction` | `<db>.pre-v140.bak` exists before any update; copy failure stops the migration | AC-05 | none |
| same | `migration_runs_once` | second open does not re-run (guard row present) | AC-05 | none |
| same | `orphaned_rows_never_dispatch` | scheduler and restore skip `orphaned` | AC-08 | B13 |
| `cli_instance_parity_test.rs` | `action_commands_refuse_without_instance_when_two_running` | `brp`, `rrp`, `rrc`, `prompt` handlers return exit code 2 and change nothing | AC-26 | B25, B27 |
| same | `list_rows_carry_instance_fields` | every list row has `instance_id` and `instance_name`, grouped by instance | AC-27 | B26 |
| same | `cli_and_ipc_enqueue_write_same_row_shape` | CLI enqueue and IPC `enqueue_prompt` produce identical columns except `source_dir` (`cli` vs `ui`), text, and time | AC-28, AC-29 | B28 |
| same | `trace_has_all_identity_keys` | trace JSON contains all nine keys | AC-30 | none |
| same | `tree_is_scoped_to_instance` | tree for A returns only A's projects | AC-31 | B14 |
| same | `prompt_n_flag_is_node_not_instance` | `-n` is parsed as `--node`; help text says so next to `-i` | AC-26 | B24 |

Run them one file at a time (`cd src-tauri` then `cargo test --test <file_stem>`), per the AGENTS.md targeted-test rule. CI compiles them without running them.
