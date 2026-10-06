# 140 - AI Instruction: Per-Instance Prompt Attribution for Running, Backup, Queue, and Restore (CLI + UI + Multi-Instance E2E)

Status: pending
Created: 2026-10-06
Owner: any AI agent (this file is the instruction; follow it in order)
Spec: `02-spec/21-app/140-per-instance-prompt-attribution-backup-queue/01-architecture-spec.md` and `02-spec/21-app/140-per-instance-prompt-attribution-backup-queue/02-component-and-e2e-spec.md`
RCA: `02-spec/22-app-issues/34-per-instance-prompt-identity-lost-rca.md`
Subtasks: `.ai-memory/plans/subtasks/140-per-instance-prompt-attribution-backup-queue/` (index: `index.md`)
Implementers start here: `.ai-memory/plans/subtasks/140-per-instance-prompt-attribution-backup-queue/06-blind-executor/00-start-here.md` (steps 01 to 22, decisions DR-1 to DR-7 settled)
Memory: `.ai-memory/memory/learned/25-per-instance-prompt-attribution-and-e2e-testing.md`
Read first: `.ai-memory/what-to-read.md`, `.ai-memory/memory/learned/24-code-failure-decoding-and-fast-codebase-learning-skill.md`, `.ai-memory/strictly-avoid.md`

## User Request (Verbatim)

> You have learned how to do the end-to-end testing, and you know I have given you the prompt to test the anti-gravity manager based on the profile, based on the switch, auto-switch, how it's going to do. Now the problem is, when we have multiple instances, or even same instances, the current prompt running, taking the backup or the enqueue prompt, all of these are not right from the CLI methods and how the code is. 1st thing that I want you to do is observe the code where the bug is, 2nd, I want you to create an AI instruction in the plans folder specifying everything that AI needs to do in order to make this thing successful. All the test cases, all the end-to-end test cases that AI would execute in the machine based on the multiple instances. The things would be a little bit complicated because every time AI needs to know the instance name now in order to know which prompts is coming to which. 1st, you need to confirm that is it possible using anti-gravity, and you need to craft in there the CLI methods and also inside the UI how it would reuse the functionalities to make sure that it is working. I don't want you to code, I want you to think, and I want you to write that plan and let me know finally where you have written that plan. Also, so far what you have coded and learned, make sure that these are everything written inside the AI memory so that anytime I throw this conversation, you or any AI would read it and understand what went wrong and how you did these codes, what are the best practices and how to make sure the code does not break, and what are the end-to-end tests that we have discussed. So everything I've won inside the AI memory properly, and inject that in the what to read file. Make sure of that, your memories are safe in there, and make sure you do a git commit and push.

The full request including the actionable items list is preserved verbatim in the architecture spec, section "User Request (Verbatim)".

## 1. What is wrong (observed in code, 2026-10-06, HEAD 93b9c825)

Root cause in one sentence: there is no canonical instance key that is set once when a prompt is captured and carried unchanged to dispatch; every layer re-derives it ("default", "__default__", "", the active instance, fuzzy `resolve_instance_id`, substring or `LIKE` matches, path-only joins), and the resume hand-off file `.antigravity_resume_task.json` is per repo instead of per (instance, repo, conversation), so identity is lost whenever two instances share a repo or a key fails to normalize.

Highest-impact sites (lead-verified with `gitmap aum search`; full table of 33 sites B01..B33 in the architecture spec):

| Flow | Site | Problem |
|---|---|---|
| Key | `src-tauri/src/modules/instance.rs:4376` | empty or "active" id resolves to whichever instance is active |
| Backup | `src-tauri/src/modules/backup_prompts_db.rs:249`, `:252` | rows with empty `instance_id` match every instance |
| Backup | `src-tauri/src/modules/backup_prompts_db.rs:353`, `:371` | `UPDATE prompt_backups SET ... instance_id = ?3` re-stamps another instance's backup |
| Queue | `src/components/instances/PromptTreeViewModal.tsx:1335` | UI calls IPC `enqueue_prompt`, which does not exist in Rust (0 hits in `src-tauri/src`) |
| Queue | `src-tauri/src/modules/repo_db.rs:2054` `check_and_dispatch_enqueued_prompts` | consumes switch backups as queue items, marks dispatched even when not sent |
| Restore | `src-tauri/src/modules/repo_db.rs:1224`, `1283`, `1382`, `1603`, `2175`, `3376`, `3534`, `3595` | resume file joined to `repo_path`: two instances on one repo overwrite each other |
| Restore | `src-tauri/src/modules/auto_switcher.rs:1691` | a `"default"` target becomes `None` (re-send every instance) |
| CLI | `src-tauri/src/bin/agm.rs:2926` | `-n` means `--node` (remote SSH), not instance; only `-i/--instance` targets an instance |
| CLI | `src-tauri/src/bin/agm.rs` `cmd_prompt_dispatch`, `cmd_which_prompts_running`, `cmd_resend_running_commands` | default to the active instance, join by path only, back up one instance and re-send all |

## 2. Is it possible with Antigravity? (feasibility)

Yes for named instances, with one live proof still required:

- AGM launches each named instance with `USERPROFILE`, `APPDATA`, `LOCALAPPDATA` (HOME on Unix) set to `<instances>/<id>/home` (`src-tauri/src/modules/instance.rs:3594-3618`). Antigravity then writes that instance's conversations to `<instances>/<id>/home/.gemini/{antigravity,antigravity-ide,antigravity-cli}`, which `gemini_dirs_for_instance` reads (`src-tauri/src/modules/repo_db.rs:824`).
- The default instance owns the global `%USERPROFILE%\.gemini` store.
- Antigravity writes no instance tag of its own (`app_data_dir` holds only product names). The instance link is therefore (a) the folder the store lives in and (b) the `instance_id` AGM stamps at capture time.
- On this machine on 2026-10-06: 5 registered instances; per-instance `conversation_summaries.db` files exist for `cli-switch-proof-6857` and `test-cli-flow-1743` with 0 rows, so no live conversation has been seen in a per-instance store yet. E2E-01 must prove it before anything else is trusted.
- macOS `open -a` may drop the environment: treat macOS as unverified.

## 3. What the AI must do (ordered; do not skip, do not reorder)

| Step | Subtask file | Content | Depends on |
|---|---|---|---|
| 1 | (none) | Read the files in "Read first" above, then the two spec files and the RCA. | - |
| 2 | `05-e2e-runbook-and-evidence-capture.md` (E2E-00, E2E-01 only) | Safety snapshot, then the feasibility gate. If E2E-01 fails, STOP and report; record an ambiguity. | 1 |
| 3 | `01-canonical-instance-key-and-per-instance-resume-file.md` | D1 canonical key, D2 identity triple and schema migration, D3 resume hand-off (first find who reads the file). | 2 |
| 4 | `02-backup-queue-restore-pipeline-fixes.md` | D4 state machine and one restore path, D5 running detection, D6 switch flows. | 3 |
| 5 | `03-cli-and-ipc-parity.md` | D7 CLI flags and NEW commands, D8 IPC wrappers and UI reuse. | 3, 4 |
| 6 | `04-e2e-multi-instance-test-catalog.md` + `05-e2e-runbook-and-evidence-capture.md` | Run every E2E case (E2E-00..E2E-19), capture evidence, report one line per case. | 3, 4, 5 |
| 7 | (none) | Update memory (`learned/25`), RCA 34 status, `.ai-memory/what-to-read.md`, move this plan to `completed/`. | 6 |

Each implementation subtask ends with: `cargo fmt -- --check`, `cargo clippy --all-targets --all-features`, targeted `cargo test --lib -- --test-threads=1 <name>`, and `npm run build` when `src/` changed (AGENTS.md pre-flight). Release only when the user asks.

## 4. How the AI knows which prompt belongs to which instance (rules for every step)

1. Test instances are created only with prefix `test-cli-flow-a-<rand4>`, `test-cli-flow-b-<rand4>`, and the clone `test-cli-flow-c-<rand4>` (spec 94 law L8). `agm instances create` appends a 4-digit timestamp (`src-tauri/src/modules/instance.rs:838`), so the real registry id is `test-cli-flow-a-<rand4>-<ts4>`: always read the id from `--json` output, never assume it.
   - Scratch repos used by tests must have no `.git` folder, because `agm prompt` runs `git pull` when the current folder has one.
   - Commands that dispatch prompts (`rrp`, `rrc`, `qs`, account switch) can send other instances' prompts, including default, until subtasks 01 and 02 ship. Run them today only in sandbox mode: `ABV_DATA_DIR` pointed at a copy of the data folder (`src-tauri/src/modules/account.rs:836`).
   - Today's `brp` pulls empty-`instance_id` rows into the target; run it live only after confirming the live databases have no such rows.
   - On a build that contains subtask 01, the first live command migrates the real databases, so E2E-14 (migration on a copy) runs right after E2E-00.
2. Every test prompt embeds a marker `E2E140-<runid>-<instance>-<case>`, so it can be found in any store by text.
3. Evidence is read with read-only SQLite (`file:<path>?mode=ro`) from `repo_prompts.db`, the backup DB, and each instance's `conversation_summaries.db`, always with `instance_id` in the `WHERE` clause. The exact queries are in the e2e spec section 4.
4. Every CLI call that acts on prompts passes `-i <instance>`. Never rely on the active instance.
5. After the CLI work in step 5 ships, `agm prompts trace -i <instance>` (NEW) prints the identity chain for each prompt.

## 5. CLI methods and UI reuse (summary; full tables in the specs)

- Existing and verified: `agm instances`, `agm status`, `agm history`, `agm wpr`, `agm prompts`, `agm prompt -i`, `agm brp -i`, `agm rrp -i`, `agm running-prompts [ls|backup|restore|export|import]`, `agm running-projects`, `agm tree [all]`, `agm tif`, `agm observe`, `agm qs [instance] [--once]`, `agm rrc`.
- Target: every prompt command takes `-i/--instance <id|#seq|name>` and `--json` with `instance_id` and `instance_name` on every row; action commands refuse to run without `-i` when more than one instance is running.
- NEW: `agm prompt enqueue -i <instance> --repo <path> [--conversation <cid>] "<text>"`, `agm prompts queue ls -i <instance>`, `agm prompts trace -i <instance> [--conversation <cid>]`.
- UI: NEW IPC `enqueue_prompt`, `backup_running_prompts_for_instance`, `restore_running_prompts_for_instance`; optional `instance_id` on `list_backed_up_prompts`, `list_running_projects`; `src/pages/Instances.tsx:281` passes the instance id to `get_project_conversation_tree`. Every IPC command wraps the same Rust function the CLI calls (spec 94 law L4). Every prompt row in the UI shows the instance badge.

## 6. E2E test cases (catalog; full steps in the e2e spec)

E2E-00 safety snapshot; E2E-01 feasibility gate (per-instance profile honoured); E2E-02 canonical key resolution; E2E-03 running detection single instance; E2E-04 two instances, different repos; E2E-05 two instances, same repo (hand-off collision); E2E-06 same conversation id in two instances (clone); E2E-07 backup isolation and idempotence; E2E-08 enqueue via CLI; E2E-09 enqueue via UI; E2E-10 queue vs backup separation; E2E-11 instance account switch keeps the prompt in the same conversation, other instance untouched; E2E-12 auto-switch cross-instance; E2E-13 default instance paths by unit test only (never switch default live); E2E-14 legacy empty `instance_id` migration on a database copy; E2E-15 restart mid-flow without duplicate resend; E2E-16 failed spawn is not marked dispatched; E2E-17 refusal without `-i`; E2E-18 trace identity chain; E2E-19 cleanup and protected PID check.

## 7. Safety rules (non-negotiable, every step)

- Never kill Cursor or `agm-alim`; never switch or close the default instance; never delete `cli-switch-proof-6857`, `test-cli-flow-1743`, `clone-from-default-4424`, `new-empty-instance-4425`.
- Create test instances only with the prefixes in section 4; delete only instances this run created, by exact id.
- BANNED commands: `agm tif` / `agm test-instance-flow` (its cleanup deletes every `test-cli-flow*` instance, including the protected `test-cli-flow-1743`) and `agm instances rm-all` (removes all sandboxes). Also banned: any command that acts on all instances or on default without `-i`. Full list in subtask 05.
- Do not click the Google data-collection checkbox; do not launch a replacement AGM GUI.
- Never print emails or tokens in full. Never claim a live reinject was watched unless it was.
- Relative repo paths only in every file, commit message, changelog, and release note.

## 8. Done when

- AC-01..AC-32 in the architecture spec each have a PASS line with evidence, or a recorded BLOCKED reason.
- RCA 34 status is `solved` with commit SHAs.
- This file is moved to `.ai-memory/plans/completed/` and `.ai-memory/plans/readme.md` is updated.

## Follow-ups (out of scope for this plan)

- Plan 139 items: `pid_refresh_seconds` UI/CLI parity, `agy` session resume flag, ambiguity `02-is-instance-running-saved-pid-fast-path.md`.
- Orphan instance folder `test-e2e-alpha-5232` exists on disk but not in the registry; ask the user before deleting.
