# Subtask 02: Status State Machine, Single Restore Path, Running Detection, Switch Flows

Status: pending

- Parent plan: `.ai-memory/plans/140-per-instance-prompt-attribution-backup-queue.md`
- Spec: `02-spec/21-app/140-per-instance-prompt-attribution-backup-queue/01-architecture-spec.md` (sections 6.5 to 6.8)
- Decisions implemented: D4, D5, D6
- Depends on: subtask 01 (`canonical_instance_id`, `InstanceScope`, new columns, `write_resume_handoff`, tagged scans).

## Goal

Make each prompt move through one explicit status machine, restore a switch backup through exactly one function for exactly one instance, count a prompt as running only when it is really live in that instance, and keep every switch flow (desktop, auto switcher, Telegram, CLI) attached to the instance where the prompt was typed.

## Bug IDs covered

B05, B08, B09, B11, B12, B13, B14, B15, B16, B17, B18, B19 (candidate part), B20, B21, B22, B23, B29, B30.

## Files and functions to change

Line numbers were re-checked on 2026-10-06. Re-check with `gitmap aum search "<literal>" <file> --ext .rs` before editing; anchor on the `fn` signature.

| File | Function | Current line | Change |
|---|---|---|---|
| `src-tauri/src/modules/repo_db.rs` | `discover_running_prompts_from_antigravity` | `:991` to `:995` | Remove `prompts.is_empty()` and the 5-idle-rows rule; apply the running predicate |
| `src-tauri/src/modules/repo_db.rs` | `backup_running_prompts` | `:1122`; stale retire `:1128` to `:1132`; Layer 2 `:1239` to `:1310`; fallback `:1316` to `:1319`; dedupe `:1337` | Replace stale retire with liveness sweep; Layer 2 only for running projects of that instance; instance filter, no `LIKE` |
| `src-tauri/src/modules/repo_db.rs` | `detect_running_projects` | `:512`; global delete `:626` | Per-instance, per-writer delete |
| `src-tauri/src/modules/repo_db.rs` | `dispatch_running_prompts` | `:1491` | Becomes an internal step of the single restore path; same-instance only |
| `src-tauri/src/modules/repo_db.rs` | `is_prompt_running_for_project` | `:1700`; Gate 4 `:2008` to `:2012` | Full normalized path equality only |
| `src-tauri/src/modules/repo_db.rs` | `check_and_dispatch_enqueued_prompts` | `:2054`; select `:2060` to `:2064`; mark `:2192` to `:2195` | Only `queued`; atomic claim; `dispatched` only on success |
| `src-tauri/src/modules/repo_db.rs` | `requeue_running_conversations_for_instance` | `:2245` | Keep only the audit record when called after a backup; no second backup |
| `src-tauri/src/modules/repo_db.rs` | `get_project_execution_status` | `:2692`; filter `:2706` to `:2709` | Exact canonical compare or exact `__{id}` suffix |
| `src-tauri/src/modules/repo_db.rs` | `spawn_prompt_via_agy` | `:3106`; worker key `:3125`; HOME branch `:3164` | Canonical id in key and in branch |
| `src-tauri/src/modules/repo_db.rs` | `resend_running_commands_for_instance` | `:3237`; select `:3248`; dedupe `:3317` | Never `running`; canonical instance required; dedupe by identity |
| `src-tauri/src/modules/repo_db.rs` | `auto_resume_recent_prompts` | `:3437`; `LIKE` `:3507`, `:3512`; file fallback `:3534` | No `LIKE`; per-instance hand-off only |
| `src-tauri/src/modules/backup_prompts_db.rs` | `backup_active_running_prompts_for_instance` | `:230`; candidates `:249` to `:252`; conv match `:300` to `:304`; stamp `:338` | Exact instance only; tagged conversation scan |
| `src-tauri/src/modules/backup_prompts_db.rs` | `restore_running_prompts_for_instance` | `:519`; resend at `:614` | Remove the resend call; add `restore_backed_up_prompts_for_instance` |
| `src-tauri/src/modules/instance.rs` | `restore_and_inject_prompts_for_instance` | `:2805` to `:2816` | Thin wrapper over the new single function |
| `src-tauri/src/modules/instance.rs` | launch reinject blocks | `:3565` to `:3575` (macOS), `:3669` to `:3680` | Call only the new single function |
| `src-tauri/src/modules/auto_switcher.rs` | `execute_profile_rotation_with_context` | `:1415`; backup `:1431`; requeue `:1595`; resend `:1690` to `:1695`; dispatch `:1715` | No `None` for default; restore the source instance only |
| `src-tauri/src/modules/integration.rs` | desktop switch | `:341` to `:437` | Keep `"default"` (real default path); one restore call |
| `src-tauri/src/modules/telegram_inbound.rs` | switch handlers | `:1354` to `:1356`, `:1385` to `:1386`, `:2729` to `:2733`, `:2788` to `:2792` | Effective instance id; one restore call |

## Step-by-step implementation

### Step 1: status constants and atomic claim (`src-tauri/src/modules/repo_db.rs`)

1. Add a `pub mod prompt_status` (or `const`s) with `RUNNING`, `BACKED_UP`, `QUEUED`, `DISPATCHING`, `DISPATCHED`, `FAILED`, `ORPHANED`. Replace string literals in the functions you touch.
2. Add:

```rust
/// Moves one row from `from_status` to `dispatching`. Returns true only if this caller won the claim.
pub fn claim_prompt_for_dispatch(conn: &Connection, id: &str, from_status: &str) -> bool;

/// Writes dispatched or failed. `reason` is required for failed.
pub fn finish_prompt_dispatch(conn: &Connection, id: &str, sent: bool, reason: Option<&str>);

/// Moves dispatching rows older than `max_age_secs` (120) to failed with dispatch_timeout.
pub fn sweep_stuck_dispatching(conn: &Connection, max_age_secs: i64) -> usize;
```

3. `claim_prompt_for_dispatch` runs `UPDATE active_prompts SET status = 'dispatching', attempts = attempts + 1, updated_at = ?1 WHERE id = ?2 AND status = ?3` and returns `conn.changes() == 1`.
4. Keep the in-memory map (`get_memory_prompts_map`) in sync in both helpers.

### Step 2: one dispatch step

Add `fn dispatch_one_prompt(conn: &Connection, prompt: &ActivePrompt) -> DispatchResult` that:

1. Checks `Path::new(&prompt.repo_path).exists()`; else `finish_prompt_dispatch(.., false, Some("repo_missing"))`.
2. Calls `write_resume_handoff(prompt, "dispatched")` (subtask 01); on `Failed` finishes with `handoff_write_failed`.
3. Calls `spawn_prompt_via_agy(prompt)`; finishes with `sent` true, or false with `spawn_failed` (or `agy_missing` when `get_antigravity_cli_executable_path` returns `None`, or `worker_busy` when an existing worker blocked it at `:3128` to `:3140`; a `worker_busy` row goes back to its previous status instead of `failed`, so it is retried by the next restore or scheduler run).

### Step 3: `spawn_prompt_via_agy` (`:3106`)

1. At the top: `let inst = canonical_instance_id(&prompt.instance_id)?` (return `false` and log on `Err`).
2. Worker key (`:3125`): `format!("{}:{}", inst, normalize_path_for_compare(&prompt.repo_path))`.
3. HOME branch (`:3164`): `if inst != default_id`. `"__default__"` can no longer reach the named branch.
4. Conversation resume: if `agy` supports resuming a conversation by id, pass `prompt.session_id`; if it does not, keep `-p` and record `status_reason = 'new_chat_no_resume_flag'` on success so `agm prompts trace` shows it. Check `agy --help` output read-only first and record the flag in the spec open questions; do not invent a flag.

### Step 4: scheduler consumes only `queued` (`check_and_dispatch_enqueued_prompts`, `:2054`)

1. Candidate query (`:2060` to `:2064`): `WHERE status = 'queued'`; add `AND instance_id = ?` when `target_instance` is `Some`.
2. Replace the `resolve_instance_id` match block (`:2086` to `:2116`) with canonical equality (the parameter becomes `Option<&InstanceScope>` or a canonical `Option<&str>`).
3. Busy check (`:2119` to `:2120`): call `is_prompt_running_for_project` with canonical ids and the normalized repo path only.
4. FIFO pick (`:2136` to `:2163`): `WHERE instance_id = ?3 AND repo_path = ?2 AND status = 'queued'` (drop `project_id = ?1 OR` and the default alias clause; ids are canonical after migration).
5. Replace the resume write, spawn, and unconditional update (`:2174` to `:2202`) with `claim_prompt_for_dispatch(.., "queued")` then `dispatch_one_prompt`.
6. Audit `action_taken` uses the real result (`dispatched` or `failed:<reason>`).
7. Call `sweep_stuck_dispatching(&conn, 120)` at the start of each run.

### Step 5: single restore path

1. In `src-tauri/src/modules/backup_prompts_db.rs` add `pub fn restore_backed_up_prompts_for_instance(instance_id: &str, keep_backup: bool, custom_file: Option<&str>) -> Result<RestoreReport, String>` (spec section 6.6):
   1. `let inst = canonical_instance_id(instance_id)?`.
   2. `wait_for_instance_prompt_channel(&inst)` (`src-tauri/src/modules/instance.rs:2987`, already bounded). Then check liveness with `is_instance_running` (`src-tauri/src/modules/instance.rs:2584`); if not running, return a report with `skipped_reason = "instance_not_running"` and change nothing.
   3. Restore unrestored `prompt_backups` rows with `instance_id = inst` into `active_prompts` through `save_or_requeue_prompt` (`src-tauri/src/modules/repo_db.rs:2777`) with `source_dir = "backup_restore"`, using `prompt_status_after_restore` (`:622`). Mark restored unless `keep_backup`.
   4. Select `active_prompts WHERE instance_id = inst AND status = 'backed_up' ORDER BY created_at ASC`. For each: `claim_prompt_for_dispatch(.., "backed_up")`, then `dispatch_one_prompt`. One row per `(normalized repo_path)` at a time; later rows of the same repo become `queued` so the scheduler sends them after the first finishes.
   5. Call `ensure_prompt_goals_running_for_instance(&inst)` once.
2. Delete the call `repo_db::resend_running_commands_for_instance(Some(target_inst), 20)` at `:614` (B30). `restore_running_prompts_for_instance` keeps only the DB move (step 1.3) for callers that want "restore without dispatch"; mark it `#[deprecated]` in a comment-free way by routing all production callers to the new function.
3. `restore_and_inject_prompts_for_instance` (`src-tauri/src/modules/instance.rs:2805`): body becomes `restore_backed_up_prompts_for_instance(instance_id, false, None).map(|r| r.dispatched)`.
4. Launch reinject (`src-tauri/src/modules/instance.rs:3563` to `:3576` and `:3668` to `:3681`): replace the four calls with one call to the new function (B29). Keep `invalidate_prompt_tree_cache`.
5. `dispatch_running_prompts` (`src-tauri/src/modules/repo_db.rs:1491`): make it call the new per-instance dispatch loop for a canonical id; remove the `resolve_instance_id` default alias logic (`:1494` to `:1503`, `:1534` to `:1567`).

### Step 6: resend never touches `running` (`resend_running_commands_for_instance`, `:3237`)

1. Signature: `instance_id: &str` (canonical, required). `resend_all_running_commands` (`:3230`) loops over running instances explicitly and calls the per-instance function; it is used only where a caller really means "all" (none remain after Step 9).
2. Query (`:3244` to `:3250`): `WHERE instance_id = ?1 AND status IN ('backed_up', 'failed')`.
3. Dedupe (`:3317`): key `(normalize_path_for_compare(repo_path), session_id)`.
4. Replace the inline resume JSON (`:3375` to `:3394`) and the spawn with `claim_prompt_for_dispatch` plus `dispatch_one_prompt`.
5. Remove the dispatched-cache shortcut that marks rows `dispatched` without sending (`:3357` to `:3367`); a row is `dispatched` only through `finish_prompt_dispatch`.

### Step 7: running detection (spec section 6.7)

1. Add `pub fn is_live_turn(not_fully_idle: i64, status: &str, last_turn_at: i64, now: i64) -> bool` returning `not_fully_idle > 0 && status.contains("RUNNING") && now - last_turn_at <= 120`. Read `last_turn_at` from the summaries row (extend `read_conversation_summary_rows` and `ConversationSummaryRow`; update every initializer, including the fallback at `:940` to `:947`, which today sets `not_fully_idle: 1` for rows without status columns; that fallback must set `0` so unknown schema rows are never treated as live).
2. `discover_running_prompts_from_antigravity` (`:991` to `:995`): `if !is_live_turn(..) { continue; }`. Live rows get `status = "running"` when called from detection and `"backed_up"` only when called from the backup path (pass a parameter; do not hardcode `"backed_up"` at `:1089`).
3. Gate the whole scan on `is_instance_running(id, data_dir, saved_pid)`; when not running, return an empty `Vec` and let the liveness sweep remove that instance's `running` rows.
4. Liveness sweep (replaces `:1128` to `:1132`): `DELETE FROM active_prompts WHERE instance_id = ?1 AND status = 'running' AND session_id NOT IN (<live cids of this instance>)`, after recording each removal with `log_instance_prompt_audit`.
5. Layer 2 (`:1239` to `:1310`): filter `projects` to `p.is_running && p.instance_id == inst` before spawning threads. Fallback query (`:1316` to `:1319`): `WHERE instance_id = ?1 AND repo_path = ?2 ORDER BY updated_at DESC LIMIT 1` with normalized path. Dedupe (`:1337`): add `AND instance_id = ?`.
6. `detect_running_projects` (`:626`): replace the global delete with a per-instance delete of legacy ids only: `DELETE FROM running_projects WHERE instance_id = ?1 AND substr(id, -length('__' || ?1)) <> '__' || ?1`. Rows from Layer 1 and from the CLI now use composite ids `{base}__{instance_id}` (subtask 01) and are kept; rows of other instances are never touched.
7. Gate 4 (`:2008` to `:2012`): `is_target_matched = has_target && clean_p == clean_target`.
8. `get_project_execution_status` (`:2706` to `:2709`): compare canonical ids; accept `p.project_id.ends_with(&format!("__{}", inst))` as the only suffix form.
9. `auto_resume_recent_prompts` (`:3507` to `:3534`): drop `proj_like`; query by `instance_id` and normalized `repo_path`; read only the per-instance hand-off (subtask 01 reader).

### Step 8: backup candidates (`backup_active_running_prompts_for_instance`, `src-tauri/src/modules/backup_prompts_db.rs:230`)

1. Parameter becomes a canonical `&str` (callers pass a real id; `None` is no longer accepted).
2. Candidates (`:249` to `:252`): `p.instance_id == inst` only.
3. Conversation match (`:300` to `:304`): use `scan_conversations_for(&InstanceScope::One(inst.clone()), 100)` (subtask 01) and match by `p.session_id == conversation_id` first, then by normalized path equality of the first workspace URI. No `contains`.
4. Stamp (`:338`): `instance_id: Some(p.instance_id.clone())` (already equal to `inst`).

### Step 9: switch flows (spec section 6.8)

1. Auto switcher (`src-tauri/src/modules/auto_switcher.rs`):
   1. `:1421` to `:1427`: the fallback to `get_active_instance_id` stays only for the daemon path where no context exists; pass the result through `canonical_instance_id` and stop the rotation on `Err` before any backup.
   2. `:1431` to `:1436`: replace the two backup calls with one `backup_prompts_for_instance(&current_instance_id, None)` (the shared function from subtask 03; if subtask 03 is not merged yet, call `backup_running_prompts` then `backup_active_running_prompts_for_instance` with the same canonical id).
   3. `:1595` to `:1597`: remove the second backup; record the requeue audit event only.
   4. `:1684` to `:1729`: compute `restore_inst = current_instance_id.clone()` (the source). Same-instance rotation: restore `restore_inst` after relaunch. Cross-instance rotation: restore `restore_inst` only if the source was relaunched; otherwise skip and log `[AutoSwitcher] source '<id>' not relaunched; <n> prompts remain backed_up`. Never pass `None`; never dispatch under `target.instance_id` rows that belong to the source.
   5. Replace `resend_running_commands_for_instance` and `dispatch_running_prompts` there with one `restore_backed_up_prompts_for_instance(&restore_inst, false, None)`.
2. Desktop switch (`src-tauri/src/modules/integration.rs:341` to `:437`): keep `"default"` (this path closes and relaunches the default instance, `:356`). Replace `:426` to `:437` with one `restore_backed_up_prompts_for_instance(&default_id, false, None)`. Never pass `effective_target` (IDE flavor) as an instance id.
3. Telegram (`src-tauri/src/modules/telegram_inbound.rs`): at `:1354` to `:1356` and `:2729` to `:2733` and `:2788` to `:2792`, compute the effective instance (explicit `--instance` from the command, else the instance the switch acts on, else default) through `resolve_instance_input` and then call the shared backup and the single restore for that id. Remove `resend_all_running_commands(20)` from these paths. `:1385` to `:1386` uses the shared backup with the same id.
4. `src-tauri/src/modules/account.rs:1743` stays `"default"`. Do not change it.

## Unit tests to add

| Test name | File | Asserts |
|---|---|---|
| `claim_prompt_for_dispatch_is_exclusive` | `repo_db.rs` | Two claims on one row: first true, second false; `attempts == 1` |
| `scheduler_ignores_backed_up_rows` | `repo_db.rs` | Seed one `backed_up` and one `queued` row for A; scheduler run touches only the `queued` row |
| `scheduler_marks_failed_when_spawn_fails` | `repo_db.rs` | Missing repo path: row ends `failed` with `repo_missing`, never `dispatched` |
| `stuck_dispatching_moves_to_failed_after_timeout` | `repo_db.rs` | Row `dispatching` with old `updated_at` becomes `failed` / `dispatch_timeout` |
| `resend_never_selects_running_rows` | `repo_db.rs` | Seed `running` and `backed_up` for A; resend selects only `backed_up` |
| `restore_for_instance_does_not_touch_other_instance` | `backup_prompts_db.rs` | Backed-up rows for A and B; restore for A leaves B unchanged |
| `restore_dispatches_each_row_at_most_once` | `backup_prompts_db.rs` | After restore, every restored row has `attempts <= 1` |
| `restore_running_prompts_for_instance_does_not_resend` | `backup_prompts_db.rs` | Calling it moves rows only; no row reaches `dispatching` |
| `empty_instance_legacy_row_not_in_any_backup` | `backup_prompts_db.rs` | Row with `instance_id = ''` (orphaned) is not a candidate for A or default |
| `two_instances_same_repo_backup_separately` | `backup_prompts_db.rs` | A and B with the same repo and same text produce two `prompt_backups` rows with their own ids |
| `first_summary_row_is_not_running_when_idle` | `repo_db.rs` | Fixture with one idle row: discovery returns no running prompt |
| `fallback_schema_row_is_never_live` | `repo_db.rs` | Summaries DB without status columns: `not_fully_idle` is 0 and the row is not live |
| `is_live_turn_respects_120_second_ttl` | `repo_db.rs` | `now - last_turn_at` of 121 returns false; 120 returns true |
| `detect_running_projects_keeps_other_writer_rows` | `repo_db.rs` | Composite Layer-1 row for A survives a `detect_running_projects(A)` call |
| `gate4_does_not_match_prefix_folder` | `repo_db.rs` | Target `.../app` does not match conversation URI `.../app-v2` |
| `execution_status_default_does_not_match_default_copy` | `repo_db.rs` | Project id `x__default-copy-8159` is not returned for instance `default` |
| `spawn_worker_key_is_canonical` | `repo_db.rs` | `"__default__"` and `"default"` produce the same worker key |
| `auto_switch_default_target_restores_default_only` | `auto_switcher.rs` | The computed restore scope for a `"default"` target is `Some(default_id)`, never all instances |
| `cross_instance_rotation_restores_source_not_target` | `auto_switcher.rs` | Computed `restore_inst` equals the source id |

Pure helpers (restore scope computation, `is_live_turn`, worker key) must be extracted into functions so they can be tested without launching processes.

## Acceptance criteria

AC-14, AC-15, AC-16, AC-17, AC-18, AC-19, AC-20, AC-21, AC-22, AC-23, AC-24, AC-25, AC-09 and AC-10 (candidate part), AC-32. Definitions are in spec section 8.

## Verification commands

```text
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features
cd src-tauri && cargo test --lib -- --test-threads=1 claim_prompt_for_dispatch_is_exclusive
cd src-tauri && cargo test --lib -- --test-threads=1 scheduler_
cd src-tauri && cargo test --lib -- --test-threads=1 restore_
cd src-tauri && cargo test --lib -- --test-threads=1 resend_never_selects_running_rows
cd src-tauri && cargo test --lib -- --test-threads=1 is_live_turn_respects_120_second_ttl
cd src-tauri && cargo test --lib -- --test-threads=1 gate4_does_not_match_prefix_folder
cd src-tauri && cargo test --lib -- --test-threads=1 execution_status_default_does_not_match_default_copy
cd src-tauri && cargo test --lib -- --test-threads=1 auto_switch_default_target_restores_default_only
cd src-tauri && cargo test --lib -- --test-threads=1 cross_instance_rotation_restores_source_not_target
```

No `npm run build` is needed for this subtask (no `src/` change).

## Do not

- Never key per-instance data by IDE flavor (`target_ide`, `effective_target`); the desktop switch passes `"default"` as the instance id and the flavor only to credential and process helpers.
- Never trust a launcher wrapper PID alone; liveness goes through `is_instance_running` with its bounded `find_pids_for_data_dir` fallback; never replace that fallback with a silent `false`.
- Never close another instance's PID during a single-instance switch; use `close_instance(id)` with `should_spare_pid`; never call `close_antigravity(None)` from that path.
- Never revert the v4.155.0 per-call scan in `is_instance_running` before ambiguity `.ai-memory/ambiguous-questions/01-new-ambiguity/02-is-instance-running-saved-pid-fast-path.md` is answered.
- Never mark a row `dispatched` unless `spawn_prompt_via_agy` returned true.
- Never re-send a `running` row; never run more than one restore path per switch.
- Never pass `None` (all instances) from a switch flow.
- Never add an unbounded wait; every wait has a timeout (prompt channel wait, dispatching sweep at 120 s).
- Never edit a range that starts or ends inside a block without re-reading the whole function; never delete a `let` without searching its other uses; never add a struct field (for example on `ConversationSummaryRow` or `ActivePrompt`) without updating every initializer, including tests.
- E2E safety (D10): never kill Cursor or `agm-alim`; never switch or close the default instance; never delete `cli-switch-proof-6857`, `test-cli-flow-1743`, `clone-from-default-4424`, `new-empty-instance-4425`; test instances only with prefix `test-cli-flow-` or `test-diag-`; delete only instances this run created; do not click the Google data-collection checkbox; do not launch a replacement AGM GUI; read databases read-only; never print emails or tokens in full; never claim a live reinject was watched unless it was.
