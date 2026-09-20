# Plan 48: Instance Smart Rotation, Process Termination, Account Selection Algorithm & Minor Release v4.37.0

## Status: Completed

## User Request (Verbatim)
```text
I think there is a misunderstanding of this button. So whenever we click on this button on that current profile, it is basically going to close those processes for that exact process which is running from that location and from those profile actually. So it's going to close those, and then just like how the selected button, what I'm going to show you, just switches the profile. It's going to use that thing, but also with additional step. The additional step is that it's going to try to find whichever workspace or the account have not been used or had most room. Let's say account with six days to refill, no use. That would be the first idea. That would be the first one to pick. And in last four hours used, there is no use by anyone, so we'll pick that. So if that is already available, we just do a refresh and then make sure that that is the this one and then pick and switch. That is the idea of this button. I'm not sure on what you are doing. So first thing you should do is write the spec into spec folder and then write the plan, and then improve this code, and then make a final bump and minor release. Do you understand?
```

## Accomplished Work & Deliverables

1. **Architectural Specification in `02-spec/`**:
   - Authored `02-spec/20-instance-management/02-smart-rotation-account-switch-spec.md` detailing exact process teardown, candidate scoring formulas, quota refresh verification, and token injection flow.
   - Updated `02-spec/20-instance-management/01-index.md` index table.

2. **Process Teardown by Profile Location**:
   - In `src-tauri/src/modules/instance.rs`, updated `find_pids_for_data_dir` to ensure processes matching the instance's directory and `--user-data-dir` are accurately identified.
   - Updated `close_instance` on Windows to pass `/F /T /PID` (tree-kill) to terminate root processes and any child processes (renderers, extension host, language servers) so no orphan process holds locks on `state.vscdb`.

3. **Smart Candidate Account Selection Algorithm**:
   - Implemented `findSmartRotationAccount` in `src/services/instanceService.ts`:
     - Filters healthy accounts (`!account.disabled`, `!account.quota?.is_forbidden`, `!account.validation_blocked`).
     - Excludes currently bound account if alternative candidates exist.
     - 4-Hour Inactivity Factor: Priority (+100,000 pts) for accounts not used in past 4h (`(now - last_used) >= 4 * 3600`) or never used.
     - Refill Runway Factor: Major bonus (+50,000 pts) for accounts with longest runway until reset (e.g. 6 to 7 days until weekly refill).
     - Headroom Factor: Proportional scoring for remaining quota percentage across models (up to 20,000 pts for 100% full).
     - Tier Bonus: Pro (+2,000 pts) and Ultra (+3,000 pts).

4. **Live Quota Refresh & Profile Account Switch**:
   - Implemented `smartRotateProfileAccount` in `src/stores/useInstanceStore.ts`.
   - Executes immediate process teardown on the target profile directory.
   - Runs live quota refresh on the candidate account via `useAccountStore.getState().refreshQuota(candidate.account.id)`.
   - Injects fresh tokens into `{data_dir}/User/globalStorage/state.vscdb` and keyring via `switchAccountToInstance`, and relaunches the profile.

5. **UI Integration**:
   - Updated FastForward (`>>`) action in `src/components/navbar/InstanceSelector.tsx` (top navbar and dropdown rows).
   - Updated FastForward action in `src/pages/Instances.tsx`.
   - Replaced old "Double Play" text with "Smart Switch" and added tooltips and success toasts.

6. **Minor Release Ceremony (`v4.37.0`)**:
   - Executed version bump from `4.36.0` to `4.37.0` across `version.json`, `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `Casks/antigravity-tools.rb`, `readme.md`, `CHANGELOG.md`, `CHANGELOG_EN.md`, and spec changelogs.
   - Verified 100% version synchronization via `python 03-ai-scripts/14-version-sync-checker.py`.
   - Verified 0 TypeScript errors via `npx tsc --noEmit`.

## Subtasks Record
- [Subtask 01: Spec & Backend Instance Process Teardown](.ai-memory/plans/subtasks/48-instance-smart-rotation-account-switch/01-spec-and-backend-instance-process-teardown.md) - Completed
- [Subtask 02: Smart Account Candidate Selection & Refresh](.ai-memory/plans/subtasks/48-instance-smart-rotation-account-switch/02-smart-account-candidate-selection-and-refresh.md) - Completed
- [Subtask 03: Frontend Instance Rotation Integration](.ai-memory/plans/subtasks/48-instance-smart-rotation-account-switch/03-frontend-instance-rotation-integration.md) - Completed
- [Subtask 04: Minor Release v4.37.0 Ceremony](.ai-memory/plans/subtasks/48-instance-smart-rotation-account-switch/04-minor-release-v4-37-0.md) - Completed
