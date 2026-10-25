---
plan: 226-antigravity-manager-clone-and-ui-enhancements
subtask: "01"
title: Non-blocking async clone engine, modal simplification, and two-level card actions
domain: backend/cloning & frontend/instances
target_files:
  - src-tauri/src/commands/instance.rs
  - src-tauri/src/modules/instance/clone.rs
  - src-tauri/src/modules/instance.rs
  - src/pages/instances/CopyInstanceDialog.tsx
  - src/components/navbar/instanceSelector/CopyInstanceModal.tsx
  - src/pages/instances/instancePageActions.ts
  - src/components/navbar/instanceSelector/useInstanceSelector.ts
  - src/pages/instances/InstanceCardActions.tsx
status: completed
---

# Subtask 01 — Non-Blocking Async Clone Engine, Modal Simplification, and Two-Level Card Actions Reversion

## Overview & Objectives
This subtask decomposes and details the concrete implementation for **Task-02**, **Task-03**, and **Task-04** defined in `.ai-memory/plans/pending/226-antigravity-manager-clone-and-ui-enhancements.md` and `02-spec/21-app/226-antigravity-manager-clone-and-ui-enhancements/01-architecture-spec.md`:
1. **Task-02**: Eliminate OS `(Not Responding)` freeze during cloning by making `copy_instance` asynchronous, offloading disk I/O to `tokio::task::spawn_blocking`, and emitting real-time progress across 6 discrete stages (`instance-clone-progress`).
2. **Task-03**: Simplify the cloning dialogs in both `src/pages/instances/CopyInstanceDialog.tsx` and `src/components/navbar/instanceSelector/CopyInstanceModal.tsx` by eliminating the `Profile Copy` option, hardcoding full IDE clone with workspace projects, and providing clean informative badges.
3. **Task-04**: Revert `src/pages/instances/InstanceCardActions.tsx` from the cramped 10-button single row into a structured two-level segmented capsule layout (Level 1: 5 buttons, Level 2: 6 buttons).

---

## Detailed Step-by-Step Implementation Instructions

### Step 1: Task-02 — Non-Blocking Clone Engine (Backend)

#### 1.1 Target File: `src-tauri/src/modules/instance/clone.rs`
- **Location**: Lines 7–482.
- **Action**: Add `CloneProgressPayload` struct and implement `copy_instance_with_progress`.
- **Implementation Details**:
  1. Define `CloneProgressPayload`:
     ```rust
     #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
     pub struct CloneProgressPayload {
         pub source_id: String,
         pub target_name: String,
         pub stage: String,
         pub percentage: u8,
         pub message: String,
         pub timestamp_ms: u64,
     }
     ```
  2. Implement `copy_instance_with_progress`:
     ```rust
     pub fn copy_instance_with_progress<F>(
         source_id: &str,
         target_name: String,
         clone_mode: Option<&str>,
         copy_projects: bool,
         mut progress: F,
     ) -> Result<InstanceConfig, String>
     where
         F: FnMut(&str, u8, &str),
     {
         // Stage 1: Initializing (10%)
         progress("initializing", 10, "Initializing clone environment and resolving target paths...");
         let resolved_source_id = resolve_instance_id(source_id).unwrap_or_else(|_| source_id.to_string());
         let registry = load_registry()?;
         let source = registry
             .instances
             .iter()
             .find(|i| i.id == resolved_source_id || i.id == source_id)
             .ok_or_else(|| format!("Source instance {} not found", source_id))?
             .clone();
         let mut new_instance = create_instance(target_name)?;

         // Stage 2: User Settings & Policies (30%)
         progress("user_settings", 30, "Replicating user settings, keybindings, snippets, and security policies...");
         // ... execute copy_required_ide_files, security_presets.json, antigravity_policies.json ...

         // Stage 3: SQLite Databases (50%)
         progress("sqlite_databases", 50, "Safely cloning SQLite state databases and storage identifiers...");
         // ... execute safe_clone_sqlite_db for state.vscdb and storage.json ...

         // Stage 4: Workspace Storage & Projects (75%)
         progress("workspace_storage", 75, "Cloning workspace storage trees, open projects, and recent folders...");
         // ... execute copy_instance_projects, clone_instance_repo_rows ...

         // Stage 5: Telemetry & Parity Sync (95%)
         progress("telemetry_sync", 95, "Scrubbing hardware telemetry salts and enforcing IDE parity...");
         // ... execute scrub_storage_json_telemetry, sync_instance_ide_parity ...

         // Stage 6: Complete (100%)
         progress("complete", 100, "Clone completed successfully.");
         // ... record task history and return new_instance ...
         Ok(new_instance)
     }
     ```
  3. Keep `copy_instance_with_options` as a backward-compatible wrapper calling `copy_instance_with_progress(..., |_, _, _| {})`.

#### 1.2 Target File: `src-tauri/src/modules/instance.rs`
- **Location**: Lines 470–530.
- **Action**: Export `CloneProgressPayload` and `copy_instance_with_progress`.

#### 1.3 Target File: `src-tauri/src/commands/instance.rs`
- **Location**: Lines 232–247.
- **Action**: Convert `copy_instance` to `pub async fn` accepting `app: tauri::AppHandle`.
- **Implementation Details**:
  ```rust
  #[tauri::command]
  pub async fn copy_instance(
      app: tauri::AppHandle,
      source_id: String,
      target_name: String,
      clone_mode: Option<String>,
      copy_projects: Option<bool>,
  ) -> Result<InstanceConfig, String> {
      let resolved_src = instance::resolve_instance_id(&source_id).unwrap_or(source_id);
      let should_copy_projs = copy_projects.unwrap_or(true);
      let effective_mode = clone_mode.unwrap_or_else(|| "full".to_string());
      let app_handle = app.clone();

      tokio::task::spawn_blocking(move || {
          let emit_progress = |stage: &str, percentage: u8, msg: &str| {
              let payload = instance::CloneProgressPayload {
                  source_id: resolved_src.clone(),
                  target_name: target_name.clone(),
                  stage: stage.to_string(),
                  percentage,
                  message: msg.to_string(),
                  timestamp_ms: std::time::SystemTime::now()
                      .duration_since(std::time::UNIX_EPOCH)
                      .unwrap_or_default()
                      .as_millis() as u64,
              };
              if let Err(err) = app_handle.emit("instance-clone-progress", payload) {
                  tracing::warn!("[Instance Clone] Failed to emit clone progress: {}", err);
              }
          };

          instance::copy_instance_with_progress(
              &resolved_src,
              target_name,
              Some(&effective_mode),
              should_copy_projs,
              emit_progress,
          )
      })
      .await
      .map_err(|e| format!("Clone task join error: {}", e))?
  }
  ```

---

### Step 2: Task-03 — Cloning Modal Simplification (Frontend)

#### 2.1 Target File: `src/pages/instances/CopyInstanceDialog.tsx`
- **Location**: Lines 72–185.
- **Action**:
  1. Remove radio cards for `cloneMode === 'full'` vs `cloneMode === 'profile'`.
  2. Remove checkbox toggle for `copyProjects`.
  3. Render informative locked card:
     - Title: `IDE Copy (Full Environment & Sessions)`
     - Badge: `Locked: Full Clone` with emerald styling and `Check` icon.
     - Description: "Clones the complete isolated runtime environment, extensions, user settings, active sessions, and workspace storage into the new profile."
  4. Render workspace locked indicator:
     - Text: `Workspace projects, recent paths, and states included`
     - Badge: `Always Included` in emerald font.
  5. Add real-time progress indicator when clone is running (`api.isBusy` or local `isCloning` state).

#### 2.2 Target File: `src/components/navbar/instanceSelector/CopyInstanceModal.tsx`
- **Location**: Lines 122–185.
- **Action**:
  1. Remove `CloneScopeCard` usage and selection for `profile`.
  2. Render the same locked full IDE copy informative card and locked workspace indicator.
  3. Ensure `copy_projects: true` and `clone_mode: "full"` are permanently supplied to `handleCopy`.

#### 2.3 Target Files: `src/pages/instances/instancePageActions.ts` & `src/components/navbar/instanceSelector/useInstanceSelector.ts`
- **Location**: Around lines 160–205.
- **Action**: Ensure default state for `cloneMode` is `'full'` and `copyProjects` is `true`.

---

### Step 3: Task-04 — Two-Level Instance Card Actions Reversion (Frontend)

#### 3.1 Target File: `src/pages/instances/InstanceCardActions.tsx`
- **Location**: Lines 62–334.
- **Action**: Revert from single 10-slot container into two contiguous segmented capsules (`space-y-1.5`):
  
  **Level 1 Capsule (`rounded-full`, 5 slots)**:
  - **Slot 1 (Lifecycle)**:
    - Running: `Stop` (`Square` in rose) + `Restart` (`RotateCcw` in sky with hover shadow).
    - Stopped: `Launch` (`Play` in teal).
  - **Slot 2 (Switch Account)**:
    - Icon: `ArrowRightLeft`.
    - 3D yellow gradient hover:
      `hover:bg-gradient-to-b hover:from-[#FDE047] hover:via-[#FACC15] hover:to-[#EAB308] hover:!text-black hover:[&_svg]:!text-black hover:font-bold hover:shadow-[0_2px_0_0_#A16207,0_3px_5px_rgba(0,0,0,0.25)] active:translate-y-0.5 active:shadow-[0_0_0_0_#A16207]`
    - Action: `setSwitchTargetInstance(inst)`.
  - **Slot 3 (Fast Forward)**:
    - Icon: `FastForward`.
    - Styling: 3D yellow gradient hover.
    - Action: `handleFastForward(inst.config.id)`.
  - **Slot 4 (Audit Trail)**:
    - Icon: `History`.
    - Action: `openAuditDialog(inst.config.id, inst.config.name)`.
  - **Slot 5 (Sync PID & Quota)**:
    - Icon: `Cpu` (with pulse indicator when syncing).
    - Action: `handleSync(inst.config.id)`.

  **Level 2 Capsule (`rounded-full`, 6 slots)**:
  - **Slot 1 (Prompt Tree)**:
    - Icon: `Layers`.
    - Visual: Cyan background + animated top-right cyan pulse dot if `hasActiveTask`.
    - Action: `openPromptTree(inst.config.id)`.
  - **Slot 2 (Settings & Sync)**:
    - Icon: `SlidersHorizontal`.
    - Action: `openSettingsDialog(inst.config.id)`.
  - **Slot 3 (Clone Profile)**:
    - Icon: `Copy`.
    - Action: `openCopyDialog(inst.config.id, inst.config.name)`.
  - **Slot 4 (Clone Executable)**:
    - Icon: `Cpu`.
    - Action: `handleCloneExecutable(inst.config.id)`.
  - **Slot 5 (Wipe Session / Reset)**:
    - Icon: `KeyRound` in amber.
    - Disabled when instance is running.
    - Action: `handleWipeSession(inst.config.id)`.
  - **Slot 6 (Delete Profile / More Actions Popover)**:
    - Icon: `MoreHorizontal`.
    - Popover contains:
      - `Resend Prompts (FIFO)` (`RotateCcw` in cyan).
      - `Export Prompts (.json)` (`FileDown` in purple).
      - `Restore Prompts (.json)` (`FileUp` in emerald).
      - Divider line.
      - `Delete Profile` (`Trash2` in rose for non-default instances).

---

## Concrete Verification Criteria & Automated Health Checks

### Verification Steps & Commands (Dry-Run / Non-Compiling)

1. **Verify No Forbidden Absolute Paths**:
   Ensure zero occurrences of forbidden drive roots:
   ```powershell
   # Ensure all references use strictly repository-relative Git paths
   python .agents/scripts/07-relative-path-fixer.py 02-spec/21-app/226-antigravity-manager-clone-and-ui-enhancements
   # Expected output: All files use strict relative paths
   ```

2. **Verify Clone Command Signature**:
   Check that `copy_instance` in `src-tauri/src/commands/instance.rs` is asynchronous:
   ```powershell
   Select-String -Path "src-tauri/src/commands/instance.rs" -Pattern "pub async fn copy_instance"
   # Expected match on src-tauri/src/commands/instance.rs
   ```

3. **Verify Modal Simplification**:
   Confirm zero occurrences of `clone_mode_profile` in `CopyInstanceDialog.tsx` and `CopyInstanceModal.tsx`:
   ```powershell
   Select-String -Path "src/pages/instances/CopyInstanceDialog.tsx", "src/components/navbar/instanceSelector/CopyInstanceModal.tsx" -Pattern "clone_mode_profile"
   # Expected output: Empty
   ```

4. **Verify Two-Level Layout Structure**:
   Confirm that `InstanceCardActions.tsx` has exactly two segmented capsule wrapper divs with `rounded-full`:
   ```powershell
   (Select-String -Path "src/pages/instances/InstanceCardActions.tsx" -Pattern "rounded-full border border-slate-200").Matches.Count
   # Expected count: 2
   ```

### Diffstat Checks & Expectations
- `src-tauri/src/commands/instance.rs`: ~25 lines modified (async conversion, `spawn_blocking`, event emission).
- `src-tauri/src/modules/instance/clone.rs`: ~50 lines added (progress checkpoints and payload).
- `src/pages/instances/CopyInstanceDialog.tsx`: ~60 lines simplified (removed profile cards and toggle).
- `src/components/navbar/instanceSelector/CopyInstanceModal.tsx`: ~60 lines simplified.
- `src/pages/instances/InstanceCardActions.tsx`: ~40 lines modified (split into two structured capsule rows).
