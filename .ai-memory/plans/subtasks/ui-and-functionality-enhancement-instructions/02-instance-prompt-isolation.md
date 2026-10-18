# Subtask 02: Instance Prompt Isolation & Idle Supremacy

Parent Task: `ui-and-functionality-enhancement-instructions`
Task-ID: `Task-03`
Owner: Worker 02

## Objectives
1. Enforce Explicit Idle Supremacy in `tree_collect.rs` and `liveness.rs`: check `has_idle_status` (IDLE, COMPLETED, FAILED, CANCELLED) before `not_fully_idle`.
2. In `tree_node_conv.rs` line 47, eliminate the unconditional `(is_inst_alive && item.status.contains('RUNNING'))` override so evaluated idle status is strictly honored.
3. In `instanceCardData.ts`, add conversation-level instance ownership verification: `(node.conversations || []).some(c => c.is_running && !isGhostConversation(c) && (c.instance_id === inst.config.id || (!c.instance_id && inst.config.id === 'default')))` so Instance 1 does not display Instance 2's running prompt.
4. In `tree_phases.rs::query_tree_projects`, iterate through all instances in registry when target is `None`.
