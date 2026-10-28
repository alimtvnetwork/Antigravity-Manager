# Plan 223: Prompt Tree View Issues

**Status:** COMPLETED

## Steps

1. Compact the tree header, icon-only Import/Export and Hide Tree, labeled Reset and Refresh.
2. Remove conversation hover borders and tighten tree spacing.
3. Refresh only on the Sync interval or the Refresh button.
4. Dispatch Current Conv through `agy --conversation`, including `conv-` ids. `new` stays a sentinel and is not passed as `--conversation`. CLI `--new` and a missing session use that sentinel instead of a synthetic `conv-{uuid}`.
5. Emit workspace-relative image markdown for dispatch. Tree display uses `file:///` for out-of-workspace media.
6. Match `prompt-{id}` strip ids to bare conversation ids. `findStripConversation` already strips the `prompt-` prefix and falls back to the sole running row. The E9001 warning still fires when the strip uuid is absent from `treeData` and more than one conversation is running.
7. Segment instance-card actions into one full-width capsule and raise yellow-hover contrast.
8. `agm errors list` reads `errors.db`. Skill: `.agents/skills/agm-error-manager-cli/skill.md`. Drawer history is in-memory and is not cleared by `agm errors clear`. `failed_to_resolve_parent` is a stale history string; `cmd_files.rs` now reports `invalid_path: parent directory cannot be opened`.
9. `agm prompts tree --json --local` rebuilds in the current binary and skips the GUI daemon cache.

## Verification

- `gitmap cargo build --bin agm --offline` finished with exit 0.
- Unit tests: `test_conversation_display_title`, `test_workspace_relative_media`, `conversation_flag_keeps_real_ids_and_skips_synthetic_keys` all passed.
- Isolated instance `test-e2e-tree-*` created, started, sent `--conversation conv-e2e-probe` (`session_id` kept), then `test-clean`. Caller `default` and `v2-3127` stayed running.
- Local tree dump: 144 conversations, 0 untitled titles, 68 `file:///` media paths, 0 raw drive paths.
