# Subtask 02: Real-Time IDE Conversation Title Renaming Synchronization

- **Slug**: `ide-title-rename-synchronization`
- **Component**: Conversation Metadata & IDE Workspace Sync
- **Primary Files**:
  - `src/components/instances/PromptTreeView/DetailHeader.tsx`
  - `src-tauri/src/commands/instance.rs`
  - `src-tauri/src/modules/repo_db/agy.rs`
- **Status**: QUEUED

---

## 1. Problem Statement
The user explicitly stated:
> "The rename, let me test out if the rename works. Rename does not work. If I double-click and rename, it does not affect to the IDE. Check that you are not syncing with the IDE. That is the main problem. Renaming of the title. I'm going to give you again the screenshot. Try to find the root cause, try to fix it."

## 2. Technical Requirements
1. **Identify IDE Conversation Title Storage**:
   - Antigravity IDE (VS Code based agentic tool) stores conversation titles in its session metadata (e.g. `state.vscdb`, `workspaceStorage`, or session JSON).
   - In addition to updating `repo_prompts.db`, AGM must synchronize the new title to the IDE's state storage so that both AGM and the IDE UI display the exact same updated title.
2. **Backend Tauri Command**:
   - Provide `rename_conversation(instance_id, conversation_id, new_title)`:
     - Updates SQLite `conversations` table in `repo_prompts.db`.
     - Locates the Antigravity instance's workspace storage / `state.vscdb` or conversation transcript/metadata file, and updates the title.
     - Logs audit record.
3. **Frontend Hook Integration**:
   - In `DetailHeader.tsx:handleCommitTitle`, invoke `rename_conversation` via `invoke('rename_conversation', ...)`.
   - Update local state and tree cache.

## 3. Verification Criteria
- Renaming a conversation title from AGM reflects in both AGM UI and the IDE session metadata.
