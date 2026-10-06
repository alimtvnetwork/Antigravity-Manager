# Step 22: Live E2E Run and Evidence

Goal: run the live multi-instance E2E catalog once on the fixed build, record one evidence file per case, and commit a short report. This step changes no source code.

Read `00-start-here.md` first. Then read the whole runbook `../05-e2e-runbook-and-evidence-capture.md` once before you run anything. This file lists only the commands, the order, and the places where the runbook must be overridden for this plan. Where this file and the runbook disagree, this file wins, because it follows the binding decisions in `00-start-here.md` and the real code from steps 01 to 21.

## 1. Depends on

- Steps 01 to 21 committed. Every gate of those steps passed.
- The user is present for the HUMAN parts (E2E-01 path 2, E2E-09, the watched reinject in E2E-11). If no human is available, those parts are recorded as BLOCKED `needs_human_ui`; the run continues.

## 2. Files you may edit

- `.ai-memory/plans/subtasks/140-per-instance-prompt-attribution-backup-queue/06-blind-executor/22-e2e-report.md` (new; the only committed file).
- Anything under `.ai-memory/temp-agents/140-per-instance-e2e-runs/<runid>/` (gitignored evidence; never staged).

Do not edit any file under `src-tauri/`, `src/` or `scripts/`. If a case fails because of a code bug, record FAIL and stop editing; the fix is a new step, not part of this one.

## 3. Safety rules (verbatim, binding for the whole step)

- Never kill Cursor or `agm-alim`; never switch or close the default instance; never delete `cli-switch-proof-6857`, `test-cli-flow-1743`, `clone-from-default-4424`, `new-empty-instance-4425`.
- New test instances only with prefix `test-cli-flow-` or `test-diag-` (spec 94 L8); delete only instances this run created.
- Do not click the Google data-collection checkbox; do not launch a replacement AGM GUI.
- Read databases read-only (`file:<path>?mode=ro`); never print emails or tokens in full.
- Record evidence per test: command, exit code, JSON excerpt, SQL row with `instance_id`. Never claim a live reinject was watched unless it was.

Also binding: never run `agm tif` or `agm instances rm-all`, and every other command on the banned list in runbook section 1 rule 1. Stop a process only by a PID this run started. No macOS verification (DR-6).

## 4. Commands to run

### 4.1 Build the CLI (sandbox build, not the GUI)

```text
$env:CARGO_TARGET_DIR="$PWD\src-tauri\target"; $env:CARGO_BUILD_JOBS="1"
cd src-tauri; cargo build --bin agm; cd ..
```

Expected: exit 0 and `src-tauri/target/debug/agm.exe` exists. Do not run `npm run tauri build` and do not start the built GUI.

### 4.2 Session setup

Open one PowerShell session at the repo root. Paste the whole helper block of runbook section 3 into it, unchanged. Keep this session until the end of the run. If the helper block throws `ABV_DATA_DIR is set`, STOP and ask the user.

Then confirm the build has steps 18 and 19:

```powershell
$probeHelp = Invoke-Agm @('prompts', 'trace', '--help')
$probeHelp.stdout_excerpt -match 'prompts trace'
$probeQueue = Invoke-Agm @('prompts', 'queue', 'ls', '--help')
$probeQueue.exit_code
```

Expected: `True` and `0`. If either is wrong, the binary is stale: rebuild with 4.1 once. If it is still wrong, STOP and report `build_missing_step_19`.

### 4.3 Run order

Wrap all cases in the `try { ... } finally { ... }` block from runbook section 4, so the instance stop of E2E-19 always runs.

| Order | Case | Runbook section | Override in section 5 of this file |
|---|---|---|---|
| 1 | E2E-00 safety snapshot | "E2E-00 Safety snapshot" | none |
| 2 | Build detection | "Build detection" | 5.1 |
| 3 | E2E-14 legacy migration on a copy | "E2E-14" | none |
| 4 | E2E-01 feasibility gate | "E2E-01 Feasibility gate" | 5.2 (DR-5) |
| 5 | E2E-02 to E2E-07 | "E2E-02" to "E2E-04 to E2E-07" | 5.3 (DR-1) |
| 6 | E2E-08 CLI enqueue | "E2E-08, E2E-10, E2E-15, E2E-16" | 5.4 |
| 7 | E2E-09 UI enqueue (HUMAN) | "E2E-09" | 5.5 |
| 8 | E2E-10, E2E-15, E2E-16 | "E2E-08, E2E-10, E2E-15, E2E-16" | 5.6 |
| 9 | E2E-11 account switch on A | "E2E-11" | 5.7 (DR-1) |
| 10 | E2E-12 auto-switch | "E2E-12" | 5.8 |
| 11 | E2E-18 trace | "E2E-18 Trace" | 5.9 |
| 12 | E2E-17 refusal | "E2E-17 Refusal" | 5.8, 5.10 |
| 13 | E2E-13 default instance paths | "E2E-13" | 5.8 |
| 14 | E2E-19 cleanup | "E2E-19 Cleanup" | 5.11 |

Never reorder. Never skip E2E-00 or E2E-19.

## 5. Overrides to the runbook

### 5.1 Build detection

Run the runbook block as written. All three gates `G_BUILD_01`, `G_BUILD_02`, `G_BUILD_03` must be `True` on this build. If one is `False`, mark every case that needs it BLOCKED with the runbook's reason code and continue; do not rebuild in the middle of the run.

### 5.2 E2E-01 is a hard gate (DR-5)

If E2E-01 is FAIL: run E2E-19, write `BLOCKED | feasibility_failed` for every other case, write the report (section 6), commit it (section 8), and stop. Do not continue to E2E-02.

Path 2 (IDE flavor) needs a human. Without one, record `PASS | cli flavor; IDE flavor BLOCKED needs_human_ui` when path 1 passed.

### 5.3 Hand-off file: legacy only (DR-1)

The runbook and catalog mention a per-instance hand-off file `.antigravity_resume_task.<A>.json`. This plan does not build it (DR-1). Wherever the runbook reads or compares that file:

1. Record whether `<repo>\.antigravity_resume_task.<id>.json` exists. Expected: it does NOT exist. If it exists, the case is FAIL with reason `per_instance_handoff_written`.
2. Check the legacy file `<repo>\.antigravity_resume_task.json` instead:
   - one owner of the repo among running instances: the file is written (record `LastWriteTimeUtc`);
   - two or more owners (E2E-05, `repo-shared`): the file is NOT rewritten by this dispatch (`LastWriteTimeUtc` unchanged), and the row stays in the database only (`DbOnly`).
3. Read the trace output: `handoff.legacy` must be `written` for one owner and `skipped_multi_owner` for two or more; `handoff.path` ends with `.antigravity_resume_task.json`.

### 5.4 E2E-08 CLI enqueue

Run the runbook block. The command shape on this build is exactly:

```powershell
Invoke-Agm @('prompt', 'enqueue', '-i', $A, '--repo', (Join-Path $Scratch 'repo-a'), (New-PromptText $m8), '--json')
```

Expected JSON: one object with `instance_id = $A`, `status = queued`, `attempts = 0`, `source_dir = cli`. Then `Invoke-Agm @('prompts', 'queue', 'ls', '-i', $A, '--json')` lists that row and no row of B.

### 5.5 E2E-09 UI enqueue (HUMAN)

Gate: the running AGM window must be a build that contains step 21. Ask the user; do not start another AGM window. If the user cannot confirm, record `BLOCKED | needs_human_ui` and continue.

Give the human the text from `New-PromptText (New-Marker $A 'E2E09a')` and this click path: Instances page, row of A, open the prompt tree, select a conversation of `repo-a`, paste the text into the prompt editor, click Enqueue. Remind them not to click the Google data-collection checkbox.

After they confirm, run the runbook step 4. Expected: one queued row for A with `source_dir = ui`, the same columns as the E2E-08 row, and no new or rewritten hand-off file in `repo-a` (5.3 rule 1 and the legacy file `LastWriteTimeUtc` unchanged, because enqueue never writes a file).

### 5.6 E2E-10, E2E-15, E2E-16

- E2E-10 (send now via CLI): `Invoke-Agm @('prompt', '-i', $A, (New-PromptText $m10)) -Cwd (Join-Path $Scratch 'repo-a')`. Expected: a row for A with `source_dir = cli`, status `dispatched`, `attempts = 1`. The row exists before the agy process starts (check `created_at` is not later than the first marker hit).
- E2E-15 (queue loop): use the runbook `Start-Process` block. Stop only `$Run.qs_pid`. Expected: the queued row moves to `dispatched` with `attempts = 1`, never above 1.
- E2E-16 (repo gone): rename after the enqueue and before `qs`, as the runbook says. Expected on this build: status `failed`, `status_reason = repo_missing`, `attempts = 1`. A second `qs $A --once` does not raise `attempts`.

### 5.7 E2E-11 account switch on A

Run the runbook block, but replace the per-instance hand-off read (the `$hand = ... .antigravity_resume_task.$A.json` line and the line after it) with the 5.3 checks. `watched_reinject` stays `$false` unless a human or a screenshot saw the restored prompt arrive (HUMAN). Restored rows have `source_dir = backup_restore`.

### 5.8 Integration test files that do not exist

The runbook calls `cargo test --test switch_flow_instance_scope_test`, `--test canonical_instance_key_test` and `--test cli_instance_parity_test`. These files are not created by this plan. Run these library filters instead, each as its own command, from `src-tauri` with the environment from 4.1:

```text
cargo test --lib -- --test-threads=1 select_instance_
cargo test --lib -- --test-threads=1 canonical_instance_id
cargo test --lib -- --test-threads=1 auto_switch_default_target_restores_default_only
cargo test --lib -- --test-threads=1 cross_instance_rotation_restores_source_not_target
```

| Runbook test | Substitute filter | Used by |
|---|---|---|
| `switch_flow_instance_scope_test` | `auto_switch_default_target_restores_default_only`, `cross_instance_rotation_restores_source_not_target` | E2E-12, E2E-13 |
| `canonical_instance_key_test` | `canonical_instance_id` | E2E-13 |
| `cli_instance_parity_test action_commands_refuse_without_instance_when_two_running` | `select_instance_` | E2E-17 |

Each output must contain `test result: ok.` with at least 1 passed. If a filter runs 0 tests, the case is BLOCKED with reason `substitute_test_missing:<filter>`. Write the substitution into the case `notes`.

### 5.9 E2E-18 trace

Run the runbook block. The trace JSON on this build has these keys: `instance_id`, `instance_name`, `source_dir`, `conversation_id`, `repo_path`, `row_id`, `status`, `status_reason`, `attempts`, `status_history`, `backup_row_id`, `handoff`, `dispatch_result`. The runbook key list is a subset; `missing_keys` must be empty.

Expected values for the E2E-11 conversation: `instance_id = $A`, `source_dir = backup_restore` for a restored row (the spec says `cli`; this plan records restored rows as `backup_restore`), `dispatch_result` starts with `sent:` or `failed:`, `handoff.legacy` follows 5.3. The trace for B with A's conversation id returns an empty array.

### 5.10 E2E-17 refusal

Run the runbook block with the 5.8 substitute for the unit test. A and B are both running at this point. Expected: `brp`, `rrp`, `rrc` and `prompt` without `-i` each exit 2, stderr contains `instance required: 2 instances running`, and the sandbox row counts are unchanged.

### 5.11 E2E-19 cleanup

Run the runbook block as written. It only touches ids stored in `$Run.instances` by this run and refuses any id without the `test-cli-flow-` prefix or on the protected list. Do not add any other delete. Do not delete the evidence folder.

## 6. Evidence to record

Per case, in `.ai-memory/temp-agents/140-per-instance-e2e-runs/<runid>/<ID>.json` (the runbook helpers `New-Case` and `Save-Case` write it): command, exit code, masked JSON excerpt, SQL rows with `instance_id`, file existence and `LastWriteTimeUtc`, result and one-line reason. Also `run.json`, `raw/`, and `report.txt` with one line per case:

```text
ID | RESULT | path | reason
```

`RESULT` is one of `PASS`, `FAIL`, `BLOCKED`, `ABORTED`. `BASELINE` is not used on this build. HUMAN parts that were not done are written as `NOT VERIFIED: needs a human` in the reason, with result `BLOCKED` and code `needs_human_ui`.

Then write `22-e2e-report.md` (the committed file) with exactly these sections:

1. `Run`: run id, date, build commit (`git rev-parse --short HEAD`), relative path of `report.txt`.
2. `Results`: the full content of `report.txt` in a `text` code block.
3. `Counts`: number of PASS, FAIL, BLOCKED, ABORTED.
4. `Failures`: every FAIL with the expected item that failed and its evidence file; or `None`.
5. `Not verified`: every HUMAN part not done, with `NOT VERIFIED: needs a human`.
6. `Substitutions`: the 5.8 table rows that were used, and the DR-1 hand-off override.
7. `Safety`: whether every protected PID stayed unchanged (from E2E-19 `final_instances` versus E2E-00 `instances`), and `leftover_run_ids` (must be empty).

No absolute path, drive letter, username, email or token in the report. Run every string through `Protect-Text` before writing it.

## 7. Gate

No build gate: this step changes no code. Before committing:

```powershell
git status --porcelain
```

Expected: the only new or changed tracked path is `.ai-memory/plans/subtasks/140-per-instance-prompt-attribution-backup-queue/06-blind-executor/22-e2e-report.md`. Nothing under `.ai-memory/temp-agents/` shows up (it is gitignored). If anything else shows up, do not stage it.

## 8. Commit

```text
Fix: prompts - record plan 140 live E2E results
```

Stage only `22-e2e-report.md` by its explicit path.

## 9. Done when

- [ ] `report.txt` has one line for every case from E2E-00 to E2E-19.
- [ ] E2E-01 is PASS, or the run stopped per 5.2 and every later case is `BLOCKED | feasibility_failed`.
- [ ] No case file mentions an existing `.antigravity_resume_task.<id>.json`, or that case is FAIL.
- [ ] Every HUMAN part is either done by a human or says `NOT VERIFIED: needs a human`.
- [ ] `watched_reinject` is `true` only where a human or a screenshot saw it.
- [ ] E2E-19 is PASS: `leftover_run_ids` is empty and every protected instance and process is unchanged.
- [ ] `22-e2e-report.md` is committed alone with the message from section 8.
