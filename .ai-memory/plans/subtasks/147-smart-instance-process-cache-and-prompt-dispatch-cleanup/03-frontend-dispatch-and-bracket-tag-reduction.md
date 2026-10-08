# Subtask 03: Frontend Dispatch Integration & Bracket Tag Reduction

## Objective
Update `src/components/instances/PromptTreeViewModal.tsx` and `src/services/instanceService.ts` to connect to new IPC commands, reduce bracket tag clutter, and ensure smooth UX.

## Actions
1. In `src/services/instanceService.ts`:
   - Export `sendPromptNow`, `enqueuePrompt`, and `getRunningInstancesProcessCount`.
2. In `src/components/instances/PromptTreeViewModal.tsx`:
   - Update `handleResendPrompt` to call `sendPromptNow`.
   - Update `handleEnqueuePrompt` to call `enqueuePrompt`.
   - Reduce visual bracket tags:
     - Replace `formatDualBadge(project.seq_code, 'P001', project.gitmap_seq_code, ...)` with a clean, compact sequence badge (e.g. `#1`, `#2` or `P001`) without bloated `[AGM:... | GM:...]` brackets.
     - Replace conversation dual-bracket tags with sleek `#25` or `C025` pill badges.
