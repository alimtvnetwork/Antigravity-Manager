---
plan: 135-instance-restart-button-sync-icons-and-running-projects-fix
subtask: "01"
title: Instance Restart Backend Validation & Frontend Segmented Split Button Capsule
domain: backend-rust-tauri-frontend-react
depends_on:
  - 02-spec/21-app/135-instance-restart-button-sync-icons-and-running-projects-fix/01-architecture-spec.md
  - 02-spec/21-app/135-instance-restart-button-sync-icons-and-running-projects-fix/02-component-spec.md
  - 02-spec/21-app/135-instance-restart-button-sync-icons-and-running-projects-fix/03-root-cause-analysis.md
citations:
  architecture_spec: 02-spec/21-app/135-instance-restart-button-sync-icons-and-running-projects-fix/01-architecture-spec.md
  component_spec: 02-spec/21-app/135-instance-restart-button-sync-icons-and-running-projects-fix/02-component-spec.md
  root_cause_analysis: 02-spec/21-app/135-instance-restart-button-sync-icons-and-running-projects-fix/03-root-cause-analysis.md
  coding_guidelines: 02-spec/02-coding-guidelines/readme.md
  error_management: 02-spec/03-error-manage/readme.md
target_files:
  - src-tauri/src/modules/instance.rs
  - src-tauri/src/commands/instance.rs
  - src-tauri/src/lib.rs
  - src/services/instanceService.ts
  - src/components/instances/InstanceTable.tsx
  - src/pages/Instances.tsx
status: pending
---

# Subtask 01: Instance Restart Backend Validation & Frontend Segmented Split Button Capsule

## 1. Objectives & User Requirements

### User Prompt Verbatim:
> "In the instance section, there should be a button to restart. That means it's going to close and restart on the current account. The same thing should actually happen with the switch button. Oh, switch button is selecting. Okay, keep it as it is. But yeah, there should be one more button, or the current play and the stop button, try to have a split. When it started, try to have a split, and here have another button, restart."

### Deliverables:
1. **Backend Validation & Robustness**:
   - Verify and harden `restart_instance` in `src-tauri/src/modules/instance.rs`: graceful child process termination, active OS polling loop for PID exit and lockfile release (<1,500ms at 80ms intervals), cache invalidation via `invalidate_prompt_tree_cache`, and relaunch on the currently bound account.
   - Verify Tauri IPC registration in `commands/instance.rs` and `lib.rs`, and frontend service wrapper in `src/services/instanceService.ts`.
2. **Frontend Segmented Split Capsule in Table View (`InstanceTable.tsx`)**:
   - When an instance is running (`inst.is_running === true`), replace the single Stop button with a contiguous segmented pill capsule `[Stop (Square) | Restart (RotateCcw)]` with a hairline vertical divider.
   - When stopped, render the standard standalone `Play` button.
3. **Frontend Segmented Split Capsule in Card View (`Instances.tsx`)**:
   - In Card view, render the matching contiguous segmented pill capsule `[Stop | Restart]` when running, and standalone `Play` button when stopped.
4. **Action Label & Progress Synchronization**:
   - In `Instances.tsx`, update `getActionLabel` to handle `'restart'`, returning `'Restarting...'` instead of fallback `'Processing...'`.
5. **Switch Button Invariance**:
   - Maintain the account switch button (`onSwitch` / `setSwitchTargetInstance`) as a strictly independent configuration action. Clicking switch opens the account selection modal and NEVER triggers process shutdown or restart.

---

## 2. File Locations & Touched Modules

| Module / Target | File Path | Focus Area |
|---|---|---|
| **Backend Instance Logic** | [`src-tauri/src/modules/instance.rs`](file:///d:/work/Antigravity-Manager/src-tauri/src/modules/instance.rs#L1073-L1117) | `restart_instance` implementation and process polling loop |
| **Tauri Command Handler** | [`src-tauri/src/commands/instance.rs`](file:///d:/work/Antigravity-Manager/src-tauri/src/commands/instance.rs#L44-L48) | `restart_instance` IPC command export |
| **IPC Registration** | [`src-tauri/src/lib.rs`](file:///d:/work/Antigravity-Manager/src-tauri/src/lib.rs#L1089) | Register `commands::instance::restart_instance` |
| **Frontend Service API** | [`src/services/instanceService.ts`](file:///d:/work/Antigravity-Manager/src/services/instanceService.ts#L262-L272) | `restartInstance` client invoke method |
| **Table Actions View** | [`src/components/instances/InstanceTable.tsx`](file:///d:/work/Antigravity-Manager/src/components/instances/InstanceTable.tsx#L380-L440) | Segmented split capsule `[Square \| RotateCcw]` |
| **Card Actions View** | [`src/pages/Instances.tsx`](file:///d:/work/Antigravity-Manager/src/pages/Instances.tsx#L1530-L1570) | Card split capsule and `getActionLabel` mapping |

---

## 3. Step-by-Step Implementation Instructions

### Step 3.1: Backend `restart_instance` Validation & Lifecycle Polling
1. In [`src-tauri/src/modules/instance.rs`](file:///d:/work/Antigravity-Manager/src-tauri/src/modules/instance.rs):
   - Ensure `restart_instance` follows the strict atomic lifecycle:
     ```rust
     pub fn restart_instance(instance_id: &str) -> AppResult<InstanceStatus> {
         let resolved_id = resolve_instance_id(instance_id).unwrap_or_else(|_| instance_id.to_string());
         
         // 1. Terminate current running process(es)
         let _ = stop_instance(&resolved_id);
         
         // 2. Poll up to 1,500ms for process cleanup and lockfile release
         let start_wait = std::time::Instant::now();
         if let Ok(registry) = load_registry() {
             if let Some(config) = registry.instances.iter().find(|i| i.id == resolved_id) {
                 let is_default = config.is_default || resolved_id == "default";
                 while start_wait.elapsed() < std::time::Duration::from_millis(1500) {
                     let pids = find_pids_for_data_dir(&config.data_dir, is_default);
                     if pids.is_empty() {
                         break;
                     }
                     std::thread::sleep(std::time::Duration::from_millis(80));
                 }
             }
         }
         
         // 3. Invalidate prompt tree cache so running status reflects fresh state
         crate::modules::repo_db::invalidate_prompt_tree_cache(Some(&resolved_id));
         
         // 4. Launch instance with its existing bound account and workspaces
         launch_instance(&resolved_id)?;
         
         // 5. Return updated InstanceStatus
         let statuses = list_instances().map_err(AppError::Unknown)?;
         let updated = statuses
             .into_iter()
             .find(|s| s.config.id == resolved_id || s.config.name == resolved_id)
             .ok_or_else(|| {
                 AppError::Process(format!("Instance '{}' not found in registry after restart", resolved_id))
             })?;
         Ok(updated)
     }
     ```
2. Verify IPC command handler in [`src-tauri/src/commands/instance.rs`](file:///d:/work/Antigravity-Manager/src-tauri/src/commands/instance.rs):
   ```rust
   #[tauri::command]
   pub async fn restart_instance(instance_id: String) -> Result<InstanceStatus, String> {
       crate::modules::instance::restart_instance(&instance_id).map_err(|e| e.to_string())
   }
   ```
3. Verify registration in [`src-tauri/src/lib.rs`](file:///d:/work/Antigravity-Manager/src-tauri/src/lib.rs) invoke handler.
4. Verify frontend service wrapper in [`src/services/instanceService.ts`](file:///d:/work/Antigravity-Manager/src/services/instanceService.ts):
   ```typescript
   export const restartInstance = async (instanceId: string): Promise<InstanceStatus> => {
       if (isTauri()) {
           return await invoke<InstanceStatus>('restart_instance', { instanceId });
       }
       const res = await fetch(`${API_BASE}/api/instances/${instanceId}/restart`, { method: 'POST' });
       if (!res.ok) throw new Error(await res.text());
       return await res.json();
   };
   ```

### Step 3.2: Frontend Table Mode Segmented Split Capsule
In [`src/components/instances/InstanceTable.tsx`](file:///d:/work/Antigravity-Manager/src/components/instances/InstanceTable.tsx):
1. In the Primary Actions column, when `inst.is_running === true`:
   - Wrap Stop and Restart buttons in a contiguous capsule with shared borders:
     ```tsx
     <div className="inline-flex items-center rounded-l-[5px] overflow-hidden bg-white dark:bg-[#071a27] border-r border-slate-300 dark:border-slate-700/80">
         {/* Stop Button */}
         <button
             type="button"
             disabled={isBusy}
             onClick={() => onStop(inst.config.id)}
             className="px-2 py-1 text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/40 transition-colors cursor-pointer disabled:opacity-50"
             title="Stop Instance"
         >
             {currentAction === 'stop' ? (
                 <RotateCw className="w-3 h-3 animate-spin text-rose-500" />
             ) : (
                 <Square className="w-3 h-3 fill-current" />
             )}
         </button>
         
         {/* Hairline Divider */}
         <div className="w-px h-3.5 bg-slate-300 dark:bg-slate-700/80 my-auto" />
         
         {/* Restart Button */}
         <button
             type="button"
             disabled={isBusy}
             onClick={() => onRestart(inst.config.id)}
             className="px-2 py-1 text-amber-600 dark:text-amber-400 hover:bg-amber-50 dark:hover:bg-amber-950/40 transition-colors cursor-pointer disabled:opacity-50"
             title="Restart Instance on Current Account"
         >
             {currentAction === 'restart' ? (
                 <RotateCw className="w-3 h-3 animate-spin text-amber-500" />
             ) : (
                 <RotateCcw className="w-3 h-3" />
             )}
         </button>
     </div>
     ```
2. When `inst.is_running === false`:
   - Render standalone `Play` button (`rounded-l-[5px]`).
3. Maintain the Context Dropdown Menu item *"Restart Instance"* calling `onRestart(inst.config.id)`.

### Step 3.3: Frontend Card Mode Segmented Split Capsule
In [`src/pages/Instances.tsx`](file:///d:/work/Antigravity-Manager/src/pages/Instances.tsx):
1. Locate the Card primary actions container.
2. When `inst.is_running === true`:
   - Render the identical contiguous segmented split capsule `[Square (Stop) | RotateCcw (Restart)]` with divider.
3. When `inst.is_running === false`:
   - Render the standalone `Play` button.

### Step 3.4: Action Label Synchronization
In [`src/pages/Instances.tsx`](file:///d:/work/Antigravity-Manager/src/pages/Instances.tsx):
1. In `getActionLabel(action: InstanceActionType)`:
   ```typescript
   function getActionLabel(action: InstanceActionType): string {
       switch (action) {
           case 'launch':
               return 'Launching...';
           case 'stop':
               return 'Stopping...';
           case 'restart':
               return 'Restarting...'; // REQUIRED: Prevents fallback to generic 'Processing...'
           case 'switch':
               return 'Switching...';
           case 'fast-forward':
               return 'Rotating...';
           case 'sync':
               return 'Syncing...';
           case 'wipe':
               return 'Wiping...';
           case 'delete':
               return 'Deleting...';
           default:
               return 'Processing...';
       }
   }
   ```

### Step 3.5: Switch Button Selection Invariance
1. Ensure the adjacent button with `ArrowLeftRight` icon remains bound exclusively to:
   - Table view: `onSwitch(inst.config.id)` -> `setSwitchTargetInstance(inst.config.id)`
   - Card view: `handleSwitchAccount(inst.config.id)` -> `setSwitchTargetInstance(inst.config.id)`
2. Ensure clicking Switch ONLY opens the account binding modal. It must NEVER initiate process shutdown or restart.

---

## 4. Safety Gates & Edge Cases

1. **Double-Click & Concurrency Lock**:
   - `disabled={isBusy}` applied to both Stop and Restart segments prevents concurrent IPC requests.
2. **Process Teardown Timeout Fallback**:
   - If an Antigravity process fails to terminate within 1,500ms, `restart_instance` logs a warning and proceeds with launch rather than hanging the worker thread indefinitely.
3. **Account Credential Continuity**:
   - Data directory, account bindings (`email`), and environment settings remain completely intact throughout the restart cycle.

---

## 5. Verification & Acceptance Criteria

- [ ] In Table view, running instances display contiguous `[Square | RotateCcw]` segmented capsule with subtle divider.
- [ ] In Card view, running instances display contiguous `[Square | RotateCcw]` segmented capsule.
- [ ] In both views, stopped instances display standalone `Play` button.
- [ ] Clicking Restart triggers `restart_instance` IPC call; button displays spinning icon and status pill displays *"Restarting..."*.
- [ ] After restart, the instance window relaunches bound to the identical account as before.
- [ ] Switch button opens the account selection modal and does not restart or stop the instance.
