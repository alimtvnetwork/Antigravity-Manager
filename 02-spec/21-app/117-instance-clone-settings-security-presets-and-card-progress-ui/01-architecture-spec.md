# Specification 117: Deep Instance Cloning, Security Presets, Settings Sync, and Card Mutex UI Overhaul

## 1. Problem Classification & Scope

### A. Deep Instance Cloning Engine Defects
1. **Source Instance ID Resolution Failure**:
   `copy_instance_with_options` performed a naive string match (`registry.instances.iter().find(|i| i.id == source_id)`). When invoked via CLI tooling (`copy-profile`, `instance copy`) or GUI with sequence numbers (`"#1"`, `"1"`) or aliases (`"Default"`, `"default"`), it failed to resolve the actual instance UUID.
2. **SQLite Locking & Concurrency Violations on Running Instances**:
   When cloning from a running Antigravity instance, SQLite database files (`state.vscdb` in `globalStorage` and within each `workspaceStorage/<hash>/`) are locked by Electron/SQLite.
   - `safe_clone_sqlite_db` called `Backup::run_to_completion`, which does not respect `busy_timeout` and failed immediately on open transactions.
   - Fallback `fs::copy` failed with Windows OS Error 32 (`ERROR_SHARING_VIOLATION`).
   - `merge_state_vscdb_recent_paths` opened SQLite in read-write mode without read-only flags or URI mode, failing on running instances and resulting in zero recent workspaces being cloned.
3. **Loss of User Settings in "Copy Settings"**:
   `copy_instance_settings` only copied keys matching `workbench.*theme*`, `workbench.*color*`, `antigravity.*`, and `*policy*`. All user preferences (`editor.*`, `terminal.*`, `files.autoSave`, `snippets/`, `keybindings.json`) were discarded.
4. **Security Presets & Policies Dropping**:
   `security_presets.json`, `antigravity_policies.json`, and `.gemini/policies` were skipped in settings replication and failed to fall back to the system profile when cloning from an uninitialized named instance.

### B. Instance Settings Modal Layout & Text Clutter
1. **Replication Dropdown Overlap**: The "Split Paste" menu used an `absolute` dropdown (`z-[9999]`) that physically covered the "Workspaces & Folders" card beneath it.
2. **Horizontal Card Squeeze**: In 2-column cards, `<select>` dropdowns and buttons were squashed into ~300px columns.
3. **Verbose Text**: Missing icons on toggle buttons; long strings ("Toggle (Disable)", "Set: Always Proceed") took excessive space.
4. **Scattered JSON Buttons**: 5 separate buttons wrapped across 2–3 jagged rows without unified grouping.

### C. In-App Delete Dialog & Card Mutex Feedback
1. **Delete Dialog**: Ensure delete flow exclusively uses an in-app React modal (`ModalDialog`) with destructive styling, sequence number, and directory path, with zero native OS `window.confirm`.
2. **Card & Row Progress Mutex**:
   When launching, stopping, syncing, or wiping an instance, lock the entire card and row (`pointer-events-none opacity-60` or full-card glass overlay), prevent duplicate clicks on all card elements (including project double-clicks), and display animated progress feedback (`RotateCw` spinner and action label).

---

## 2. Technical Architecture

### A. Backend Cloning & Storage Engine (`src-tauri/src/modules/instance.rs`)
1. **Universal Source ID Resolution**:
   Call `resolve_instance_id(source_id)` at the entry point of `copy_instance_with_options` and across all CLI cloning commands in `cli.rs`.
2. **Immutable Read-Only SQLite Cloning**:
   In `safe_clone_sqlite_db`:
   - Open source database with `OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_URI`.
   - Append `?mode=ro&immutable=1` URI parameters to bypass write locks.
   - Run incremental backup loop (`step(100)`) with retry backoff up to 3 seconds for transient busy states.
   - If backup fails, use memory-buffered fallback reading via standard read-share file streams.
3. **Full User Settings Replication**:
   - Deep-merge ALL valid JSON keys in `settings.json` between source and target.
   - Copy `keybindings.json`, `snippets/`, `security_presets.json`, and `antigravity_policies.json`.
   - In `copy_gemini_trees`, fall back to `%USERPROFILE%/.gemini/policies` if the source instance home has not yet seeded policy definitions.
4. **Recent Workspaces & WorkspaceStorage Cloning**:
   - In `merge_state_vscdb_recent_paths`, open source DB in read-only URI mode.
   - Migrate `history.recentlyOpenedPathsList`, `profileAssociations.*`, `workbench.colorTheme`, and active workspaces into target DB.
   - Clone all workspace folders under `User/workspaceStorage/` using resilient file traversal that catches per-file lock errors without aborting the parent directory.

### B. Frontend UI & Interaction Improvements
1. **Instance Settings Modal (`src/components/instances/InstanceSettingsModal.tsx`)**:
   - Replace verbose toggle buttons with compact segmented pill switches and Lucide icons (`Zap`, `CheckCircle2`, `Sliders`, `Sparkles`).
   - Fix replication bar dropdown positioning and card vertical flow so dropdowns never overlap cards below.
   - Group JSON tools into a contiguous segmented capsule (`[Copy | Paste | Export | Import | Raw Editor]`).
2. **In-App Delete Dialog**:
   - Maintain sleek `ModalDialog` in `src/pages/Instances.tsx` for deletion with profile details, sequence badge, and path.
3. **Card & Row Mutex Overlay (`src/pages/Instances.tsx` & `src/components/instances/InstanceTable.tsx`)**:
   - When `isBusy` (action in progress for an instance):
     - Card view: Render a glass backdrop overlay with spinning `RotateCw` and active state label ("Launching...", "Stopping...", "Syncing...", etc.), intercepting all mouse events.
     - Table view: Apply `pointer-events-none select-none opacity-60` to the table row and display an animated transition pill in the Status & PID column.
