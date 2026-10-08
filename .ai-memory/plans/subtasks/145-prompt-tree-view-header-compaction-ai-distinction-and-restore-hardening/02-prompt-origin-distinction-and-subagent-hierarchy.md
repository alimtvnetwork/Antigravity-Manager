# Subtask 02: Prompt Origin Distinction and Subagent Hierarchy

**Parent Plan:** [145-prompt-tree-view-header-compaction-ai-distinction-and-restore-hardening.md](../../145-prompt-tree-view-header-compaction-ai-distinction-and-restore-hardening.md)  
**Status:** `IN PROGRESS`  

---

## Objectives
1. Implement 3-tier classification in `repo_db.rs` and `PromptTreeViewModal.tsx`:
   - `USER_PROMPT`: Human user-initiated prompt (top-level root node).
   - `AI_SUBAGENT`: Autonomous subagent instruction (`invoke_subagent`, background worker, etc.) rendered as nested sub-points (`↳ [AI Subagent]`).
   - `NON_PROMPT`: Background task completion notices, system messages, empty drafts; filtered from the root tree.
2. Provide visual differentiation in the tree view: sky badge for User Prompt, purple badge for AI Subagent instruction.
3. Allow filtering specifically by User Prompts vs AI Subagent instructions.
