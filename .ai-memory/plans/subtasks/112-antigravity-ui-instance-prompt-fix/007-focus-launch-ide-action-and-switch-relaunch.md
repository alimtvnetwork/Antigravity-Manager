---
plan: 112-antigravity-ui-instance-prompt-fix
subtask: "007"
title: Implement focus or launch IDE action and enforce full switch relaunch
domain: fullstack
depends_on:
  - 006-prompt-details-modal-and-markdown-newline-preview.md
citations:
  app_spec: .ai-memory/plans/112-antigravity-ui-instance-prompt-fix.md
  coding_guidelines: AGENTS.md
  strictly_avoid: 02-spec/02-coding-guidelines/01-strictly-avoid.md
target_files:
  - src-tauri/src/modules/instance.rs
  - src-tauri/src/commands/instance.rs
  - src-tauri/src/lib.rs
  - src/services/instanceService.ts
  - src/components/instances/PromptTreeViewModal.tsx
status: pending
---

# 007 — Implement Focus or Launch IDE Action and Enforce Full Switch Relaunch

## 1. Context & Rationale
When working with prompt management in `PromptTreeViewModal.tsx`, users frequently enqueue or resend prompts and need to immediately jump into the running IDE window to verify that the conversation turns and AI responses are executing. Currently, there is no direct mechanism from the prompt modal to bring the target IDE window to the foreground. If the IDE is not running, users are forced to close the modal, navigate to Instances, and click Launch.

Additionally, when users click "Switch Account" for an instance, the IDE must reliably relaunch with the newly applied account profile. In the current implementation of `switch_account_to_instance`, default instances follow a bifurcated legacy path that returns early without executing the unified prompt backup, clean process closure, and deterministic relaunch sequence that non-default instances use. The user explicitly requested:
> *"the switch button should act as a relaunch of the IDE make sure of it. please"*

## 2. Target Files and Symbols
- **`src-tauri/src/modules/instance.rs`**:
  - `focus_or_launch_instance(&str) -> Result<bool, crate::error::AppError>`: New function checking whether the instance is running; brings it to the foreground if alive, or launches it if dead.
  - `switch_account_to_instance`: Unify the switch lifecycle so that switching always closes the running process and relaunches the IDE with the newly injected credentials and prompts.
- **`src-tauri/src/commands/instance.rs`**:
  - `focus_or_launch_instance(instance_id: String) -> Result<bool, crate::error::AppError>`: Expose as Tauri IPC command.
- **`src-tauri/src/lib.rs`**:
  - Register `commands::instance::focus_or_launch_instance` in `generate_handler!`.
- **`src/services/instanceService.ts`**:
  - `focusOrLaunchInstance(instanceId: string): Promise<boolean>`: Expose IPC helper.
- **`src/components/instances/PromptTreeViewModal.tsx`**:
  - Action Header: Add a **"Focus IDE"** button adjacent to Resend and Enqueue actions.

## 3. Detailed Technical Requirements

### 3.1. `focus_or_launch_instance` Core Logic in `instance.rs`
1. Resolve the target `instance_id` in `instances.json` (fallback to `"default"`).
2. Scan active processes for this instance's data directory via `find_pids_for_data_dir(&instance.data_dir, true)`.
3. If alive:
   - Call `crate::modules::process::focus_instance_pids(&pids)`.
   - On Windows: Uses Win32 `ShowWindow(SW_RESTORE)` and `SetForegroundWindow` on the main window handle.
   - On macOS: Uses AppleScript `osascript` to bring the process frontmost.
   - On Linux: Uses `xdotool windowactivate` or `wmctrl`.
   - Return `Ok(true)` indicating the instance was focused.
4. If not alive (or if focus activation failed):
   - Call `launch_instance(&instance.id)` to launch the IDE with proper workspace folders and parameters.
   - Return `Ok(false)` indicating the instance was launched.

```rust
pub fn focus_or_launch_instance(instance_id: &str) -> Result<bool, crate::error::AppError> {
    let registry = load_registry().map_err(crate::error::AppError::Config)?;
    let inst = registry
        .instances
        .iter()
        .find(|i| i.id == instance_id)
        .ok_or_else(|| {
            crate::error::AppError::Config(format!("Instance '{}' not found", instance_id))
        })?;

    let pids = find_pids_for_data_dir(&inst.data_dir, true);
    if !pids.is_empty() {
        if crate::modules::process::focus_instance_pids(&pids) {
            crate::modules::logger::log_info(&format!(
                "[Instance] Successfully focused running instance '{}' (PIDs: {:?})",
                instance_id, pids
            ));
            return Ok(true);
        }
    }

    crate::modules::logger::log_info(&format!(
        "[Instance] Instance '{}' is not running. Launching new instance...",
        instance_id
    ));
    launch_instance(&inst.id)?;
    Ok(false)
}
```

### 3.2. Mandatory IDE Relaunch on Account Switch
In `switch_account_to_instance`:
1. Ensure the default instance no longer returns early without relaunching.
2. Follow the strict 5-stage transition:
   - **Stage 1 (Snapshot)**: Backup running prompts for the target instance (`backup_running_prompts`).
   - **Stage 2 (Kill First)**: Close existing IDE processes running under the target data directory (`close_instance(&instance.id)` and wait 300ms for locks to clear).
   - **Stage 3 (Inject Second)**: Inject new OAuth credentials and device profiles into `state.vscdb`, `storage.json`, and system keyring.
   - **Stage 4 (Start Third)**: Unconditionally launch the target instance (`launch_instance_with_workspaces` or `start_antigravity_with_fallback_path`).
   - **Stage 5 (Restore)**: Spawn non-blocking 5-second asynchronous delayed prompt restoration (`restore_and_inject_prompts_for_instance`).

### 3.3. Frontend IPC & "Focus IDE" Button in `PromptTreeViewModal.tsx`
In `src/services/instanceService.ts`:
```typescript
export async function focusOrLaunchInstance(instanceId: string): Promise<boolean> {
    try {
        return await invoke('focus_or_launch_instance', { instanceId });
    } catch (e: any) {
        useErrorStore.getState().captureError(e, {
            source: 'instanceService.focusOrLaunchInstance',
            endpoint: 'focus_or_launch_instance',
            triggerAction: 'focus_or_launch_instance',
            context: { instanceId },
        });
        throw e;
    }
}
```

In `PromptTreeViewModal.tsx`:
Add state and handler:
```typescript
const [isFocusing, setIsFocusing] = useState(false);

const handleFocusIDE = async () => {
    const targetInstId = selectedProject?.instance_id || instanceId || 'default';
    try {
        setIsFocusing(true);
        const focused = await focusOrLaunchInstance(targetInstId);
        setActionMsg(focused ? 'IDE brought to foreground!' : 'IDE launched and opened!');
        setTimeout(() => setActionMsg(null), 3000);
    } catch (err: any) {
        setError(err?.toString() || 'Failed to focus/launch IDE');
    } finally {
        setIsFocusing(false);
    }
};
```

Render button inside the actions capsule:
```tsx
{/* Focus IDE Action Button */}
<button
    type="button"
    onClick={handleFocusIDE}
    disabled={isFocusing}
    className="flex items-center gap-1.5 px-3 py-1.5 rounded-[5px] bg-cyan-50 dark:bg-[#0c2438] text-cyan-700 dark:text-cyan-300 border border-cyan-300/60 dark:border-[#15334d] hover:bg-cyan-100 dark:hover:bg-[#15334d] text-xs font-semibold transition-colors cursor-pointer disabled:opacity-50"
    title="Focus running IDE or launch it if not open"
>
    <ExternalLink className={cn('w-3.5 h-3.5 text-cyan-500', isFocusing && 'animate-spin')} />
    <span>Focus IDE</span>
</button>
```

## 4. Constraints & Non-Negotiables
- Must work cross-platform: Windows (Win32), macOS (osascript), Linux (xdotool/wmctrl).
- Switching accounts must always terminate old IDE process and spawn a new IDE process.
- Non-blocking 5-second asynchronous delayed prompt restoration must not block the IPC return.
- Unit and pre-flight checks: `cargo fmt -- --check`, `cargo clippy`, and `npm run build`.

## 5. Out of Scope
- Linux AppImage sandbox permission fixes (covered in 008).
- Progress bar and Instance Card UI redesign (handled by Worker 01).

## 6. Verification & Acceptance Tests
1. **Focus IDE When Alive**: Start Antigravity IDE. Click "Focus IDE" in `PromptTreeViewModal`. Verify the IDE window immediately pops to the foreground.
2. **Launch IDE When Dead**: Ensure Antigravity IDE is closed. Click "Focus IDE". Verify the IDE process spawns and opens to the active project workspace.
3. **Switch Account Relaunch**: Switch account on an instance. Verify that the running IDE closes, reopens with the new profile credentials, and restored prompts appear after 5 seconds.
4. **Pre-flight Checks**:
   - `cd src-tauri && cargo fmt -- --check`
   - `npm run build`
