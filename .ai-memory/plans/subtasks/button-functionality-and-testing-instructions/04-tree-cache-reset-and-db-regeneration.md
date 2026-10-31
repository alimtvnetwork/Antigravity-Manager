# Subtask 04: Prompt Tree Cache & DB Reset

**Subtask ID**: `04-tree-cache-reset-and-db-regeneration`  
**Parent Plan**: `.ai-memory/plans/button-functionality-and-testing-instructions.md`  
**Status**: QUEUED  

## Intent
Implement full reset behavior in the Reset button. When clicked, it must invoke a backend command to clear `repo_prompts.db` records for the instance, purge the memory cache, and completely re-scan all transcripts from the filesystem to regenerate the tree view fresh from disk.

## Files
- `src/components/instances/PromptTreeView/DetailHeader.tsx`
- `src-tauri/src/commands/cmd_repo_db.rs`
- `src-tauri/src/modules/repo_db/mod.rs`
