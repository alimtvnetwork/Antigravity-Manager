# Subtask 002: Frontend N Key Hotkey, Send Now Dispatch, Header Metadata Trio & Tail Excerpt

## Owner: Worker 01 (Frontend Specialist)
## Target File: `src/components/instances/PromptTreeViewModal.tsx`

### Objective
1. **'N' Key Hotkey Listener**:
   - Register a `window.addEventListener('keydown', ...)` when the modal is open:
     - Check if `(e.key === 'n' || e.key === 'N') && !e.ctrlKey && !e.altKey && !e.metaKey`.
     - Check if event target is an `HTMLInputElement` or `HTMLTextAreaElement` or `HTMLSelectElement` or `isContentEditable`. If so, do not trigger.
     - Otherwise, prevent default and invoke `handleResendPrompt()`.
2. **"Send Now" Button Dispatch**:
   - Ensure `handleResendPrompt`:
     - Checks that `selectedConversation` exists.
     - Writes `.antigravity_resume_task.json` into `selectedProject.repo_path`.
     - Calls Tauri command `resume_recent_project_prompts`.
     - Sets action message: `"Prompt Dispatched (via Hotkey 'N' or Send Now)!"` and clears after 3.5s.
3. **Prompt Header Metadata Trio & Tail Snippet**:
   - Display in the prompt view header (lines 1760-1820) and prompt instruction banner (lines 1910-1940):
     - Prompt Sequence badge: `#P001` or `C001` (from `selectedConversation.gitmap_seq_code` or `selectedConversation.seq_code`).
     - Instance Trio pill:
       `[#${selectedProject.instance_seq_num || 1} · ${selectedProject.instance_exe_name || 'Antigravity'} · ${selectedProject.instance_name || 'default'}]`
     - Tail excerpt snippet:
       Extract the last 12 words of `activePromptText` and display:
       `“… ending with: '${tailSnippet}'”`
4. **Filter Empty / Untitled Conversations in Tree**:
   - In conversation list rendering, hide items where `conv.prompt_word_count === 0 && isStaleOrEmptyConversation(conv)` from active navigation.
