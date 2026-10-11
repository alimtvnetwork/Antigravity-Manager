# Execution Plan: Auto Switch & Rotate Fix and Application Performance & Synchronization

**Slug**: `auto-switch-and-rotate-fix` & `application-performance-and-synchronization-issues`  
**Parent Task ID**: 1  
**Budget**: 300 steps  
**Status**: ACTIVE  

---

## User Request (Verbatim)

```text
# Auto Switch and Rotate Fix: high priority instruction, non-negotiable task

Yeah, I think it's been quite some time that you have been trying to fix the auto switch and rotate. None of this is working. The biggest problem here is that the switch of the account and fast forward, both of this does not work at all. So you can compare with 162, v4.162 version. That's auto-rotate. Auto-rotate and fast forward, restarting instance, all of these functionalities work. So try to go deep, understand very well what is the difference between your code and there, and also fix the CI/CD issues by checking the gitmap.pe. Don't try to run anything. Check the issues, try to find the files, the root cause, and try to fix it. Again, send it to GitHub CI/CD to fix it, okay? Don't try to be over smart at this moment. And try to figure out and write the root cause for me why your account switching is failing. I tried to revert back to the old version, version 1.62, and why I cannot do that automatically using the `Antigravity Manager (`Antigravity Manager (AGM)`)` update to v4.162 or just 4.162. Why I cannot? So that update option from the CLI also does not revert the version. Make sure you fix it properly everything. Okay? It's very crucial. I've been doing this for several time. You have been breaking stuff. This is stupid. Write the root cause, write the problem first, and then try to create the instruction for the AI first, okay? And then you start doing the task. So before doing that, plan it properly, write the instruction properly, find the root problem, go deep inside the code, do the comparison of this, what you have, what is there that is working. It is kind of stupid that you tried several times and it is not working.

## slug: auto-switch-and-rotate-fix

# Actionable Items Must Follow Non-Negotiable

1. Write spec under 02-spec/21-app/auto-switch-and-rotate-fix/ and enqueue plan task in .ai-memory/plans/auto-switch-and-rotate-fix.md (subtasks in .ai-memory/plans/subtasks/auto-switch-and-rotate-fix/) first
2. Search codebase exclusively via GitMap (gitmap aum search, gitmap find, gitmap cat, gitmap ps, gitmap py, gitmap llm train); TOTAL BAN on rg, ripgrep, grep, git grep, Select-String
3. Strictly use relative Git paths (02-spec/..., .ai-memory/..., cmd/...); only add the relative paths, never add the absolute path during your work, and ensure this is respected on the release page and in release notes as well
4. Use `gitmap` AI agents to enter data
5. Task completion includes committing and pushing to Git
6. Compare current code with version 162, v4.162 to identify differences in auto-rotate and fast forward functionalities.
7. Investigate and fix CI/CD issues by checking gitmap.pe.
8. Identify and document the root cause of account switching failure.
9. Attempt to revert to version 1.62 using `Antigravity Manager (`Antigravity Manager (AGM)`)` and document why the CLI update option fails to revert the version.
10. Plan and write detailed instructions for AI before starting the task.

# Application Performance and Synchronization Issues: high priority instruction, non-negotiable task

This list view, make sure we have the exe path that is visible in that column. I mean, the ending path will be enough, exe path. And also, make sure that there is no code that actually swallows the error. And there are some glitches, like when we try to open, let's say IDE or Switch, it takes a long time. And this is a process-to-process communication, that is all right. But sometimes the application gets stuck. So can you please check that part, actually, why the application gets stuck? That's one thing. Another is that your account needs to be synchronized, even if we don't have the super base, using the email or the Telegram. So make sure that you pass something as a JSON, as an instruction, so that we understand which machine is using which, let's say, instance, which account, which Id, which workspace, things like that. So using the internal database concept and other options, you would understand what is happening in each machine, so that as a summary, you can know it from the top level.

## slug: application-performance-and-synchronization-issues

# Actionable Items Must Follow Non-Negotiable

1. Write spec under 02-spec/21-app/application-performance-and-synchronization-issues/ and enqueue plan task in .ai-memory/plans/application-performance-and-synchronization-issues.md (subtasks in .ai-memory/plans/subtasks/application-performance-and-synchronization-issues/) first
2. Search codebase exclusively via GitMap (gitmap aum search, gitmap find, gitmap cat, gitmap ps, gitmap py, gitmap llm train); TOTAL BAN on rg, ripgrep, grep, git grep, Select-String
3. Strictly use relative Git paths (02-spec/..., .ai-memory/..., cmd/...); only add the relative paths, never add the absolute path during your work, and ensure this is respected on the release page and in release notes as well
4. Use `gitmap` AI agents to enter data
5. Task completion includes committing and pushing to Git
6. Ensure the exe path is visible in the list view column and check for any code that swallows errors.
7. Investigate and resolve application glitches and performance issues, particularly when opening IDE or Switch.
8. Synchronize accounts using email or Telegram, passing JSON instructions to track machine usage, instances, accounts, IDs, and workspaces.
9. Utilize internal database concepts to monitor and summarize machine activities at a top-level view.
```

---

## Root Causes Identified from Deep Comparative Analysis vs v4.162.0

1. **Account Switch & Auto-Rotate Failure**:
   - `src-tauri/src/modules/instance/pid_scan.rs:302`: Cross-instance PID contamination where `instance_root_pids` accumulated across all instances. Stopped instances falsely adopted running sibling processes, causing `switch.rs:59` to abort with `"Account is actively bound to running sibling instance"`.
   - `src-tauri/src/modules/auto_switcher/selection.rs:254, 294` and `scoring.rs:205`: 100% quota gate returning `0.0` for accounts with `< 100%` quota, and Step 3 fallback blocked whenever `cooldown_pool` was not empty.

2. **CLI Downgrade/Revert Failure (`agm update v4.162` or `4.162`)**:
   - SemVer 2-part normalization gap: inputs like `v4.162` or `4.162` were passed directly without normalizing to `v4.162.0`, causing GitHub release tag lookups to 404.
   - `src-tauri/src/bin/agm/update_helpers.rs:86`: `download_release_zip` ignored requested version and hardcoded `releases/latest`.
   - `install.ps1:1265, 1423, 1461`: `install.ps1` rejected lower versions unless `$isPinned` bypassed the check, and guessed mismatched asset filenames (`agm-alim_${cleanPinned}_x64-setup.exe`).

3. **Application Freezes / Glitches on IDE Launch & Switch**:
   - `src-tauri/src/modules/instance/projects.rs:150-157`: Synchronous 6,000ms (`std::thread::sleep(Duration::from_millis(6000))`) sleep in `wait_for_instance_prompt_channel` called directly on the Tauri IPC thread during launch.
   - `src-tauri/src/modules/instance/close.rs:105-135`: Synchronous blocking loops in `close_instance_verified` sleeping 200ms up to 10 seconds.
   - `src-tauri/src/modules/instance/pid_scan.rs:20-44`: Heavy global mutex lock contention on `PROCESS_SCAN_CACHE` blocking all concurrent IPC calls during process refreshes.

4. **Instance List View Executable Path**:
   - `src/components/instances/InstanceTable.tsx` and `src/components/instances/table/InstanceRowCells.tsx`: Only displayed `resolveDataDir(inst)` in the File/Data Path column without displaying the executable path (`inst.config.executable_path` or default IDE fallback).

5. **Silent / Swallowed Errors**:
   - `instance_fastforward.rs:78, 92-93`: Bare `let _ = repo_db::...`
   - `src-tauri/src/commands/instance.rs:25-31`: `record_ignored(..., "block_on")` on account switch.
   - `switch.rs:155, 240, 413, 418`: `.unwrap_or(0)` and `.unwrap_or_default()` discarding errors without `AppError` logging.
   - `useAutoSwitcher.ts:34, 55` and `useInstanceSettings.ts:63, 112`: Silent frontend `.catch(() => {})`.

6. **Decentralized Machine Sync via Email/Telegram (without Supabase)**:
   - Need structured JSON telemetry payload (`agm-fleet-sync-v1.0`) dispatching node ID, bound instances, accounts, and workspace prompts via Email IMAP/SMTP and Telegram, storing aggregated state in local SQLite `fleet_sync.db` for top-level cluster monitoring.

---

## Discrete Task Breakdown

- **Task-01**: Fix PID scanner contamination in `src-tauri/src/modules/instance/pid_scan.rs` and quota fallback scoring in `src-tauri/src/modules/auto_switcher/selection.rs` & `scoring.rs`.
- **Task-02**: Fix CLI updater downgrade & SemVer normalization in `src-tauri/src/bin/agm/update_cmds.rs`, `update_helpers.rs`, `src-tauri/src/modules/delegate_updater/args.rs`, and `install.ps1`.
- **Task-03**: Eliminate launch & switch blocking sleeps/deadlocks (`wait_for_instance_prompt_channel` 6s sleep, synchronous `close_instance_verified` locks).
- **Task-04**: Add Executable Path display to instance list view (`src/components/instances/InstanceTable.tsx`, `src/components/instances/table/InstanceRowCells.tsx`).
- **Task-05**: Audit and eliminate swallowed errors across instance switching, fast-forward, launch, and frontend settings.
- **Task-06**: Implement decentralized machine synchronization JSON schema & internal SQLite tracking via Email/Telegram in `src-tauri/src/modules/` and IPC commands.
- **Task-07**: Update documentation, audit specs, verify targeted checks, and commit/push atomically via `gitmap cpb`.
