# Subtask 01: Coding Guidelines (CG) Refactoring & Guard Clause Simplification

**Parent Task:** `ui-and-functionality-enhancement-instructions`  
**Subtask Code:** `Subtask-01`  
**Status:** `READY`

### Objective
Apply concrete coding guideline fixes across touched modules:
1. Simplify complex `if-else` conditionals and invert guard clauses (`booleans-and-complex-conditions`).
2. Enforce naming conventions and positive boolean prefixes (`is`/`has`) (`naming-conventions-and-boolean-prefixes`).
3. Replace magic strings and numbers with typed enums and named constants (`constants-and-enums`).
4. Ensure CLI commands have clean help formatting (`cli-commands-and-help`).
5. Verify function return types follow sequence-14 rules (`function-signatures-and-return-types`).

### Target Files
- `src/pages/instances/instancePageUtils.ts`
- `src-tauri/src/modules/repo_db/tree_node_conv.rs`
- `src-tauri/src/modules/repo_db/liveness.rs`
- `src/components/instances/PromptTreeView/DetailHeader.tsx`
