# Plan 223: Prompt Tree View Issues

**Status:** IN PROGRESS

## Steps

1. Compact the tree header, icon-only Import/Export and Hide Tree, labeled Reset and Refresh.
2. Remove conversation hover borders and tighten tree spacing.
3. Refresh only on the Sync interval or the Refresh button.
4. Dispatch Current Conv through `agy --conversation`. New Conv starts a new chat.
5. Emit workspace-relative image markdown.
6. Match `prompt-{id}` strip ids to bare conversation ids.
7. Segment instance-card actions and raise yellow-hover contrast.
8. Record the error-manager CLI skill.

## Verification

- `cargo check` in `src-tauri` for `src-tauri/src/modules/repo_db/agy.rs`.
- UI files listed in the spec compile with the frontend typecheck when a full build is run.
