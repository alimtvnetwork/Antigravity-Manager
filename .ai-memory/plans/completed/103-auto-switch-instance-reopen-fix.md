# Parent Plan: 103 - Auto-Switch Instance Reopen & Cross-Instance Rotation Fix

**Version:** 1.0.0  
**Status:** Completed & Verified  
**Spec Reference:** `02-spec/21-app/103-auto-switch-instance-reopen-fix.md`  
**Total Steps Budget (N):** 300  
**Concurrency Capacity:** A = 2, H = 2

---

## 1. User Request (Verbatim)
> Hi there. You can see the default workspace or instance was zero in credits, but it did not reopen to the new instance. That is a bug current system. So I hope you find the root cause and try to fix it and make a release of this. Do you understand? Also improve the UI to modern please
> 
> # Actionable Items Must Follow Non-Negotiable
> 1. Write a plan and spec first 
> 2. Identify the root cause of the bug in the current system
> 3. Fix the bug to ensure the instance reopens correctly
> 4. Make a release after the fix is implemented
> 
> Must follow and spawn agent using 
> @[.agents/skills/execute-parent-task-with-n-steps-v6]
> 
> ## Additional Instructions
> learn /learn if you have to learn something and /plan stuff before working please.

---

## 2. Granular Subtask Decomposition

- [x] **Subtask-01: Auto-Switcher Candidate Ownership & Cross-Instance Dispatch** (`01-candidate-ownership-and-dispatch.md`)
  - Fix candidate instance ID preservation in `select_candidate_profiles` (`auto_switcher.rs:880, 950`).
  - Implement cross-instance rotation lifecycle in `execute_profile_rotation_with_context` (`auto_switcher.rs`): close old instance, set active instance, and launch new instance window.
  - Verified with `cargo fmt -- --check`.

- [x] **Subtask-02: Default Instance Relaunch & Windows Process Detach** (`02-default-instance-relaunch-and-detach.md`)
  - Fix short-circuit in `switch_account_to_instance` (`instance.rs:3037-3065`) to ensure default instance relaunches when running.
  - Purge stale lockfiles (`lockfile`, `code.lock`, `DevToolsActivePort`) in `integration.rs`.
  - Verified with `cargo fmt -- --check`.

- [x] **Subtask-03: Configuration Flag & Settings Schema** (`03-config-flag-and-schema.md`)
  - Added `auto_reopen_on_switch: bool` (default `true`) to `AutoProfileSwitcherConfig` in `src-tauri/src/models/config.rs`.
  - Verified deserialization defaults.

- [x] **Subtask-04: Frontend State Refresh & Modernized Instance Display** (`04-frontend-state-and-notification.md`)
  - Redesigned Auto Profile Switcher banner to modern glassmorphism card with glowing pulse indicator and structured quota pill.
  - Completely modernized Instance Card: dark glass Account pill and Quota box (`dark:bg-[#0c2438]/90 dark:border-[#15334d]`), eliminating all glaring light-gray boxes in dark mode.
  - Added compact badge layout for Status, Profile ID, and Path with copy action.
  - Polished primary and secondary action buttons.
  - Updated `useInstanceStore.ts` with cross-instance launch delegation and event listeners.
  - Verified with `npm run build`.

- [x] **Subtask-05: Pre-Flight Verification & Release Ceremony** (`05-verification-and-release.md`)
  - Run `cargo fmt -- --check` and `npm run build`.
  - Execute release bump via `npm run bump patch` and synchronize `CHANGELOG.md`, `README.md`, and `README_EN.md` attributed strictly to `@aukgit`.
