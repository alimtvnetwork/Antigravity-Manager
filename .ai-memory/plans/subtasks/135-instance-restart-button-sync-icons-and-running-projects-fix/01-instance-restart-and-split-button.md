# Subtask 01: Instance Restart Backend Validation & Frontend Segmented Split Button Capsule

- **Subtask Identifier**: `01-instance-restart-and-split-button`
- **Parent Task**: `135-instance-restart-button-sync-icons-and-running-projects-fix`
- **Specification References**:
  - Architecture Spec: [`02-spec/21-app/135-instance-restart-button-sync-icons-and-running-projects-fix/01-architecture-spec.md`](file:///d:/work/Antigravity-Manager/02-spec/21-app/135-instance-restart-button-sync-icons-and-running-projects-fix/01-architecture-spec.md)
  - Component Spec: [`02-spec/21-app/135-instance-restart-button-sync-icons-and-running-projects-fix/02-component-spec.md`](file:///d:/work/Antigravity-Manager/02-spec/21-app/135-instance-restart-button-sync-icons-and-running-projects-fix/02-component-spec.md)
- **Status**: Ready for Execution

---

## 1. Objectives & User Requirements

The user specified:
> "In the instance section, there should be a button to restart. That means it's going to close and restart on the current account. The same thing should actually happen with the switch button. Oh, switch button is selecting. Okay, keep it as it is. But yeah, there should be one more button, or the current play and the stop button, try to have a split. When it started, try to have a split, and here have another button, restart."

### Deliverables:
1. **Backend Validation & Robustness**:
   - Ensure `restart_instance` in `src-tauri/src/modules/instance.rs` gracefully closes processes, actively polls PID exit (< 1,500ms, 80ms interval), invalidates `prompt_tree_cache`, and relaunches on the currently bound account.
   - Verify Tauri IPC registration in `commands/instance.rs` and `lib.rs`, and service wrapper in `src/services/instanceService.ts`.
2. **Frontend Segmented Split Capsule in Table View (`InstanceTable.tsx`)**:
   - When an instance is running (`inst.is_running === true`), replace the single Stop button with a contiguous segmented pill capsule `[Stop (Square) | Restart (RotateCcw)]` with a hairline divider.
   - When stopped, render the standard standalone `Play` button.
3. **Frontend Segmented Split Capsule in Card View (`Instances.tsx`)**:
   - In Card view, render the identical contiguous segmented pill capsule `[Stop | Restart]` when running, and standalone `Play` button when stopped.
4. **Action Label & Progress Synchronization**:
   - In `Instances.tsx`, update `getActionLabel` to handle `'restart'`, returning `'Restarting...'` instead of fallback `'Processing...'`.
5. **Switch Button Invariant**:
   - Keep the account switch button (`onSwitch` / `setSwitchTargetInstance`) strictly independent. It must remain an account selection mechanism, never overloaded with restart.

---

## 2. File Locations & Touched Modules

| Module / Target | File Path | Focus Area |
|---|---|---|
| **Backend Instance Logic** | [`src-tauri/src/modules/instance.rs`](file:///d:/work/Antigravity-Manager/src-tauri/src/modules/instance.rs#L1073-L1117) | `restart_instance` implementation and process polling |
| **Tauri Command Handler** | [`src-tauri/src/commands/instance.rs`](file:///d:/work/Antigravity-Manager/src-tauri/src/commands/instance.rs#L44-L48) | `restart_instance` IPC command export |
| **IPC Registration** | [`src-tauri/src/lib.rs`](file:///d:/work/Antigravity-Manager/src-tauri/src/lib.rs#L1089) | Register `commands::restart_instance` |
| **Frontend Service API** | [`src/services/instanceService.ts`](file:///d:/work/Antigravity-Manager/src/services/instanceService.ts#L262-L272) | `restartInstance` client invoke method |
| **Table Actions View** | [`src/components/instances/InstanceTable.tsx`](file:///d:/work/Antigravity-Manager/src/components/instances/InstanceTable.tsx#L380-L440) | Segmented split capsule `[Square \| RotateCcw]` |
| **Card Actions View** | [`src/pages/Instances.tsx`](file:///d:/work/Antigravity-Manager/src/pages/Instances.tsx#L1530-L1570) | Card split capsule and `getActionLabel` mapping |

---

## 3. Step-by-Step Implementation Instructions

### Step 3.1: Backend `restart_instance` Validation
1. Verify [`src-tauri/src/modules/instance.rs`](file:///d:/work/Antigravity-Manager/src-tauri/src/modules/instance.rs):
   ```rust
   pub fn restart_instance(instance_id: &str) -> AppResult<InstanceStatus> {
       let resolved_id = resolve_instance_id(instance_id).unwrap_or_else(|_| instance_id.to_string());
       // 1. Terminate current running process(es)
       let _ = stop_instance(&resolved_id);
       
       // 2. Poll up to 1500ms for process cleanup and lockfile release
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
2. Verify IPC registration in `src-tauri/src/commands/instance.rs` and `src-tauri/src/lib.rs`.

### Step 3.2: Frontend Table Mode Segmented Split Capsule
In [`src/components/instances/InstanceTable.tsx`](file:///d:/work/Antigravity-Manager/src/components/instances/InstanceTable.tsx):
1. In the Primary Actions column, when `inst.is_running === true`:
   - Wrap the Stop and Restart buttons inside a contiguous container:
     ```tsx
     <div className="inline-flex items-center rounded-l-[5px] overflow-hidden bg-white dark:bg-[#071a27] border-r border-slate-300 dark:border-slate-700/80">
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
         <div className="w-px h-3.5 bg-slate-300 dark:bg-slate-700/80 my-auto" />
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
1. Locate the Card primary actions pill (around line 1530).
2. Render the matching contiguous segmented split capsule `[Square | RotateCcw]` when `inst.is_running === true`.
3. Render standalone `Play` button when `inst.is_running === false`.

### Step 3.4: Action Label Synchronization
In [`src/pages/Instances.tsx`](file:///d:/work/Antigravity-Manager/src/pages/Instances.tsx):
1. In `getActionLabel(action: InstanceActionType)`:
   - Ensure `case 'restart': return 'Restarting...';` is explicitly present.
   - Prevents the UI from displaying generic `'Processing...'`.

### Step 3.5: Switch Button Invariant Verification
1. Ensure the adjacent button with `ArrowLeftRight` icon remains bound exclusively to:
   - Table view: `onSwitch(inst.config.id)` -> `setSwitchTargetInstance(inst.config.id)`
   - Card view: `handleSwitchAccount(inst.config.id)` -> `setSwitchTargetInstance(inst.config.id)`
2. Ensure clicking Switch NEVER initiates process shutdown or restart.

---

## 4. Safety Gates & Edge Cases

1. **Busy State Concurrency Lock**:
   - `disabled={isBusy}` on both Stop and Restart buttons prevents double-click race conditions.
2. **Process Polling Timeout**:
   - If an Antigravity process fails to terminate within 1,500ms, `restart_instance` logs a warning and proceeds with launch rather than hanging the IPC worker thread indefinitely.
3. **Data Integrity**:
   - Profile data directory, credentials, and settings remain untouched; only processes and caches cycle.

---

## 5. Verification & Acceptance Criteria

- [ ] In Table view, running instances display contiguous `[Square | RotateCcw]` segmented capsule with subtle divider.
- [ ] In Card view, running instances display contiguous `[Square | RotateCcw]` segmented capsule.
- [ ] In both views, stopped instances display standalone `Play` button.
- [ ] Clicking Restart triggers `restart_instance` IPC call; button displays spinning icon and status pill displays *"Restarting..."*.
- [ ] After restart, the instance window relaunches bound to the identical account as before.
- [ ] Switch button opens the account selection modal and does not restart or stop the instance.
