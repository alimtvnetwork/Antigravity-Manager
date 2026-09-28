# [Completed] GitMap AGY Parity, Dual AGM/GitMap Bracketed Tree View (`P001`/`C001` + `GM:#1`/`GM:<cid>`), & Multi-Instance / Multi-Project Account Swap Isolation

- **Status**: `COMPLETED`
- **Completed At**: 2026-09-28
- **Specification**: [02-gitmap-agm-tree-instance-swap-spec.md](../../../02-spec/21-app/02-gitmap-agm-tree-instance-swap-spec.md)

## Consolidated Subtasks & Verified Outcomes
1. **Task-01: Latest GitMap CLI Parity (`agy`, `ssh`, `update`, `prompt`, `backup/restore`) in AGM & Telegram (`[Completed]`)**:
   - Added full command parity and interactive examples for `gitmap agy active`, `gitmap agy running-prompts ls|backup|restore`, `gitmap backup-running-prompts`, `gitmap restore-running-prompts`, `gitmap agy prompt -n read-all|is-done -t "..."`, `gitmap agy prompt-project P001 -n is-done -t "..."`, `gitmap agy fpug`, `gitmap agy sug`, `gitmap agy rerun`, `gitmap ssh nodes|exec|update agm`, and `gitmap agy ssh` across `src-tauri/src/modules/telegram_inbound.rs`, `src-tauri/src/bin/agm.rs`, and `src/components/settings/EmailNotificationSettings.tsx`.
2. **Task-02: Bracketed Project → Conversation → 200-Word Prompt Tree View, Dual AGM/GitMap Sequence IDs, & Instance/Machine-Scoped Prompt Injection (`[Completed]`)**:
   - Upgraded `AgmProjectTreeNode`, `AgmConversationNode`, `AgmSequenceResolution`, `get_project_conversation_tree`, `format_tree_view_cli`, and `format_tree_view_telegram_html` in `src-tauri/src/modules/repo_db.rs` to render bracketed hierarchy with dual sequence IDs (`[AGM:P001 | GM:#1]` and `[AGM:C001 | GM:<cid>]`) and `[Prompt ≤200w (<N> words)]` across all registered instances.
   - Added `prompt_target_by_sequence_scoped` and `--instance <id|#seq|name>` / `--node <node>` flag extraction so prompts can target any conversation or project by AGM or GitMap sequence ID on a specific instance or remote SSH cluster machine.
3. **Task-03: Multi-Instance Creation, Multi-Project Workspace Binding, and Multi-Account Swapping End-to-End Verification & Isolation (`[Completed]`)**:
   - Added `get_instance_workspace_folders` and `assign_project_to_instance` (`agm instances assign <#seq|id> <repo_paths...>`) in `src-tauri/src/modules/instance.rs`.
   - Fixed `launch_instance` and `switch_account_to_instance` so secondary instances (`!is_default`) never overwrite the global OS Keyring or global `current_account_id`, always re-open all bound project workspace folders on relaunch, and scope prompt backup/restoration (`backup_running_prompts(&instance.id)` and `resend_running_commands_for_instance(Some(&instance.id), 20)`) strictly to the target instance.
