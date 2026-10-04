---
plan: 117-instance-clone-settings-security-presets-and-card-progress-ui
subtask: "005"
title: Frontend in-app delete confirmation dialog and card/row action mutex overlay
domain: frontend/ui
target_files:
  - src/pages/Instances.tsx
  - src/components/instances/InstanceTable.tsx
status: pending
---

# 005 — Frontend In-App Delete Confirmation Dialog and Card/Row Action Mutex Overlay

## Scope
1. In `src/pages/Instances.tsx`:
   - Delete confirmation:
     - Ensure delete flow exclusively uses the in-app `ModalDialog` with destructive red styling, sequence number, and directory path.
     - Add guard checking if instance is running before allowing deletion.
     - Reset `deleteModalTarget` cleanly in `finally` blocks.
   - Card Mutex & Progress Feedback:
     - When an action (`launch`, `stop`, `sync`, `wipe`, `fast-forward`) is executing on an instance (`isBusy` is true):
       - Render a sleek glass mutex overlay across the entire card container with `backdrop-blur-[1.5px]`, spinning `RotateCw` icon, and status label (e.g. "Launching...", "Stopping...", "Processing...").
       - Intercept and block all clicks, double-clicks on projects, and button presses on that card.
2. In `src/components/instances/InstanceTable.tsx`:
   - When `isBusy` is true for a table row:
     - Apply `pointer-events-none select-none opacity-60` to the row.
     - In the Status & PID column, replace static badges with an animated transition pill showing the spinner and active action label.
