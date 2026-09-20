# Subtask 03: Frontend Instance Rotation Integration

## Status: Completed

## Parent Plan
[Plan 48: Instance Smart Rotation, Process Termination, Account Selection Algorithm & Minor Release v4.37.0](.ai-memory/plans/completed/48-instance-smart-rotation-account-switch.md)

## Goal
Integrate the complete Smart Switch lifecycle into the FastForward (`>>`) buttons across the UI with clear feedback.

## Key Actions
1. In `src/stores/useInstanceStore.ts`:
   - Implement `smartRotateProfileAccount(targetInstanceId?: string)` orchestrating:
     - Process teardown of target instance location.
     - Candidate account discovery using `findSmartRotationAccount`.
     - Live quota refresh and token validation.
     - Profile account token injection via `switchAccountToInstance`.
     - Refreshing instance and account states.
2. In `src/components/navbar/InstanceSelector.tsx`:
   - Connect active profile FastForward button and dropdown row FastForward buttons to `smartRotateProfileAccount`.
   - Update tooltip and toasts with clear explanations (candidate account email, runway days).
3. In `src/pages/Instances.tsx`:
   - Connect FastForward buttons to `smartRotateProfileAccount`.
4. Run `npx tsc --noEmit` to ensure TypeScript compilation without errors.

## Constraints
- Zero Chinese characters in new/modified comments or UI strings.
- Implicit boolean checks only.
