# Subtask 04: Prompt Tree Context Menus, z-Index, Conversation View & Enqueued Prompts

Parent Task: `ui-and-functionality-enhancement-instructions`
Task-ID: `Task-05`
Owner: Worker 01

## Objectives
1. Fix Global z-Index Stacking: Elevate `ConversationContextMenu.tsx`, `ProjectContextMenu.tsx`, `DetailsModal.tsx`, `InspectorModal.tsx`, `VerificationModal.tsx`, and `PromptImagePreviewModal.tsx` from `z-[10000]` / `z-50` to `z-[100050]`.
2. Connect `onProjectContextMenu` in `TreePanel.tsx` -> `<ProjectNode />`.
3. In `tree_node_conv.rs`, do not drop queued/running active prompts (`continue`). Instead append them to `target_conv.prompts` as turns and set `is_queued = true`.
4. In `PromptInstructionView.tsx`, when `selectedTurnNumber === null`, render the Conversation Session Summary Dashboard (metrics, duration, enqueued prompt chips, and chronological turn navigation).
