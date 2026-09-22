# Subtask 02: Robust Instance Switching & Terminal CLI Fast-Forward Support

Traceability ID: Task-02
Target Files:
- src-tauri/src/modules/instance.rs
- src-tauri/src/commands/mod.rs
- src/components/InstancesView.tsx
- src/components/Navbar.tsx

Action:
- Refactor the instance switching workflow so that switching to a target profile:
  1. Identifies the active instance's PID.
  2. Safely signals and terminates only that specific instance PID.
  3. Updates the active profile pointer and editor database bindings.
  4. Launches the new instance with its dedicated `--user-data-dir`.
  5. Records the newly spawned PID.
- Expose a CLI command/arg flag (`--fast-forward` or `--switch <profile_name>`) enabling headless profile switching from the terminal.
- Update frontend Fast-Forward and Switch buttons with optimistic loading states, error toast reporting, and prevention of double-click collisions.

Acceptance Criteria:
- Switching instances transitions cleanly between profiles without crashing or hanging.
- External profiles launched under other directories remain untouched.
- Fast-forward can be invoked programmatically from terminal CLI.

Targeted Verification:
- python 03-ai-scripts/05-guideline-autofixer.py src-tauri/src/modules/instance.rs

Status: COMPLETED
- Removed global `switchAccount` call from `smartRotateProfileAccount` in `useInstanceStore.ts` that killed all instances.
- Refactored `rotateToNextBestProfile` to close only previous profile cleanly before launching target.
- Added `--fast-forward` / `-ff` CLI command in `cli.rs` triggering headless background rotation.
