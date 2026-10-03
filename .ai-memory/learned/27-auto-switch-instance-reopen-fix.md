# Auto-Switch Instance Reopen & Modern UI Guidelines

## 1. Overview & Architecture
When an active profile or workspace exhausts its quota (0% credits or drops below <= 15%), the Auto Profile Switcher automatically evaluates candidate profiles across all registered instances and fast-forwards to the healthiest account.

This involves:
- Multi-instance candidate evaluation in `src-tauri/src/modules/auto_switcher.rs`.
- Clean instance lifecycle execution: closing depleted instances, updating active pointer, launching target instances, and restoring in-flight prompt queues.
- Visual state synchronization in `src/pages/Instances.tsx` and `src/stores/useInstanceStore.ts`.

---

## 2. Root Cause Analysis (RCA) & Critical Fixes

### Defect 1: Candidate Instance ID Overwritten with `current_instance_id`
- **Root Cause**: In `select_candidate_profiles` (`auto_switcher.rs:880, 950`), when evaluating candidate accounts registered under secondary instances (e.g. Instance #2 `default-copy`), the code set `instance_id: current_instance_id.to_string()`.
- **Consequence**: The true owner instance ID was destroyed and replaced with `'default'`. The system treated all rotations as same-instance credential replacements inside the depleted workspace instead of switching to the target instance.
- **Fix**: In `select_candidate_profiles`, candidate profiles now find and preserve their bound `instance_id` (`inst.id.clone()`).

### Defect 2: Missing Cross-Instance Window Launch
- **Root Cause**: `execute_profile_rotation_with_context` only called `service.switch_account` without checking if `target.instance_id != current_instance_id`.
- **Consequence**: The candidate instance was never launched, leaving the user with an idle workspace and no running window.
- **Fix**: When `target.instance_id != current_instance_id`:
  1. In-flight prompts are backed up via `requeue_running_conversations_for_instance(&current_instance_id)`.
  2. The depleted instance is closed via `instance::close_instance(&current_instance_id)`.
  3. The active pointer is updated via `instance::set_active_instance_id(&target.instance_id)`.
  4. If `auto_reopen_on_switch` is enabled, the target instance window is launched via `instance::launch_instance(&target.instance_id)`.

### Defect 3: Default Instance Relaunch Bypassed & Stale Lockfiles
- **Root Cause**: `switch_account_to_instance` for `is_default_inst` returned `Ok(())` without relaunching the process when it was previously running. Furthermore, killed Electron instances left stale `lockfile` / `code.lock` in AppData.
- **Fix**:
  1. `was_running` state is tracked; if `was_running`, `launch_instance_without_prompt_reinject("default")` is called and child PID is recorded.
  2. In `DesktopIntegration::on_account_switch`, stale `lockfile`, `code.lock`, and `DevToolsActivePort` are purged before restarting.

---

## 3. UI Modernization Invariants
- **Dark Theme Consistency**: All inner containers in dark mode must use deep dark glass styling (`dark:bg-[#0c2438]/90 dark:border-[#15334d]`). Never use `bg-base-100` or `bg-gray-50` without explicit dark overrides that match the theme.
- **Active Pulsing Status Badges**: Running instances show an animated emerald pulse with PID; Idle instances show a slate pill.
- **Structured Banner Cards**: Auto-switcher notifications render as glassmorphism cards with structured metric pills and vibrant CTA buttons rather than unformatted run-on text.
