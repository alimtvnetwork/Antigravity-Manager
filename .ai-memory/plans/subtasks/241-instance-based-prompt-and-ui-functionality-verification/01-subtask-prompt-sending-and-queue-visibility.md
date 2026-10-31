# Subtask 01: Verify Instance-Based Prompt Sending & Queued Prompts Visibility

- **Task**: Instance-Based Prompt and UI Functionality Verification
- **Subtask**: 01 of 06
- **Status**: Completed
- **Path**: `.ai-memory/plans/subtasks/241-instance-based-prompt-and-ui-functionality-verification/01-subtask-prompt-sending-and-queue-visibility.md`

## Audit Outcomes
- Verified `src-tauri/src/modules/repo_db/tree_nodes.rs` and `tree_node_conv.rs`: `is_owning_instance_alive` and `is_queued_or_pending_status` correctly support `"enqueued_ide"`, `"dispatched"`, and `"in_progress"`.
- Verified CDP queued messages harvester in `src-tauri/src/modules/instance/cdp_onboarding.rs` (`extract_queued_composer_messages_for_instance`).
- Verified `src/components/instances/PromptTreeView/usePromptTreeData.ts`: `queuedPromptCount` strictly avoids spurious review queue counts.
- Tenant scoping and multi-instance alias matching confirmed functional across default and secondary instances.
