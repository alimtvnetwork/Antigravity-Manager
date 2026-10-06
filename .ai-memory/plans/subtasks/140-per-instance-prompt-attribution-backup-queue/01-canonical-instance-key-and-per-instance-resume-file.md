# Subtask 01: Canonical Instance Key, Identity Triple, Schema Migration, Per-Instance Resume Hand-off

Status: pending

- Parent plan: `.ai-memory/plans/140-per-instance-prompt-attribution-backup-queue.md`
- Spec: `02-spec/21-app/140-per-instance-prompt-attribution-backup-queue/01-architecture-spec.md` (sections 6.1 to 6.4)
- Decisions implemented: D1, D2, D3
- Depends on: nothing. Subtasks 02 and 03 depend on this one.

## Goal

Give every prompt one canonical instance id that is stamped once at capture and never rewritten, make the identity triple `(instance_id, conversation_id, repo_path)` the key of every prompt table, migrate legacy rows safely, and stop two instances on one repo from overwriting each other's resume hand-off.

## Bug IDs covered

B01, B02, B03, B04, B06, B07, B10, B19 (schema and dedupe key part), B32 (tagging), B33.

## Files and functions to change

Line numbers were re-checked on 2026-10-06. Re-check with `gitmap aum search "<literal>" <file> --ext .rs` before editing; if a line moved, anchor on the `fn` signature.

| File | Function or item | Current line | Change |
|---|---|---|---|
| `src-tauri/src/modules/instance.rs` | `resolve_instance_id` | `:4372` (empty/active branch `:4375`, suffix match `:4413` to `:4432`) | Keep for input parsing only; add `canonical_instance_id` and `resolve_instance_input` next to it |
| `src-tauri/src/modules/instance.rs` | `get_instance_home_dir` | `:25` to `:33` | Use `canonical_instance_id`; return `Err` for unknown ids instead of creating `instances/<raw>/home` |
| `src-tauri/src/modules/repo_db.rs` | `gemini_dirs_for_instance` | `:824` to `:865` | Take a canonical id; never treat `"all"` as default |
| `src-tauri/src/modules/repo_db.rs` | `gemini_dirs_tagged` | `:867` | Take `InstanceScope`; return `(instance_id, dir)` pairs, one owner per dir |
| `src-tauri/src/modules/repo_db.rs` | `discover_running_prompts_from_antigravity` | `:956`; id built at `:1080`, stamp at `:1084` | Row id `prompt-{instance_id}-{cid}`; set `source_dir` |
| `src-tauri/src/modules/repo_db.rs` | `connect_db` schema block | `:149` to `:236` (`active_prompts` `:180`, `agm_conversation_sequences` `:226`) | Add columns, indexes, migration table (section 6.2 of the spec) |
| `src-tauri/src/modules/repo_db.rs` | sequence upsert | `:3786`, `:3791`, `:3799`, `:3806`, `:3812` | Add `instance_id` to every lookup and insert |
| `src-tauri/src/modules/repo_db.rs` | `backup_running_prompts` Layer 1 upsert | `:1195` to `:1203` | Conflict target becomes the new id; never update `instance_id` |
| `src-tauri/src/modules/repo_db.rs` | Layer 2 insert | `:1355` to `:1371`; `session_id` at `:1366`, `:1390` | `session_id` is a conversation id or `NULL`, never `project.id` |
| `src-tauri/src/modules/repo_db.rs` | all `resolve_instance_id` calls | `:514`, `:1496`, `:1537`, `:1706`, `:2088`, `:2090`, `:2534`, `:2618`, `:2650`, `:2700`, `:2785`, `:3274`, `:3283`, `:3288`, `:4028`, `:4152`, `:4235`, `:4316`, `:4442` | Replace with `canonical_instance_id` or plain equality on canonical values |
| `src-tauri/src/modules/repo_db.rs` | `normalize_path_for_compare` | `:2473` | Make `pub(crate)`; the only path normalizer for joins |
| `src-tauri/src/modules/repo_db.rs` | `resume_task_document` | `:780` | Unchanged content; add a new `write_resume_handoff` that uses it |
| `src-tauri/src/modules/repo_db.rs` | resume-file writers | `:1224`, `:1382`, `:1603`, `:2175`, `:3376`, `:3595` | Replace inline writes with `write_resume_handoff` |
| `src-tauri/src/modules/repo_db.rs` | resume-file readers | `:1283`, `:3534` | Ignore documents whose `instance_id` is not the target |
| `src-tauri/src/modules/notification_hub.rs` | resume-file reader | `:570` | Same instance check |
| `src-tauri/src/modules/agy_cleaner.rs` | resume-file reader | `:377` | Same instance check |
| `src-tauri/src/modules/agy_cleaner.rs` | `get_gemini_candidate_dirs`, `scan_conversations` | `:95` to `:124`, `:192` | Add `scan_conversations_for(InstanceScope, keep_count)` returning `(instance_id, ConversationItem)`, `seen` keyed by `(instance_id, cid)` |
| `src-tauri/src/bin/agm.rs` | `cmd_prompt_dispatch` resume write | `:3154` | Replace with `write_resume_handoff` (full CLI rework is subtask 03) |
| `src-tauri/src/modules/backup_prompts_db.rs` | `connect_backup_db` schema | `:140` to `:183` | Add `idx_prompt_backups_identity`; add migration twin |
| `src-tauri/src/modules/backup_prompts_db.rs` | dedupe and reactivation | `:343` to `:375` (UPDATEs at `:353`, `:371`) | Add `instance_id = ?` to both SELECTs; remove `instance_id = ?3` from both UPDATEs |

## Step-by-step implementation

### Step 1: research the resume-file reader (no code change)

1. Read-only search the installed Antigravity resources and the `agy` binary strings for `antigravity_resume_task`. Record each hit (file and offset) or "not found".
2. In a scratch repo opened only by a `test-cli-flow-*` instance, write a resume file and observe whether the IDE reacts. Record the observation as evidence; never claim a reaction that was not watched.
3. Write the answer under "Open Questions" item 1 of the spec and in the e2e evidence section of `02-component-and-e2e-spec.md` (ask the lead to merge it; do not edit files you do not own in a docs-only run).
4. Decision: if no external reader exists, use per-instance file names (Step 7 option A). If an external reader needs the legacy name, use the DB queue as source of truth and keep the legacy file as a pointer (Step 7 option B).

### Step 2: add the key functions in `src-tauri/src/modules/instance.rs`

1. Add `pub enum InstanceScope { One(String), All }` (derive `Debug, Clone, PartialEq, Eq`).
2. Add `pub fn canonical_instance_id(raw: &str) -> Result<String, String>` with the rules in spec section 6.1:
   1. Trim. `""`, `"default"`, `"__default__"` (case-insensitive) return the id of the registry entry with `is_default || id == "default"`, else `"default"`.
   2. Exact registry id (case-insensitive compare) returns the registry casing.
   3. Anything else, including `"all"` and `"active"`, returns `Err("unknown instance id '<raw>'; known: <ids>")`.
   4. Never call `get_active_instance_id`; never create folders.
3. Add `pub fn resolve_instance_input(input: &str) -> Result<String, String>`:
   1. Empty input returns `Err("instance required")`.
   2. Literal `active` returns `get_active_instance_id()`.
   3. Exact id or name, then `#N` / `N` / `ins-N` / `instance-N` sequence match, then suffix match.
   4. Suffix match with more than one candidate returns `Err("ambiguous instance '<input>': candidates <id1>, <id2>")`.
   5. Pass the result through `canonical_instance_id`.
4. Leave `resolve_instance_id` in place for current non-storage callers (`src-tauri/src/modules/cli.rs`, `telegram_inbound.rs:2281`, `:2299`, `auto_switcher.rs:2206`, and `instance.rs` callers). Do not change its behavior in this subtask.
5. Change `get_instance_home_dir` (`:25`) to:

```rust
pub fn get_instance_home_dir(instance_id: &str) -> Result<PathBuf, String> {
    let resolved_id = canonical_instance_id(instance_id)?;
    let instances_root = get_instances_dir()?;
    let home_dir = instances_root.join(&resolved_id).join("home");
    if !home_dir.exists() {
        fs::create_dir_all(&home_dir)
            .map_err(|e| format!("Failed to create instance home directory: {}", e))?;
    }
    Ok(home_dir)
}
```

6. Search every caller of `get_instance_home_dir` with `gitmap aum search "get_instance_home_dir(" src-tauri/src --ext .rs` and confirm each handles `Err` (most already use `if let Ok`). Re-read each enclosing function after editing.

### Step 3: tag gemini dirs in `src-tauri/src/modules/repo_db.rs`

1. `gemini_dirs_for_instance(instance_id)` (`:824`): call `canonical_instance_id`; on `Err` log and return an empty `Vec`. Named means "canonical id is not the default id". Remove the `"all"` and `"__default__"` string checks.
2. `gemini_dirs_tagged` (`:867`): change the parameter to `&InstanceScope`. For `All`, iterate registry ids (default first) and return `(id, dir)` for each existing dir. A dir path must appear once; if a named instance's dir equals a default dir (should not happen), keep the default owner and log.
3. Update callers of `gemini_dirs_tagged` (search `gemini_dirs_tagged(`) to pass a scope.

### Step 4: stamp once at capture

1. `discover_running_prompts_from_antigravity(instance_id)` (`:956`): take a canonical id. Build `p_id = format!("prompt-{}-{}", instance_id, cid)` at `:1080`. Set `source_dir` to `base_dir` (new `ActivePrompt` field `source_dir: Option<String>`; update every `ActivePrompt {` initializer, including tests; search `ActivePrompt {` with gitmap).
2. Layer 1 upsert (`:1195` to `:1203`): keep `ON CONFLICT(id)`; the `DO UPDATE SET` list must not contain `instance_id`. Add `source_dir = excluded.source_dir`.
3. Layer 2 insert (`:1355` to `:1371`): `session_id` is `NULL` unless a conversation id is known from the tagged scan of the same instance. Remove `"session_id": project.id` from the resume payload at `:1390` (the hand-off now comes from `write_resume_handoff`).

### Step 5: schema changes in `connect_db` (`:149`)

1. Add idempotent migrations (ignore "duplicate column" errors the same way `:199` to `:202` does):
   - `ALTER TABLE active_prompts ADD COLUMN status_reason TEXT`
   - `ALTER TABLE active_prompts ADD COLUMN source_dir TEXT`
   - `ALTER TABLE active_prompts ADD COLUMN attempts INTEGER NOT NULL DEFAULT 0`
2. Create `agm_schema_migrations(name TEXT PRIMARY KEY, applied_at INTEGER NOT NULL)`.
3. Call `migrate_prompt_instance_ids_v140(&conn)` (Step 6) before creating the partial unique index.
4. Create `idx_active_prompts_identity` exactly as in spec section 6.2.1, after migration (the index fails if duplicates remain; migration resolves them first).
5. Sequence queries (`:3786` to `:3812`): add `instance_id` to the SELECT `WHERE`, the UPDATE `WHERE`, and the INSERT conflict target.

### Step 6: one-time migration `migrate_prompt_instance_ids_v140`

1. If `agm_schema_migrations` has `v140_instance_ids`, return `Ok(0)`.
2. Copy the database file to `<db>.pre-v140.bak` (use `get_repo_db_path`, `:142`). If the copy fails, log `[Migration v140] backup copy failed; migration skipped` and return `Err`. Do not proceed.
3. Begin a transaction.
4. `UPDATE active_prompts SET instance_id = ?1 WHERE instance_id = '__default__'` with the canonical default id. Same for `running_projects`, `agm_project_sequences`, `agm_conversation_sequences`.
5. Rename legacy status: `UPDATE active_prompts SET status = 'queued' WHERE status = 'pending'`.
6. Build an owner index once: for each `(id, dir)` in `gemini_dirs_tagged(&InstanceScope::All)`, read conversation ids from `conversation_summaries.db` (read-only, `SQLITE_OPEN_READ_ONLY | SQLITE_OPEN_URI`) and the names of `brain/<cid>` folders. Map `cid -> set of instance ids`.
7. For each row with `instance_id IS NULL OR instance_id IN ('', 'all')`:
   1. Exactly one owner for `session_id`: set `instance_id`; if the id is `prompt-{cid}`, rewrite it to `prompt-{owner}-{cid}` (skip the rewrite and mark `orphaned` with `legacy_owner_ambiguous` if the new id already exists).
   2. Otherwise: `status = 'orphaned'`, `status_reason = 'legacy_owner_unknown'` or `'legacy_owner_ambiguous'`, keep `instance_id = ''`.
8. Rebuild `agm_conversation_sequences` as `agm_conversation_sequences_v2` with `PRIMARY KEY (instance_id, conversation_id)` and `seq_id INTEGER NOT NULL UNIQUE`; copy rows; drop old; rename.
9. Insert `('v140_instance_ids', now)` into `agm_schema_migrations`; commit.
10. Backup DB twin in `src-tauri/src/modules/backup_prompts_db.rs` (`migrate_backup_instance_ids_v140`): same steps 1 to 3, then `prompt_backups` rows with `instance_id IS NULL OR instance_id IN ('', '__default__', 'all')` use the same owner index on `conversation_id`; on no unique owner set `status = 'orphaned'` and `is_restored = 1` with `restored_at = NULL` so restore never picks them, and keep `instance_id = ''`.

### Step 7: per-instance resume hand-off

1. Add to `src-tauri/src/modules/repo_db.rs`:

```rust
pub enum ResumeHandoffOutcome {
    PerInstanceFile(PathBuf),
    LegacyFile(PathBuf),
    DbOnly { legacy_skipped_owners: Vec<String> },
    Failed(String),
}

pub fn write_resume_handoff(prompt: &ActivePrompt, status: &str) -> ResumeHandoffOutcome;
pub fn repo_owner_instances(repo_path: &str) -> Vec<String>;
```

2. `repo_owner_instances` returns canonical ids whose `running_projects` rows (normalized `repo_path`) or workspace folders contain the repo.
3. `write_resume_handoff`:
   - Option A (no external reader): write `<repo>/.antigravity_resume_task.<instance_id>.json` from `resume_task_document`; also write the legacy file only if `repo_owner_instances(repo).len() == 1`; otherwise log `[ResumeHandoff] legacy file skipped: repo <path> owned by <ids>`.
   - Option B (external reader needs legacy name): the `active_prompts` row is the source of truth; write the legacy file only when exactly one owner; otherwise return `DbOnly` and log.
   - If no write succeeded and the DB row is missing, return `Failed(reason)`; the caller marks the row `failed` with `handoff_write_failed`.
4. Replace every writer listed in the table above with `write_resume_handoff`. The writers at `:3376` to `:3390` must stop building their own JSON (they drop `session_id` today).
5. Readers (`repo_db.rs:1283`, `:3534`; `notification_hub.rs:570`; `agy_cleaner.rs:377`): read the per-instance file first, then the legacy file, and skip any document whose `instance_id` (after `canonical_instance_id`) differs from the instance being evaluated.
6. Add `.antigravity_resume_task.*.json` next to the existing names in the gitignore helper list at `src-tauri/src/bin/agm.rs:8836` to `:8839`.

### Step 8: backup dedupe key (schema part of B19)

In `backup_active_running_prompts_for_instance` (`src-tauri/src/modules/backup_prompts_db.rs:230`):

1. The unrestored lookup at `:345` becomes `WHERE instance_id = ?4 AND (prompt_id = ?1 OR (project_path = ?2 AND prompt_text = ?3)) AND is_restored = 0`.
2. The UPDATE at `:353` becomes `UPDATE prompt_backups SET backup_batch_id = ?1, created_at = ?2, is_restored = 0 WHERE id = ?3`.
3. The restored lookup at `:363` drops `OR instance_id IS NULL OR instance_id = ''`.
4. The UPDATE at `:371` drops `instance_id = ?3`.
5. Create `idx_prompt_backups_identity` after the migration twin runs.

The candidate selection fix (`:249` to `:252`) and the conversation matching fix (`:300` to `:304`) belong to subtask 02; do not change them here.

## Unit tests to add

Put tests in the `#[cfg(test)]` module of each file. Tests that touch the registry or databases use a temp `HOME`/`USERPROFILE` the way existing tests in `src-tauri/src/modules/instance.rs` do; never touch the real `%USERPROFILE%\.antigravity_tools`.

| Test name | File | Asserts |
|---|---|---|
| `canonical_instance_id_maps_default_aliases` | `instance.rs` | `""`, `"default"`, `"__default__"`, `"DEFAULT"` all return the default id |
| `canonical_instance_id_rejects_unknown_all_and_active` | `instance.rs` | `"nope"`, `"all"`, `"active"` return `Err`; error lists known ids |
| `canonical_instance_id_never_returns_active_instance` | `instance.rs` | With active set to a named instance, `canonical_instance_id("")` still returns the default id |
| `resolve_instance_input_rejects_ambiguous_suffix` | `instance.rs` | Registry with `a-8159` and `b-8159`: input `8159` returns `Err` naming both |
| `resolve_instance_input_accepts_seq_and_name` | `instance.rs` | `#2`, `2`, display name resolve to the same id |
| `get_instance_home_dir_does_not_create_unknown_folder` | `instance.rs` | `get_instance_home_dir("__ghost__")` returns `Err` and `instances/__ghost__` does not exist |
| `gemini_dirs_tagged_all_never_stamps_all` | `repo_db.rs` | No returned pair has id `"all"`; each dir appears once |
| `captured_prompt_id_includes_instance` | `repo_db.rs` | Fixture summaries DB under a named home gives `prompt-<id>-<cid>` and `instance_id == <id>` |
| `same_conversation_id_in_two_instances_gives_two_rows` | `repo_db.rs` | Two fixture homes with the same `cid` produce two rows with distinct ids and instance ids; two sequence rows |
| `layer1_upsert_never_rewrites_instance_id` | `repo_db.rs` | Insert row for A, upsert same id from B's capture path: `instance_id` stays A |
| `migration_assigns_unique_owner_for_empty_instance_row` | `repo_db.rs` | Legacy row with empty `instance_id` and a `cid` found only in A becomes A with new id format |
| `migration_orphans_ambiguous_or_unknown_rows` | `repo_db.rs` | `cid` in two instances, or in none: `status = 'orphaned'`, reason set, `instance_id = ''` |
| `migration_is_idempotent` | `repo_db.rs` | Second run returns 0 and changes nothing |
| `migration_aborts_when_backup_copy_fails` | `repo_db.rs` | Unwritable backup path: no rows changed, `Err` returned |
| `two_instances_same_repo_do_not_share_legacy_handoff` | `repo_db.rs` | With owners A and B on one repo, `write_resume_handoff` for A does not write the legacy file and returns the skip outcome |
| `single_owner_repo_keeps_legacy_handoff` | `repo_db.rs` | One owner: legacy file written, content has `instance_id`, `session_id`, `conversation_id` |
| `resume_reader_ignores_other_instance_document` | `repo_db.rs` | Legacy file with `instance_id = A` is ignored when evaluating B |
| `backup_dedupe_does_not_steal_other_instance_row` | `backup_prompts_db.rs` | Row owned by A with same text and path; backup for B inserts a new row for B and A's row keeps `instance_id = A` |
| `instance_id_and_ide_flavor_are_not_confused` | `repo_db.rs` | A call path that has both an instance id (`"default"`) and an IDE flavor (`"ide"`) uses the instance id for the row (strictly-avoid rule) |

## Acceptance criteria

AC-01 (evidence for Step 1 and the feasibility claim), AC-02, AC-03, AC-04 (input layer only), AC-05, AC-06, AC-07, AC-08, AC-09 (dedupe part), AC-11, AC-12, AC-13, AC-21 (id format part). Definitions are in spec section 8.

## Verification commands

Run from the repo root unless noted:

```text
cd src-tauri && cargo fmt -- --check
cd src-tauri && cargo clippy --all-targets --all-features
cd src-tauri && cargo test --lib -- --test-threads=1 canonical_instance_id
cd src-tauri && cargo test --lib -- --test-threads=1 resolve_instance_input
cd src-tauri && cargo test --lib -- --test-threads=1 get_instance_home_dir_does_not_create_unknown_folder
cd src-tauri && cargo test --lib -- --test-threads=1 migration_
cd src-tauri && cargo test --lib -- --test-threads=1 same_conversation_id_in_two_instances_gives_two_rows
cd src-tauri && cargo test --lib -- --test-threads=1 handoff
cd src-tauri && cargo test --lib -- --test-threads=1 backup_dedupe_does_not_steal_other_instance_row
```

No `npm run build` is needed for this subtask (no `src/` change).

## Do not

- Never key per-instance data by IDE flavor (`target_ide`, values like `"ide"` or `"agy"`). The default instance id is `"default"`; `src-tauri/src/modules/account.rs:1743` stays `"default"`.
- Never trust a launcher wrapper PID alone as proof that an instance runs; use `is_instance_running` with its bounded fallback search.
- Never close another instance's PID, and never call `close_antigravity(None)` from a single-instance path.
- Never call `get_active_instance_id` or `resolve_instance_id` from storage code.
- Never add a `LIKE %name%`, substring `contains`, or folder-name prefix join.
- Never rename the resume file before Step 1 has an answer.
- Never run the migration without the `.pre-v140.bak` copy succeeding first.
- Never edit a code range that starts or ends inside a block without re-reading the whole enclosing function; never add a struct field without updating every initializer, including tests.
- Never push before `cargo fmt -- --check` and `cargo clippy --all-targets --all-features` pass locally.
- E2E safety (D10): never kill Cursor or `agm-alim`; never switch or close the default instance; never delete `cli-switch-proof-6857`, `test-cli-flow-1743`, `clone-from-default-4424`, `new-empty-instance-4425`; create test instances only with prefix `test-cli-flow-` or `test-diag-` and delete only those this run created; read databases read-only (`file:<path>?mode=ro`); never print emails or tokens in full.
