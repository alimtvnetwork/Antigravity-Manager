# Subtask 03: CLI Flags, NEW Commands, NEW IPC Wrappers, UI Reuse

Status: pending

- Parent plan: `.ai-memory/plans/140-per-instance-prompt-attribution-backup-queue.md`
- Spec: `02-spec/21-app/140-per-instance-prompt-attribution-backup-queue/01-architecture-spec.md` (sections 6.9 to 6.11)
- Decisions implemented: D7, D8
- Depends on: subtask 01 (`resolve_instance_input`, `canonical_instance_id`, `InstanceScope`) and subtask 02 (`restore_backed_up_prompts_for_instance`, claim and dispatch helpers).

## Goal

Every prompt-related CLI command and every UI action names its instance, prints or returns `instance_id` and `instance_name` on every row, and goes through the same Rust function. Action commands refuse to guess when more than one instance is running. The missing `enqueue_prompt` IPC exists, and the UI stops writing prompt state to disk.

## Bug IDs covered

B24, B25, B26, B27, B28, B31. Also the UI side of B07 (`src/components/instances/PromptTreeViewModal.tsx:1237`, `:1346`).

## Files and functions to change

Line numbers were re-checked on 2026-10-06. Re-check with `gitmap aum search "<literal>" <file> --ext .rs` (or `--ext .tsx`) before editing; anchor on the `fn` signature or a unique line.

| File | Function or item | Current line | Change |
|---|---|---|---|
| `src-tauri/src/bin/agm.rs` | dispatch table | `:51` to `:145` | Route NEW subcommands; no new top-level names except as listed |
| `src-tauri/src/bin/agm.rs` | `cmd_which_prompts_running` | `:1597`; `--json` `:1620`; refresh loop `:1623` to `:1627`; join `:1640` | NEW `-i`; join by instance and normalized path |
| `src-tauri/src/bin/agm.rs` | `cmd_prompts` | `:1773`; `--json` `:1881` | NEW `-i`; NEW subcommands `queue ls`, `trace`; `prompts backup` / `prompts restore` follow the refusal rule |
| `src-tauri/src/bin/agm.rs` | `cmd_prompt_dispatch` | `:2913`; help `:2923` to `:2939`; `-i` `:2995` to `:3003`; `-n` `:3008`; local branch `:3124` to `:3134`; inserts `:3139` to `:3152`; resume write `:3154`; spawn `:3184` | NEW `enqueue` subcommand; cwd-owner rule; shared function; `failed` on spawn error |
| `src-tauri/src/bin/agm.rs` | `cmd_resend_running_commands` | `:3326`; backup and resend `:3371` to `:3382` | `-i` required by refusal rule; same instance for backup and resend |
| `src-tauri/src/bin/agm.rs` | `cmd_queue_scheduler` | `:3843` | Accept `-i` besides positional instance |
| `src-tauri/src/bin/agm.rs` | `cmd_backup_running_prompts` | `:3928`; default `:4036` to `:4037`; parse `:4044` to `:4056`; calls `:4060` to `:4061` | Refusal rule; shared backup function |
| `src-tauri/src/bin/agm.rs` | `cmd_restore_running_prompts` | `:4119`; default `:4145` to `:4146`; calls `:4169` to `:4175` | Refusal rule; only `restore_backed_up_prompts_for_instance` (B31) |
| `src-tauri/src/bin/agm.rs` | `cmd_running_prompts` | `:4240`; `--json` `:4304` | NEW `-i` on `ls` |
| `src-tauri/src/bin/agm.rs` | `cmd_running_projects` | `:4587`; `--json` `:4609` | NEW `-i` |
| `src-tauri/src/bin/agm.rs` | `cmd_tree` | `:7019`; `--json` `:7050` | NEW `-i` |
| `src-tauri/src/modules/repo_db.rs` | NEW shared functions | near `save_or_requeue_prompt` (`:2777`) | `enqueue_prompt_for_instance`, `send_prompt_now_for_instance`, `list_queued_prompts_for_instance`, `trace_prompts_for_instance` |
| `src-tauri/src/modules/repo_db.rs` | `list_running_projects`, `list_backed_up_prompts` | `:2328`, `:2364` | Take `&InstanceScope` |
| `src-tauri/src/modules/backup_prompts_db.rs` | NEW `backup_prompts_for_instance` | near `:222` | One backup entry point for CLI, IPC, and switch flows |
| `src-tauri/src/commands/instance.rs` | `list_running_projects`, `list_backed_up_prompts` | `:207`, `:212` | Add `instance_id: Option<String>` |
| `src-tauri/src/commands/instance.rs` | NEW IPC commands | after `:214` | `enqueue_prompt`, `send_prompt_now`, `backup_running_prompts_for_instance`, `restore_running_prompts_for_instance`, `list_queued_prompts`, `trace_prompts` |
| `src-tauri/src/commands/mod.rs` | re-exports | search `pub use` with gitmap | Export the new commands the same way existing ones are exported |
| `src-tauri/src/lib.rs` | `invoke_handler` list | `:1104` (`commands::list_backed_up_prompts`) | Register the new commands next to it |
| `src/pages/Instances.tsx` | `fetchRunningTasks` | `:279` to `:291` (invoke at `:281`) | Pass the instance id (or call once per instance and merge with an instance tag) |
| `src/components/instances/PromptTreeViewModal.tsx` | `handleResendPrompt` | `:1235` to `:1268` (file write `:1237`, `save_text_file` `:1250`, resume call `:1262`) | Replace with `send_prompt_now` IPC |
| `src/components/instances/PromptTreeViewModal.tsx` | `handleEnqueuePrompt` | `:1325`; invoke `:1335`; fallback `:1344` to `:1363` | Keep the invoke (now real); remove the file fallback; show the error |
| `src/components/instances/PromptTreeViewModal.tsx` | `'default'` fallbacks | `:1241`, `:1260`, `:1338`, `:1350` | Use the modal `instanceId`; error when neither is known |

## Step-by-step implementation

### Step 1: shared instance selection helper for the CLI (`src-tauri/src/bin/agm.rs`)

Add one helper and use it in every prompt command (do not copy argument loops into each command):

```rust
enum InstanceSelection {
    One { id: String, name: String },
    All,
}

/// Parses -i/--instance/-instance/--instance=/-i=. For action commands, refuses when no -i
/// is given and more than one instance is running.
fn select_instance(args: &[String], is_action: bool) -> Result<InstanceSelection, String>;
```

Rules:

1. With `-i <value>`: `resolve_instance_input(value)`; on `Err`, print the error (it already lists candidates) and exit with code 2.
2. Without `-i`, list command: `InstanceSelection::All`.
3. Without `-i`, action command: count running instances with `is_instance_running` over the registry. Exactly one: use it and print `[instance] using #<seq> <name> (only running instance)` on stderr (so `--json` stdout stays clean). Zero: use the default id and print the same note. Two or more: print `instance required: <n> instances running (#<seq> <name>, ...); pass -i <id|#seq|name>` and exit with code 2 before any side effect.
4. `instance_name` comes from the registry entry; for the default instance use its registry name, else `"default"`.

### Step 2: JSON and row fields

1. Every JSON object printed by `wpr`, `prompts`, `prompts queue ls`, `prompts trace`, `running-prompts ls`, `running-projects`, `tree`, `brp`, `rrp`, `rrc`, `qs`, `prompt`, `prompt enqueue` includes `"instance_id"` and `"instance_name"`.
2. Table output prints a group header per instance: `== #<seq> <name> (<instance_id>) ==`.
3. Add one helper `fn instance_label(id: &str) -> (String, String)` and reuse it.

### Step 3: list commands with NEW `-i`

1. `cmd_which_prompts_running` (`:1597`): use `select_instance(args, false)`. Refresh only the selected instance (or each registered instance for `All`, as today at `:1623` to `:1627`). Replace the join at `:1640` with `p.instance_id == proj.instance_id && normalize_path_for_compare(&p.repo_path) == normalize_path_for_compare(&proj.repo_path)` (B26). Use `scan_conversations_for(&scope, 100)` instead of `agy_cleaner::scan_conversations(100)` at `:1631`.
2. `cmd_prompts` `ls` (`:1773`): filter rows by selection; show `status_reason`.
3. `cmd_running_prompts` `ls` (`:4240`): call `list_backed_up_prompts(&scope)`.
4. `cmd_running_projects` (`:4587`): call `list_running_projects(&scope)`.
5. `cmd_tree` (`:7019`): pass `Some(id)` for `One`, `None` for `All` to `get_project_conversation_tree_cached`.
6. Update each command's help text with `-i, --instance <id|#seq|name>`.

### Step 4: NEW subcommands (proposed; none exist today)

1. NEW `agm prompt enqueue -i <inst> --repo <path> [--conversation <cid>] "<text>" [--json]`: handled at the start of `cmd_prompt_dispatch` when `args[0] == "enqueue"`. `-i` is required (no refusal fallback for enqueue: an enqueue always names its instance). `--repo` defaults to the current folder. Calls `repo_db::enqueue_prompt_for_instance`. Prints the row id, `instance_id`, `instance_name`, `repo_path`, `conversation_id`, `status`.
2. NEW `agm prompts queue ls -i <inst> [--json]`: in `cmd_prompts` when `args[0] == "queue"` and `args[1] == "ls"`. Calls `list_queued_prompts_for_instance` (statuses `queued`, `dispatching`, `failed`).
3. NEW `agm prompts trace -i <inst> [--conversation <cid>] [--json]`: in `cmd_prompts` when `args[0] == "trace"`. Calls `trace_prompts_for_instance`. JSON keys exactly: `source_dir`, `conversation_id`, `repo_path`, `row_id`, `status`, `status_reason`, `status_history`, `backup_row_id`, `handoff` (`{ "path": ..., "legacy": "written" | "skipped_multi_owner" | "not_applicable" }`), `dispatch_result`, `instance_id`, `instance_name`.

### Step 5: shared Rust functions (`src-tauri/src/modules/repo_db.rs`)

```rust
pub fn enqueue_prompt_for_instance(
    instance_id: &str,
    repo_path: &str,
    text: &str,
    conversation_id: Option<&str>,
    source: &str, // "cli" or "ui"
) -> Result<ActivePrompt, String>;

pub fn send_prompt_now_for_instance(
    instance_id: &str,
    repo_path: &str,
    text: &str,
    conversation_id: Option<&str>,
    source: &str,
) -> Result<ActivePrompt, String>; // enqueue, claim, dispatch_one_prompt; returns final row

pub fn list_queued_prompts_for_instance(instance_id: &str) -> Result<Vec<ActivePrompt>, String>;

pub fn trace_prompts_for_instance(
    instance_id: &str,
    conversation_id: Option<&str>,
) -> Result<Vec<PromptTrace>, String>;
```

Rules:

1. Each function starts with `let inst = canonical_instance_id(instance_id)?;` and validates `repo_path` exists and `text.trim()` is not empty before writing anything.
2. `enqueue_prompt_for_instance` inserts `id = format!("queued-{}-{}", inst, Uuid::new_v4())`, `project_id = format!("{}__{}", repo_slug, inst)`, `session_id = conversation_id`, `status = 'queued'`, `source_dir = source`. It ensures a composite `running_projects` row exists for `(repo, inst)` with `is_running = 0` (never `''` storage path without `__`, B25). It calls `invalidate_prompt_tree_cache(Some(&inst))`.
3. `send_prompt_now_for_instance` uses `cli-{inst}-{uuid}` ids for `source == "cli"`, then `claim_prompt_for_dispatch` and `dispatch_one_prompt` (subtask 02). The row ends `dispatched` or `failed` with a reason (B25).
4. `trace_prompts_for_instance` reads `active_prompts` (instance filter), `prompt_backups` (same `instance_id` and `prompt_id`), audit events from `task_history_db` for the row id, and the hand-off outcome (per-instance file path and legacy outcome from `repo_owner_instances`).
5. `list_running_projects(scope)` and `list_backed_up_prompts(scope)`: add `WHERE instance_id = ?` for `One`.

In `src-tauri/src/modules/backup_prompts_db.rs`:

```rust
pub fn backup_prompts_for_instance(
    instance_id: &str,
    custom_file: Option<&str>,
) -> Result<BackupReport, String>; // repo_db::backup_running_prompts(inst) then backup_active_running_prompts_for_instance(inst)
```

### Step 6: action commands

1. `cmd_prompt_dispatch` (`:2913`):
   1. Keep `--node/-n` as remote SSH (`:3008`). Update help (`:2923` to `:2939`) to say `-n is --node (remote SSH), not instance; use -i for instance`.
   2. Local branch without a sequence target (`:3124` to `:3134`): if `-i` is given use it; else compute `repo_owner_instances(cwd)` (subtask 01): exactly one owner, use it; otherwise apply the action refusal rule from Step 1.
   3. Sequence branch (`:3112` to `:3123`): `seq.instance_id` is used only after `canonical_instance_id`; an explicit `-i` that differs from the sequence's instance is an error (`sequence C001 belongs to <id>; remove -i or use the matching instance`), instead of silently overriding.
   4. Replace the inserts, the resume write, and the spawn (`:3139` to `:3184`) with `send_prompt_now_for_instance(&inst, &repo, &final_prompt, conversation_id, "cli")`. `session_id` is the sequence conversation id or `None`, never the repo slug.
   5. Exit code 1 when the final status is `failed`; print the reason.
2. `cmd_backup_running_prompts` (`:3928`): replace the parse loop and the active default (`:4036` to `:4058`) with `select_instance(args, true)`; call `backup_prompts_for_instance`.
3. `cmd_restore_running_prompts` (`:4119`): same selection; delete the resend and dispatch calls at `:4169` to `:4170`; call only `restore_backed_up_prompts_for_instance(&inst, keep_backup, custom_file)` (B31). Print the `RestoreReport`.
4. `cmd_resend_running_commands` (`:3326`): `select_instance(args, true)`; replace `:3371` to `:3382` with `backup_prompts_for_instance(&inst, None)` then `resend_running_commands_for_instance(&inst, limit_n)` (B27).
5. `cmd_queue_scheduler` (`:3843`): accept `-i` as well as the positional instance; pass a canonical id or `All`.
6. `agm prompts backup` and `agm prompts restore` subcommands: route through the same functions as `brp` and `rrp` with the same refusal rule.
7. Update scripts that call these commands without `-i` (search `scripts/` with `gitmap aum search "agm brp" scripts`, and the same for `rrp`, `rrc`, `backup-running-prompts`, `restore-running-prompts`) to pass `-i`.

### Step 7: IPC wrappers (`src-tauri/src/commands/instance.rs`)

Each wrapper is a thin call; no logic, no shelling out to `agm`:

```rust
#[tauri::command]
pub fn enqueue_prompt(
    instance_id: String,
    repo_path: String,
    prompt_content: String,
    conversation_id: Option<String>,
    project_id: Option<String>,
) -> Result<crate::modules::repo_db::ActivePrompt, String> {
    let _ = project_id;
    crate::modules::repo_db::enqueue_prompt_for_instance(
        &instance_id,
        &repo_path,
        &prompt_content,
        conversation_id.as_deref(),
        "ui",
    )
}
```

Same pattern for:

- `send_prompt_now(instance_id, repo_path, prompt_content, conversation_id?)` over `send_prompt_now_for_instance(.., "ui")`.
- `backup_running_prompts_for_instance(instance_id)` over `backup_prompts_db::backup_prompts_for_instance(&instance_id, None)`.
- `restore_running_prompts_for_instance(instance_id, keep_backup?)` over `backup_prompts_db::restore_backed_up_prompts_for_instance(&instance_id, keep_backup.unwrap_or(false), None)`.
- `list_queued_prompts(instance_id)` over `list_queued_prompts_for_instance`.
- `trace_prompts(instance_id, conversation_id?)` over `trace_prompts_for_instance`.
- `list_running_projects(instance_id?)` and `list_backed_up_prompts(instance_id?)`: map `Some(id)` through `resolve_instance_input` to `InstanceScope::One`, `None` to `All`.

The UI already sends `conversationId`, `projectId`, `instanceId`, `repoPath`, `promptContent` (`src/components/instances/PromptTreeViewModal.tsx:1335` to `:1341`); Tauri maps camelCase to the snake_case parameter names above. `project_id` is accepted for compatibility and ignored, because the project id is derived from `(repo_path, instance_id)`.

Register every new command in `src-tauri/src/lib.rs` next to `commands::list_backed_up_prompts` (`:1104`) and export it from `src-tauri/src/commands/mod.rs` the same way as the existing instance commands.

### Step 8: UI changes

1. `src/pages/Instances.tsx:281`: pass `instanceId` for the instance being rendered. If the page shows all instances at once, call the IPC once per instance and tag each node with its `instance_id` (the IPC result already carries it).
2. `src/components/instances/PromptTreeViewModal.tsx`:
   1. `handleResendPrompt` (`:1235` to `:1268`): delete the `save_text_file` resume write and the `resume_recent_project_prompts` call; call `invoke('send_prompt_now', { instanceId, repoPath, promptContent, conversationId })`. Keep clipboard copy and focus (`:1270` to `:1280`).
   2. `handleEnqueuePrompt` (`:1325`): keep `invoke('enqueue_prompt', ...)`; delete the fallback at `:1344` to `:1363`; on error show it with `setError` and leave `enqueued = false`.
   3. Replace `|| 'default'` at `:1241`, `:1260`, `:1338`, `:1350` with the modal `instanceId`; if both `selectedProject?.instance_id` and `instanceId` are empty, show `Instance unknown; open the tree from an instance row` and do not call the IPC.
3. Instance badge: every prompt row shows `#seq name` from `instance_id` and `instance_name` (markup and placement are owned by `02-component-and-e2e-spec.md`; reuse the existing badge component style, no new control groups).
4. Write JSX conditionals as `{cond && (...)}` or `{cond ? (...) : null}`; never mix the two forms.

### Step 9: CLI and UI parity check

Confirm each row of this table calls the same Rust function from both sides (read the wrapper and the CLI command after editing):

| User action | CLI | IPC | Shared function |
|---|---|---|---|
| List running prompts | `agm wpr -i <inst> --json` | `get_project_conversation_tree(instanceId, onlyRunning: true)` | `get_project_conversation_tree_cached` |
| List backed-up prompts | `agm running-prompts ls -i <inst> --json` | `list_backed_up_prompts(instanceId)` | `list_backed_up_prompts(&InstanceScope)` |
| List running projects | `agm running-projects -i <inst> --json` | `list_running_projects(instanceId)` | `list_running_projects(&InstanceScope)` |
| Enqueue | NEW `agm prompt enqueue -i <inst> --repo <path> "<text>"` | NEW `enqueue_prompt` | `enqueue_prompt_for_instance` |
| Send now | `agm prompt -i <inst> "<text>"` | NEW `send_prompt_now` | `send_prompt_now_for_instance` |
| List queue | NEW `agm prompts queue ls -i <inst>` | NEW `list_queued_prompts` | `list_queued_prompts_for_instance` |
| Backup | `agm brp -i <inst>` | NEW `backup_running_prompts_for_instance` | `backup_prompts_for_instance` |
| Restore | `agm rrp -i <inst>` | NEW `restore_running_prompts_for_instance` | `restore_backed_up_prompts_for_instance` |
| Trace | NEW `agm prompts trace -i <inst>` | NEW `trace_prompts` | `trace_prompts_for_instance` |

## Unit tests to add

| Test name | File | Asserts |
|---|---|---|
| `select_instance_refuses_action_when_two_running` | `src-tauri/src/bin/agm.rs` test module (or the module that hosts the helper if it is moved to `src-tauri/src/modules/cli.rs`) | With a stubbed running set of two instances, an action without `-i` returns `Err` naming both |
| `select_instance_uses_only_running_instance` | same | One running instance: returns it |
| `select_instance_list_without_flag_is_all` | same | List command without `-i` returns `All` |
| `enqueue_prompt_for_instance_writes_one_queued_row` | `repo_db.rs` | Row id starts with `queued-<id>-`, status `queued`, `source_dir` set, `session_id` equals the given conversation id |
| `enqueue_prompt_for_instance_rejects_unknown_instance` | `repo_db.rs` | Unknown id returns `Err`; no row written; no `instances/<id>` folder created |
| `enqueue_two_instances_same_repo_gives_two_rows` | `repo_db.rs` | Enqueue for A and B on one repo: two rows with distinct `instance_id` and composite project ids |
| `send_prompt_now_marks_failed_on_missing_repo` | `repo_db.rs` | Status `failed`, reason `repo_missing` |
| `cli_dispatch_session_id_is_never_repo_slug` | `repo_db.rs` | `send_prompt_now_for_instance` without a conversation id stores `session_id = NULL` |
| `list_backed_up_prompts_scope_one_filters_instance` | `repo_db.rs` | Rows for A and B; scope `One(A)` returns only A |
| `trace_includes_identity_chain_keys` | `repo_db.rs` | Trace JSON for a seeded row contains every key listed in Step 4.3 |
| `ipc_enqueue_matches_cli_enqueue_row_shape` | `src-tauri/src/commands/instance.rs` test module | Calling the IPC function and the shared function with the same input produces rows that differ only in `id`, timestamps, and `source_dir` |

## Acceptance criteria

AC-02, AC-03, AC-04, AC-11, AC-15, AC-26, AC-27, AC-28, AC-29, AC-30, AC-31, plus AC-17 and AC-18 for the `rrp` path. Definitions are in spec section 8.

## Verification commands

```text
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features
cd src-tauri && cargo test --lib -- --test-threads=1 select_instance_
cd src-tauri && cargo test --lib -- --test-threads=1 enqueue_
cd src-tauri && cargo test --lib -- --test-threads=1 send_prompt_now_marks_failed_on_missing_repo
cd src-tauri && cargo test --lib -- --test-threads=1 cli_dispatch_session_id_is_never_repo_slug
cd src-tauri && cargo test --lib -- --test-threads=1 list_backed_up_prompts_scope_one_filters_instance
cd src-tauri && cargo test --lib -- --test-threads=1 trace_includes_identity_chain_keys
cd src-tauri && cargo test --lib -- --test-threads=1 ipc_enqueue_matches_cli_enqueue_row_shape
npm run build
```

If a helper test lives in the `agm` binary target instead of the library, run it with `cd src-tauri && cargo test --bin agm -- --test-threads=1 select_instance_`.

## Do not

- Never key per-instance data by IDE flavor (`target_ide`); IPC and CLI pass instance ids only.
- Never trust a launcher wrapper PID alone when counting running instances for the refusal rule; use `is_instance_running`.
- Never close another instance's PID from any CLI or IPC path added here.
- Never let an action command fall back to the active instance silently; never treat `-n` as an instance flag.
- Never shell out to `agm` from an IPC command; never write prompt state to disk from the UI (`save_text_file` for resume files is removed).
- Never invent CLI commands beyond the NEW ones listed in spec section 6.9; label any further proposal NEW in the spec before coding it.
- Never mix `&&` and ternary forms in one JSX conditional; never use `== true` / `=== true`; boolean names start with `is`, `has`, `should`, or `can`; use `Id` / `id`, never `ID`.
- Never push before `cargo fmt -- --check`, `cargo clippy --all-targets --all-features`, and `npm run build` pass locally.
- E2E safety (D10): never kill Cursor or `agm-alim`; never switch or close the default instance; never delete `cli-switch-proof-6857`, `test-cli-flow-1743`, `clone-from-default-4424`, `new-empty-instance-4425`; test instances only with prefix `test-cli-flow-` or `test-diag-`; delete only instances this run created; do not click the Google data-collection checkbox; do not launch a replacement AGM GUI; read databases read-only; never print emails or tokens in full.
