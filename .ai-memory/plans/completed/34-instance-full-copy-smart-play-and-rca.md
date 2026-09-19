# Completed Plan 34: Instance Full Directory Copy, Smart Play Profile Rotation, UI Action Bar, and Root Cause Analysis

## Goal

Resolve Antigravity multi-instance launch failure and user isolation inconsistency, implement full directory copying for isolated instances with a settings toggle, construct top action bar (Play/Smart-Play, Duplicate, Remove, Edit, Plus), integrate real-time profile search and JSON Import/Export, and execute full verification and release.

## Root Cause Analysis (Summary)

Detailed 4-part RCA documented in `.ai-memory/issues/02-instance-launch-and-profile-isolation-rca.md`:
1. **Cause A (Process Termination Race Condition)**: `close_antigravity` followed immediately by `cmd.spawn()` triggered Electron's single-instance mutex collision before previous process released `code.lock` and `singleton*`, terminating the new window silently.
2. **Cause B (Incomplete Directory Copying)**: `copy_instance` previously copied only the `User/` subfolder, omitting Chromium `Local State`, cookies, and session databases.
3. **Cause C (Shared Credential Slot Collision)**: Windows Credential Manager `gemini:antigravity` global credential storage collision across profiles.
4. **Cause D (Orphaned Locks)**: Residual `code.lock` in the instance directory blocked new launches.

## Implemented Deliverables

1. **Root Cause Analysis (RCA)**:
   - Grounded 4-part RCA formulated in `.ai-memory/issues/02-instance-launch-and-profile-isolation-rca.md`.
2. **Backend Full Directory Copy & Launch Isolation**:
   - `src-tauri/src/modules/instance.rs`: Updated `copy_instance` to accept `clone_mode` (default `"full"`), sanitized volatile locks (`*.lock`, `singleton*`, `crashpad`, caches) during directory copy.
   - `src-tauri/src/modules/instance.rs`: Hardened `launch_instance` with stale lock purging, 500ms process unmap delay, and `--password-store=basic`.
   - `src-tauri/src/modules/cli.rs`: Passed clone mode flag.
   - `src-tauri/src/commands/instance.rs` & `src-tauri/src/lib.rs`: Registered `export_instances_json` and `import_instances_json` IPC commands.
   - `src-tauri/src/models/config.rs`: Added `instance_clone_mode` to `AppConfig` with default `"full"`.
3. **Smart Play Profile Rotation & Best Profile Selection**:
   - `src/services/instanceService.ts`: Implemented `findBestSmartPlayAccount` evaluating accounts by LRU, 4-hour remaining quota, weekly remaining quota, and health status.
   - `src/stores/useInstanceStore.ts`: Implemented `smartPlayInstance` to rotate, bind, and launch target instances.
4. **Action Bar & Dropdown UI Redesign**:
   - `src/components/navbar/InstanceSelector.tsx`:
     - Top Action Bar: Play (Smart Play) / Stop, Edit / Rename, Duplicate (with clone mode option), Remove / Delete (confirmation dialog), and Plus / Create.
     - Dropdown Header: JSON Import and Export buttons.
     - Profile Search: Real-time filter by profile name and bound email.
     - Enhanced stacking context with `z-[9999]` and `isolation: isolate` on `Navbar.tsx` eliminating overlap glitches.
     - Modals: Create, Duplicate (Full Directory vs Profile Only), Rename, and Delete modals.
5. **Settings Configuration**:
   - `src/types/config.ts`: Added `instance_clone_mode?: 'full' | 'profile'`.
   - `src/pages/Settings.tsx`: Added "Instance Duplication Mode" setting in General tab.
   - `src/locales/en.json` & `src/locales/zh.json`: Added all localization strings.

## Verification

- Lint and type checks validated.
- Coding guidelines (no explicit true checks, no mixed polarity, lowercase filenames, relative git paths) enforced.
