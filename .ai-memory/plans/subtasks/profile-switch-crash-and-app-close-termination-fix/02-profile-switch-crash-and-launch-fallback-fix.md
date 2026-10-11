# Subtask 02: Profile Switch Crash & Launch Fallback Fix

## Context
Profile / account switching was failing with "Process error: Failed to spawn instance process: Access is denied. (os error 5)" and opening a crash modal.

## Files to Modify
- `src-tauri/src/modules/instance/launch.rs`
- `src-tauri/src/modules/instance/switch.rs`
- `src/services/instanceService/lifecycle.ts`
- `src/pages/instances/SwitchAccountDialog.tsx`

## Acceptance Criteria
1. `launch.rs` implements a 4-stage Windows launch fallback ladder (Direct -> Base Detected -> cmd.exe /s /c start -> powershell Start-Process).
2. `switch.rs` treats Stage 5 auto-relaunch failure as non-fatal since credentials and account binding are already successfully persisted.
3. Switch account button does not crash or display an error modal.
