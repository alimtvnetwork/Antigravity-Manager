# 15. Automated Quota Polling, Profile Auto-Switching, and Task Resumption (Completed)

> **Execution Milestone**: Completed across 200 continuous self-loop steps.  
> **Initial Task Start**: User requested a configurable background timer (default: 60s, range: 15s–600s) to monitor the active IDE profile's remaining quota, automatically failover to the next best profile when quota drops below threshold (<10%), snapshot pending QE/agent tasks, resume them automatically on relaunch, and integrate full UI controls and status displays.  
> **Scope**: Background Supervisor Daemon (`src-tauri/src/modules/auto_switcher.rs`), Config Models (`AutoProfileSwitcherConfig`), IPC Commands, Task State Snapshot Recovery (`<config_dir>/task_recovery/`), Settings UI Component (`src/components/settings/AutoSwitcherSettings.tsx`), and Instances Page Live Status Banner (`src/pages/Instances.tsx`).

---

## Summary of Consolidated Subtasks

### 1. Subtask 01: Backend Auto Switcher Daemon & Task Snapshotting
- **Configuration Schema** (`src-tauri/src/models/config.rs`):
  - Defined `AutoProfileSwitcherConfig` with strict positive booleans (`is_enabled: bool`, `check_interval_seconds: u32`, `low_quota_threshold_percent: f64`, `target_model: String`, `has_auto_resume: bool`, `cooldown_seconds: u32`).
  - Added `auto_profile_switcher` field to `AppConfig` and re-exported in `models/mod.rs`.
- **Core Supervisor Daemon** (`src-tauri/src/modules/auto_switcher.rs`):
  - `start_auto_switcher()`: Async Tokio background worker polling every `check_interval_seconds`.
  - `check_and_rotate_if_needed()`: Inspects active instance's bound account quota against `low_quota_threshold_percent`.
  - `select_next_best_profile()`: Intelligently scores candidate profiles and account pools by available quota and cooldown status.
  - `snapshot_task_state()`: Captures active instance, account ID, timestamp, and recovery intent into `<config_dir>/task_recovery/snapshot_<instance_id>.json`.
  - `execute_profile_rotation()`: Injects target credentials into SQLite `state.vscdb`, updates the active registry pointer, and launches the target profile.
  - `trigger_manual_rotation()`: Provides instant test rotation to the next best profile.

### 2. Subtask 02: Tauri IPC Commands & Lifecycle Hooking
- **Tauri Commands** (`src-tauri/src/commands/instance.rs` & `commands/mod.rs`):
  - `get_auto_switcher_status() -> Result<AutoSwitcherStatus, String>`
  - `update_auto_switcher_config(config: AutoProfileSwitcherConfig) -> Result<(), String>`
  - `trigger_manual_profile_rotation() -> Result<String, String>`
- **Lifecycle Integration** (`src-tauri/src/lib.rs`):
  - Initialized `modules::auto_switcher::start_auto_switcher()` at application startup.
  - Registered commands in `tauri::generate_handler!`.

### 3. Subtask 03: Frontend Settings, Store & Instances View
- **Frontend Service & Store** (`src/services/instanceService.ts` & `src/stores/useInstanceStore.ts`):
  - Added `AutoProfileSwitcherConfig` and `AutoSwitcherStatus` TypeScript interfaces.
  - Added `fetchSwitcherStatus`, `updateSwitcherConfig`, and `triggerManualRotation` store actions.
- **Settings Component** (`src/components/settings/AutoSwitcherSettings.tsx` & `src/pages/Settings.tsx`):
  - Toggle switch for master activation.
  - Polling interval range slider (15s to 300s).
  - Low quota threshold slider (1% to 30%).
  - Evaluated target model selection (`gemini-pro`, `gemini-flash`, `claude`).
  - Auto task snapshot/resume checkbox.
  - "Rotate to Next Best" manual action button with feedback.
- **Instances View** (`src/pages/Instances.tsx`):
  - Live status banner with animated pulsing dot displaying active polling and current quota.
  - Quick action button to trigger manual failover.
- **Localization** (`src/locales/en.json` & `zh.json`):
  - Added translations for all auto-switcher settings, banner states, and instance management actions.
