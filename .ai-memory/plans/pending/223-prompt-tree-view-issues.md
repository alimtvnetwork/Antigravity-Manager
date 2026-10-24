# Plan 223: Prompt Tree View Issues

**Status:** IN PROGRESS

## Steps

1. Compact the tree header, icon-only Import/Export and Hide Tree, labeled Reset and Refresh.
2. Remove conversation hover borders and tighten tree spacing.
3. Refresh only on the Sync interval or the Refresh button.
4. Dispatch Current Conv through `agy --conversation`, including `conv-` ids. `new` stays a sentinel and is not passed as `--conversation`. CLI `--new` and a missing session use that sentinel instead of a synthetic `conv-{uuid}`.
5. Emit workspace-relative image markdown.
6. Match `prompt-{id}` strip ids to bare conversation ids. `findStripConversation` already strips the `prompt-` prefix and falls back to the sole running row. The E9001 warning still fires when the strip uuid is absent from `treeData` and more than one conversation is running.
7. Segment instance-card actions and raise yellow-hover contrast.
8. Done for the dump path. `agm errors list` reads `errors.db`. Skill: `.agents/skills/agm-error-manager-cli/skill.md`. Drawer history is in-memory and is not cleared by `agm errors clear`. `failed_to_resolve_parent` is a stale history string; `cmd_files.rs` now reports `invalid_path: parent directory cannot be opened`.

## Verification

- `cargo check` in `src-tauri` for `src-tauri/src/modules/repo_db/agy.rs`.
- UI files listed in the spec compile with the frontend typecheck when a full build is run.
