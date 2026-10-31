# Subtask 03: Reset Button Functionality and Version Invalidation

## 1. Overview
- **Subtask ID**: `03-reset-button-and-version-invalidation`
- **Component**: `src-tauri/src/commands/` & `src/components/instances/PromptTreeView/`
- **Owner**: Full-stack Worker
- **Status**: COMPLETED

## 2. Objectives
1. Implement backend cache purge command: `cmd_reset_prompt_tree` / `/api/prompts/reset` to delete SQLite rows from `prompt_tree_cache`.
2. Connect frontend Reset button in `PromptTreeHeader.tsx` to the cache reset command, reloading the tree immediately with visual spin feedback.
3. Track application version in cache metadata; automatically invalidate cache when `current_version != cached_version`.
4. Display the application version badge (e.g. `v4.238.0`) in `PromptTreeHeader.tsx`.

## 3. Verification Criteria
- [x] Backend cache reset command `reset_prompt_tree_cache` purges SQLite `prompt_tree_cache` rows and invalidates memory cache (`tree.rs`).
- [x] Version embedded into SQLite cache key `tree:v{pkg_version}:{}:{}:{}` and dynamic `localStorage` version check auto-resets cache on upgrade (`tree.rs`, `PromptTreeViewModal.tsx`).
- [x] Version badge (`v{appVersion}`) visible in `ModalHeader.tsx` with purple dark-glass styling.
- [x] Frontend Reset button flushes in-memory SWR cache, clears filters, invokes backend reset, and triggers full reload with spin animation.
