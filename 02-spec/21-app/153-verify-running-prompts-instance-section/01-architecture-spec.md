# Specification: Verification of Running Prompts in Instance Section

> **Spec ID:** `153-verify-running-prompts-instance-section`  
> **Status:** APPROVED & VERIFIED  
> **Scope:** Backend Liveness Architecture, Instance Card Scoping, Projects Section Display, Prompt Tree Deep-Linking  

---

## 1. Executive Summary

This specification verifies the complete end-to-end data flow that enables active running prompts to accurately appear in the prompts section of an Antigravity instance.

### System Verification Points:
1. **Backend Liveness & Dead-Process Gating**:
   - `tree_node_liveness.rs` and `tree_node_conv.rs` check if the instance process is alive (`is_inst_alive`).
   - If the instance process is dead, `is_running` is strictly forced to `false`.
   - If the instance is alive, active conversation turns from `conversation_summaries.db` (`RUNNING` / `not_fully_idle = 1`) or active SQLite prompts (`active_prompts`) within the TTL window (45s) are flagged `is_running = true`.
2. **Per-Instance Ownership Isolation**:
   - `isNodeOwnedByInstance` maps projects and conversation nodes strictly to the owning instance configuration (`instConfig.id` or default aliases `default`/`__default__`), preventing cross-instance state bleed.
3. **Instance Card Prompts Badging & Projects Section**:
   - `useInstanceCardData` calculates `hasActiveTask`. When `true`, a pulsing emerald `[● Prompt]` button is displayed in the card header.
   - `InstanceCardProjects` renders the "Active / Recent Projects" section, sorts running projects to the top, and renders the green `[● RUNNING]` pill badge.
4. **Prompt Tree View Modal (`PromptTreeViewModal`)**:
   - Opening the modal passes `instanceId` and optional `initialSelectedProjectId`.
   - `usePromptTreeData` automatically prioritizes non-archived projects, detects running conversations, and auto-selects the active running conversation.
   - The `Running` segmented filter button allows instant filtering to active running prompts.
