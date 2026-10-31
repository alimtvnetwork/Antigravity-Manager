# Subtask Plan: TASK-01 - Prompt Subtree Hierarchy & Real-Time Liveness Detection

> **Task Slug**: `prompt-tree-subtree-and-import-export`  
> **Subtask Code**: `TASK-01`  
> **Plan File**: `.ai-memory/plans/subtasks/prompt-tree-subtree-and-import-export/01-prompt-subtree-and-liveness.md`  
> **Target Version**: `v4.222.0`  
> **Assigned Worker**: Worker 01 (Backend Liveness & Prompt Subtree Architecture)  
> **Dependencies**: None (Kickoff Wave 1)  
> **Reference Specification**: `02-spec/21-app/prompt-tree-subtree-and-import-export/01-subtree-and-liveness-spec.md`  

---

## Subtask Objective

Remediate the 3 isolated root causes that produced the false `IDLE` status in conversation `C226` (as seen in Screenshot 02 vs Screenshot 03), construct the complete 3-Tier Hierarchy (`Project` → `Conversation` → `Prompt Subtree`) across Rust backend models and React frontend components, extract all individual user prompt turns with attached screenshot images from conversation transcripts, and render an interactive, expandable prompt subtree with pulsing emerald RUNNING badges, amber QUEUED badges, and image thumbnail indicators.

---

## Target File Ownership & Changes

| Layer | Relative File Path | Planned Modifications |
| :--- | :--- | :--- |
| **Backend Models** | `src-tauri/src/modules/repo_db/models.rs` | Define `AgmPromptTurnNode` struct with `turn_number`, `step_index`, `prompt_text`, `preview_200w`, `status`, `is_running`, `is_queued`, `timestamp`, `created_at`, `media_paths`, `media_thumbnails`. Add `prompts: Vec<AgmPromptTurnNode>` to `AgmConversationNode`. |
| **Backend Parser** | `src-tauri/src/modules/repo_db/transcript.rs` | Fix Flaw 1 (prevent premature `is_terminal_done = true` on intermediate planner turns). Implement `extract_conversation_prompt_turns` to parse all user turns, image attachments (`.user_uploaded/media_*.png`), and determine per-turn execution states. |
| **Backend Collection** | `src-tauri/src/modules/repo_db/tree_collect.rs` | Fix Flaw 2: enforce absolute precedence of `is_owning_inst_alive && not_fully_idle > 0 && is_recent` over `is_terminal_done`. Collect prompt turns for each conversation node. |
| **Backend Node Conv** | `src-tauri/src/modules/repo_db/tree_node_conv.rs` | Fix Flaw 3: stop mutating active `CASCADE_RUN_STATUS_RUNNING` database records into `IDLE`. Attach `prompts` vector to `AgmConversationNode`. |
| **Backend Assembly** | `src-tauri/src/modules/repo_db/tree_nodes.rs` | Preserve and thread `prompts` array through deduplication and node transformation passes. |
| **Frontend Types** | `src/components/instances/PromptTreeView/types.ts` | Define `AgmPromptTurnNode` interface. Update `AgmConversationNode` to include optional `prompts?: AgmPromptTurnNode[]`. |
| **Frontend Component** | `src/components/instances/PromptTreeView/ConversationNode.tsx` | Add expandable prompt subtree under conversation item. Render indented guide lines, turn index badge (`Turn #1`), pulsing emerald badge for `RUNNING`, amber badge for `QUEUED`, image thumbnail indicators, and Poppins preview snippet. |
| **Frontend Container** | `src/components/instances/PromptTreeView/TreePanel.tsx` | Manage conversation subtree expansion state (`expandedConversations`) and auto-expand active running conversations. |

---

## Detailed Step-by-Step Implementation Checklist

### Phase 2.1: Rust Backend Liveness Remediation & Data Models

- [ ] **Step 1: Add `AgmPromptTurnNode` and Update `AgmConversationNode` in `src-tauri/src/modules/repo_db/models.rs`**
  - [ ] Define `AgmPromptTurnNode`:
    ```rust
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AgmPromptTurnNode {
        pub turn_number: usize,
        pub step_index: usize,
        pub prompt_text: String,
        pub preview_200w: String,
        pub status: String,
        pub is_running: bool,
        pub is_queued: bool,
        pub timestamp: String,
        pub created_at: i64,
        pub media_paths: Vec<String>,
        pub media_thumbnails: Vec<String>,
    }
    ```
  - [ ] Add `#[serde(default, skip_serializing_if = "Vec::is_empty")] pub prompts: Vec<AgmPromptTurnNode>` to `AgmConversationNode`.

- [ ] **Step 2: Fix Flaw 1 (Terminal Done Short-Circuit) in `src-tauri/src/modules/repo_db/transcript.rs`**
  - [ ] Audit lines 484–535 in `inspect_conversation_transcript`.
  - [ ] Ensure that a `PLANNER_RESPONSE` or `MODEL` step with zero tool calls does NOT set `is_terminal_done = true` if subsequent steps exist or if it represents an intermediate reasoning/orchestration turn.
  - [ ] Only flag `is_terminal_done = true` if the step is explicitly the last turn in the transcript and contains terminal completion markers (`DONE`, `COMPLETED`).

- [ ] **Step 3: Implement `extract_conversation_prompt_turns` in `src-tauri/src/modules/repo_db/transcript.rs`**
  - [ ] Read `transcript_full.jsonl` (with fallback to `transcript.jsonl`) from `brain/<cid>/.system_generated/logs/`.
  - [ ] Filter lines where `type == "USER_INPUT"` or `source == "USER_EXPLICIT"`.
  - [ ] Discard internal subagent messages (`<SYSTEM_MESSAGE>`, `[Message] timestamp=`, `You are ...`) via `extract_clean_user_prompt`.
  - [ ] Extract attached image paths from `media` array (resolving `.user_uploaded/media_*.png` and `.antigravity/attachments/`).
  - [ ] Calculate `turn_number` (1-indexed sequence), `preview_200w`, and timestamp.
  - [ ] Determine per-turn status:
    * If `is_conv_running`: Mark currently executing turn as `RUNNING` (`is_running = true`), any turns appended after as `QUEUED` (`is_queued = true`), and preceding turns as `COMPLETED`.
    * If `!is_conv_running`: Mark all turns as `COMPLETED`.

- [ ] **Step 4: Fix Flaw 2 (Precedence of `not_fully_idle`) in `src-tauri/src/modules/repo_db/tree_collect.rs`**
  - [ ] Refactor lines 169–177.
  - [ ] Evaluate `if is_owning_inst_alive && not_fully_idle > 0 && is_recent` FIRST to guarantee that live running IDE tasks always yield `is_conv_running = true`.
  - [ ] Restrict `inspection.is_terminal_done` so it can only confirm idleness when `not_fully_idle == 0`.
  - [ ] Invoke `extract_conversation_prompt_turns` for each conversation and attach result to the intermediate record.

- [ ] **Step 5: Fix Flaw 3 & Attach Prompts in `src-tauri/src/modules/repo_db/tree_node_conv.rs`**
  - [ ] In `tree_node_conv.rs:112-120`, prevent mutating `item.status` into `"IDLE"` when the owning instance is running.
  - [ ] Populate `prompts: item.prompts` when constructing `AgmConversationNode`.

- [ ] **Step 6: Thread Prompts Through `src-tauri/src/modules/repo_db/tree_nodes.rs`**
  - [ ] Ensure that helper methods aggregating, cloning, or converting conversation nodes preserve the `prompts` vector.

---

### Phase 2.2: Frontend TypeScript Interfaces & Prompt Subtree Rendering

- [ ] **Step 7: Update TypeScript Types in `src/components/instances/PromptTreeView/types.ts`**
  - [ ] Export `AgmPromptTurnNode` interface matching backend serialization.
  - [ ] Add `prompts?: AgmPromptTurnNode[];` to `AgmConversationNode`.

- [ ] **Step 8: Implement Subtree Expansion in `src/components/instances/PromptTreeView/ConversationNode.tsx`**
  - [ ] Add local `isExpanded` state (defaulting to `true` if `conv.is_running === true`).
  - [ ] Render interactive expansion chevron next to conversation icon when `conv.prompts && conv.prompts.length > 0`.
  - [ ] Prevent click bubbling on chevron from triggering conversation selection or inspector opening.

- [ ] **Step 9: Render Child Prompt Turns in `src/components/instances/PromptTreeView/ConversationNode.tsx`**
  - [ ] Beneath the conversation row, render an indented container (`border-l border-slate-200 dark:border-purple-900/40 ml-4 pl-2 space-y-1 mt-1`).
  - [ ] For each turn in `conv.prompts`:
    * Turn pill badge: `Turn #${turn.turn_number}` (`font-mono text-[9px] px-1 py-0.2 rounded bg-slate-200 dark:bg-purple-950/60 text-purple-300`).
    * Status badge:
      - If `turn.is_running`: Pulsing emerald badge (`animate-pulse bg-emerald-500/15 text-emerald-700 dark:text-[#1af18d] border border-emerald-500/40 text-[8.5px] px-1.5 py-0.2 rounded-full font-bold`).
      - If `turn.is_queued`: Amber badge (`bg-amber-500/15 text-amber-700 dark:text-amber-300 border border-amber-500/40 text-[8.5px] px-1.5 py-0.2 rounded-full font-bold flex items-center gap-1` with `Clock` icon).
      - If `turn.status === 'COMPLETED'`: Zinc/muted badge (`text-slate-400 dark:text-zinc-500 text-[8.5px]`).
    * Image attachment indicator: If `turn.media_paths.length > 0`, render image badge (`🖼️ ${turn.media_paths.length} img`) with hover tooltip.
    * Prompt snippet text: Display `turn.preview_200w` in Poppins typography (`font-[Poppins,sans-serif] text-[11px] truncate text-slate-700 dark:text-slate-300`).
    * Click turn handler: Select prompt turn and display its details in the right-hand inspection view.

- [ ] **Step 10: Enhance Subtree State Management in `src/components/instances/PromptTreeView/TreePanel.tsx`**
  - [ ] Support expanding and collapsing conversation subtrees.
  - [ ] Maintain consistent scroll positioning when subtrees expand.

---

### Phase 2.3: Verification & Pre-flight Quality Gates

- [ ] **Step 11: Rust Formatting Gate**
  - [ ] Run `cd src-tauri && cargo fmt -- --check`. Must return zero formatting errors.

- [ ] **Step 12: Rust Clippy Compilation Gate**
  - [ ] Run `cd src-tauri && cargo clippy --all-targets --all-features`. Must compile cleanly with zero warnings.

- [ ] **Step 13: Frontend Build Gate**
  - [ ] Run `npm run build`. Must compile with zero TypeScript errors and zero missing symbols.

- [ ] **Step 14: Functional & Visual Verification Against Screenshots**
  - [ ] Verify Screenshot 02 scenario: Active IDE task `Continuing Previous Task` displays pulsing emerald `RUNNING` status in AGM tree.
  - [ ] Verify that enqueued messages display amber `QUEUED` badges with clock glyphs.
  - [ ] Verify that prompt turns with images render image chips and file attachment references.
