# 27 — End-to-end, instance, and auto-switch testing playbook

Read this file in full before acting when the user says any of these:

- "end-to-end testing" or "perform the end-to-end testing"
- "auto-switch end-to-end testing" or "perform the auto-switch test"
- "instance testing" or "instance test testing"

**Instance testing always includes auto-switch testing, opening an instance, opening the prompt preview (Prompt Tree), and entering / sending a prompt.** Do not stop after `agm instances ls`.

Shipped proof for the fixes below is **v4.198.0** (tag `v4.198.0`, commit `fa8fb3c6` for the release, plus `8d9f9d86` for the later Supabase and email import fix). CI run `38556786736` and release run `38556790966` both completed with conclusion `success` on `alimtvnetwork/Antigravity-Manager`.

Related writes:

- Spec: `02-spec/21-app/end-to-end-testing-and-ui-update-instructions/01-architecture-spec.md`
- Spec: `02-spec/21-app/end-to-end-testing-and-ui-update-instructions/02-supabase-and-email-system-spec.md`
- Plan: `.ai-memory/plans/end-to-end-testing-and-ui-update-instructions.md`
- Earlier CLI notes: `.ai-memory/plans/subtasks/antigravity-manager-and-gitmap-testing/02-e2e-cases-for-ai.md`
- Issue write-up: `.ai-memory/issues/63-e2e-prompt-tree-supabase-email-and-autoswitch-rca.md`

## 1. Which phrase runs which cases

| User phrase | Run these sections, in order | Do not skip |
| :--- | :--- | :--- |
| end-to-end testing | 0, then A, B, C, D, E | Restore the 15% threshold. Record every pass/fail line. |
| instance testing | 0, then A, B, C | Auto-switch is part of instance testing. Prompt preview and a real send are part of instance testing. |
| auto-switch end-to-end testing | 0, then B | Still restore the live threshold to 15% before you finish. |

Section D (Supabase) and section E (email) are part of a full end-to-end request. They are extra when the user asked only for instance testing or only for auto-switch.

## 2. Rules that previous runs broke

Follow every line. Each line is a mistake that already happened.

1. **Test the binary that contains the change.** On 2026-10-14 the installed `agm` 4.197.0 still wiped `prompt_tree_cache`. The locally compiled binary at 4.197.1 kept the cache (first load 1.54s, second load 0.23s). After `npm run bump minor`, the released binary is 4.198.0. `cargo build` on this machine writes to `$env:CARGO_TARGET_DIR`, which is not always `src-tauri/target/debug/agm.exe`. Run `& "$env:CARGO_TARGET_DIR\debug\agm.exe" --version` and refuse to score a case until that version is the one you just built.
2. **Run one command at a time.** This host has an 8.7 GB commit ceiling. A parallel Cargo build plus a Prompt Tree scan pushed `agm-alim.exe` from 71 threads / 33 MB to 565 threads / 461 MB and crashed WebView2 with `0xe0000008`. Use `cargo build --bin agm -j 1` and `cargo build --bin agm-alim -j 1`. Do not start a second build while a scan is running.
3. **Do not mark a case passed by reading source.** A green reading of `usePromptTreeData.ts` is not a pass. The process must exit 0, or the window must show the screen, and you must write the observed numbers.
4. **Do not close the user's IDE or the manager window.** `close_instance` must not kill Cursor or `agm-alim.exe`. Prompt commands must not close the manager. If a command closes the window, that case fails.
5. **Restore auto-switch to 15%.** `agm auto-switch test 98` and `agm auto-switch threshold 98` are temporary. Leaving 98% makes the live daemon rotate on the next tick. The default in `src-tauri/src/models/config.rs` is `low_quota_threshold_percent: 15.0` and `critical_threshold_percent: 15.0`. Migration in `src-tauri/src/modules/config/migration.rs` rewrites a saved `25.0` or `12.0` to `15.0`. A value you set yourself (98, 50, anything else) stays until you set it back. End every auto-switch case with `agm auto-switch status` showing `15.0`.
6. **Search with GitMap.** `gitmap aum search <pattern> <dir>`. Do not use `rg`, `grep`, or `Select-String` for repository search.
7. **Relative paths only in tracked files.** Never write `D:\work\...` or `C:\Users\...` into a spec, changelog, or memory file.
8. **Supabase `NXDOMAIN` is a paused cloud project, not a reason to invent a new project.** `pezjuuddecbyfmqxytrv.supabase.co` and `ikwmurmjynhxdpmhzekt.supabase.co` have returned `DNS name does not exist` while `supabase.co` still resolves. Free-tier projects pause after 7 days and unpublish the project DNS name. Restore the project in the Supabase or Lovable dashboard. Do not create a replacement project from this test.
9. **Do not claim the CLI opens every settings screen.** `agm open-ui` and `agm instances open-ui` launch the whole desktop app and print `[OK] Antigravity Manager UI launched (PID: ...)`. That is process-start confirmation. There is no command that opens Settings, Supabase, or the Prompt Tree by itself and then reports that specific screen. Prompt Tree, Settings, and Supabase still need a click in the window, or a screenshot that shows that screen. `agm instances open <id>` starts the IDE for that instance. `agm instances ls` lists instances. Those are different commands.
10. **Do not print tokens, passwords, or prompt bodies** in the result block. Print ids, counts, timings, and status words.
11. **`--json` stdout must be JSON.** Tracing logs go to stderr when `--json` or `-j` is passed (`src-tauri/src/modules/logger.rs`). A log line on stdout is a fail.
12. **One prompt send is enough.** Use a short instruction such as `Review the latest change in this repo.` Do not invent a repository path. Use a path already bound to `default` or `default-copy-5597`.

## 3. How the failures showed up, and the fix that stopped them

### 3.1 Prompt Tree froze the machine (`0xe0000008`)

**What you saw.** Opening Instances, then Prompt Tree, made `msedgewebview2.exe` fault with `0xe0000008` (`STATUS_OOM`). `agm-alim.exe` climbed from about 71 threads and 33 MB to 565 threads and 461 MB in about one second. Cursor froze on the same machine.

**Why it happened.**

1. `src/components/instances/PromptTreeView/usePromptTreeData.ts` put `archivedProjectIds` and `pinnedProjectIds` in the `useCallback` dependency list of `loadTree`.
2. The modal-open `useEffect` depended on `loadTree` and called `setArchivedProjectIds(...)` and `setPinnedProjectIds(...)` on every run.
3. Both helpers return a new array every time, so every render rebuilt `loadTree`, which re-ran the effect, which called `invoke('get_project_conversation_tree')` again.
4. Each IPC call used Tokio `spawn_blocking` (`src-tauri/src/commands/instance.rs`). The blocking pool filled (512 workers). Each worker scanned about 48 workspaces at once.

**The wrong fix.** Debouncing the button, raising the thread pool, or catching the WebView crash. The loop was still scheduling scans.

**The fix that holds.**

- Keep the archived and pinned id lists in `useRef` inside `loadTree`. Do not list those arrays as `useCallback` dependencies.
- In `src-tauri/src/modules/repo_db/tree.rs`, a process-wide `Mutex<()>` serializes an uncached scan. After the lock is taken, read `prompt_tree_cache` again so the waiters use the row the first scanner wrote.
- `execute_prompts_tree` in `src-tauri/src/modules/cli/prompts.rs` must default to `force: false`. `--force` and `--refresh` are the only ways to bypass the cache. An earlier build hardcoded `force: true`, so every CLI tree call looked like a cold scan.

**Pass numbers from the fixed build.** Cold `agm prompts tree -i default` about 2402 ms. Warm cache about 183 ms. With the window open, thread count stayed at 51–54 and working set near 91 MB. Reset did not close the window. Header showed All Projects (48).

### 3.2 CLI hung on restore and on auto-switch test

**What you saw.** `agm prompts restore` produced no JSON for 78 seconds while Default was running. `agm auto-switch test 98` could sit forever if its output was piped.

**Why.** `spawn_with_job_breakaway_fallback` in `src-tauri/src/modules/process/launch.rs` left `stdin`, `stdout`, and `stderr` inherited. The detached `Antigravity.exe` held the caller's pipe until the IDE exited.

**The fix.** Set those three handles to `Stdio::null()` before spawn. Also keep the job-breakaway fallback ladder. Never pass `CREATE_BREAKAWAY_FROM_JOB` as the only attempt. Inside a restricted Windows job, `CreateProcessW` returns `os error 5`.

**Pass.** `agm prompts restore -i default --json` returned in 162 ms with exit 0.

### 3.3 `-r .` stored a dot

**What you saw.** A prompt sent with `-r .` later reopened in the IDE install directory.

**Why.** `CliContext::parse` in `src-tauri/src/modules/cli/types.rs` stored the literal `"."` in `active_prompts.repo_path`.

**The fix.** Canonicalize `-r`, `--repo`, and `--workspace` against the caller's working directory before insert.

### 3.4 Restore deleted a running project and hit a foreign key

**What you saw.** Stale-row cleanup in `running_projects` failed with `FOREIGN KEY constraint failed`.

**Why.** `src-tauri/src/modules/repo_db/detection.rs` deleted a `running_projects` row that `active_prompts` still referenced.

**The fix.** The `DELETE` excludes rows that `active_prompts` still points at.

### 3.5 "Update Available" for the version already running

**What you saw.** The toast said Update Available for v4.197.1 while the app was already v4.197.1. The installed binary on disk was still 4.197.0, and the GitHub tag had no Windows `.exe` yet.

**Why.**

1. `src/stores/use-update-store.ts` called `check_update_via_script` (`install.ps1 -CheckUpdate`) before `check_for_updates`. The script compared the on-disk binary with `package.json` on `main` and ignored whether the release had assets.
2. `src-tauri/src/modules/update_checker/channels.rs` set `has_update` from the version string alone.

**The fix.**

- Prefer `check_for_updates`.
- `has_update` is true only when the tag is strictly newer than `CURRENT_VERSION` and the release has an installer for this OS: `.exe` or `.msi` on Windows, `.dmg` or `.app.tar.gz` or `.pkg` on macOS, `.AppImage` or `.deb` or `.rpm` on Linux.
- `UpdateNotification.tsx` renders nothing while state is `checking`.
- A local build of the current version must not announce itself.

**Pass.** With no newer tag that has a Windows `.exe`, the Update Available popup stays hidden.

### 3.6 Auto-switch looked "on" at 25% or 98% and then rotated live

**What you saw.** Settings copy and an old `gui_config.json` still said 25% or 12%. A test set the slider to 98% to force a decision, then the daemon kept that value and would rotate on the next tick. An earlier UI build also treated 98% as the product default.

**The fix that is in v4.198.0.**

- `default_low_quota_threshold` and `default_critical_threshold` both return `15.0` in `src-tauri/src/models/config.rs`.
- `src/components/settings/AutoSwitcher/switcherTypes.ts` uses `low_quota_threshold_percent: 15.0`.
- `BackgroundTaskRunner.tsx` falls back with `?? 15.0`.
- Migration rewrites only `25.0` and `12.0` to `15.0`. It does not rewrite 98. The test must put 15 back.
- `agm auto-switch test <percent>` is a dry decision. It must not be left as the saved threshold.

**Pass.** `agm auto-switch status --json` shows `15.0` at the end. During the test, `98` reads back as `98.0` and then the status returns to `15.0`.

### 3.7 Supabase "credentials are wrong" when the URL was a mock or a typo

**What you saw.** `~/.antigravity_tools/supabase_config.json` had four endpoints. Two were mocks (`https://mock-root.supabase.co`, `https://mock-secondary.supabase.co`). One real host was misspelled `pezjuuddecbyrmqxytrv.supabase.co` (`rmq` instead of `fmq`). `02-antigravity-and-event-manager/vault/supabase_config.json` in repo-secrets had `"endpoints": []` while the manager vault had the two real endpoints. `scripts/import-supabase-secrets.ps1` read the Base64 envelope as raw text. `scripts/setup-supabase.ps1` declared `$RootUrl` twice and failed to parse.

**Why loaders kept the junk.** `auto_seed_from_repo_secrets` and `cmd_supabase_load_json` skipped an endpoint when the same `id` already existed. They never replaced the mock URL, never fixed the typo, and never dropped a second row with the same normalized URL.

**The fix.**

- `normalize_supabase_url` in `src-tauri/src/modules/supabase_client/types.rs` rewrites the `rmq` host to `pezjuuddecbyfmqxytrv.supabase.co`.
- `is_placeholder_or_mock_endpoint` treats `mock-*`, `your-project.supabase.co`, and `.mock` keys as placeholders.
- Seed, discover, and `agm supabase load-json` update a row in place by id or by normalized URL, delete placeholders when a real endpoint is present, and dedupe by normalized URL.
- The PowerShell importer decodes `"encoding_format": "base64"` fields.
- `classify_transport_error` in `src-tauri/src/modules/supabase_client/client.rs` says the free-tier project is paused when DNS returns `NXDOMAIN` / Windows `os error 11001`.

**Pass for the local file.** `agm supabase status` lists exactly `ep-root-lovable-01` → `https://pezjuuddecbyfmqxytrv.supabase.co` and `ep-secondary-01` → `https://ikwmurmjynhxdpmhzekt.supabase.co`.

**Expected remote result until someone unpauses the project.** DNS `NXDOMAIN`. Record it as a cloud pause. Do not call the local config fail if those two URLs are the only rows.

### 3.8 Email import duplicated the recipient and required the whole secret in the file

**What you saw.** Repeated imports inserted `devorg.bd@gmail.com` three times. The compact JSON had no password, so import either failed or people pasted the password into the JSON.

**The fix.**

- `add_notify_recipient` in `src-tauri/src/modules/email_vault_db/recipients.rs` updates the existing row when `LOWER(TRIM(email))` matches, then deletes the extra rows.
- `init_vault_tables` deletes historical duplicates and creates `idx_notify_recipients_email_lower`.
- `upsert_email_account` reuses the account id for the same email.
- `agm email import` accepts `--json '<compact spec>'`, `-f <file>`, `--password`, and `--ask-password`. When the JSON has no password, the command prompts. SMTP `mail.hire-seoexperts.com:465` SSL and IMAP `mail.hire-seoexperts.com:993` SSL stay inside the JSON.

**Pass.** `agm email ls` shows one account `ai-agm-tool-v1@hire-seoexperts.com` and one recipient `devorg.bd@gmail.com`. `agm email status` dispatches a status mail and prints `[SUCCESS]`.

## 4. Section 0 — prepare, every time

1. Read this file.
2. `git log -1 --oneline` and `& "$env:CARGO_TARGET_DIR\debug\agm.exe" --version`. If the binary is older than the commit you are testing, build with `-j 1` and run that binary for every later command. Prefix examples below with that binary when the installed `agm` is older.
3. `agm instances ls`. Record the ids. The known pair on this host is `default` (running) and `default-copy-5597` (often idle). Use those ids. Do not create a third instance unless both are missing.
4. Note the current auto-switch percent: `agm auto-switch status --json`. You must return to `15.0` even if you found a different number.
5. Write a result block as you go. One line per case: command, exit code, one number or status, PASS or FAIL.

## 5. Section A — instance open

Instance testing starts here and continues through B and C.

| # | Do this | Pass | Fail |
| :--- | :--- | :--- | :--- |
| A1 | `agm instances ls` and `agm instances status -i default` | Default is listed. Status prints pid or idle without exiting 1. | No instances, or the command closes `agm-alim.exe`. |
| A2 | `agm instances open-ui` | Stdout contains `[OK] Antigravity Manager UI launched (PID:` or the window was already open and stayed open. | Spawn error with no fallback, or the manager exits. |
| A3 | In the window, open Instances. | Both Default and Default Copy are visible. Auto-Switch shows ON. | The list is empty, or only one card exists when `ls` printed two. |
| A4 | `agm instances start -i default-copy-5597` only if that copy is idle and the user asked to open an instance. Do not stop `default` if it is the IDE the user is using. | The copy reports running, or a clear error that the binary is missing. The manager stays open. | Default's PID dies. Cursor dies. |
| A5 | Screenshot `assets/screenshots/` of the Instances view. | File exists and shows the two cards. | Screenshot is the desktop with no manager window. |

`agm instances open <id>` starts the IDE. `agm instances open-ui` starts the manager. Do not swap those names in the result block.

## 6. Section B — auto-switch (required for instance testing)

| # | Do this | Pass | Fail |
| :--- | :--- | :--- | :--- |
| B1 | `agm auto-switch status --json` | JSON on stdout. Record `low_quota_threshold_percent` and `is_enabled`. | Logs on stdout, or the command hangs. |
| B2 | `agm auto-switch threshold 98` then `agm auto-switch status --json` | The read-back is `98.0` or `98`. | The command hangs (stdio inherited by the IDE — section 3.2 regressed). |
| B3 | `agm auto-switch test 98` | Exit 0. A candidate decision or a clear "no candidate" error. No pipe hang. | Process still running after 30 seconds. |
| B4 | Set the threshold back to 15: `agm auto-switch threshold 15` then `agm auto-switch status --json` | Status reads `15.0`. Enabled state matches what you found in B1, unless you had to turn it on for the test, in which case say so. | Status still 98, 25, or 12. |
| B5 | If the window is open, open Settings → Auto-Switch. | The threshold control shows 15. The guide text says the default is under 15%. | The screen shows 98 because B4 was skipped. |

Do not run a real account rotation against the user's live Default instance as part of B3. `test` is the decision check. A full switch is a separate request.

## 7. Section C — prompt preview, enter, and send (required for instance testing)

| # | Do this | Pass | Fail |
| :--- | :--- | :--- | :--- |
| C1 | `agm prompts tree -i default` twice, back to back. | First load prints projects. Second load is faster. Log may say `prompt_tree_cache kept`. Record both durations. | Second load is as slow as a cold scan and the log says the cache was wiped. Exit with no message. |
| C2 | `agm prompts reset-tree -i default` then `agm prompts tree -i default` once more. | Reset prints that the cache was cleared for default. The next tree is slow again (a fresh scan). | Reset closes the manager. |
| C3 | `agm prompts running -i default` and `agm prompts running -i default --json` | Table, then a JSON envelope. Record the count. Do not paste prompt text. | JSON mixed with tracing lines. |
| C4 | Open Prompt Tree in the window on Default (Prompts and History). | Tree paints. Header can show Backup, Restore, Reset, Refresh. Project count is in the header (the fixed run showed All Projects (48)). Thread count stays near 50, not hundreds. | Window closes, WebView `0xe0000008`, or threads climb past 200. |
| C5 | Click Reset once in that tree. | Tree paints again. Window stays open. | Window closes. |
| C6 | `agm prompts copy -i default --json` | Exit 0 and a JSON envelope. | Clipboard command hangs. |
| C7 | Enter a prompt. Pick a repo path already bound to the instance. `agm prompts send -i default -r "<that path>" "Review the latest change in this repo."` | Exit 0. `agm prompts running -i default --json` shows the new id as running or dispatched. | Command closes the manager, stores repo path `.`, or never returns. |
| C8 | `agm prompts queue -i default -r "<that path>" "Queue a second review."` | Exit 0, status `queued`, `fifo_ordered true`. | The queued row is marked running immediately, or the window closes. |
| C9 | `agm prompts backup -i default --json` then `agm prompts restore -i default --json` | Backup returns a path or a count. Restore returns within a few seconds (the fixed run was 162 ms) and does not delete a project that still has an active prompt. | 78-second hang, or `FOREIGN KEY constraint failed`. |
| C10 | `agm prompts resend -i default --json` only if C3 showed a running prompt. | Exit 0 or one clear error. | Hang, or a second IDE spawned for an instance you did not name. |
| C11 | Screenshot the Prompt Tree after C4. | File shows the tree, not only the desktop. | Missing file. |

Order is fixed: preview (C1–C5) before send (C7). Sending first and then calling the tree "preview" hides a broken tree behind a successful dispatch.

## 8. Section D — Supabase (full end-to-end only)

| # | Do this | Pass | Fail |
| :--- | :--- | :--- | :--- |
| D1 | `agm supabase status` | Exactly two real endpoints, hosts spelled `pezjuuddecbyfmqxytrv` and `ikwmurmjynhxdpmhzekt`. No `mock-root`, no `mock-secondary`, no `rmq` typo. | Four rows, or a mock URL still present. |
| D2 | `agm supabase export --file` to a scratch path you delete afterwards. Then `agm supabase load-json` of `02-antigravity-manager/vault/supabase_config.json` from repo-secrets, using a relative path. | Load prints success and status still shows two endpoints. | Duplicate rows after load. |
| D3 | Resolve both project hosts. | If both are `NXDOMAIN` and `supabase.co` resolves, write `CLOUD PAUSED`. That is the expected current result. | Treating NXDOMAIN as a bad API key and writing a new project ref into the vault. |

## 9. Section E — email (full end-to-end only)

| # | Do this | Pass | Fail |
| :--- | :--- | :--- | :--- |
| E1 | `agm email ls` | One account, one recipient. | The same recipient listed two or three times. |
| E2 | Import with the password prompt only if the vault is empty. Use `--ask-password` and the JSON from `02-antigravity-manager/email-system-spec.md` in repo-secrets. Do not write the password into the repo. | `agm email ls` still shows one recipient. | A second row for the same email. |
| E3 | `agm email status` | `[SUCCESS]` and a send to `devorg.bd@gmail.com` via `mail.hire-seoexperts.com:465`. | Auth error, or a hang. |

## 10. Checklist the agent ticks before saying "done"

Copy this into the reply and mark each line.

```text
[ ] Used the agm binary whose version matches the commit under test
[ ] Builds and scans were sequential (-j 1), one command at a time
[ ] A1 instances ls / status recorded
[ ] A2 open-ui printed a PID or the existing window stayed open
[ ] A3 Instances screen seen (two cards, Auto-Switch ON) or marked NOT SHOWN with a reason
[ ] B1 status recorded
[ ] B2 threshold 98 read back
[ ] B3 auto-switch test 98 returned
[ ] B4 status is 15.0 again
[ ] C1 tree cold and warm timings recorded
[ ] C2 reset cleared cache and the next tree was cold
[ ] C3 running table and JSON recorded (no prompt body)
[ ] C4 Prompt Tree window painted and the process stayed near 50 threads
[ ] C5 Reset click did not close the window, or NOT CLICKED is written
[ ] C7 a prompt was entered with send, or FAIL with the error
[ ] C8 a second prompt was queued
[ ] C9 backup and restore returned without a foreign-key error
[ ] D and E ran when the user asked for full end-to-end testing
[ ] No password, token, or prompt body in the reply
[ ] No new Supabase project was created
[ ] Result block lists PASS or FAIL per case
```

If the user asked only for auto-switch, the C lines can stay unchecked and the reply must say instance and prompt cases were out of scope. If the user asked for instance testing, A, B, and C must all be checked. Unchecked C7 means instance testing is not finished.

## 11. Result block from the run that produced v4.198.0

```text
Binary: locally compiled agm, then release v4.198.0 (tag v4.198.0).
CLI instances: PASS. Default running. Default Copy idle (default-copy-5597).
Tree cold/warm: PASS. About 2402 ms cold, about 183 ms warm on the fixed build. Earlier 4.197.1 binary: 1.54s then 0.23s; after reset-tree, 1.46s. Installed 4.197.0 still wiped the cache.
Reset: PASS. Cache cleared for default.
Running / copy / resend: PASS with --json on stdout.
Backup: PASS for default and default-copy-5597.
Restore: first binary hung 78s while Default was running (FAIL, section 3.2). After Stdio::null, PASS in 162 ms.
Auto-switch: PASS. threshold 98 read back 98.0. test 98 returned. status restored to 15.0.
Window: PASS via agm instances open-ui. Instances showed both cards, Auto-Switch ON. Prompt Tree showed All Projects (48). Threads stayed 51–54. Reset and Copy worked. Screenshots under assets/screenshots/.
Update toast: PASS. Hidden when the running version has no newer tag with a Windows .exe.
Supabase file: PASS after the loader fix (two real endpoints). DNS: CLOUD PAUSED (NXDOMAIN). No new project created.
Email: PASS. One account, one recipient after dedupe. Live SMTP 465 status mail returned SUCCESS.
Release: CI 38556786736 success. Release 38556790966 success.
Screen-by-screen CLI confirmation: NOT AVAILABLE. open-ui confirms process start only.
```

## 12. What a later agent must not reintroduce

- `archivedProjectIds` or `pinnedProjectIds` in the `loadTree` dependency array.
- `force: true` as the default for `agm prompts tree`.
- Inherited stdio on a detached IDE spawn.
- `has_update` from a version compare with no platform asset check.
- A saved auto-switch threshold left at 98 after a test.
- A product default of 25% or 98% for `low_quota_threshold_percent`.
- Mock Supabase URLs kept because their endpoint id already existed.
- The host spelling `pezjuuddecbyrmqxytrv`.
- A second `notify_recipients` row for the same email.
- A new Supabase project to "fix" `NXDOMAIN`.
- A claim that `agm` can open Settings or Supabase and confirm that screen. Add that command only if it prints the route name after the window navigates, and then add a case to section A.
