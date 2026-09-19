# Completed Plan 35: Instance Launch RCA, Full Copy Isolation, Smart Play Rotation, and Release

## Task Metadata
- **Parent Task**: Instance launch failure RCA, full directory cloning, smart play account auto-rotation, top action bar with in-dropdown search/JSON transfer, and automated release.
- **Execution Loops / Steps**: 15 steps.
- **Completed Date**: 2026-09-19
- **Outcome**: 100% verified and released.

---

## 1. Initial State & Problem Statement
Users reported two distinct issues when operating Antigravity with custom instance profiles:
1. When selecting a non-default profile, Antigravity failed to launch or silently terminated immediately.
2. When instances opened, both instances displayed the identical Google account and profile.
3. Users requested a full directory clone by default with a settings toggle, a top action bar (Play, Edit, Duplicate, Remove, Plus), in-dropdown search, JSON import/export, and smart account auto-rotation.

---

## 2. Root Cause Analysis (RCA Summary)
- **Root Cause A (Lock Contention)**: Electron creates `code.lock` and `singleton*` files in the data directory. Spawning a new instance before prior processes fully unmapped caused immediate exit.
- **Root Cause B (Incomplete Cloning)**: Copying only `User/` omitted `Local State`, cookies, and session databases.
- **Root Cause C (Credential Keyring Collision)**: Windows Credential Manager writes to a single global key `gemini:antigravity`. Separate `--user-data-dir`, `--password-store=basic`, and isolated `state.vscdb` injection prevent collisions.
- **Root Cause D (Stale Singleton Locks)**: Crashed processes left orphaned lock files.
- **Root Cause E (Detection Order & Parallel Process Scope)**: `launch_instance` previously called global `close_antigravity` before detection, breaking process-based executable discovery and terminating all other running parallel instances. Fixed by discovering executable first, and scoping process termination strictly to `find_pids_for_data_dir`.

---

## 3. Implemented Features & Architecture

### A. Full Directory Cloning with Sanitization
- `src-tauri/src/modules/instance.rs`: Implemented recursive cloning of full root directory (`Local State`, databases, cookies, `User/`) while skipping volatile locks (`code.lock`, `singleton*`, `crashpad/`, and caches).
- Added `instance_clone_mode` configuration (`"full"` vs `"profile"`, default `"full"`).
- Settings UI and Duplicate modal provide interactive toggle.

### B. Top Action Bar & Dropdown UI
- `src/components/navbar/InstanceSelector.tsx`:
  - Top Action Bar: **Play/Stop** (Smart Play), **Edit/Rename** (Pencil), **Duplicate** (Copy), **Remove** (Trash2), **Add (+)**.
  - In-Dropdown real-time profile search filter.
  - JSON Import / Export buttons for profile configurations.
  - Dropdown elevation (`z-[9999]`, `isolation: isolate`).

### C. Smart Play Account Auto-Rotation
- `src/services/instanceService.ts` & `src/stores/useInstanceStore.ts`:
  - Multi-factor scoring prioritizing least recently used (`last_used`), lowest recent 4-hour quota, lowest weekly quota, and active health status.
  - Automatically binds best account to target instance and launches.

### D. Scoped Multi-Instance Process Management
- `launch_instance` checks `find_pids_for_data_dir(&data_dir, is_default)` and closes only the target instance's process, preserving other parallel running instances.
- Detects executable prior to terminating target process so Strategy 1 process detection succeeds.

---

## 4. Verification & Release
- All 27 CI/CD quality gates verified green via `python 03-ai-scripts/06-cicd-local-runner.py`.
- Automated release orchestrated via `python 03-ai-scripts/29-release-orchestrator.py`.
