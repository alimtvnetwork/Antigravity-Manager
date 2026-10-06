# Plan 140 Blind Executor: Start Here

Read this whole file before touching any code. It is written for an AI that has never seen this repository and has no memory of how the plan was made. If something in a step file disagrees with this file, this file wins. If something in this file disagrees with `agents.md` at the repo root, `agents.md` wins.

## 1. What you are building (one paragraph)

AGM (Antigravity Manager) runs several copies ("instances") of the Antigravity IDE on one machine. Each instance has its own home folder and its own `.gemini` store. AGM captures the prompt that is running in an instance, backs it up before an account switch, queues it, and sends it again afterwards. Today the instance a prompt belongs to is guessed again at almost every step, so a prompt typed in instance A can be backed up, queued, or re-sent as if it belonged to instance B or to the default instance. You will make the instance id a fixed part of every prompt row, stamped once when the prompt is captured and never rewritten, and you will make every CLI command and UI action name its instance.

## 2. Glossary

| Term | Meaning |
|---|---|
| Instance | One Antigravity IDE profile managed by AGM. Registry file: `<data dir>/instances/instances.json`. Struct: `InstanceConfig` in `src-tauri/src/models/instance.rs`. |
| Default instance | The registry entry with `is_default == true` (normally id `"default"`). It uses the real user home. |
| Named instance | Any non-default instance. Its home is `<data dir>/instances/<id>/home`. |
| Canonical instance id | The exact registry `id` string. Never `""`, `"all"`, `"active"`, `"__default__"`, or an IDE flavor. |
| IDE flavor | `"ide"` or `"agy"` (`target_ide`, `effective_target`). It is NOT an instance id. Never store it in an `instance_id` column. |
| Conversation id (`cid`) | Antigravity's conversation UUID. Stored in `session_id` / `conversation_id`. |
| Identity triple | `(instance_id, conversation_id, repo_path)`. Two rows with different `instance_id` are different prompts even if the other two match. |
| Resume hand-off file | `<repo>/.antigravity_resume_task.json`, written by AGM before it re-sends a prompt. |
| `agy` | The Antigravity command-line binary AGM uses to send a prompt (`spawn_prompt_via_agy`). |
| Repo DB | SQLite at `get_repo_db_path()` (`src-tauri/src/modules/repo_db.rs`). Tables `active_prompts`, `running_projects`, `agm_conversation_sequences`, `agm_project_sequences`. |
| Backup DB | SQLite opened by `connect_backup_db` (`src-tauri/src/modules/backup_prompts_db.rs`). Table `prompt_backups`. |
| Gate | The commands at the end of each step. A step is done only when its gate exits 0. |

## 3. Decisions already made (do not reopen)

The spec lists open questions. They are settled here so you never have to guess.

| ID | Question | Decision | Why |
|---|---|---|---|
| DR-1 | Who reads `.antigravity_resume_task.json`, and may it be renamed? | **Option B.** The `active_prompts` row is the source of truth. The file name stays `.antigravity_resume_task.json`. AGM writes it only when exactly one instance owns the repo; otherwise it writes nothing and returns `ResumeHandoffOutcome::DbOnly`. Never create `.antigravity_resume_task.<id>.json`. | Safe whether or not an external reader exists. Renaming could break an unknown reader. |
| DR-2 | Does `agy` have a "resume conversation by id" flag? | Assume **no**. Keep the current `-p` call. On a successful send without a resume flag, set `status_reason = 'new_chat_no_resume_flag'`. Only add a flag if `agy --help` output, saved as evidence, literally shows it. | Never invent flags. |
| DR-3 | Is the default instance id literally `"default"`? | Do not assume. Use the registry entry with `is_default || id == "default"`; if none, `"default"`. This is already how `resolve_instance_id` treats `"default"` in `src-tauri/src/modules/instance.rs`. | Covers both cases. |
| DR-4 | Change the saved-PID fast path in `is_instance_running`? | **No.** Do not edit `is_instance_running` at all. Ambiguity `.ai-memory/ambiguous-questions/01-new-ambiguity/02-is-instance-running-saved-pid-fast-path.md` is still open. | Out of scope. |
| DR-5 | Is `USERPROFILE` honoured by Antigravity for named instances? | Assumed yes (the launcher sets it). Proven only by E2E-01 (step 22). If E2E-01 shows a named instance writing into the default `.gemini`, stop and report; do not work around it. | Feasibility gate. |
| DR-6 | macOS | Code must compile on all platforms (`cfg` blocks unchanged). Live macOS verification is out of scope. | No macOS machine in the loop. |
| DR-7 | Legacy rows whose owner cannot be proven | Mark `status = 'orphaned'`, keep `instance_id = ''`, never auto-dispatch them. | Never guess an owner. |

## 4. How every step file is laid out

Each step file `NN-<slug>.md` has the same sections, in this order:

1. **Depends on**: step numbers that must be committed first.
2. **Files you may edit**: the only files you may change in this step.
3. **Find it**: for each change, the `fn` signature and one unique literal to search for. Line numbers are hints only.
4. **Current code**: the code as it is today, copied verbatim. If what you find is different, STOP (section 6).
5. **New code**: the replacement, complete. Copy it exactly; change only what the compiler forces (it names the line).
6. **Tests**: complete test functions to paste into the file's existing `#[cfg(test)] mod tests`.
7. **Gate**: the exact commands. All must exit 0.
8. **Commit**: the exact commit message.
9. **Done when**: a checklist you tick before committing.

## 5. The loop for every step

1. `git pull --rebase` on `main`. Confirm `git log --oneline -1` contains the previous step's commit message.
2. Read the step file top to bottom.
3. For each change, open the file and find the anchor with GitMap: `gitmap aum search "<literal>" <file> --ext .rs`. Read the whole enclosing function before editing.
4. Compare the "Current code" block with what is there. Whitespace differences are fine. Any other difference: STOP.
5. Apply the "New code". Add the tests.
6. Run the gate from the repo root:

```text
$env:CARGO_TARGET_DIR="$PWD\src-tauri\target"; $env:CARGO_BUILD_JOBS="1"
cd src-tauri; cargo fmt; cargo fmt -- --check; cargo clippy --all-targets --all-features; cargo test --lib -- --test-threads=1 <test filter from the step>; cd ..
```

7. If the gate fails, read the first error only, fix it, run the gate again. After two failed attempts on the same error, STOP.
8. Stage only the files listed in "Files you may edit" (explicit paths, never `git add -A`), commit with the exact message, push. If the push is rejected because the remote is ahead: `git pull --rebase`, then push again. Never force-push. Never `--no-verify`.

## 6. Stop rules (report instead of improvising)

Stop and write a short report (step number, what you expected, what you found, the exact error) when:

- The "Current code" block does not match the file (someone changed it).
- The gate fails twice on the same error.
- A change would need a file that is not in "Files you may edit".
- A step asks for live evidence you cannot produce (a human must type in an IDE window). Write `NOT VERIFIED: needs a human` instead of claiming it.
- Anything would kill a process, switch the default instance, or delete an instance not created by this run.

## 7. Rules that apply to every step

**Code**

- Pure core plus thin loader. Every new function that needs the registry or a database takes it as a parameter (`&InstanceRegistry`, `&Connection`). A thin wrapper loads the registry or opens the database and calls the core. Tests call only the core with an in-memory registry or `Connection::open_in_memory()` plus `init_tables(&conn)`.
- Never call `load_registry()` from a test. With no registry file it creates a default instance pointing at the real Antigravity data folder and writes settings into it.
- Never set `HOME`, `USERPROFILE`, `APPDATA`, or `ABV_DATA_DIR` in a new test. If a test truly needs a data dir, lock `crate::modules::account::TEST_DATA_DIR_MUTEX`, set `ABV_DATA_DIR` to a temp folder, pre-write `instances/instances.json`, and restore the variable at the end (pattern: `test_set_current_account_id_with_target` in `src-tauri/src/modules/account.rs`). Prefer the pure core instead.
- When you add a field to a struct (for example `source_dir` on `ActivePrompt`), search every initializer with `gitmap aum search "ActivePrompt {" src-tauri/src --ext .rs` and update all of them, including tests.
- Before you delete a `let`, search for its other uses in the function.
- Never edit a range that starts or ends inside a block without re-reading the whole function.
- Never use `LIKE '%..%'`, `.contains()` on paths, or folder-name prefix matching to join prompts to projects. Compare normalized paths with `normalize_path_for_compare` (made `pub(crate)` in step 08).
- Never call `get_active_instance_id` or `resolve_instance_id` from storage code. Use `canonical_instance_id`.
- Booleans are named `is_`, `has_`, `should_`, `can_`. Use `id` / `Id`, never `ID`. No `== true`.
- Do not add comments that narrate the change. Match the comment density of the surrounding code.

**Process**

- Search only with GitMap: `gitmap aum search`, `gitmap lf`, `gitmap cat`. A backslash in a pattern is interpreted, and a pattern starting with `-` is read as a flag; use a plain literal or a regex such as `node.{3,12}-n`.
- Relative paths only in docs, logs and commit messages.
- One step, one commit. Never combine steps.
- Never disable a CI check. Never commit secrets, caches, or files outside the step's list.

**E2E safety (step 22 and any live check)**

- Never kill Cursor or `agm-alim`. Never switch or close the default instance.
- Never delete `cli-switch-proof-6857`, `test-cli-flow-1743`, `clone-from-default-4424`, `new-empty-instance-4425`.
- Create test instances only with the prefix `test-cli-flow-` or `test-diag-`, and delete only those this run created.
- Never run `agm tif` or `agm instances rm-all`.
- Open databases read-only (`file:<path>?mode=ro`). Never print emails or tokens in full.
- Do not click the Google data-collection checkbox. Do not launch a replacement AGM GUI.

## 8. Step order

| Step | File | Area | Depends on |
|---|---|---|---|
| 01 | `01-instance-scope-and-canonical-id.md` | `InstanceScope`, `canonical_instance_id`, `resolve_instance_input` | none |
| 02 | `02-instance-home-dir-rejects-unknown.md` | `get_instance_home_dir` returns `Err` for unknown ids | 01 |
| 03 | `03-schema-columns-and-source-dir-field.md` | new columns, `agm_schema_migrations`, `ActivePrompt.source_dir` | 01 |
| 04 | `04-tagged-gemini-dirs.md` | `gemini_dirs_for_instance`, `gemini_dirs_tagged(&InstanceScope)` | 01 |
| 05 | `05-capture-stamps-instance-once.md` | row id `prompt-{instance}-{cid}`, Layer 1 and Layer 2 upserts | 03, 04 |
| 06 | `06-migration-v140-repo-db.md` | one-time migration, identity index, sequences v2 | 03, 04 |
| 07 | `07-backup-db-migration-and-dedupe.md` | backup DB migration twin, dedupe key | 06 |
| 08 | `08-storage-uses-canonical-id.md` | replace `resolve_instance_id` in `repo_db.rs`; `normalize_path_for_compare` `pub(crate)` | 01 |
| 09 | `09-resume-handoff-db-first.md` | `write_resume_handoff` (DR-1), `repo_owner_instances`, readers | 05, 08 |
| 10 | `10-status-machine-and-dispatch-one.md` | status constants, claim, finish, sweep, `dispatch_one_prompt`, canonical `spawn_prompt_via_agy` | 09 |
| 11 | `11-scheduler-consumes-queued-only.md` | `check_and_dispatch_enqueued_prompts` | 10 |
| 12 | `12-single-restore-path.md` | `restore_backed_up_prompts_for_instance`, launch reinject | 10 |
| 13 | `13-resend-never-touches-running.md` | `resend_running_commands_for_instance` | 10 |
| 14 | `14-running-detection-live-turn.md` | `is_live_turn`, liveness sweep, Gate 4, execution status | 05, 08 |
| 15 | `15-backup-candidates-exact-instance.md` | `backup_active_running_prompts_for_instance` candidates | 07, 04 |
| 16 | `16-switch-flows-restore-source.md` | auto switcher, desktop switch, Telegram | 12, 15 |
| 17 | `17-shared-enqueue-send-list-trace.md` | `enqueue_prompt_for_instance`, `send_prompt_now_for_instance`, list, trace, `backup_prompts_for_instance` | 10, 12 |
| 18 | `18-cli-select-instance-and-lists.md` | `select_instance`, `instance_label`, `-i` on list commands | 17 |
| 19 | `19-cli-actions-and-new-subcommands.md` | `prompt enqueue`, `prompts queue ls`, `prompts trace`, refusal rule on actions | 18 |
| 20 | `20-ipc-wrappers-and-registration.md` | new Tauri commands, `lib.rs` registration | 17 |
| 21 | `21-ui-instance-aware-actions.md` | `PromptTreeViewModal.tsx`, `Instances.tsx`; `npm run build` | 20 |
| 22 | `22-live-e2e-and-evidence.md` | run `05-e2e-runbook-and-evidence-capture.md` in full | 21 |

Before step 01, run E2E-00 and E2E-01 from `../05-e2e-runbook-and-evidence-capture.md` once to record the baseline. If E2E-01 fails, stop (DR-5).

## 9. Where the background lives

You do not need these to execute a step, but they explain why:

- `../../../140-per-instance-prompt-attribution-backup-queue.md`: parent plan.
- `../01-canonical-instance-key-and-per-instance-resume-file.md`, `../02-backup-queue-restore-pipeline-fixes.md`, `../03-cli-and-ipc-parity.md`: the original subtasks these steps were cut from. If a step file is unclear, the matching section there is the tie-breaker, except where section 3 above overrides it.
- `02-spec/21-app/140-per-instance-prompt-attribution-backup-queue/01-architecture-spec.md`: bug list B01 to B33 and acceptance criteria AC-01 to AC-32.
- `.ai-memory/memory/learned/24-code-failure-decoding-and-fast-codebase-learning-skill.md`: how past edits in this repo broke the build, and how to avoid it.
