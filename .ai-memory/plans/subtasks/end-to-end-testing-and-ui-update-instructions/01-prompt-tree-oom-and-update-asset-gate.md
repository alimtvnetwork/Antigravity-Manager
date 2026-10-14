# Subtask 01: Prompt Tree OOM Fix, Update Asset Gate, and CLI Prompt Copy/Resend

- **Status**: `COMPLETED`

## Target Files
- `src/components/instances/PromptTreeView/usePromptTreeData.ts`
- `src-tauri/src/modules/repo_db/tree.rs`
- `src-tauri/src/modules/repo_db/tree_format.rs`
- `src-tauri/src/modules/cli/prompts.rs`
- `src-tauri/src/modules/process/launch.rs`
- `src-tauri/src/modules/cli/types.rs`
- `src-tauri/src/modules/repo_db/detection.rs`
- `src-tauri/src/modules/update_checker/channels.rs`
- `src-tauri/src/modules/update_checker/platform.rs`
- `src-tauri/src/modules/delegate_updater/ui.rs`
- `src-tauri/src/modules/logger.rs`
- `src/stores/use-update-store.ts`
- `src/components/UpdateNotification.tsx`
- `src-tauri/src/bin/agm/prompt_cmds.rs`
- `src-tauri/src/bin/agm/instance_cmds.rs`
- `src-tauri/src/bin/agm/main.rs`

## Verification Results
- [x] Opening Prompt Tree in `agm-alim` keeps process thread count stable (`51–54` threads, `~91 MB` working set) instead of climbing to `565` threads (`0xe0000008` OOM resolved).
- [x] `check_for_updates` returns `has_update: false` when running the latest version or when a newer tag lacks the current platform installer asset (`.exe`/`.msi` on Windows, `.dmg`/`.app.tar.gz` on macOS, `.AppImage`/`.deb`/`.rpm` on Linux).
- [x] `agm prompts running --json`, `agm prompts resend --json`, `agm prompts copy --json`, `agm prompts backup --json`, `agm prompts restore --json`, and `agm prompts reset-tree --json` emit pure JSON on stdout and succeed from the terminal.
