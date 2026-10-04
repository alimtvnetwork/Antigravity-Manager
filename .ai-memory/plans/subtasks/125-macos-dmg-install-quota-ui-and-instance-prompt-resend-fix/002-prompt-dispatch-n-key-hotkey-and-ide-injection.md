# Subtask 002: Prompt Dispatch Engine (N Key Hotkey & Live IDE Injection)

## Owner: Worker 01 (Frontend Specialist)
## Target File: `src/components/instances/PromptTreeViewModal.tsx`

### Requirements
1. **'N' Key Hotkey Listener**:
   - Register a `useEffect` modal-level `keydown` listener for `N` and `n` keys.
   - Ignore events if target is an `HTMLInputElement`, `HTMLTextAreaElement`, `HTMLSelectElement`, or `isContentEditable`.
   - Prevent default and call `handleResendPrompt()`.
2. **"Send Now" Button & IDE Injection**:
   - Write `.antigravity_resume_task.json` to `selectedProject.repo_path`.
   - Invoke `resume_recent_project_prompts` with the instance ID.
   - If instance is not active, call `focusOrLaunchInstance`.
   - Provide immediate toast visual feedback: `"Prompt Dispatched (via Hotkey 'N' or Send Now)!"` clearing after 3.5s.
