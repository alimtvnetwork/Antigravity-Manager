# Subtask 003: Frontend In-App Delete Dialog and Card Progress Mutex

## Objective
1. Replace native browser / OS `window.confirm` dialogs for profile deletion and session wiping with custom in-app modal dialogs matching the Antigravity glass aesthetic.
2. Implement an action mutex and animated progress state on instance cards and table rows during Play (launch), Stop, and Switch actions, disabling interactive elements and providing visible progress indicators to prevent duplicate clicks.

## Target Files
- `src/pages/Instances.tsx`
- `src/components/instances/InstanceTable.tsx`
- `src/components/common/ModalDialog.tsx` (reuse or extend if needed)

## Root Causes Addressed
1. `handleDelete` and `handleWipeSession` in `src/pages/Instances.tsx` invoked `window.confirm(...)`, triggering the native OS/WebView message box instead of the in-app React modal.
2. `handleLaunch`, `stopInstance`, `handleWipeSession`, and `handleDelete` lacked per-instance in-flight tracking. Clicking "Play" or "Stop" did not lock adjacent controls, allowing rapid repeated clicks during Electron process spawn/termination.

## Implementation Blueprint
1. **Delete Confirmation State**:
   - Add `deleteModalTarget: InstanceStatus | null` to `Instances.tsx`.
   - When clicking delete icon on a card or table row, set `deleteModalTarget(inst)`.
   - Render in-app confirmation modal (`ModalDialog` with `isDestructive={true}`, title `Delete Profile`, message with profile name and sequence number, and `Confirm Delete` button).
   - Add same in-app confirmation for wipe session (`wipeModalTarget: InstanceStatus | null`).
2. **Action Mutex & Progress Indicators**:
   - Add `actionState: Record<string, 'launching' | 'stopping' | 'switching' | 'wiping' | 'deleting' | null>` in `Instances.tsx`.
   - When an action begins: set `actionState[id] = 'launching'` (or corresponding action).
   - Disable all buttons on the instance card / row (`disabled={Boolean(actionState[id])}`).
   - Render an animated spinner (`RotateCw className="w-3 h-3 animate-spin"` or custom pulsing badge) on the active button.
   - When the promise resolves or rejects, clear `actionState[id] = null`.

## Acceptance Criteria
- [x] Deleting an instance opens an in-app glass confirmation modal; zero `window.confirm` calls remain.
- [x] Clicking Play/Stop immediately disables the instance card/row buttons and renders an animated spinner.
- [x] Concurrent or spam clicks are completely blocked.
