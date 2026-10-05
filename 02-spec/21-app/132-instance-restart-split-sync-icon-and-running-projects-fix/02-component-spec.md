# Component Specification: Instance Restart Split Button, Distinct Sync Icons & Running Projects Detection

## 1. Backend IPC Command Interface

### 1.1 `restart_instance`
- **File**: `src-tauri/src/modules/instance.rs` & `src-tauri/src/commands/instance.rs`
- **Signature**:
  ```rust
  pub fn restart_instance(instance_id: &str) -> AppResult<InstanceStatus>
  ```
  ```rust
  #[tauri::command]
  pub async fn restart_instance(instance_id: String) -> Result<InstanceStatus, String>
  ```
- **Lifecycle Steps**:
  1. Resolves `instance_id` (handling `default`, `__default__`, or instance name).
  2. Calls `stop_instance(&resolved_id)` to terminate all child processes and IDE windows.
  3. Polls process status for up to 1500ms (100ms intervals) until no active process for `resolved_id` is running.
  4. Calls `launch_instance(&resolved_id)`.
  5. Queries updated `InstanceStatus` and returns it.

---

## 2. Frontend Component Interfaces

### 2.1 `src/components/instances/InstanceTable.tsx`
- **Action Types**:
  ```typescript
  export type InstanceActionType = 'launch' | 'stop' | 'restart' | 'switch' | 'fast-forward' | 'wipe' | 'delete' | 'sync' | null;
  ```
- **Prop Extension**:
  ```typescript
  interface InstanceTableProps {
      // ... existing props
      onRestart: (id: string) => void;
  }
  ```
- **Split Button Rendering**:
  - When `inst.is_running`:
    ```tsx
    <div className="inline-flex items-center rounded-[5px] border border-slate-200 dark:border-[#15334d] bg-white dark:bg-[#071a27] shadow-2xs overflow-hidden">
        <button
            type="button"
            onClick={() => onStop(inst.config.id)}
            disabled={isBusy}
            title="Stop Instance"
            className="p-1.5 text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/40 transition-colors"
        >
            <Square className="w-3.5 h-3.5 fill-current" />
        </button>
        <div className="w-px h-4 bg-slate-200 dark:bg-[#15334d]" />
        <button
            type="button"
            onClick={() => onRestart(inst.config.id)}
            disabled={isBusy}
            title="Restart Instance on Current Account"
            className="p-1.5 text-amber-600 dark:text-amber-400 hover:bg-amber-50 dark:hover:bg-amber-950/40 transition-colors"
        >
            <RotateCcw className="w-3.5 h-3.5" />
        </button>
    </div>
    ```
  - When `!inst.is_running`:
    ```tsx
    <button
        type="button"
        onClick={() => onLaunch(inst.config.id)}
        disabled={isBusy}
        title="Launch Instance"
        className="p-1.5 rounded-[5px] text-teal-600 dark:text-teal-400 hover:bg-teal-50 dark:hover:bg-teal-950/40 border border-slate-200 dark:border-[#15334d] transition-colors"
    >
        <Play className="w-3.5 h-3.5 fill-current" />
    </button>
    ```

### 2.2 `src/pages/Instances.tsx` Card Mode Split Button
- In Card action footer:
  - If `inst.is_running`: render contiguous split button `[Square | RotateCcw]`.
  - If `!inst.is_running`: render standard `Play` button.
  - Sync button replaced with `Cpu` icon with clear label *"Sync PIDs"*.

---

## 3. Running Projects Detection Engine (`repo_db.rs`)

### 3.1 Strict Isolation Rules
1. **Explicit Instance Binding**:
   - Matches only rows where `instance_id = ?` or composite key matches `__{instance_id}`.
   - For `default` instance, matches strictly `default` and `__default__`, NEVER matching rows where `instance_id IS NULL` or empty string.
2. **Turn Recency Evaluation**:
   - `is_run = is_inst_alive && ap.status == "running" && (now - ap.updated_at <= 120)`
   - Any turn older than 120 seconds without active heartbeats is marked idle.
3. **Ghost Conversation Filter**:
   - Conversations with 0 words and title starting with "untitled" or empty are skipped from running count.
